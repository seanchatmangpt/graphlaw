//! WebAssembly FFI shell for `graphlaw-wasm`: the only `unsafe` in the crate.
//
// Consumed query columns (ffi.rq): crate_name, export_prefix, abi_version,
// max_request_bytes.
// Allocation-discipline columns (ffi.rq): has_abi_version_export, max_outstanding_bytes,
// buffer_style, abi_path.
// Rendered by ggen (rust-wasi-wasmex-pack) from the wja: graph.
// Edit the ontology and re-render; never edit this file by hand.
// The ownership table below is the soundness contract for `buffer_style = vec`.
// A re-render must keep it (see the pack template): reconstructing a `Vec`
// with `capacity = len.max(1)` is undefined behavior when the allocator's
// capacity is larger, and freeing an unread `gl_alloc` buffer with
// `length = len` claims uninitialized bytes.
//
// Protocol (ABI version 1, request limit 16777216 bytes; all
// integers are wasm `i32`/`i64`):
// 1. `gl_alloc(len) -> ptr`: host reserves `len` bytes and writes a UTF-8
//    JSON request there. Returns null when `len` exceeds the request limit.
// 2. `gl_call(ptr, len) -> packed`: runs the request and consumes (frees)
//    the request buffer. `packed = (out_ptr << 32) | out_len`.
// 3. host reads `out_len` bytes at `out_ptr` (UTF-8 JSON), then
//    `gl_free(out_ptr, out_len)`.
//
// A null request pointer, an unknown pointer, a length that is not the one
// this module handed out, or an oversize length never traps: the call returns
// a typed error response built by the safe core. A second free is a no-op.
//
// Contract with the hand-written safe core (`graphlaw::abi`):
//   pub fn call(request: &[u8]) -> Vec<u8>
//   pub fn limit_response(name: &str, observed: usize, max: usize) -> Vec<u8>
//   pub fn missing_buffer_response() -> Vec<u8>
//   pub fn unknown_buffer_response() -> Vec<u8>
// and with the generated `crate::abi_meta`: `MAX_REQUEST_BYTES: usize`, `MAX_OUTSTANDING_BYTES: usize`.
//
// Allocation discipline: every buffer is a zero-filled `Vec<u8>` whose real
// `capacity()` is recorded in `LIVE` at the moment it is leaked to the host.
// Reclaim looks the pointer up, checks the host length against the recorded
// declared length, and rebuilds with that capacity. Unknown pointers and
// mismatched lengths are not rebuilt.
// Outstanding-byte cap: bytes handed to the host (`_alloc` buffers and `_call` responses)
// and not yet freed are counted; `_alloc` returns null once a reservation would exceed
// `MAX_OUTSTANDING_BYTES`, so a host that never frees cannot fill linear memory.
#![allow(unsafe_code)]
#![deny(missing_docs)]
// Native builds only exercise the helpers through tests; the exports are wasm32-only.
#![cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]

use crate::abi_meta::MAX_OUTSTANDING_BYTES;
use crate::abi_meta::MAX_REQUEST_BYTES;
use core::sync::atomic::{AtomicUsize, Ordering};
use std::collections::BTreeMap;
use std::sync::Mutex;

/// Bytes handed to the host (`_alloc` buffers and `_call` responses) and not yet freed.
static OUTSTANDING: AtomicUsize = AtomicUsize::new(0);

/// Live host-owned buffers. The key is the leaked `Vec` pointer.
/// Absent on purpose from the ABI: the host only round-trips `(ptr, len)`.
static LIVE: Mutex<BTreeMap<usize, LiveAlloc>> = Mutex::new(BTreeMap::new());

/// One buffer this module handed the host.
struct LiveAlloc {
    /// `Vec::capacity()` at [`publish`]. This is the layout `from_raw_parts` must use.
    capacity: usize,
    /// Initialized prefix. Allocations are zero-filled, so this is at least 1.
    length: usize,
    /// Length the host must pass back (`gl_alloc`'s argument, or the response length).
    declared: u32,
    /// Bytes reserved against [`MAX_OUTSTANDING_BYTES`].
    budget: usize,
}

fn live() -> std::sync::MutexGuard<'static, BTreeMap<usize, LiveAlloc>> {
    LIVE.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Reserve `bytes` against the outstanding cap; false when the cap would be exceeded.
fn reserve(bytes: usize) -> bool {
    OUTSTANDING
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| {
            n.checked_add(bytes)
                .filter(|total| *total <= MAX_OUTSTANDING_BYTES)
        })
        .is_ok()
}

/// Return `bytes` to the outstanding budget (saturating at zero).
fn release(bytes: usize) {
    let _ = OUTSTANDING.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| {
        Some(n.saturating_sub(bytes))
    });
}

/// Record `buf` and leak it to the host. `buf` must be non-empty and initialized.
fn publish(mut buf: Vec<u8>, declared: u32, budget: usize) -> *mut u8 {
    let length = buf.len();
    let capacity = buf.capacity();
    debug_assert!(length > 0 && capacity >= length);
    let ptr = buf.as_mut_ptr();
    core::mem::forget(buf);
    live().insert(
        ptr as usize,
        LiveAlloc {
            capacity,
            length,
            declared,
            budget,
        },
    );
    ptr
}

/// Why a host `(ptr, len)` was not rebuilt.
enum Reject {
    /// Null pointer. The caller answers with [`graphlaw::abi::missing_buffer_response`].
    Missing,
    /// Not a pointer this module currently owns (forged, or already reclaimed).
    Unknown,
    /// The pointer is live, but `len` is not the declared length. Left in the table.
    Len,
}

struct Taken {
    buf: Vec<u8>,
    budget: usize,
}

/// Take a live buffer back. A mismatch leaves the allocation in the table.
fn reclaim(ptr: *mut u8, len: u32) -> Result<Taken, Reject> {
    if ptr.is_null() {
        return Err(Reject::Missing);
    }
    let mut map = live();
    let key = ptr as usize;
    let Some(slot) = map.get(&key) else {
        return Err(Reject::Unknown);
    };
    if slot.declared != len {
        return Err(Reject::Len);
    }
    let slot = map.remove(&key).expect("key was just present");
    // SAFETY: `ptr`, `length`, and `capacity` were stored by `publish` from this
    // `Vec`. `length` bytes were initialized before the leak (zero-filled alloc
    // buffers, or a response `Vec` built in safe code). The map entry is gone,
    // so a second reclaim cannot rebuild the same allocation.
    let buf = unsafe { Vec::from_raw_parts(ptr, slot.length, slot.capacity) };
    Ok(Taken {
        buf,
        budget: slot.budget,
    })
}

/// Reserve a zero-filled buffer of length `len.max(1)`.
/// Null when `len` exceeds the request limit or the reservation would exceed the outstanding cap.
pub fn alloc_buf(len: u32) -> *mut u8 {
    if len as usize > MAX_REQUEST_BYTES {
        return core::ptr::null_mut();
    }
    let n = (len as usize).max(1);
    if !reserve(n) {
        return core::ptr::null_mut();
    }
    // Zero-fill so the bytes are initialized even if the host never writes.
    // `capacity()` may exceed `n`; `publish` records the real value.
    publish(vec![0u8; n], len, n)
}

/// Release a buffer obtained from [`alloc_buf`] or returned by [`call_buf`].
///
/// Unknown pointers and mismatched lengths do not rebuild a `Vec`.
///
/// # Safety
/// Safe against a hostile `(ptr, len)`. The `unsafe` marker matches the
/// export: a correct host passes a pair this module handed out, once.
pub unsafe fn free_buf(ptr: *mut u8, len: u32) {
    if let Ok(taken) = reclaim(ptr, len) {
        release(taken.budget);
        drop(taken.buf);
    }
}

/// Run one request buffer and return the JSON response bytes.
///
/// Null, unknown, mismatched, and oversize inputs yield typed error responses.
/// The request buffer is consumed only when it is a live allocation whose
/// declared length is `len`.
///
/// # Safety
/// Safe against a hostile `(ptr, len)`. See [`free_buf`].
pub unsafe fn call_buf(ptr: *mut u8, len: u32) -> Vec<u8> {
    let response = if ptr.is_null() {
        graphlaw::abi::missing_buffer_response()
    } else if len as usize > MAX_REQUEST_BYTES {
        // Not a buffer this module could have handed out; do not reconstruct it.
        graphlaw::abi::limit_response("request_bytes", len as usize, MAX_REQUEST_BYTES)
    } else {
        match reclaim(ptr, len) {
            Ok(taken) => {
                let usable = (len as usize).min(taken.buf.len());
                let response = graphlaw::abi::call(&taken.buf[..usable]);
                drop(taken.buf);
                release(taken.budget);
                response
            }
            Err(Reject::Missing) => graphlaw::abi::missing_buffer_response(),
            Err(Reject::Unknown | Reject::Len) => graphlaw::abi::unknown_buffer_response(),
        }
    };
    if response.is_empty() {
        b"{}".to_vec()
    } else {
        response
    }
}

/// Hand a response to the host; returns `(out_ptr << 32) | out_len`.
fn pack_response(response: Vec<u8>) -> u64 {
    let response = if response.is_empty() {
        b"{}".to_vec()
    } else {
        response
    };
    let budget = response.len().max(1);
    if reserve(budget) {
        return publish_response(response, budget);
    }
    // The response itself would exceed the outstanding cap. Hand back a small
    // typed limit error, counted when it fits.
    let err = graphlaw::abi::limit_response(
        "outstanding_bytes",
        budget,
        MAX_OUTSTANDING_BYTES,
    );
    let err_budget = err.len().max(1);
    if reserve(err_budget) {
        return publish_response(err, err_budget);
    }
    // The cap is already saturated. Track the error buffer with a zero budget
    // so `gl_free` is still defined, without wrapping the counter.
    publish_response(err, 0)
}

fn publish_response(response: Vec<u8>, budget: usize) -> u64 {
    let declared = u32::try_from(response.len()).unwrap_or(u32::MAX);
    let ptr = publish(response, declared, budget);
    ((ptr as usize as u64) << 32) | u64::from(declared)
}

/// Reserve `len` bytes of linear memory for the host. Returns null (0) when
/// `len` exceeds the request limit or the outstanding-byte cap; a following `gl_call` on that null
/// buffer yields a typed error response, never a trap.
#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn gl_alloc(len: u32) -> *mut u8 {
    alloc_buf(len)
}

/// Release a buffer obtained from `gl_alloc` or returned by `gl_call`.
///
/// A pointer this module does not own, or a length that is not the declared
/// one, is ignored. That includes a second free.
///
/// # Safety
/// Safe against a hostile `(ptr, len)`. A correct host passes a pair this
/// module handed out, once.
#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn gl_free(ptr: *mut u8, len: u32) {
    unsafe { free_buf(ptr, len) }
}

/// Execute one JSON request; returns `(out_ptr << 32) | out_len`.
///
/// # Safety
/// Safe against a hostile `(ptr, len)`. A correct host passes a buffer from
/// `gl_alloc` filled with the request, once. `gl_call` consumes that buffer.
#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn gl_call(ptr: *mut u8, len: u32) -> u64 {
    pack_response(unsafe { call_buf(ptr, len) })
}

#[cfg(test)]
fn is_live(ptr: *mut u8) -> bool {
    live().contains_key(&(ptr as usize))
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;

    #[test]
    fn alloc_over_limit_is_null() {
        assert!(alloc_buf((MAX_REQUEST_BYTES as u32).saturating_add(1)).is_null());
    }

    #[test]
    fn alloc_free_roundtrip_including_zero_len() {
        for len in [0u32, 1, 17] {
            let p = alloc_buf(len);
            assert!(!p.is_null());
            assert!(is_live(p));
            unsafe { free_buf(p, len) };
            assert!(!is_live(p));
        }
    }

    #[test]
    fn null_and_oversize_call_are_typed_not_traps() {
        let a = unsafe { call_buf(core::ptr::null_mut(), 0) };
        assert_eq!(a, graphlaw::abi::missing_buffer_response());
        let oversized_len = (MAX_REQUEST_BYTES as u32).saturating_add(1);
        let b = unsafe { call_buf(core::ptr::dangling_mut::<u8>(), oversized_len) };
        assert_eq!(
            b,
            graphlaw::abi::limit_response(
                "request_bytes",
                oversized_len as usize,
                MAX_REQUEST_BYTES
            )
        );
    }

    #[test]
    fn unknown_pointer_and_double_free_are_not_rebuilt() {
        let dangling = core::ptr::dangling_mut::<u8>();
        let unknown = unsafe { call_buf(dangling, 4) };
        assert_eq!(unknown, graphlaw::abi::unknown_buffer_response());
        unsafe { free_buf(dangling, 4) };

        let p = alloc_buf(8);
        assert!(is_live(p));
        // Wrong length must not drop the allocation.
        unsafe { free_buf(p, 7) };
        assert!(is_live(p), "mismatched len left the buffer live");
        unsafe { free_buf(p, 8) };
        assert!(!is_live(p));
        // Second free is a no-op, not a double-free.
        unsafe { free_buf(p, 8) };
        assert!(!is_live(p));
    }

    #[test]
    fn call_consumes_the_request_once() {
        let p = alloc_buf(2);
        unsafe {
            p.write(b'{');
            p.add(1).write(b'}');
            let response = call_buf(p, 2);
            assert!(!is_live(p), "gl_call consumes the request");
            free_buf(p, 2);
            assert!(!is_live(p));
            let text = String::from_utf8(response).expect("json");
            assert!(text.contains("\"ok\""), "{text}");
        }
    }
}
