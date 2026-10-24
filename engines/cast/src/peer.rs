//! The life of one Cast peer.
//!
//! WHY: a Cast connection is a long-lived TLS socket with a small request and
//! response protocol on top. The rules that keep that socket harmless are
//! local to the connection: a handshake deadline, an idle deadline, a message
//! size limit and a rule that a malformed message ends the connection
//! instead of being repaired. Keeping them in one place makes them
//! auditable.
//!
//! WHAT: [`run_peer`] owns one connection from the TLS handshake to the
//! final `Ended` event.
//!
//! EXAMPLE:
//!
//! ```no_run
//! # use phairplay_cast::peer::run_peer;
//! # async fn example() {
//! # let inner = unimplemented!();
//! # let stream = unimplemented!();
//! # let tls = unimplemented!();
//! # let ctx = unimplemented!();
//! # let addr = unimplemented!();
//! run_peer(inner, stream, tls, ctx, addr).await;
//! # }
//! ```

use std::net::SocketAddr;
use std::sync::Arc;

use phairplay_core::protocol::EngineContext;
use phairplay_core::session::{SessionEndReason, SessionEvent, SessionId, SessionSnapshot};
use tokio::net::TcpStream;
use tokio_rustls::TlsAcceptor;

use crate::engine::Inner;
use crate::frame::{FrameError, read_frame, write_frame};
use crate::message::CastMessage;
use crate::session::{PeerSession, SessionAction};

/// Runs one Cast connection until it ends.
///
/// The function never returns an error: every failure is reported as a
/// [`SessionEndReason`] on the event channel, because the peer is not there
/// to read an error page.
pub(crate) async fn run_peer(
    inner: Arc<Inner>,
    stream: TcpStream,
    tls: TlsAcceptor,
    ctx: EngineContext,
    peer_addr: SocketAddr,
) {
    let handshake =
        tokio::time::timeout(inner.config.handshake_timeout(), tls.accept(stream)).await;
    let tls_stream = match handshake {
        Ok(Ok(stream)) => stream,
        Ok(Err(_)) => {
            tracing::debug!(%peer_addr, "cast peer failed the TLS handshake");
            return;
        }
        Err(_) => {
            tracing::debug!(%peer_addr, "cast peer did not finish the TLS handshake in time");
            return;
        }
    };

    let (mut reader, mut writer) = tokio::io::split(tls_stream);
    let index = inner.next_transport_index();
    let transport_id = format!("web-{index}");
    let session_id = format!("session-{index}");
    let mut peer = PeerSession::new(
        transport_id.clone(),
        session_id,
        inner.config.app_id.clone(),
    );
    let session = SessionId::next();
    inner.register_session(session, transport_id);
    tracing::info!(%peer_addr, %session, "cast peer connected");

    let snapshot = SessionSnapshot {
        id: session,
        protocol: phairplay_core::protocol::ProtocolKind::Cast,
        sender_name: None,
        sender_address: Some(peer_addr.to_string()),
        video: None,
        audio: None,
        started_at: std::time::Instant::now(),
    };
    let _ = ctx.events.send(SessionEvent::Started { session: snapshot });

    for message in peer.greeting() {
        if write_message(&mut writer, &message, inner.config.max_message_bytes)
            .await
            .is_err()
        {
            break;
        }
    }

    let reason = peer_loop(&inner, &mut reader, &mut writer, &mut peer, &ctx, session).await;

    inner.unregister_session(session);
    let _ = ctx.events.send(SessionEvent::Ended {
        session,
        reason: reason.clone(),
    });
    tracing::info!(%peer_addr, %session, %reason, "cast peer disconnected");
}

/// The read-dispatch-write loop of a peer.
async fn peer_loop<R, W>(
    inner: &Arc<Inner>,
    reader: &mut R,
    writer: &mut W,
    peer: &mut PeerSession,
    ctx: &EngineContext,
    session: SessionId,
) -> SessionEndReason
where
    R: tokio::io::AsyncRead + Unpin,
    W: tokio::io::AsyncWrite + Unpin,
{
    let idle = inner.config.idle_timeout();
    loop {
        let frame =
            tokio::time::timeout(idle, read_frame(reader, inner.config.max_message_bytes)).await;
        let payload = match frame {
            Err(_) => {
                return SessionEndReason::Failed(
                    "the peer sent nothing within the idle timeout".into(),
                );
            }
            Ok(Err(FrameError::Io(_))) => return SessionEndReason::SenderDisconnected,
            Ok(Err(FrameError::TooLarge(size))) => {
                return SessionEndReason::LimitExceeded(format!(
                    "a message of {size} bytes exceeds the limit"
                ));
            }
            Ok(Err(FrameError::Empty)) => {
                return SessionEndReason::ProtocolViolation("an empty message was sent".into());
            }
            Ok(Ok(payload)) => payload,
        };

        let message = match CastMessage::decode(&payload) {
            Ok(message) => message,
            Err(error) => {
                return SessionEndReason::ProtocolViolation(format!(
                    "the message could not be decoded: {error}"
                ));
            }
        };

        let dispatch = match peer.handle(&message) {
            Ok(dispatch) => dispatch,
            Err(error) => {
                return SessionEndReason::ProtocolViolation(error.to_string());
            }
        };

        for action in &dispatch.actions {
            apply_action(action, ctx, session);
        }
        for reply in &dispatch.replies {
            if write_message(writer, reply, inner.config.max_message_bytes)
                .await
                .is_err()
            {
                return SessionEndReason::SenderDisconnected;
            }
        }
        if dispatch
            .actions
            .iter()
            .any(|action| matches!(action, SessionAction::Close))
        {
            return SessionEndReason::SenderDisconnected;
        }
    }
}

/// Applies one action of the dispatch to the sinks and the event channel.
fn apply_action(action: &SessionAction, ctx: &EngineContext, session: SessionId) {
    match action {
        SessionAction::SetVolume { level, muted } => {
            let _ = ctx.audio_sink.set_volume(*level);
            if *muted {
                let _ = ctx.audio_sink.set_volume(0.0);
            }
        }
        SessionAction::MetadataChanged {
            title,
            artist,
            album,
        } => {
            let _ = ctx.events.send(SessionEvent::MetadataChanged {
                session,
                title: title.clone(),
                artist: artist.clone(),
                album: album.clone(),
            });
        }
        SessionAction::PlaybackStateChanged { state } => {
            let _ = ctx.events.send(SessionEvent::PlaybackStateChanged {
                session,
                state: state.clone(),
            });
        }
        SessionAction::Close => {}
    }
}

/// Writes one message to the peer.
async fn write_message<S>(
    writer: &mut S,
    message: &CastMessage,
    max_bytes: usize,
) -> Result<(), FrameError>
where
    S: tokio::io::AsyncWrite + Unpin,
{
    let payload = message
        .encode()
        .map_err(|error| FrameError::TooLarge(error.to_string().len()))?;
    write_frame(writer, &payload, max_bytes).await
}
