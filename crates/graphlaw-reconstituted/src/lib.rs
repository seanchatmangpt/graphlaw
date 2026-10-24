//! Clean-room reconstituted GraphLaw kernel synthesized via ggen-legacy.
//! Operating under zero-ambient-authority law.

pub mod egress;
pub mod runner;
pub mod security;

use blake3::Hasher;
use serde::{Deserialize, Serialize};

/// Domain-separation literal for receipt_v2 (exactly 15 bytes per ADR-CANONARROW).
pub const RECEIPT_V2_PREFIX: &[u8; 15] = b"GLAW_RECEIPT_V2";

/// Dual-witness cryptographic execution receipt for sovereign query evaluation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionReceiptV2 {
    pub h_pre: [u8; 32],
    pub h_post: [u8; 32],
    pub h_query: [u8; 32],
    pub h_triples: [u8; 32],
    pub h_columnar: [u8; 32],
    pub epoch: u64,
    pub receipt_root: [u8; 32],
}

impl ExecutionReceiptV2 {
    /// Seals a v2 execution receipt into its immutable 32-byte BLAKE3 receipt root.
    pub fn seal(
        h_pre: &[u8; 32],
        h_post: &[u8; 32],
        h_query: &[u8; 32],
        h_triples: &[u8; 32],
        h_columnar: &[u8; 32],
        epoch: u64,
    ) -> Self {
        let mut hasher = Hasher::new();
        hasher.update(RECEIPT_V2_PREFIX);
        hasher.update(h_pre);
        hasher.update(h_post);
        hasher.update(h_query);
        hasher.update(h_triples);
        hasher.update(h_columnar);
        hasher.update(&epoch.to_le_bytes());

        let root = *hasher.finalize().as_bytes();
        Self {
            h_pre: *h_pre,
            h_post: *h_post,
            h_query: *h_query,
            h_triples: *h_triples,
            h_columnar: *h_columnar,
            epoch,
            receipt_root: root,
        }
    }
}

/// Computes the content-addressed query hash: H_query = BLAKE3(Canon(G_AST) || P_params)
pub fn compute_h_query(canon_ast_bytes: &[u8], params_bytes: &[u8]) -> [u8; 32] {
    let mut hasher = Hasher::new();
    hasher.update(b"GLAW_QUERY_AST_V1");
    hasher.update(canon_ast_bytes);
    hasher.update(params_bytes);
    *hasher.finalize().as_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_receipt_v2_determinism() {
        let h = [7u8; 32];
        let r1 = ExecutionReceiptV2::seal(&h, &h, &h, &h, &h, 42);
        let r2 = ExecutionReceiptV2::seal(&h, &h, &h, &h, &h, 42);
        assert_eq!(r1, r2);
        assert_eq!(r1.receipt_root, r2.receipt_root);
    }
}
