//! Sterile Wasi host environment runner with fuel consumption and epoch deadlines.
//! Resolves G4, G5, and E5 vulnerabilities.

#[cfg(all(not(target_arch = "wasm32"), feature = "runner"))]
pub mod host {
    use std::io::{Read, Write};
    use wasmtime::*;
    use wasmtime_wasi::p1::{add_to_linker_sync, WasiP1Ctx};
    use wasmtime_wasi::WasiCtxBuilder;

    pub const MAX_REQUEST_BYTES: usize = 16 * 1024 * 1024; // 16 MiB
    pub const FRAME_OVERSIZE_EXIT_CODE: i32 = 92;

    #[derive(Debug)]
    pub enum HostError {
        FrameOversize(usize, usize),
        Io(std::io::Error),
        Wasmtime(wasmtime::Error),
        Trap(wasmtime::Trap),
    }

    impl std::fmt::Display for HostError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::FrameOversize(len, max) => {
                    write!(f, "exit 92: frame size {len} exceeds cap {max}")
                }
                Self::Io(e) => write!(f, "IO error: {e}"),
                Self::Wasmtime(e) => write!(f, "Wasmtime engine error: {e}"),
                Self::Trap(t) => write!(f, "WASM trap: {t}"),
            }
        }
    }

    impl std::error::Error for HostError {}

    /// Reads a 4-byte big-endian framed packet with strict MAX_REQUEST_BYTES bound.
    pub fn read_bounded_frame(reader: &mut impl Read) -> Result<Option<Vec<u8>>, HostError> {
        let mut len_buf = [0u8; 4];
        match reader.read_exact(&mut len_buf) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
            Err(e) => return Err(HostError::Io(e)),
        }

        let len = u32::from_be_bytes(len_buf) as usize;
        if len > MAX_REQUEST_BYTES {
            return Err(HostError::FrameOversize(len, MAX_REQUEST_BYTES));
        }

        let mut body = vec![0u8; len];
        reader.read_exact(&mut body).map_err(HostError::Io)?;
        Ok(Some(body))
    }

    /// Creates a sterile Wasmtime instance with:
    /// 1. NO host environment variables passed in (no inherit_env).
    /// 2. Piped, memory-buffered stdio.
    /// 3. Active fuel consumption and epoch metering.
    pub fn create_sterile_engine() -> Result<Engine, HostError> {
        let mut config = Config::new();
        config.wasm_exceptions(true);
        config.consume_fuel(true);
        config.epoch_interruption(true);
        config.cranelift_opt_level(OptLevel::Speed);
        Engine::new(&config).map_err(HostError::Wasmtime)
    }

    pub fn create_sterile_store(engine: &Engine, fuel: u64) -> Result<Store<WasiP1Ctx>, HostError> {
        // Sterile WASI: No inherit_env()! Host credentials stay isolated.
        let wasi_ctx = WasiCtxBuilder::new()
            .build_p1();

        let mut store = Store::new(engine, wasi_ctx);
        store.set_fuel(fuel).map_err(HostError::Wasmtime)?;
        store.set_epoch_deadline(100);
        Ok(store)
    }
}
