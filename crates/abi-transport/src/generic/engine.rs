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
/// Source: `oracle/oracle/codemp/game/g_syscalls.c:8` (the module `syscall` ptr).
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
    /// Source: `oracle/oracle/codemp/qcommon/vm.cpp:363-376`
    unsafe fn raw_syscall_words(&self, _import: c_int, _words: &[isize]) -> isize {
        let _ = self.syscall;
        todo!("Port CEngine::raw_syscall_words — oracle/oracle/codemp/qcommon/vm.cpp:363-376")
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
