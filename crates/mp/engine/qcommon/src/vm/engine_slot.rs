//! `EngineSlot` — the per-module-slot inbound-syscall trampoline cell (SEAM-D11).
//!
//! When our engine hosts a real mod DLL, the module's poisoned `syscall` slot
//! must be handed a raw C variadic fn pointer (Raven's `VM_DllSyscall`,
//! `vm.cpp:363-380`) — the inbound dual of `CEngine::raw_syscall_words`. That raw
//! fn is stateless, so it reads a per-slot `*mut Engine` cell set for exactly the
//! duration of each engine→module call. SEAM-D11 replaces Raven's `currentVM`
//! global with one such cell per module slot (never one global — STATE-D2).

use std::cell::Cell;
use std::ffi::c_void;

//TODO: Port EngineSlot engine-pointer type
// Frozen shape (SEAM-D11) is `Cell<*mut Engine>` where `Engine` =
// `mp_engine_core::Engine` (the engine-island aggregate). `mp_engine_qcommon`
// sits a tier BELOW `mp_engine_core` and cannot name that type (uphill edge),
// so the cell is spelled with an opaque `*mut c_void` placeholder here. This is
// a real doc/layering tension surfaced by the seed — see skeleton FINDINGS.
// Source: docs/architecture/engine-seam.md § Inbound raw syscall trampoline (SEAM-D11)
type EnginePtr = *mut c_void;

/// One per hosted module slot. The cell holds a live `*mut Engine` ONLY while an
/// engine→module call into this slot is on the stack — the porting-rules §D11
/// engine-side seam exemption, the twin of the module shell's
/// `OnceLock<CEngine>` (SEAM-D1), one cell per slot (STATE-D2).
///
/// Source: `docs/architecture/engine-seam.md` § Inbound raw syscall trampoline (SEAM-D11).
pub struct EngineSlot {
    engine: Cell<EnginePtr>,
}

/// RAII: set the slot's cell on entry to an engine→module call, restore on Drop.
pub struct EngineSlotGuard<'a> {
    slot: &'a EngineSlot,
    prev: EnginePtr,
}

impl EngineSlot {
    /// cell = engine.
    pub fn enter(&self, engine: EnginePtr) -> EngineSlotGuard<'_> {
        let _ = (&self.engine, engine);
        todo!("Port EngineSlot::enter — docs/architecture/engine-seam.md SEAM-D11")
    }
}

impl Drop for EngineSlotGuard<'_> {
    fn drop(&mut self) {
        // cell = prev.
        let _ = (self.slot, self.prev);
        //TODO: Port EngineSlotGuard::drop restore — docs/architecture/engine-seam.md SEAM-D11
    }
}

//TODO: Port game_syscall_trampoline
// The raw fn assigned to the hosted module's `syscall` slot is frozen (SEAM-D11)
// as `extern "C-unwind" fn game_syscall_trampoline(arg: isize, ...) -> isize` —
// a C-variadic DEFINITION, which stable Rust cannot express (requires the
// unstable `c_variadic` feature). Emitting it would not compile, so it is left
// as this marker; Slice 0 does not exercise it (SEAM-D11 non-blocking note).
// This is a doc/stable-Rust contradiction surfaced by the seed — see FINDINGS.
// Source: docs/architecture/engine-seam.md § Inbound raw syscall trampoline (SEAM-D11)
