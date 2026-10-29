// wasi_compat34.h — force-included into every DuckDB C++ TU via
// `-include $TMP/wasi_compat34.h` (see action.yml). Committed as a sibling
// file rather than an in-script heredoc: a column-0 heredoc terminator is
// impossible inside the action's YAML block scalar (a less-indented line ends
// the literal block), so the previous heredoc form never terminated in bash
// and made action.yml YAML-invalid. Witnessed 2026-10-24.
//
// wasi-sdk 34 libc lacks TIOCGWINSZ and a `struct winsize` compatible with
// what DuckDB's terminal probing expects, and sched_getcpu has no WASI
// definition.
#include <signal.h>
#include <sys/resource.h>
#include <sys/types.h>
#include <unistd.h>
#ifndef TIOCGWINSZ
#define TIOCGWINSZ 0x40087468
#endif
struct wasi_compat_winsize { unsigned short ws_row, ws_col, ws_xpixel, ws_ypixel; };
#define winsize wasi_compat_winsize
#ifdef __cplusplus
extern "C" int sched_getcpu(void);
// wasi-libc declares kill only when POSIX feature-test macros are defined;
// DuckDB's process_util.cpp calls kill(pid, 0) with no signal.h in its
// include chain, so the call hits "use of undeclared identifier 'kill'"
// (witnessed on CI). Declared here — the -include forces this decl into
// every C++ TU; the symbol itself is provided at link time by
// frozen-duckdb-sys wasi_posix.c stubs.
extern "C" int kill(pid_t, int);

// wasi-libc carries the rlimit surface behind
// __wasilibc_unmodified_upstream: struct rlimit, RLIMIT_NOFILE and
// RLIM_INFINITY are absent from <sys/resource.h> on wasm32-wasip1
// (witnessed: duckdb http_transport_manager.cpp GetFileDescriptorLimit
// fails to compile against wasi-sdk-34). Mirror the POSIX layout locally;
// the getrlimit symbol itself is provided at link time by
// frozen-duckdb-sys's wasi_posix.c stubs (getrlimit(int, void*) — the
// void* ABI matches this layout exactly).
struct wasi_compat_rlimit { unsigned long long rlim_cur, rlim_max; };
#define rlimit wasi_compat_rlimit
#define RLIMIT_NOFILE 7
#define RLIM_INFINITY ((unsigned long long)-1)
extern "C" int getrlimit(int, void *);
#endif
