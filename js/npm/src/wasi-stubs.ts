// wasi-stubs.ts — shared wasi_snapshot_preview1 import object for the
// graphlaw analytical kernel module.
//
// The DuckDB-linked artifact (the coming re-pin) imports 28 preview1
// functions. This kernel is a sandboxed, filesystem-free analytical engine:
// fd-backed calls FAIL CLOSED with honest wasi errno values rather than
// faking success on reads of fds that do not exist. Only the surfaces the
// kernel genuinely uses (entropy, clock, environment, panic-time stdout,
// yield) return success.
//
// errno values per wasi_preview1 (wasi/api.h):
//   ERRNO_SUCCESS   = 0
//   ERRNO_BADF      = 8    (EBADF — bad file descriptor)
//   ERRNO_IO        = 31   (EIO)
//   ERRNO_EXIST     = 21   (EEXIST)
//   ERRNO_ISDIR     = 33   (EISDIR)
//   ERRNO_NOENT     = 46   (ENOENT — no such file or directory)
//   ERRNO_NOTDIR    = 56   (ENOTDIR)
//   ERRNO_NOTEMPTY  = 57   (ENOTEMPTY)
//   ERRNO_NOTSUP    = 60   (ENOTSUP)
//   ERRNO_NOSYS     = 54   (ENOSYS — operation not supported)

const ERRNO_SUCCESS: number = 0;
const ERRNO_BADF: number = 8;
const ERRNO_EXIST: number = 21;
const ERRNO_ISDIR: number = 33;
const ERRNO_NOENT: number = 46;
const ERRNO_NOSYS: number = 54;
const ERRNO_NOTDIR: number = 56;
const ERRNO_NOTEMPTY: number = 57;
const ERRNO_NOTSUP: number = 60;

/**
 * Builds the full `wasi_snapshot_preview1` import object (28 functions).
 *
 * @param mem accessor for the instance memory — imports run only after
 *   instantiation, when the export is available.
 * @param randomFill fills a buffer with real entropy (node:crypto
 *   randomFillSync on Node, crypto.getRandomValues on the web).
 * @param onProcExit handler for proc_exit (throw-path in both runtimes).
 */
export function buildWasiImports(
  mem: () => WebAssembly.Memory,
  randomFill: (buf: Uint8Array<ArrayBuffer>) => void,
  onProcExit: (code: number) => never,
): Record<string, unknown> {
  return {
    // --- genuinely served surfaces (success) ------------------------------

    // Entropy: real randomness, never zeros.
    random_get: (bufPtr: number, bufLen: number): number => {
      randomFill(new Uint8Array(mem().buffer, bufPtr, bufLen));
      return ERRNO_SUCCESS;
    },

    // Empty environment: sizes_get writes (0, 0); environ_get is a no-op
    // (the kernel never reads a non-empty env, and there is nothing to copy).
    environ_sizes_get: (envCountPtr: number, envBufSizePtr: number): number => {
      const view = new DataView(mem().buffer);
      view.setUint32(envCountPtr, 0, true);
      view.setUint32(envBufSizePtr, 0, true);
      return ERRNO_SUCCESS;
    },
    environ_get: (): number => ERRNO_SUCCESS,

    // Cooperative yield: nothing to schedule in a single-threaded host.
    sched_yield: (): number => ERRNO_SUCCESS,

    // Monotonic nanoseconds; performance.now() is monotonic per spec. The
    // kernel only requests clock_monotonic; all ids get the same clock,
    // which is honest for a deterministic analytical kernel.
    clock_time_get: (clockId: number, precision: bigint, timePtr: number): number => {
      void clockId;
      void precision;
      const view = new DataView(mem().buffer);
      const nanos = BigInt(Math.round(performance.now() * 1e6));
      view.setBigUint64(timePtr, nanos, true);
      return ERRNO_SUCCESS;
    },

    // Panic-path stdout/stderr: accept the iovec list, count the bytes, and
    // discard the payload (the module surfaces panics through thrown
    // GraphlawError, not console text). fds other than 1/2 are EBADF —
    // the kernel has no open files.
    fd_write: (fd: number, iovecsPtr: number, iovecsLen: number, nwrittenPtr: number): number => {
      if (fd !== 1 && fd !== 2) return ERRNO_BADF;
      const view = new DataView(mem().buffer);
      let written = 0;
      for (let i = 0; i < iovecsLen; i++) {
        written += view.getUint32(iovecsPtr + i * 8 + 4, true);
      }
      view.setUint32(nwrittenPtr, written, true);
      return ERRNO_SUCCESS;
    },

    // Exit from a cdylib module is a contract violation on this wire —
    // surfaced as a typed init error by the host runtime.
    proc_exit: onProcExit,

    // --- fail-closed surfaces (honest nonzero errnos) ----------------------

    // No descriptor table exists: every fd-numbered call is EBADF. Returning
    // success here would lie about reads/ seeks on fds that do not exist.
    fd_close: (): number => ERRNO_BADF,
    fd_datasync: (): number => ERRNO_BADF,
    fd_fdstat_get: (): number => ERRNO_BADF,
    fd_fdstat_set_flags: (): number => ERRNO_BADF,
    fd_filestat_get: (): number => ERRNO_BADF,
    fd_filestat_set_size: (): number => ERRNO_BADF,
    fd_pread: (): number => ERRNO_BADF,
    fd_pwrite: (): number => ERRNO_BADF,
    fd_read: (): number => ERRNO_BADF,
    fd_readdir: (): number => ERRNO_BADF,
    fd_seek: (): number => ERRNO_BADF,

    // No preopened descriptor table: a prestat probe cannot succeed.
    fd_prestat_get: (): number => ERRNO_BADF,
    fd_prestat_dir_name: (): number => ERRNO_BADF,

    // There is no filesystem behind the kernel: every path call is ENOENT.
    // (create_directory is ENOENT rather than EEXIST/ENOSPC — there is no
    // parent directory to create it in.)
    path_create_directory: (): number => ERRNO_NOENT,
    path_filestat_get: (): number => ERRNO_NOENT,
    path_open: (): number => ERRNO_NOENT,
    path_readlink: (): number => ERRNO_NOENT,
    path_remove_directory: (): number => ERRNO_NOENT,
    path_rename: (): number => ERRNO_NOENT,
    path_unlink_file: (): number => ERRNO_NOENT,

    // Blocking poll on a no-IO kernel would hang by contract; refuse loudly.
    poll_oneoff: (): number => ERRNO_NOSYS,
  } as Record<string, unknown>;
}

// Re-exported for runtimes/tests that assert on specific errno codes.
export const WASI_ERRNO: Readonly<Record<string, number>> = {
  SUCCESS: ERRNO_SUCCESS,
  BADF: ERRNO_BADF,
  EXIST: ERRNO_EXIST,
  ISDIR: ERRNO_ISDIR,
  NOENT: ERRNO_NOENT,
  NOTDIR: ERRNO_NOTDIR,
  NOTEMPTY: ERRNO_NOTEMPTY,
  NOTSUP: ERRNO_NOTSUP,
  NOSYS: ERRNO_NOSYS,
};
