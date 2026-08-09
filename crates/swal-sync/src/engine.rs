use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use serde::{Deserialize, Serialize};

/// Represents an individual CRDT operation on a document.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SyncOp {
    pub id: String,
    pub doc_id: String,
    pub op_type: String,
    pub payload: serde_json::Value,
    pub ts: u64,
}

/// The outcome of merging a set of remote operations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct MergeResult {
    pub applied: usize,
    pub conflicts: usize,
}

/// SyncEngine is a wasm-clean (no I/O, pure-logic) CRDT state manager.
/// It maintains a history log of operations and computes Merkle-like hashes
/// for identifying divergence and enabling fast state handshakes.
#[derive(Debug, Clone)]
pub struct SyncEngine {
    pub log: Vec<SyncOp>,
    pub merkle: HashMap<String, u64>,
}

impl SyncEngine {
    /// Creates a new, empty SyncEngine.
    pub fn new() -> Self {
        Self {
            log: Vec::new(),
            merkle: HashMap::new(),
        }
    }

    /// Appends a local operation to the log, updates its Merkle hashes, and returns it.
    pub fn apply_local(&mut self, op: SyncOp) -> SyncOp {
        let op_hash = self.compute_op_hash(&op);

        self.merkle.insert(op.id.clone(), op_hash);
        let doc_hash = self.merkle.entry(op.doc_id.clone()).or_insert(0);
        *doc_hash = doc_hash.wrapping_add(op_hash);

        self.log.push(op.clone());
        op
    }

    /// Merges remote operations idempotently.
    /// If an operation ID is already present in the log:
    /// - The operation is skipped.
    /// - If the payloads differ, a conflict is counted.
    pub fn merge_remote(&mut self, ops: Vec<SyncOp>) -> MergeResult {
        let mut result = MergeResult::default();

        for remote_op in ops {
            if let Some(existing) = self.log.iter().find(|o| o.id == remote_op.id) {
                if existing.payload != remote_op.payload {
                    result.conflicts += 1;
                }
            } else {
                let op_hash = self.compute_op_hash(&remote_op);
                self.merkle.insert(remote_op.id.clone(), op_hash);
                let doc_hash = self.merkle.entry(remote_op.doc_id.clone()).or_insert(0);
                *doc_hash = doc_hash.wrapping_add(op_hash);

                self.log.push(remote_op);
                result.applied += 1;
            }
        }

        result
    }

    /// Computes a deterministic state hash of the entire operation log.
    /// Sorts operations by ID to guarantee determinism regardless of order.
    pub fn state_hash(&self) -> String {
        let mut sorted_ops = self.log.clone();
        sorted_ops.sort_by(|a, b| a.id.cmp(&b.id));

        let mut hasher = DefaultHasher::new();
        for op in sorted_ops {
            op.id.hash(&mut hasher);
            op.doc_id.hash(&mut hasher);
            op.op_type.hash(&mut hasher);
            let payload_str = serde_json::to_string(&op.payload).unwrap_or_default();
            payload_str.hash(&mut hasher);
            op.ts.hash(&mut hasher);
        }

        format!("{:016x}", hasher.finish())
    }

    /// Helper function to compute the hash of a single operation.
    fn compute_op_hash(&self, op: &SyncOp) -> u64 {
        let mut hasher = DefaultHasher::new();
        op.id.hash(&mut hasher);
        op.doc_id.hash(&mut hasher);
        op.op_type.hash(&mut hasher);
        let payload_str = serde_json::to_string(&op.payload).unwrap_or_default();
        payload_str.hash(&mut hasher);
        op.ts.hash(&mut hasher);
        hasher.finish()
    }
}

impl Default for SyncEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_idempotence() {
        let mut engine = SyncEngine::new();
        let op = SyncOp {
            id: "op1".to_string(),
            doc_id: "doc1".to_string(),
            op_type: "insert".to_string(),
            payload: serde_json::json!({"text": "hello"}),
            ts: 100,
        };
        engine.apply_local(op.clone());
        assert_eq!(engine.log.len(), 1);

        // Merging the same op again should be idempotent (0 applied)
        let result = engine.merge_remote(vec![op]);
        assert_eq!(result.applied, 0);
        assert_eq!(result.conflicts, 0);
        assert_eq!(engine.log.len(), 1);
    }

    #[test]
    fn test_convergence() {
        let mut engine_a = SyncEngine::new();
        let mut engine_b = SyncEngine::new();

        let op_a = SyncOp {
            id: "op_a".to_string(),
            doc_id: "doc1".to_string(),
            op_type: "insert".to_string(),
            payload: serde_json::json!({"x": 1}),
            ts: 100,
        };
        let op_b = SyncOp {
            id: "op_b".to_string(),
            doc_id: "doc1".to_string(),
            op_type: "insert".to_string(),
            payload: serde_json::json!({"y": 2}),
            ts: 110,
        };

        engine_a.apply_local(op_a.clone());
        engine_b.apply_local(op_b.clone());

        assert_ne!(engine_a.state_hash(), engine_b.state_hash());

        // Merge each other's ops
        let res_a = engine_a.merge_remote(engine_b.log.clone());
        let res_b = engine_b.merge_remote(engine_a.log.clone());

        assert_eq!(res_a.applied, 1);
        assert_eq!(res_b.applied, 1);
        assert_eq!(res_a.conflicts, 0);
        assert_eq!(res_b.conflicts, 0);

        // Check convergence
        assert_eq!(engine_a.state_hash(), engine_b.state_hash());
    }

    #[test]
    fn test_conflict_detection() {
        let mut engine = SyncEngine::new();
        let op_local = SyncOp {
            id: "op_conf".to_string(),
            doc_id: "doc1".to_string(),
            op_type: "insert".to_string(),
            payload: serde_json::json!({"val": "local"}),
            ts: 100,
        };
        engine.apply_local(op_local);

        let op_remote = SyncOp {
            id: "op_conf".to_string(),
            doc_id: "doc1".to_string(),
            op_type: "insert".to_string(),
            payload: serde_json::json!({"val": "remote"}),
            ts: 100,
        };

        let result = engine.merge_remote(vec![op_remote]);
        assert_eq!(result.applied, 0);
        assert_eq!(result.conflicts, 1);
    }
}
