//! WebAssembly FFI shell for `graphlaw-wasm`: the only `unsafe` in the crate.
//
// Consumed query columns (ffi.rq): crate_name, export_prefix, abi_version,
// max_request_bytes.
// Allocation-discipline columns (ffi.rq): has_abi_version_export, max_outstanding_bytes,
// buffer_style, abi_path.
// Rendered by ggen (rust-wasi-wasmex-pack) from the wja: graph.
// Edit the ontology and re-render; never edit this file by hand.
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
// A null request pointer or an oversize length never traps: the call returns a
// typed error response built by the safe core.
//
// Contract with the hand-written safe core (`graphlaw::abi`):
//   pub fn call(request: &[u8]) -> Vec<u8>
//   pub fn limit_response(name: &str, observed: usize, max: usize) -> Vec<u8>
//   pub fn missing_buffer_response() -> Vec<u8>
// and with the generated `crate::abi_meta`: `MAX_REQUEST_BYTES: usize`, `MAX_OUTSTANDING_BYTES: usize`.
//
// Allocation discipline: every buffer crossing the boundary is a `Vec<u8>` of capacity
// `len.max(1)` (responses are shrunk to capacity == length), so `_free` / `_call`
// reconstruct the exact allocation with `Vec::from_raw_parts`.
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

/// Bytes handed to the host (`_alloc` buffers and `_call` responses) and not yet freed.
static OUTSTANDING: AtomicUsize = AtomicUsize::new(0);

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

/// Leak a vec buffer to the host as a raw pointer.
fn into_raw(mut buf: Vec<u8>) -> *mut u8 {
    let ptr = buf.as_mut_ptr();
    core::mem::forget(buf);
    ptr
}

/// Reclaim a buffer previously produced by [`into_raw`].
///
/// # Safety
/// `ptr` must come from this module and `len` must be the length that was
/// requested / returned for it; each pair may be reclaimed exactly once.
unsafe fn from_raw(ptr: *mut u8, len: u32) -> Vec<u8> {
    let n = (len as usize).max(1);
    // SAFETY: caller guarantees ptr/len describe a live allocation of capacity `n`.
    unsafe { Vec::from_raw_parts(ptr, len as usize, n) }
}

/// Reserve a buffer of capacity `len.max(1)`; null when `len` exceeds the request limit or the reservation would exceed the outstanding cap.
pub fn alloc_buf(len: u32) -> *mut u8 {
    if len as usize > MAX_REQUEST_BYTES {
        return core::ptr::null_mut();
    }
    // A host that allocates and never frees must not be able to fill linear memory.
    if !reserve((len as usize).max(1)) {
        return core::ptr::null_mut();
    }
    into_raw(Vec::with_capacity((len as usize).max(1)))
}

/// Release a buffer obtained from [`alloc_buf`] or returned by [`call_buf`].
///
/// # Safety
/// `ptr`/`len` must be exactly a pair previously handed out by this module.
pub unsafe fn free_buf(ptr: *mut u8, len: u32) {
    if !ptr.is_null() {
        release((len as usize).max(1));
        // SAFETY: forwarded caller contract.
        drop(unsafe { from_raw(ptr, len) });
    }
}

/// Run one request buffer and return the JSON response bytes. Null pointers and
/// oversize lengths yield typed error responses; the request buffer is consumed
/// only when it is a buffer this module could have handed out.
///
/// # Safety
/// `ptr`/`len` must describe a buffer from [`alloc_buf`] filled by the host.
pub unsafe fn call_buf(ptr: *mut u8, len: u32) -> Vec<u8> {
    let response = if ptr.is_null() {
        graphlaw::abi::missing_buffer_response()
    } else if len as usize > MAX_REQUEST_BYTES {
        // Not a buffer this module could have handed out; do not reconstruct it.
        graphlaw::abi::limit_response("request_bytes", len as usize, MAX_REQUEST_BYTES)
    } else {
        // SAFETY: forwarded caller contract; len <= MAX_REQUEST_BYTES.
        let request = unsafe { from_raw(ptr, len) };
        let response = graphlaw::abi::call(&request);
        drop(request);
        release((len as usize).max(1));
        response
    };
    if response.is_empty() {
        // Keep len >= 1 so the host's free reclaims the exact allocation.
        return b"{}".to_vec();
    }
    response
}

/// Hand a response to the host; returns `(out_ptr << 32) | out_len`.
fn pack_response(response: Vec<u8>) -> u64 {
    let out_len = response.len() as u64;
    // The response is now host-owned until `_free`; count it (never zero-sized here).
    OUTSTANDING.fetch_add(response.len().max(1), Ordering::Relaxed);
    let mut response = response;
    // Capacity must equal length so `_free` can reconstruct the allocation.
    response.shrink_to_fit();
    let out_ptr = into_raw(response) as usize as u64;
    (out_ptr << 32) | out_len
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
/// # Safety
/// `ptr`/`len` must be exactly a pair previously handed out by this module.
#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn gl_free(ptr: *mut u8, len: u32) {
    // SAFETY: forwarded caller contract.
    unsafe { free_buf(ptr, len) }
}

/// Execute one JSON request; returns `(out_ptr << 32) | out_len`.
///
/// # Safety
/// `ptr`/`len` must describe a buffer from `gl_alloc` filled by the host.
#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn gl_call(ptr: *mut u8, len: u32) -> u64 {
    // SAFETY: forwarded caller contract.
    pack_response(unsafe { call_buf(ptr, len) })
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
            unsafe { free_buf(p, len) };
        }
    }

    #[test]
    fn null_and_oversize_call_are_typed_not_traps() {
        let a = unsafe { call_buf(core::ptr::null_mut(), 0) };
        assert!(!a.is_empty());
        let b = unsafe {
            call_buf(
                core::ptr::dangling_mut::<u8>(),
                (MAX_REQUEST_BYTES as u32).saturating_add(1),
            )
        };
        assert!(!b.is_empty());
    }
}
