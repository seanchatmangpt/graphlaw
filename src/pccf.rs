//! PCCf server admission leaf (scaffold).
//!
//! # Law
//!
//! - Hot envelope is **736 bytes** (`64` header + `672` Bulletproof). Wire SoT:
//!   cindy `L-pccf-736-envelope.md`.
//! - Negation in admission rules is stratified **SRL `NOT { }`**, checked with
//!   [`purrdf_datalog::schedule::stratify_rules`] — never unstratified N3
//!   `log:notIncludes`.
//! - This module is a **leaf**. Do **not** add a `Step` variant in
//!   [`crate::law::Step`]. Do **not** introduce a `duckdb` cargo feature
//!   (`frozen-duckdb` is already unconditional; that is why `graphlaw.wasm`
//!   is ~63 MB). Client/edge verify lives in
//!   `/Users/sac/cgap/crates/pccf-verifier-wasm`, not in this crate's wasm ABI.
//!
//! Full `admit_pccf` body (SRL load → stratify → evaluate → marking/lease/chain
//! checks) is intentionally unfinished; the stub refuses closed.

/// Exact hot-path frame length (header + Bulletproof).
pub const PCCF_LEN: usize = 736;
/// Header bytes (marking + AID + epoch digest + γ).
pub const PCCF_HEADER_LEN: usize = 64;
/// Bulletproof body bytes.
pub const PCCF_BULLETPROOF_LEN: usize = 672;

const _: () = assert!(PCCF_HEADER_LEN + PCCF_BULLETPROOF_LEN == PCCF_LEN);

/// Parsed PCCf frame (owned copy of the 736 raw bytes).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pccf {
    /// Exact wire bytes.
    pub raw: [u8; PCCF_LEN],
}

/// Append-only refusal codes for parse / admit.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PccfRefusal {
    /// Frame length ≠ [`PCCF_LEN`].
    BadLength {
        /// Observed length.
        got: usize,
    },
    /// Scaffold: SRL stratify + admit body not yet wired.
    NotImplemented(&'static str),
}

/// Structural parse only (length fence). Magic / reserved checks land with
/// the wire SoT courts.
pub fn parse_pccf(bytes: &[u8]) -> Result<Pccf, PccfRefusal> {
    if bytes.len() != PCCF_LEN {
        return Err(PccfRefusal::BadLength { got: bytes.len() });
    }
    let mut raw = [0u8; PCCF_LEN];
    raw.copy_from_slice(bytes);
    Ok(Pccf { raw })
}

/// Full server admission (scaffold).
///
/// Intended body:
/// 1. Load PCCf SRL rules that use `NOT { }` only.
/// 2. `stratify_rules(&rules, &run_once)?` — refuse if unstratifiable.
/// 3. Evaluate schedule against envelope + carrier.
/// 4. Marking reachability, lease subset, chain, Bulletproof, Andon.
///
/// Until that lands, every call refuses closed.
pub fn admit_pccf(_p: &Pccf) -> Result<(), PccfRefusal> {
    Err(PccfRefusal::NotImplemented(
        "admit_pccf awaits stratified SRL rules via stratify_rules; Step enum untouched",
    ))
}

#[cfg(test)]
mod courts {
    use super::*;

    #[test]
    fn parse_exact_736() {
        let frame = [0u8; PCCF_LEN];
        assert!(parse_pccf(&frame).is_ok());
    }

    #[test]
    fn parse_wrong_length_refuses() {
        assert!(matches!(
            parse_pccf(&[0u8; 10]),
            Err(PccfRefusal::BadLength { got: 10 })
        ));
    }

    #[test]
    fn admit_stub_refuses_closed() {
        let p = parse_pccf(&[0u8; PCCF_LEN]).expect("parse");
        assert!(matches!(
            admit_pccf(&p),
            Err(PccfRefusal::NotImplemented(_))
        ));
    }
}
