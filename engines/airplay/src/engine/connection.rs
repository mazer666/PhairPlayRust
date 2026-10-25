//! The connection handling of the AirPlay engine.

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use phairplay_core::media::EncodedVideoFrame;
use phairplay_core::privacy::mask_socket_addr;
use phairplay_core::protocol::EngineContext;
use phairplay_core::session::{SessionEndReason, SessionEvent};
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};

use crate::engine::ports::MirrorPorts;
use crate::engine::{AirPlayConfig, EngineInner};
use crate::pairing::identity::PairingIdentity;
use crate::rtp::{self, FuAReassembler};
use crate::rtsp::reader::{RtspRequestReader, StreamBuffer};
use crate::rtsp::request::RtspMethod;
use crate::rtsp::response::RtspResponse;
use crate::rtsp::router::{PhotoSlot, RouteContext};
use crate::session::AirPlaySession;

pub(super) async fn accept_loop(
    listener: TcpListener,
    inner: Arc<EngineInner>,
    config: AirPlayConfig,
    ctx: EngineContext,
    identity: Arc<PairingIdentity>,
    ports: Arc<MirrorPorts>,
) {
    loop {
        let accepted = listener.accept().await;
        let (socket, peer) = match accepted {
            Ok(accepted) => accepted,
            Err(error) => {
                tracing::warn!(%error, "the AirPlay listener failed");
                return;
            }
        };
        // WHY the limit comes from the context: the rule that says how many
        // peers a protocol serves has **one** owner — the list
        // `session.multi_peer_protocols` of the configuration — and the
        // engine asks it instead of carrying a number of its own.
        let busy = {
            let Ok(mut state) = inner.state.lock() else {
                return;
            };
            if state.peer_count as usize >= ctx.peer_limit.max_peers() {
                true
            } else {
                state.peer_count = 1;
                false
            }
        };
        if busy {
            // AirPlay serves one sender at a time. The second one is told so
            // instead of being queued.
            tracing::warn!(peer = %mask_socket_addr(&peer), "rejecting a second AirPlay sender");
            reject_second_peer(socket).await;
            continue;
        }
        let inner = inner.clone();
        let config = config.clone();
        let ctx = ctx.clone();
        let identity = identity.clone();
        let ports = ports.clone();
        tokio::spawn(async move {
            handle_connection(socket, peer, inner, config, ctx, identity, ports).await;
        });
    }
}

/// Answers a second sender with `503 Service Unavailable` and disconnects it.
async fn reject_second_peer(mut socket: TcpStream) {
    let response = RtspResponse::service_unavailable().to_bytes(0);
    let _ = socket.write_all(&response).await;
    let _ = socket.flush().await;
    let _ = socket.shutdown().await;
}

/// Handles one sender from the first request to the last.
async fn handle_connection(
    socket: TcpStream,
    peer: SocketAddr,
    inner: Arc<EngineInner>,
    config: AirPlayConfig,
    ctx: EngineContext,
    identity: Arc<PairingIdentity>,
    ports: Arc<MirrorPorts>,
) {
    let mut stream = StreamBuffer::new(socket);
    let reader = RtspRequestReader::new(config.limits.clone());
    let mut session = AirPlaySession::with_identity(identity.clone());
    let photos: PhotoSlot = Arc::new(Mutex::new(None));
    let pairing_store = inner
        .pairing_store
        .lock()
        .ok()
        .and_then(|guard| guard.clone());

    let context = RouteContext {
        identity: &ctx.identity,
        capabilities: &ctx.capabilities,
        limits: &config.limits,
        require_pin: config.require_pin,
        events: Some(&ctx.events),
        photos: Some(&photos),
        pairing_store: pairing_store.as_deref(),
        stream_ports: ports.as_ref(),
        pairing_public_key: Some(identity.public_key()),
    };

    tracing::info!(peer = %mask_socket_addr(&peer), "an AirPlay sender connected");
    let mut switch_to_rtp = false;
    loop {
        let request = match reader.read_request(&mut stream).await {
            Ok(Some(request)) => request,
            Ok(None) => break,
            Err(error) => {
                tracing::warn!(peer = %mask_socket_addr(&peer), %error, "the AirPlay request could not be read");
                break;
            }
        };
        let cseq = request.cseq().unwrap_or(0);
        let response = crate::rtsp::router::route(&request, &mut session, &context);
        let bytes = response.to_bytes(cseq);
        let write_result = {
            let socket = stream.inner_mut();
            socket.write_all(&bytes).await
        };
        if let Err(error) = write_result {
            tracing::warn!(peer = %mask_socket_addr(&peer), %error, "the AirPlay answer could not be written");
            break;
        }
        if request.method == RtspMethod::Record
            && response.status == 200
            && !session.mirror
            && session.has_video()
        {
            switch_to_rtp = true;
            break;
        }
    }

    if switch_to_rtp {
        read_media(&mut stream, &session, &ctx, &config).await;
    }

    if session.streaming {
        let _ = ctx.events.try_send(SessionEvent::Ended {
            session: session.id,
            reason: SessionEndReason::SenderDisconnected,
        });
    }
    ctx.video_sink.reset();
    ctx.audio_sink.reset();
    if let Ok(mut guard) = inner.photos.lock() {
        *guard = None;
    }
    if let Ok(mut state) = inner.state.lock() {
        state.peer_count = 0;
    }
    tracing::info!(peer = %mask_socket_addr(&peer), "the AirPlay sender disconnected");
}

/// Reads the interleaved media of a mirroring session.
async fn read_media<R: tokio::io::AsyncRead + Unpin>(
    stream: &mut StreamBuffer<R>,
    session: &AirPlaySession,
    ctx: &EngineContext,
    config: &AirPlayConfig,
) {
    let mut reassembler = FuAReassembler::new();
    loop {
        let frame = match rtp::read_interleaved_frame(
            stream,
            config.limits.max_rtp_frame_bytes,
            config.limits.idle_timeout,
        )
        .await
        {
            Ok(Some(frame)) => frame,
            Ok(None) => break,
            Err(error) => {
                tracing::warn!(%error, "the interleaved media stream failed");
                break;
            }
        };
        if frame.channel != rtp::CHANNEL_VIDEO_RTP {
            continue;
        }
        let (header, payload) = match rtp::parse_rtp(&frame.payload) {
            Ok(parsed) => parsed,
            Err(error) => {
                tracing::warn!(%error, "an RTP packet could not be parsed");
                continue;
            }
        };
        let pts_us = rtp::timestamp_to_micros(header.timestamp);
        let Some(nal) = reassembler.push(payload, pts_us) else {
            continue;
        };
        let data = crate::mirror::avcc_to_annex_b(&nal.data);
        if let Err(error) = ctx.video_sink.submit(EncodedVideoFrame {
            sequence: phairplay_core::media::next_frame_sequence(),
            pts_us: nal.pts_us,
            data: Arc::new(data),
            pixel_format: None,
        }) {
            tracing::warn!(%error, "the video sink rejected a frame");
        }
    }
    let _ = session;
}
