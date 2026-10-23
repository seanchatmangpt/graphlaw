//! One OCEL causal frame + the denial lane bitfield + the rolling BLAKE3
//! receipt chain (byte-exact port of `bcinr-powl-receipt 26.7.28`).

/// A packed object reference encoding type index (high 8 bits) and object
/// id (low 24 bits) in a single `u32`.
///
/// ```
/// use graphlaw::receipt_chain::frame::PackedObjRef;
///
/// let r = PackedObjRef(0x12_00AB_C7);
/// assert_eq!(r.0 >> 24, 0x12);          // type index: high 8 bits
/// assert_eq!(r.0 & 0x00FF_FFFF, 0x00AB_C7); // object id: low 24 bits
/// ```
#[repr(transparent)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct PackedObjRef(pub u32);

/// Denial polarity bitfield: 8 single-byte lanes over a `u64` word
/// (verbatim port of `bcinr-powl-receipt 26.7.28 denial.rs`).
///
/// ```
/// use graphlaw::receipt_chain::frame::DenialPolarity;
///
/// // ADMITTED is the identity of compose.
/// let p = DenialPolarity::PRECONDITION_FAILED;
/// assert_eq!(p.compose(DenialPolarity::ADMITTED), p);
///
/// // Lanes OR: lane 1 (byte 1) + lane 2 (byte 2).
/// let both = DenialPolarity::PRECONDITION_FAILED
///     .compose(DenialPolarity::SLA_BREACH);
/// assert_eq!(both.0, 0x0000_0000_00FF_FF00);
/// ```
#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct DenialPolarity(pub u64);

impl DenialPolarity {
    /// No denial — the step is admitted.
    pub const ADMITTED: Self = Self(0);

    /// Watchdog timer drained before the step completed (lane 7).
    pub const WATCHDOG_DRAINED: Self = Self(0xFF00_0000_0000_0000);

    /// A declared precondition was not satisfied (lane 1).
    pub const PRECONDITION_FAILED: Self = Self(0x0000_0000_0000_FF00);

    /// Service-level agreement deadline exceeded (lane 2).
    pub const SLA_BREACH: Self = Self(0x0000_0000_00FF_0000);

    /// Authorization proof absent or expired (lane 3).
    pub const AUTHORIZATION_DENIED: Self = Self(0x0000_0000_FF00_0000);

    /// Capacity or quota exhausted (lane 4).
    pub const RESOURCE_EXHAUSTED: Self = Self(0x0000_00FF_0000_0000);

    /// Object lifecycle law violated (lane 5).
    pub const OBJECT_LIFECYCLE_VIOLATION: Self = Self(0x0000_FF00_0000_0000);

    /// A process-conformance gate rejected the step (lane 6).
    pub const CONFORMANCE_GATE_FAILED: Self = Self(0x00FF_0000_0000_0000);

    /// Returns `true` when no denial lane is set.
    ///
    /// ```
    /// use graphlaw::receipt_chain::frame::DenialPolarity;
    /// assert!(DenialPolarity::ADMITTED.is_admitted());
    /// assert!(!DenialPolarity::SLA_BREACH.is_admitted());
    /// ```
    #[inline]
    #[must_use]
    pub fn is_admitted(self) -> bool {
        self.0 == 0
    }

    /// Compose two words by OR-ing their lanes. Admitted (zero) is the
    /// identity element: `compose(x, ADMITTED) == x`.
    ///
    /// ```
    /// use graphlaw::receipt_chain::frame::DenialPolarity;
    /// let x = DenialPolarity::AUTHORIZATION_DENIED;
    /// assert_eq!(x.compose(DenialPolarity::ADMITTED), x);
    /// ```
    #[inline]
    #[must_use]
    pub fn compose(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Branchless lane-to-bit scatter: each active byte lane in the word
    /// produces a distinct bit in the returned `u64`
    /// (verbatim port of `DenialPolarity::to_fired_mask`).
    ///
    /// ```
    /// use graphlaw::receipt_chain::frame::DenialPolarity;
    /// // Admitted scatters to zero.
    /// assert_eq!(DenialPolarity::ADMITTED.to_fired_mask(), 0);
    /// // Lane 1 (PRECONDITION_FAILED) scatters to bit 1.
    /// assert_eq!(DenialPolarity::PRECONDITION_FAILED.to_fired_mask(), 0b10);
    /// // Two lanes OR into two distinct bits.
    /// let both = DenialPolarity::PRECONDITION_FAILED
    ///     .compose(DenialPolarity::SLA_BREACH);
    /// assert_eq!(both.to_fired_mask(), 0b110);
    /// ```
    #[inline]
    #[must_use]
    pub fn to_fired_mask(self) -> u64 {
        let w = self.0;
        let lane = |shift: u32| -> u64 {
            let byte = (w >> shift) & 0xFF;
            (byte | byte.wrapping_neg()) >> 63
        };

        lane(0)       // bit 0
        | (lane(8)  << 1)  // bit 1
        | (lane(16) << 2)  // bit 2
        | (lane(24) << 3)  // bit 3
        | (lane(32) << 4)  // bit 4
        | (lane(40) << 5)  // bit 5
        | (lane(48) << 6)  // bit 6
        | (lane(56) << 7) // bit 7
    }
}

/// One OCEL causal frame: a cache-line-sized record of a single
/// manufacturing step, its denial verdict, its object set, and the rolling
/// hash of its causal predecessor. Size: 128 bytes, aligned to 64.
#[derive(Clone)]
#[repr(C, align(64))]
pub struct OcelCausalFrame {
    /// Monotonically increasing step identity within a run.
    pub instruction_id: u64,
    /// Scatter of active denial lanes (from
    /// [`DenialPolarity::to_fired_mask`]).
    pub fired_mask: u64,
    /// Denial polarity at the time this step was manufactured.
    pub denial: DenialPolarity,
    /// Up to 8 packed object references participating in this step.
    pub obj_refs: [PackedObjRef; 8],
    /// Wall-clock timestamp in nanoseconds.
    pub ts_ns: u64,
    /// Index into the activity table for this step's activity.
    pub activity_idx: u16,
    /// Classifier byte for the POWL node kind (XOR, SEQ, LOOP, etc.).
    pub node_kind: u8,
    /// Internal padding to maintain 128-byte alignment.
    pub pad: [u8; 5],
    /// BLAKE3 hash of the preceding frame (or genesis zeros for the first
    /// frame).
    pub prior_hash: [u8; 32],
}

const _: () = assert!(core::mem::size_of::<OcelCausalFrame>() == 128);

impl OcelCausalFrame {
    /// Serialise this frame into a fixed-size byte buffer for hashing.
    /// Layout (all integers little-endian):
    /// `[0..8] instruction_id | [8..16] fired_mask | [16..24] denial.0 |
    /// [24..56] obj_refs (8 x u32 LE) | [56..64] ts_ns | [64..66]
    /// activity_idx | [66..67] node_kind | [67..99] prior_hash (verbatim)`.
    /// Total: 99 bytes.
    ///
    /// Worked example (real bytes, little-endian): a frame with
    /// `instruction_id = 1`, everything else zero except
    /// `obj_refs = [PackedObjRef(1); 8]`, `activity_idx = 3`,
    /// `node_kind = 9`, `prior_hash = zeros`:
    ///
    /// ```
    /// use graphlaw::receipt_chain::frame::{OcelCausalFrame, PackedObjRef};
    ///
    /// let f = OcelCausalFrame {
    ///     instruction_id: 1,
    ///     fired_mask: 0,
    ///     denial: graphlaw::receipt_chain::frame::DenialPolarity::ADMITTED,
    ///     obj_refs: [PackedObjRef(1); 8],
    ///     ts_ns: 0,
    ///     activity_idx: 3,
    ///     node_kind: 9,
    ///     pad: [0; 5],
    ///     prior_hash: [0u8; 32],
    /// };
    /// let b = f.to_hash_bytes();
    /// assert_eq!(b.len(), 99);
    /// assert_eq!(&b[0..8], &[1, 0, 0, 0, 0, 0, 0, 0]);   // instruction_id LE
    /// assert_eq!(&b[8..16], &[0u8; 8]);                  // fired_mask
    /// assert_eq!(&b[16..24], &[0u8; 8]);                 // denial.0
    /// assert_eq!(&b[24..28], &[1, 0, 0, 0]);             // obj_refs[0] u32 LE
    /// assert_eq!(&b[56..64], &[0u8; 8]);                 // ts_ns
    /// assert_eq!(&b[64..67], &[3, 0, 9]);                // activity_idx LE + node_kind
    /// assert!(b[67..99].iter().all(|&x| x == 0));        // prior_hash (verbatim)
    /// ```
    #[must_use]
    pub fn to_hash_bytes(&self) -> [u8; 99] {
        let mut buf = [0u8; 99];
        let mut pos = 0;

        for i in 0..8 {
            buf[pos + i] = ((self.instruction_id >> (i * 8)) & 0xFF) as u8;
        }
        pos += 8;

        for i in 0..8 {
            buf[pos + i] = ((self.fired_mask >> (i * 8)) & 0xFF) as u8;
        }
        pos += 8;

        for i in 0..8 {
            buf[pos + i] = ((self.denial.0 >> (i * 8)) & 0xFF) as u8;
        }
        pos += 8;

        for r in &self.obj_refs {
            let v = r.0;
            for i in 0..4 {
                buf[pos + i] = ((v >> (i * 8)) & 0xFF) as u8;
            }
            pos += 4;
        }

        for i in 0..8 {
            buf[pos + i] = ((self.ts_ns >> (i * 8)) & 0xFF) as u8;
        }
        pos += 8;

        buf[pos] = (self.activity_idx & 0xFF) as u8;
        buf[pos + 1] = ((self.activity_idx >> 8) & 0xFF) as u8;
        pos += 2;

        buf[pos] = self.node_kind;
        pos += 1;

        buf[pos..pos + 32].copy_from_slice(&self.prior_hash);

        buf
    }
}

/// Rolling BLAKE3 receipt for an ordered sequence of [`OcelCausalFrame`]s:
/// `chain_hash(t+1) = BLAKE3(chain_hash(t) || frame_bytes(t+1))`.
/// The genesis hash is BLAKE3 of 32 zero bytes.
///
/// Worked example of the chain rule, with real hash values produced by
/// this exact code path:
///
/// ```
/// use graphlaw::receipt_chain::frame::{DenialPolarity, OcelCausalFrame,
///     OcelCausalReceipt, PackedObjRef};
///
/// let frame = |id: u64, prior: [u8; 32]| OcelCausalFrame {
///     instruction_id: id,
///     fired_mask: 0,
///     denial: DenialPolarity::ADMITTED,
///     obj_refs: [PackedObjRef(1); 8],
///     ts_ns: 0,
///     activity_idx: 3,
///     node_kind: 9,
///     pad: [0; 5],
///     prior_hash: prior,
/// };
///
/// let mut r = OcelCausalReceipt::genesis([7u8; 32]);
/// // Genesis hash: BLAKE3 of 32 zero bytes.
/// assert_eq!(
///     hex(&r.chain_hash),
///     "2ada83c1819a5372dae1238fc1ded123c8104fdaa15862aaee69428a1820fcda"
/// );
///
/// let f1 = frame(1, r.chain_hash);
/// r.chain(&f1);
/// assert_eq!(r.frame_count, 1);
///
/// let f2 = frame(2, r.chain_hash);
/// r.chain(&f2);
/// assert_eq!(r.frame_count, 2);
/// assert_eq!(r.replay_ptr, 1);
///
/// // The two-link chain recomputes byte-exactly via chain_from_frame:
/// let h1 = graphlaw::receipt_chain::chain_from_frame(&[0u8; 32], &frame(1, [0u8; 32]));
/// assert_eq!(
///     hex(&h1),
///     "15abdafad661fb353049079245043fcaa0d42161609a5b5c3ef21788cc3de01e"
/// );
/// let h2 = graphlaw::receipt_chain::chain_from_frame(&h1, &frame(2, h1));
/// assert_eq!(
///     hex(&h2),
///     "a93bb8663dede31b5edfe9c71bef9c82f60199d194efba391edc13868e719ad1"
/// );
///
/// fn hex(b: &[u8]) -> String { b.iter().map(|x| format!("{:02x}", x)).collect() }
/// ```
pub struct OcelCausalReceipt {
    /// Current rolling hash (advances with each chain() call).
    pub chain_hash: [u8; 32],
    /// Number of frames chained so far.
    pub frame_count: u64,
    /// Opaque run identifier supplied at genesis.
    pub run_id: [u8; 32],
    /// Replay pointer: index of the last frame that can serve as a replay
    /// root.
    pub replay_ptr: u64,
}

impl OcelCausalReceipt {
    /// Create a genesis receipt for the given `run_id`. The initial
    /// `chain_hash` is BLAKE3 of 32 zero bytes.
    ///
    /// ```
    /// use graphlaw::receipt_chain::frame::OcelCausalReceipt;
    /// let g = OcelCausalReceipt::genesis([0u8; 32]);
    /// assert_eq!(g.frame_count, 0);
    /// assert_eq!(g.replay_ptr, 0);
    /// // BLAKE3 of 32 zero bytes (independent of run_id):
    /// assert_eq!(
    ///     g.chain_hash,
    ///     hex("2ada83c1819a5372dae1238fc1ded123c8104fdaa15862aaee69428a1820fcda")
    /// );
    ///
    /// fn hex(s: &str) -> [u8; 32] {
    ///     let mut out = [0u8; 32];
    ///     for (i, b) in out.iter_mut().enumerate() {
    ///         *b = u8::from_str_radix(&s[2 * i..2 * i + 2], 16).unwrap();
    ///     }
    ///     out
    /// }
    /// ```
    #[must_use]
    pub fn genesis(run_id: [u8; 32]) -> Self {
        let chain_hash: [u8; 32] = *blake3::hash(&[0u8; 32]).as_bytes();
        Self {
            chain_hash,
            frame_count: 0,
            run_id,
            replay_ptr: 0,
        }
    }

    /// Advance the chain by one frame:
    /// `BLAKE3(chain_hash || frame.to_hash_bytes())`.
    ///
    /// ```
    /// use graphlaw::receipt_chain::frame::{DenialPolarity, OcelCausalFrame,
    ///     OcelCausalReceipt, PackedObjRef};
    ///
    /// let mut r = OcelCausalReceipt::genesis([0u8; 32]);
    /// let f = OcelCausalFrame {
    ///     instruction_id: 1,
    ///     fired_mask: 0,
    ///     denial: DenialPolarity::ADMITTED,
    ///     obj_refs: [PackedObjRef(1); 8],
    ///     ts_ns: 0,
    ///     activity_idx: 3,
    ///     node_kind: 9,
    ///     pad: [0; 5],
    ///     prior_hash: r.chain_hash,
    /// };
    /// r.chain(&f);
    /// assert_eq!(r.frame_count, 1);
    /// assert_eq!(r.replay_ptr, 0);
    /// // chain_hash advances off the genesis value:
    /// assert_ne!(hex(&r.chain_hash),
    ///     "2ada83c1819a5372dae1238fc1ded123c8104fdaa15862aaee69428a1820fcda");
    ///
    /// fn hex(b: &[u8]) -> String { b.iter().map(|x| format!("{:02x}", x)).collect() }
    /// ```
    pub fn chain(&mut self, frame: &OcelCausalFrame) {
        let frame_bytes = frame.to_hash_bytes();
        let mut h = blake3::Hasher::new();
        h.update(&self.chain_hash);
        h.update(&frame_bytes);
        self.chain_hash = *h.finalize().as_bytes();
        self.frame_count += 1;
        self.replay_ptr = self.frame_count - 1;
    }
}
