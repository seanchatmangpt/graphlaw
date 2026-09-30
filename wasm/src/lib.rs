//! WebAssembly entry points. Deliberately tiny: all behaviour lives in the
//! safe, native-testable `graphlaw::abi`. The only `unsafe` in the
//! GraphLaw workspace is the pointer handling required to exchange buffers
//! with the host through linear memory.
//!
//! Protocol (all integers are wasm `i32`/`i64`):
//! 1. `gl_alloc(len) -> ptr` — host reserves `len` bytes and writes a UTF-8
//!    JSON request there.
//! 2. `gl_call(ptr, len) -> packed` — runs the request and consumes (frees)
//!    the request buffer. `packed = (out_ptr << 32) | out_len`.
//! 3. host reads `out_len` bytes at `out_ptr` (UTF-8 JSON), then
//!    `gl_free(out_ptr, out_len)`.
//!
//! A response is always JSON: `{"ok":true,...}` or `{"ok":false,"error":{...}}`.
#![allow(unsafe_code)]
#![deny(missing_docs)]

use std::sync::atomic::{AtomicUsize, Ordering};

/// Bytes handed to the host (`gl_alloc` buffers and `gl_call` responses) and not yet freed.
static OUTSTANDING: AtomicUsize = AtomicUsize::new(0);
/// Ceiling on `OUTSTANDING`; `gl_alloc` returns null beyond it.
const MAX_OUTSTANDING_BYTES: usize = 256 * 1024 * 1024;

fn release(bytes: usize) {
    let _ = OUTSTANDING.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| {
        Some(n.saturating_sub(bytes))
    });
}

/// Reserve `len` bytes of linear memory for the host. Returns null (0) when
/// `len` exceeds `graphlaw::abi::MAX_REQUEST_BYTES`; a following `gl_call` on
/// that null buffer yields a typed error response, never a trap.
#[unsafe(no_mangle)]
pub extern "C" fn gl_alloc(len: u32) -> *mut u8 {
    if len as usize > graphlaw::abi::MAX_REQUEST_BYTES {
        return std::ptr::null_mut();
    }
    // A host that allocates and never frees must not be able to fill linear memory.
    let held = len.max(1) as usize;
    let reserved = OUTSTANDING.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| {
        (n + held <= MAX_OUTSTANDING_BYTES).then_some(n + held)
    });
    if reserved.is_err() {
        return std::ptr::null_mut();
    }
    let mut buf = Vec::<u8>::with_capacity(len.max(1) as usize);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr
}

/// Release a buffer obtained from `gl_alloc` or returned by `gl_call`.
///
/// # Safety
/// `ptr` and `len` must be exactly a pair previously handed out by this module.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn gl_free(ptr: *mut u8, len: u32) {
    if !ptr.is_null() {
        release(len.max(1) as usize);
        drop(unsafe { Vec::from_raw_parts(ptr, len as usize, len.max(1) as usize) });
    }
}

/// Execute one JSON request; returns `(out_ptr << 32) | out_len`.
///
/// # Safety
/// `ptr`/`len` must describe a buffer from `gl_alloc` filled by the host.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn gl_call(ptr: *mut u8, len: u32) -> u64 {
    let response = if ptr.is_null() {
        graphlaw::abi::missing_buffer_response()
    } else if len as usize > graphlaw::abi::MAX_REQUEST_BYTES {
        // Not a buffer this module could have handed out; do not reconstruct it.
        graphlaw::abi::limit_response(
            "request_bytes",
            len as usize,
            graphlaw::abi::MAX_REQUEST_BYTES,
        )
    } else {
        let request = unsafe { Vec::from_raw_parts(ptr, len as usize, len.max(1) as usize) };
        let response = graphlaw::abi::call(&request);
        drop(request);
        release(len.max(1) as usize);
        response
    };
    let out_len = response.len() as u64;
    OUTSTANDING.fetch_add(response.len().max(1), Ordering::Relaxed);
    let mut response = response.into_boxed_slice().into_vec();
    // Capacity must equal length so gl_free can reconstruct the allocation.
    response.shrink_to_fit();
    let out_ptr = response.as_mut_ptr() as u64;
    std::mem::forget(response);
    (out_ptr << 32) | out_len
}
