use std::time::Duration;

use bytes::{Buf, BufMut, BytesMut};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use tokio::time::{interval, timeout, MissedTickBehavior};
use tokio_tungstenite::{
    connect_async_with_config,
    tungstenite::{protocol::WebSocketConfig, Message as WsMessage},
};
use tokio_util::codec::{Decoder, Encoder};

use crate::utils::{brotli_decode, zlib_decode};

const DANMAKU_WSS_URL: &str = "wss://broadcastlv.chat.bilibili.com/sub";
const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);
const MAX_WEBSOCKET_MESSAGE_BYTES: usize = 12 * 1024 * 1024;
const MAX_SUB_PACKETS_PER_FRAME: usize = 4096;

#[derive(Debug, Clone)]
struct ProtocolHeader {
    total_size: u32,
    header_size: u16,
    protocol_version: u16,
    opcode: u32,
    sequence: u32,
}

#[derive(Debug, Clone)]
pub struct Message {
    header: ProtocolHeader,
    payload: Vec<u8>,
}

pub struct Opcode;

#[allow(dead_code)]
impl Opcode {
    pub const HEADERBEAT: u32 = 2;
    pub const HEADERBEAT_REPLY: u32 = 3;
    pub const NORMAL: u32 = 5;
    pub const AUTH: u32 = 7;
    pub const AUTH_REPLAY: u32 = 8;
}

pub struct ProtocolVersion;

#[allow(dead_code)]
impl ProtocolVersion {
    pub const NORMAL: u16 = 0;
    pub const AUTH_HEARTBEAT: u16 = 1;
    pub const COMPRESSED_ZLIB: u16 = 2;
    pub const COMPRESSED_BROTLI: u16 = 3;
}

impl Message {
    fn new(opcode: u32, sequence: u32, payload: Vec<u8>) -> Self {
        let header_size = 16u16;
        let total_size = header_size as u32 + payload.len() as u32;

        Message {
            header: ProtocolHeader {
                total_size,
                header_size,
                protocol_version: 1,
                opcode,
                sequence,
            },
            payload,
        }
    }

    fn verification(sequence: u32, auth_data: Vec<u8>) -> Self {
        Self::new(Opcode::AUTH, sequence, auth_data)
    }

    fn heartbeat(sequence: u32) -> Self {
        Self::new(Opcode::HEADERBEAT, sequence, vec![])
    }
}

pub struct ProtocolCodec;

impl Decoder for ProtocolCodec {
    type Item = Message;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < 16 {
            return Ok(None);
        }

        let total_size = u32::from_be_bytes([src[0], src[1], src[2], src[3]]);
        let header_size = u16::from_be_bytes([src[4], src[5]]);
        let protocol_version = u16::from_be_bytes([src[6], src[7]]);
        let opcode = u32::from_be_bytes([src[8], src[9], src[10], src[11]]);
        let sequence = u32::from_be_bytes([src[12], src[13], src[14], src[15]]);

        if total_size > 10_000_000 {
            return Err(Self::Error::new(
                std::io::ErrorKind::InvalidData,
                "Message too large",
            ));
        }

        // A hostile peer must not be able to make the subsequent
        // `total_size - header_size` subtraction underflow or make `advance`
        // panic. `header_size` is always 16 in practice; anything outside
        // [16, total_size] is a malformed frame.
        if header_size < 16 || total_size < header_size as u32 {
            return Err(Self::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid header size",
            ));
        }

        if src.len() < total_size as usize {
            src.reserve(total_size as usize - src.len());
            return Ok(None);
        }

        let header = ProtocolHeader {
            total_size,
            header_size,
            protocol_version,
            opcode,
            sequence,
        };

        src.advance(header_size as usize);
        let payload_size = total_size as usize - header_size as usize;
        let payload = src[..payload_size].to_vec();
        src.advance(payload_size);

        Ok(Some(Message { header, payload }))
    }
}

impl Encoder<Message> for ProtocolCodec {
    type Error = std::io::Error;

    fn encode(&mut self, item: Message, dst: &mut BytesMut) -> Result<(), Self::Error> {
        dst.put_u32(item.header.total_size);
        dst.put_u16(item.header.header_size);
        dst.put_u16(item.header.protocol_version);
        dst.put_u32(item.header.opcode);
        dst.put_u32(item.header.sequence);
        dst.extend_from_slice(&item.payload);

        Ok(())
    }
}

#[derive(Clone, Serialize)]
struct ErrorEvent {
    error: String,
}

#[derive(Clone, Serialize)]
struct ConnectionEvent {
    status: String,
}

#[derive(Clone, Serialize)]
struct MessageEvent {
    payload: String,
}

#[derive(Serialize, Deserialize)]
struct AuthPayload {
    uid: u64,
    roomid: u32,
    protover: u32,
    platform: String,
    r#type: u16,
    key: String,
}

fn parse_header(view: &[u8]) -> Option<ProtocolHeader> {
    if view.len() < 16 {
        return None;
    }

    Some(ProtocolHeader {
        total_size: u32::from_be_bytes(view[0..4].try_into().ok()?),
        header_size: u16::from_be_bytes(view[4..6].try_into().ok()?),
        protocol_version: u16::from_be_bytes(view[6..8].try_into().ok()?),
        opcode: u32::from_be_bytes(view[8..12].try_into().ok()?),
        sequence: u32::from_be_bytes(view[12..16].try_into().ok()?),
    })
}

/// Splits a decompressed payload into sub-packet bodies.
///
/// Every length is attacker-controlled, so a sub-packet that declares zero
/// bytes must be rejected instead of advancing `offset` by zero forever.
fn split_sub_packets(buffer: &[u8]) -> Result<Vec<&[u8]>, &'static str> {
    let mut packets = Vec::new();
    let mut offset = 0usize;

    while offset < buffer.len() {
        if packets.len() >= MAX_SUB_PACKETS_PER_FRAME {
            return Err("too many sub packets");
        }

        let sub_buffer = &buffer[offset..];
        let header = parse_header(sub_buffer).ok_or("truncated sub packet header")?;

        if header.total_size == 0 {
            return Err("zero-sized sub packet");
        }
        if header.header_size < 16 {
            return Err("invalid sub packet header size");
        }

        let total_size = header.total_size as usize;
        let header_size = header.header_size as usize;
        if total_size < header_size {
            return Err("sub packet total size is smaller than its header");
        }

        let body_end = offset
            .checked_add(total_size)
            .ok_or("sub packet size overflow")?;
        if body_end > buffer.len() {
            return Err("truncated sub packet body");
        }

        packets.push(&buffer[offset + header_size..body_end]);
        offset = body_end;
    }

    Ok(packets)
}

fn encode_message(message: Message) -> Result<Vec<u8>, std::io::Error> {
    let mut codec = ProtocolCodec;
    let mut buffer = BytesMut::new();
    codec.encode(message, &mut buffer)?;
    Ok(buffer.to_vec())
}

pub struct SocketClient {
    app_handle: AppHandle,
}

impl SocketClient {
    pub fn new(value: AppHandle) -> SocketClient {
        SocketClient { app_handle: value }
    }

    fn parse_message(&self, payload: &[u8]) {
        match String::from_utf8(payload.to_vec()) {
            Ok(result) => {
                let _ = self
                    .app_handle
                    .emit("message-received", MessageEvent { payload: result });
            }
            Err(_) => eprintln!("Invalid message payload"),
        }
    }

    fn depack_sub_packet(&self, buffer: &[u8]) {
        match split_sub_packets(buffer) {
            Ok(packets) => {
                for payload in packets {
                    self.parse_message(payload);
                }
            }
            Err(error) => self.emit_error(format!("Invalid sub packets: {error}")),
        }
    }

    fn emit_error(&self, msg: impl Into<String>) {
        let _ = self
            .app_handle
            .emit("connection-error", ErrorEvent { error: msg.into() });
    }

    fn handle_incoming_message(&self, msg: Message) {
        if msg.header.opcode != Opcode::NORMAL {
            return;
        }

        match msg.header.protocol_version {
            ProtocolVersion::COMPRESSED_BROTLI => match brotli_decode(msg.payload) {
                Ok(decoded) => self.depack_sub_packet(&decoded),
                Err(e) => self.emit_error(format!("Brotli decompress error: {}", e)),
            },
            ProtocolVersion::COMPRESSED_ZLIB => match zlib_decode(msg.payload) {
                Ok(decoded) => self.depack_sub_packet(&decoded),
                Err(e) => self.emit_error(format!("Zlib decompress error: {}", e)),
            },
            _ => self.parse_message(&msg.payload),
        }
    }

    pub async fn connection(
        &self,
        uid: u64,
        room: u32,
        auth_token: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Bilibili exposes the danmaku stream over TLS. Connecting to the
        // plaintext port lets anyone on the network read the session token,
        // so always use the fixed WSS endpoint instead of accepting a
        // caller-supplied host/port.
        let websocket_config = WebSocketConfig::default()
            .max_message_size(Some(MAX_WEBSOCKET_MESSAGE_BYTES))
            .max_frame_size(Some(MAX_WEBSOCKET_MESSAGE_BYTES));
        let connection = timeout(
            CONNECT_TIMEOUT,
            connect_async_with_config(DANMAKU_WSS_URL, Some(websocket_config), true),
        )
        .await;

        let mut websocket = match connection {
            Ok(Ok((websocket, _))) => websocket,
            Ok(Err(e)) => {
                self.emit_error(format!("Connection failed: {}", e));
                return Err(e.into());
            }
            Err(_) => {
                self.emit_error("Connection timed out");
                return Err(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "connection timed out",
                )
                .into());
            }
        };

        let auth_obj = AuthPayload {
            uid,
            roomid: room,
            protover: 3,
            platform: "web".to_owned(),
            r#type: 2,
            key: auth_token,
        };
        let auth_json = serde_json::to_string(&auth_obj)?;
        let verification = Message::verification(1, auth_json.into_bytes());
        if let Err(e) = websocket
            .send(WsMessage::Binary(encode_message(verification)?.into()))
            .await
        {
            self.emit_error(format!("Failed to send auth: {}", e));
            return Err(e.into());
        }

        // Only report success after the authentication packet was sent. If an
        // earlier step fails, the frontend must not be left thinking that the
        // connection is healthy.
        let _ = self.app_handle.emit(
            "connection-success",
            ConnectionEvent {
                status: "connected".to_string(),
            },
        );

        let mut sequence = 2u32;
        let mut heartbeat_interval = interval(Duration::from_secs(30));
        heartbeat_interval.set_missed_tick_behavior(MissedTickBehavior::Delay);
        heartbeat_interval.tick().await;

        let mut codec = ProtocolCodec;
        let mut incoming = BytesMut::new();

        loop {
            tokio::select! {
                message = websocket.next() => {
                    let Some(message) = message else {
                        break;
                    };

                    match message {
                        Ok(WsMessage::Binary(data)) => {
                            incoming.extend_from_slice(&data);
                        }
                        Ok(WsMessage::Text(text)) => {
                            incoming.extend_from_slice(text.as_bytes());
                        }
                        Ok(WsMessage::Close(_)) => break,
                        Ok(_) => continue,
                        Err(e) => {
                            self.emit_error(format!("Receive error: {}", e));
                            break;
                        }
                    }

                    loop {
                        match codec.decode(&mut incoming) {
                            Ok(Some(msg)) => self.handle_incoming_message(msg),
                            Ok(None) => break,
                            Err(e) => {
                                self.emit_error(format!("Protocol error: {}", e));
                                incoming.clear();
                                break;
                            }
                        }
                    }
                }
                _ = heartbeat_interval.tick() => {
                    let heartbeat = Message::heartbeat(sequence);
                    sequence = sequence.wrapping_add(1);

                    let payload = match encode_message(heartbeat) {
                        Ok(payload) => payload,
                        Err(e) => {
                            self.emit_error(format!("Heartbeat encode error: {}", e));
                            break;
                        }
                    };

                    if let Err(e) = websocket.send(WsMessage::Binary(payload.into())).await {
                        self.emit_error(format!("Heartbeat error: {}", e));
                        break;
                    }
                }
            }
        }

        let _ = websocket.close(None).await;

        let _ = self.app_handle.emit(
            "connection-closed",
            ConnectionEvent {
                status: "disconnected".to_string(),
            },
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(total_size: u32, header_size: u16, payload: &[u8]) -> BytesMut {
        let mut buf = BytesMut::new();
        buf.put_u32(total_size);
        buf.put_u16(header_size);
        buf.put_u16(1); // protocol_version
        buf.put_u32(Opcode::NORMAL);
        buf.put_u32(0); // sequence
        buf.extend_from_slice(payload);
        buf
    }

    fn decode(buf: &mut BytesMut) -> Result<Option<Message>, std::io::Error> {
        ProtocolCodec.decode(buf)
    }

    #[test]
    fn decodes_well_formed_frame() {
        let payload = vec![0xABu8; 700];
        let mut buf = frame(716, 16, &payload);

        let msg = decode(&mut buf).expect("valid frame must decode").unwrap();
        assert_eq!(msg.payload, payload);
        assert!(buf.is_empty(), "frame must be fully consumed");
    }

    #[test]
    fn waits_for_incomplete_frame() {
        let mut buf = frame(716, 16, &[0u8; 100]);
        assert!(decode(&mut buf)
            .expect("incomplete frame is not an error")
            .is_none());
    }

    /// A frame whose declared `header_size` exceeds its `total_size` made
    /// `total_size - header_size` underflow and panicked, which aborted the
    /// entire process because release builds set `panic = "abort"`.
    #[test]
    fn rejects_header_size_larger_than_total_size() {
        let mut buf = frame(100, 200, &[0u8; 700]);
        let err = decode(&mut buf).expect_err("must be rejected, not panic");
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    }

    #[test]
    fn rejects_header_size_below_minimum() {
        let mut buf = frame(32, 15, &[0u8; 16]);
        let err = decode(&mut buf).expect_err("must be rejected, not panic");
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    }

    #[test]
    fn rejects_oversized_frame() {
        let mut buf = frame(10_000_001, 16, &[]);
        let err = decode(&mut buf).expect_err("must be rejected, not panic");
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    }

    /// A small compressed payload must not be able to expand past the
    /// decompression limit: the stream compresses ~1000x, so a tiny frame
    /// could otherwise force a huge allocation.
    #[test]
    fn zlib_decompression_is_bounded() {
        use flate2::write::ZlibEncoder;
        use flate2::Compression;
        use std::io::Write;

        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::best());
        encoder.write_all(&vec![0u8; 32 * 1024 * 1024]).unwrap();
        let compressed = encoder.finish().unwrap();

        assert!(
            compressed.len() < 64 * 1024,
            "expected a tiny compressed payload, got {} bytes",
            compressed.len()
        );
        assert!(crate::utils::zlib_decode(compressed).is_err());
    }

    #[test]
    fn zlib_decompresses_normal_payload() {
        use flate2::write::ZlibEncoder;
        use flate2::Compression;
        use std::io::Write;

        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::best());
        encoder.write_all(b"hello danmaku").unwrap();
        let compressed = encoder.finish().unwrap();

        assert_eq!(
            crate::utils::zlib_decode(compressed).unwrap(),
            b"hello danmaku"
        );
    }

    #[test]
    fn brotli_decompression_is_bounded() {
        use std::io::Write;

        let mut buf = Vec::new();
        {
            let mut encoder = brotli::CompressorWriter::new(&mut buf, 4096, 5, 22);
            encoder.write_all(&vec![0u8; 32 * 1024 * 1024]).unwrap();
        }

        assert!(
            buf.len() < 1024,
            "expected a tiny brotli payload, got {} bytes",
            buf.len()
        );
        assert!(crate::utils::brotli_decode(buf).is_err());
    }

    #[test]
    fn brotli_decompresses_normal_payload() {
        use std::io::Write;

        let mut buf = Vec::new();
        {
            let mut encoder = brotli::CompressorWriter::new(&mut buf, 4096, 5, 22);
            encoder.write_all(b"hello danmaku").unwrap();
        }

        assert_eq!(crate::utils::brotli_decode(buf).unwrap(), b"hello danmaku");
    }

    /// The old loop advanced `offset` by `total_size`, so a sub-packet that
    /// declared zero bytes looped forever and emitted empty messages.
    #[test]
    fn rejects_zero_sized_sub_packet() {
        let mut buffer = BytesMut::new();
        buffer.put_u32(0);
        buffer.put_u16(0);
        buffer.put_u16(1);
        buffer.put_u32(Opcode::NORMAL);
        buffer.put_u32(0);

        assert!(split_sub_packets(&buffer).is_err());
    }

    #[test]
    fn splits_valid_sub_packets() {
        let mut buffer = BytesMut::new();
        buffer.extend_from_slice(&frame(21, 16, b"hello"));
        buffer.extend_from_slice(&frame(19, 16, b"bye"));

        let packets = split_sub_packets(&buffer).unwrap();
        assert_eq!(packets, vec![&b"hello"[..], &b"bye"[..]]);
    }

    #[test]
    fn rejects_truncated_sub_packet() {
        let mut buffer = BytesMut::new();
        buffer.extend_from_slice(&frame(100, 16, b"short"));

        assert!(split_sub_packets(&buffer).is_err());
    }

    #[test]
    fn rejects_too_many_sub_packets() {
        let mut buffer = BytesMut::new();
        for _ in 0..=MAX_SUB_PACKETS_PER_FRAME {
            buffer.extend_from_slice(&frame(16, 16, &[]));
        }

        assert!(split_sub_packets(&buffer).is_err());
    }
}
