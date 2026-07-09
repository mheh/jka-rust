//! The SEAM-D11 inbound raw syscall trampoline — Rust half.
//!
//! The C-variadic definition lives in the committed shim
//! `game_syscall_trampoline.c` (built by this crate's `cc` build script; stable
//! Rust cannot define a C-variadic fn — skeleton-findings resolution 1,
//! 2026-07-03). The shim unpacks the va_list into a 16-word `intptr_t` frame
//! exactly as oracle `VM_DllSyscall` does (`vm.cpp:366-375`) and forwards here.

extern "C-unwind" {
    /// The raw C-variadic fn assigned to a hosted module's `syscall` slot — our
    /// `VM_DllSyscall` equivalent, defined in `game_syscall_trampoline.c`.
    /// Declared (not defined) in Rust so the loader can hand its address to the
    /// module handshake (`dllEntry(syscall)`, `win_main.cpp:879-887`).
    ///
    /// Source: `oracle/codemp/qcommon/vm.cpp:363-380`
    pub fn game_syscall_trampoline(arg: isize, ...) -> isize;
}

/// The typed forwarding target the C shim calls with the unpacked 16-word frame
/// (`args[0]` = syscall number) — our `currentVM->systemCall( args )`
/// (`vm.cpp:377`). Reads the slot's injected `EngineSlot` (ctx + syscall,
/// engine_slot.rs) and dispatches through it. `extern "C-unwind"` so a
/// `com_error` panic unwinds back through the shim's live C frame (SEAM-D12).
///
/// Source: `oracle/codemp/qcommon/vm.cpp:363-377`
///
/// # Safety
/// `args` must point at the shim's 16-word frame; called only from
/// `game_syscall_trampoline` while an engine→module call into the owning slot
/// is on the stack (porting-rules §D11 engine-side seam exemption).
#[no_mangle]
pub extern "C-unwind" fn game_syscall_trampoline_words(args: *const isize) -> isize {
    // SAFETY: the shim always passes its full 16-word frame (vm.cpp:366).
    let slot = unsafe {
        (*GAME_SLOT.0.get())
            .as_ref()
            .expect("game slot armed before any module syscall")
    };
    (slot.syscall)(slot.ctx, args)
}

/// The game slot's copy of the injected `EngineSlot` pair, readable by the
/// stateless trampoline above — the SEAM-D11 per-slot cell in its post-
/// injection form ("one monomorphic trampoline per slot; e.g. the game slot"),
/// the §D11 engine-side static exemption twin of the shell's `OnceLock`.
///
/// PROVISIONAL (checkpoint-7 finding): the injected-EngineSlot amendment
/// leaves the trampoline→slot channel unspecified; this cell + `arm_game_slot`
/// are the minimal faithful bridge, armed by the load call site alongside its
/// `load_module` injection.
static GAME_SLOT: GameSlotCell = GameSlotCell(std::cell::UnsafeCell::new(None));

struct GameSlotCell(std::cell::UnsafeCell<Option<super::engine_slot::EngineSlot>>);

// SAFETY (Sync only): armed once at module load, read single-threaded per
// Raven's contract (the same argument as the module shell's cells).
unsafe impl Sync for GameSlotCell {}

/// Arm the game slot's trampoline cell with the same `(ctx, system_calls)`
/// pair the load call site injects into `load_module` (LOAD-D8 injection).
pub fn arm_game_slot(ctx: *mut core::ffi::c_void, syscall: super::engine_slot::SlotSyscall) {
    // SAFETY: single-threaded module-load path; no syscall can race the arm.
    unsafe {
        *GAME_SLOT.0.get() = Some(super::engine_slot::EngineSlot { ctx, syscall });
    }
}
