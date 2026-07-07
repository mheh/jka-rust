//! `cgame` — the MP cgame module cdylib shell (SEAM-D10). Interim stub exports
//! relocated here from the retired `abi_transport::entrypoints::qvm` module
//! (LOAD-D4 supersession: exports are per-shell); the live SEAM-D10 shape
//! (`ENGINE` OnceLock + `Dispatch` match, mirroring `jampgame`) lands in a
//! later slice. Bodies verbatim from the old stubs; plain `extern "C"` kept —
//! the `extern "C-unwind"` flip is the SEAM-D12 sweep, untouched here.
//!
//! //TODO: Port cgame live entrypoint exports (vmMain match, SEAM-D10)
//! // Source: oracle/oracle/codemp/cgame/cg_main.c:190

use abi_transport::entrypoints::{AbiCommand, AbiWord, RawSyscall};

/// Raven/OpenJK QVM-style `dllEntry` export (interim stub).
#[no_mangle]
pub extern "C" fn dllEntry(_syscall: RawSyscall) {}

/// Raven/OpenJK QVM-style `vmMain` export (interim stub).
#[no_mangle]
#[allow(clippy::too_many_arguments)]
pub extern "C" fn vmMain(
    _command: AbiCommand,
    _arg0: AbiWord,
    _arg1: AbiWord,
    _arg2: AbiWord,
    _arg3: AbiWord,
    _arg4: AbiWord,
    _arg5: AbiWord,
    _arg6: AbiWord,
    _arg7: AbiWord,
    _arg8: AbiWord,
    _arg9: AbiWord,
    _arg10: AbiWord,
    _arg11: AbiWord,
) -> AbiWord {
    0
}

// `GetModuleAPI` is deliberately NOT exported (SEAM-Q7 ruling, 2026-07-06,
// same as `jampgame`): OpenJK hard-fails on a present-but-NULL-returning
// symbol and falls back to the legacy `dllEntry`/`vmMain` path only when it
// is absent. Tracked in https://github.com/mheh/jka-rust/issues/1.
