//! The data ports of the mirrored streams.

use std::sync::{Arc, Mutex};

use phairplay_core::media::{AudioSink, EncodedAudioFrame, EncodedVideoFrame, VideoSink};
use tokio::net::{TcpListener, TcpStream};

use crate::limits::AirPlayLimits;
use crate::rtsp::reader::StreamBuffer;
use crate::rtsp::router::StreamPortAllocator;
use crate::rtp::{self, FuAReassembler};

#[derive(Debug)]
pub struct MirrorPorts {
    streams: Mutex<Vec<MirrorStream>>,
    video_sink: Arc<dyn VideoSink>,
    audio_sink: Arc<dyn AudioSink>,
    limits: AirPlayLimits,
}

#[derive(Debug)]
struct MirrorStream {
    stream_type: u8,
    port: u16,
}

impl MirrorPorts {
    /// Creates the pool.
    #[must_use]
    pub fn new(
        video_sink: Arc<dyn VideoSink>,
        audio_sink: Arc<dyn AudioSink>,
        limits: AirPlayLimits,
    ) -> Self {
        Self {
            streams: Mutex::new(Vec::new()),
            video_sink,
            audio_sink,
            limits,
        }
    }

    /// The ports that are currently allocated.
    #[must_use]
    pub fn allocated_ports(&self) -> Vec<(u8, u16)> {
        self.streams
            .lock()
            .map(|guard| {
                guard
                    .iter()
                    .map(|stream| (stream.stream_type, stream.port))
                    .collect()
            })
            .unwrap_or_default()
    }
}

impl StreamPortAllocator for MirrorPorts {
    fn allocate(&self, stream_type: u8) -> Option<u16> {
        let standard = std::net::TcpListener::bind(("0.0.0.0", 0)).ok()?;
        standard.set_nonblocking(true).ok()?;
        let port = standard.local_addr().ok()?.port();
        let listener = TcpListener::from_std(standard).ok()?;
        {
            let mut guard = self.streams.lock().ok()?;
            guard.push(MirrorStream {
                stream_type,
                port,
            });
        }
        let video_sink = self.video_sink.clone();
        let audio_sink = self.audio_sink.clone();
        let limits = self.limits.clone();
        tokio::spawn(async move {
            if let Ok((socket, _peer)) = listener.accept().await {
                pump_stream(socket, stream_type, video_sink, audio_sink, limits).await;
            }
        });
        Some(port)
    }
}

/// Reads the interleaved media of one mirrored stream.
async fn pump_stream(
    socket: TcpStream,
    stream_type: u8,
    video_sink: Arc<dyn VideoSink>,
    audio_sink: Arc<dyn AudioSink>,
    limits: AirPlayLimits,
) {
    let mut stream = StreamBuffer::new(socket);
    let mut reassembler = FuAReassembler::new();
    loop {
        let frame = match rtp::read_interleaved_frame(
            &mut stream,
            limits.max_rtp_frame_bytes,
            limits.idle_timeout,
        )
        .await
        {
            Ok(Some(frame)) => frame,
            Ok(None) => break,
            Err(error) => {
                tracing::warn!(%error, "a mirrored stream failed");
                break;
            }
        };
        if stream_type == 96 || stream_type == 103 {
            let pts_us = 0;
            if let Err(error) = audio_sink.submit(EncodedAudioFrame {
                sequence: phairplay_core::media::next_frame_sequence(),
                pts_us,
                data: Arc::new(frame.payload),
            }) {
                tracing::warn!(%error, "the audio sink rejected a packet");
            }
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
        if let Err(error) = video_sink.submit(EncodedVideoFrame {
            sequence: phairplay_core::media::next_frame_sequence(),
            pts_us: nal.pts_us,
            data: Arc::new(data),
            pixel_format: None,
        }) {
            tracing::warn!(%error, "the video sink rejected a frame");
        }
    }
}
