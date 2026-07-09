//! Outbound execution backends — `CEngine` (real C engine) and `Static`
//! (our Rust engine), plus the forward-declared `RunStatic` per-call handler.
//!
//! SEAM-D9: both backend types live here; `Execute<C>`/`Dispatch<C>` (the seam
//! traits) live in `outbound.rs`/`inbound.rs`.

use core::ffi::c_int;

use super::outbound::{Execute, OutboundSysCall};
use super::transport::{DecodeSysCallReturn, EncodeSysCall};
use crate::entrypoints::RawSyscall;

/// The C-engine outbound backend: wraps the raw syscall pointer (SEAM-D9).
///
/// Source: `oracle/codemp/game/g_syscalls.c:8` (the module `syscall` ptr).
pub struct CEngine {
    syscall: RawSyscall,
}

// The single `OnceLock<CEngine>` module-seam static (SEAM-D1) requires
// `CEngine: Send + Sync`, but `RawSyscall` is `*const c_void` (neither). The
// pointer is set once at `dllEntry` and read single-threaded from `vmMain`, so
// these impls are sound; the frozen `static ENGINE: OnceLock<CEngine>` cannot
// compile without them and the docs do not spell them (see skeleton FINDINGS).
unsafe impl Send for CEngine {}
unsafe impl Sync for CEngine {}

impl CEngine {
    pub fn new(syscall: RawSyscall) -> Self {
        Self { syscall }
    }

    /// The SOLE unsafe choke point (porting-rules §D11): forwards a runtime
    /// `&[isize]` to the C variadic `syscall` by spelling out a fixed 16-slot
    /// frame matching Raven's outbound trampoline `VM_DllSyscall`'s `int
    /// args[16]` (`vm.cpp:363-376`), extras zero-filled.
    ///
    /// Source: `oracle/codemp/qcommon/vm.cpp:363-376`
    unsafe fn raw_syscall_words(&self, import: c_int, words: &[isize]) -> isize {
        // Fixed 16-slot frame: the import word + 15 argument words, extras
        // zero-filled — exactly the frame VM_DllSyscall's unpack loop consumes
        // (vm.cpp:363-376; the callee reads only the indices its trap defines).
        let mut w = [0isize; 15];
        let n = words.len().min(15);
        w[..n].copy_from_slice(&words[..n]);
        // The C variadic call — through a VARIADIC fn-pointer type so the
        // va_list ABI is correct on all targets (arm64 stack-passed va-args).
        let f: unsafe extern "C-unwind" fn(isize, ...) -> isize =
            core::mem::transmute(self.syscall);
        f(
            import as isize,
            w[0], w[1], w[2], w[3], w[4], w[5], w[6], w[7], w[8], w[9], w[10], w[11], w[12],
            w[13], w[14],
        )
    }
}

impl<C> Execute<C> for CEngine
where
    C: EncodeSysCall + DecodeSysCallReturn,
    C::Import: Into<i32>,
{
    fn execute(&self, args: C::Args) -> C::Output {
        let t = C::encode_syscall(&args);
        // Module-side encode direction: `From<import enum> for i32` (SEAM-D6).
        let ret = unsafe { self.raw_syscall_words(C::IMPORT.into(), t.args()) };
        C::decode_return(ret)
    }
}

/// The Rust-engine outbound backend (SEAM-D9): the same call defs run against
/// our Rust engine as ordinary safe Rust, `IMPORT` unused. Shape only.
pub struct Static;

/// Forward-declared per-call handler bound for `Static` (engine-seam § `Static`).
/// The engine service traits `C::run` calls are specified in state-ownership.md
/// + `docs/subsystems/*` (pending).
//TODO: Port RunStatic per-call handler surface
// Source: docs/architecture/engine-seam.md § `Static` — the Rust-engine outbound backend
pub trait RunStatic: OutboundSysCall {
    fn run(engine: &Static, args: Self::Args) -> Self::Output;
}

impl<C> Execute<C> for Static
where
    C: RunStatic,
{
    fn execute(&self, args: C::Args) -> C::Output {
        C::run(self, args)
    }
}
