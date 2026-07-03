//! Raw ABI entrypoint type aliases — the Raven-free platform-ABI vocabulary.
//!
//! Relocated here from `abi-transport` per module-loading.md LOAD-D6 (base tier;
//! `native/platform` cannot take an uphill edge to `abi-transport`, so the five
//! aliases the loader signatures are written in terms of live here and
//! `abi-transport` re-exports them). Spelled `extern "C-unwind"` per engine-seam
//! SEAM-D12 (a `Com_Error` panic must traverse a real host's live C frames
//! mid-trap; plain `extern "C"` aborts on unwind).
//!
//! Source: `crates/abi-transport/src/entrypoints.rs:3-24` (relocated under LOAD-D6).

use core::ffi::{c_int, c_void};

/// `vmMain` command selector word.
pub type AbiCommand = c_int;

/// Pointer-width trap word (engine-seam SEAM-D4).
pub type AbiWord = isize;

/// Opaque `VM_DllSyscall` trampoline pointer handed to a module at `dllEntry`.
///
/// Modeled as opaque (`*const c_void`) so oracle's untyped-variadic
/// `int (QDECL *)(int, ...)` exposes no arity; the module casts it.
pub type RawSyscall = *const c_void;

/// `"dllEntry"` export: hands the module the engine syscall trampoline.
///
/// Source: `oracle/oracle/codemp/game/g_syscalls.c:14-16`
pub type RawDllEntry = extern "C-unwind" fn(syscall: RawSyscall);

/// `"vmMain"` export: command selector + twelve `int`-width argument words.
///
/// Source: `oracle/oracle/codemp/game/g_main.c:515`
pub type RawVmMain = extern "C-unwind" fn(
    command: AbiCommand,
    arg0: AbiWord,
    arg1: AbiWord,
    arg2: AbiWord,
    arg3: AbiWord,
    arg4: AbiWord,
    arg5: AbiWord,
    arg6: AbiWord,
    arg7: AbiWord,
    arg8: AbiWord,
    arg9: AbiWord,
    arg10: AbiWord,
    arg11: AbiWord,
) -> AbiWord;
