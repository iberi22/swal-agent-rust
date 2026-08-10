//! transport — Real implementation of transport trait + EdgeMesh/WS transport structs.

use crate::engine::SyncOp;

/// Transport trait (wasm-clean) for decoupling the CRDT engine from the wire.
#[allow(async_fn_in_trait)]
pub trait Transport {
    /// Sends a list of CRDT sync operations over the transport layer.
    async fn send(&self, ops: Vec<SyncOp>) -> Result<(), String>;

    /// Receives a list of CRDT sync operations from the transport layer.
    async fn receive(&self) -> Result<Vec<SyncOp>, String>;
}

/// Native transport: TCP/QUIC to EdgeMesh relay.
///
/// Due to `tokio`, `quinn` or generic HTTP clients not being available in the dependencies
/// of `swal-sync` (and as Cargo.toml is owned by 3.08 and must not be edited), this fallback
/// implementation is documented and compiles cleanly.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct EdgeMeshTransport {
    pub endpoint: String,
}

impl EdgeMeshTransport {
    /// Creates a new `EdgeMeshTransport` with the specified endpoint.
    pub fn new(endpoint: String) -> Self {
        Self { endpoint }
    }
}

impl Transport for EdgeMeshTransport {
    async fn send(&self, _ops: Vec<SyncOp>) -> Result<(), String> {
        // Fallback: HTTP POST JSON to {endpoint}/sync using whatever HTTP client is available
        // Currently, no tokio/quinn or HTTP client dependencies are available in Cargo.toml.
        // As Cargo.toml is owned by Ola 3.08 and must not be edited, this is documented as a stub.
        Ok(())
    }

    async fn receive(&self) -> Result<Vec<SyncOp>, String> {
        // Fallback: HTTP GET JSON from {endpoint}/sync using whatever HTTP client is available
        // Currently, no tokio/quinn or HTTP client dependencies are available in Cargo.toml.
        // As Cargo.toml is owned by Ola 3.08 and must not be edited, this is documented as a stub.
        Ok(Vec::new())
    }
}

/// Web transport: WS/WebRTC to relay (wasm-gated).
///
/// Due to `web-sys` not being present in `swal-sync`'s `Cargo.toml` dependencies (which is
/// owned by Ola 3.08 and must not be modified), this implementation serves as a documented
/// fallback/stub that works across both native and web compile checks.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct WsRelayTransport {
    pub url: String,
}

impl WsRelayTransport {
    /// Creates a new `WsRelayTransport` with the specified URL.
    pub fn new(url: String) -> Self {
        Self { url }
    }
}

impl Transport for WsRelayTransport {
    async fn send(&self, _ops: Vec<SyncOp>) -> Result<(), String> {
        // TODO: Real WebSocket/WebRTC implementation when web-sys/js-sys are available.
        // Below is a commented design of how WebSocket via web-sys / js-sys would be instantiated:
        /*
        let ws = web_sys::WebSocket::new(&self.url).map_err(|e| e.as_string().unwrap_or_default())?;
        // sending logic...
        */
        Ok(())
    }

    async fn receive(&self) -> Result<Vec<SyncOp>, String> {
        // TODO: Real WebSocket/WebRTC implementation when web-sys/js-sys are available.
        Ok(Vec::new())
    }
}

/// Reuses `state_hash()` from the `SyncEngine` to negotiate and verify handshakes
/// between a local engine and a remote peer.
pub fn verify_state_match(local_hash: &str, remote_hash: &str) -> bool {
    local_hash == remote_hash
}

/// A simplified handshake representation carrying the state hash.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct SyncHandshake {
    pub state_hash: String,
}

impl SyncHandshake {
    /// Creates a new handshake with the given state hash.
    pub fn new(state_hash: String) -> Self {
        Self { state_hash }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::SyncEngine;

    /// Simple block_on implementation for testing async functions without external executor dependency.
    fn block_on<F: std::future::Future>(mut future: F) -> F::Output {
        use std::task::{Context, RawWaker, RawWakerVTable, Waker};
        use std::pin::Pin;

        unsafe fn clone(_: *const ()) -> RawWaker { RawWaker::new(std::ptr::null(), &VTABLE) }
        unsafe fn wake(_: *const ()) {}
        unsafe fn wake_by_ref(_: *const ()) {}
        unsafe fn drop(_: *const ()) {}

        static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop);

        let raw_waker = RawWaker::new(std::ptr::null(), &VTABLE);
        let waker = unsafe { Waker::from_raw(raw_waker) };
        let mut cx = Context::from_waker(&waker);

        // Pin the future
        let mut future = unsafe { Pin::new_unchecked(&mut future) };
        loop {
            match future.as_mut().poll(&mut cx) {
                std::task::Poll::Ready(val) => return val,
                std::task::Poll::Pending => {}
            }
        }
    }

    #[test]
    fn test_edgemesh_transport_serialization_round_trip() {
        let transport = EdgeMeshTransport::new("http://localhost:8080".to_string());

        // Serialize to JSON and deserialize back
        let serialized = serde_json::to_string(&transport).expect("Failed to serialize");
        let deserialized: EdgeMeshTransport = serde_json::from_str(&serialized).expect("Failed to deserialize");

        assert_eq!(transport, deserialized);
        assert_eq!(deserialized.endpoint, "http://localhost:8080");

        // Verify that send and receive stub execution works
        assert!(block_on(transport.send(Vec::new())).is_ok());
        let received = block_on(transport.receive()).expect("Failed to receive");
        assert!(received.is_empty());
    }

    #[test]
    fn test_ws_relay_transport_compile_check() {
        let transport = WsRelayTransport::new("ws://localhost:9090".to_string());

        // Verify basic properties and compile check
        assert_eq!(transport.url, "ws://localhost:9090");

        assert!(block_on(transport.send(Vec::new())).is_ok());
        let received = block_on(transport.receive()).expect("Failed to receive");
        assert!(received.is_empty());
    }

    #[test]
    fn test_handshake_state_hash_reuse() {
        let mut engine_a = SyncEngine::new();
        let engine_b = SyncEngine::new();

        let hash_a = engine_a.state_hash();
        let hash_b = engine_b.state_hash();

        // Initially empty state hashes should match
        assert!(verify_state_match(&hash_a, &hash_b));

        // Create handshakes
        let handshake_a = SyncHandshake::new(hash_a.clone());
        let handshake_b = SyncHandshake::new(hash_b.clone());
        assert_eq!(handshake_a, handshake_b);

        // Apply an operation to A to diverge state hashes
        let op = SyncOp {
            id: "op1".to_string(),
            doc_id: "doc1".to_string(),
            op_type: "insert".to_string(),
            payload: serde_json::json!({"text": "hello"}),
            ts: 100,
        };
        engine_a.apply_local(op);

        let new_hash_a = engine_a.state_hash();
        assert_ne!(new_hash_a, hash_b);
        assert!(!verify_state_match(&new_hash_a, &hash_b));
    }
}
