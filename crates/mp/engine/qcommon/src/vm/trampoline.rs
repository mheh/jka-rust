//! The SEAM-D11 inbound raw syscall trampolines — Rust half.
//!
//! The C-variadic definitions live in the committed shim
//! `game_syscall_trampoline.c` (built by this crate's `cc` build script; stable
//! Rust cannot define a C-variadic fn — skeleton-findings resolution 1,
//! 2026-07-03). Each shim unpacks the va_list into a 16-word `intptr_t` frame
//! exactly as oracle `VM_DllSyscall` does (`vm.cpp:366-375`) and forwards here.
//!
//! One trampoline and one armed cell per module slot, which is SEAM-D11's own
//! "one monomorphic trampoline per slot". The oracle keeps a single
//! `VM_DllSyscall` and reads the `currentVM` global to pick the dispatcher.
//! Here the slot a syscall belongs to is the entry address the module received
//! at `dllEntry`. The game slot serves `jampgame`, and the cgame and ui slots
//! serve the client's two dylibs (DEC-55).

extern "C-unwind" {
    /// The raw C-variadic fn assigned to the game module's `syscall` slot — our
    /// `VM_DllSyscall` equivalent, defined in `game_syscall_trampoline.c`.
    /// Declared (not defined) in Rust so the loader can hand its address to the
    /// module handshake (`dllEntry(syscall)`, `win_main.cpp:879-887`).
    ///
    /// Source: `oracle/codemp/qcommon/vm.cpp:363-380`
    pub fn game_syscall_trampoline(arg: isize, ...) -> isize;

    /// The cgame module's own entry address: same body, own armed cell.
    ///
    /// Source: `oracle/codemp/qcommon/vm.cpp:363-380`
    pub fn cgame_syscall_trampoline(arg: isize, ...) -> isize;

    /// The ui module's own entry address: same body, own armed cell.
    ///
    /// Source: `oracle/codemp/qcommon/vm.cpp:363-380`
    pub fn ui_syscall_trampoline(arg: isize, ...) -> isize;
}

/// One slot's copy of the injected `EngineSlot` pair, readable by the stateless
/// trampoline that owns it — the SEAM-D11 per-slot cell in its post-injection
/// form ("one monomorphic trampoline per slot; e.g. the game slot"), the §D11
/// engine-side static exemption twin of the shell's `OnceLock`.
///
/// PROVISIONAL (checkpoint-7 finding): the injected-EngineSlot amendment
/// leaves the trampoline→slot channel unspecified; these cells and their
/// `arm_*_slot` writers are the minimal faithful bridge, armed by the load call
/// site alongside its `load_module` injection.
struct SlotCell(std::cell::UnsafeCell<Option<super::engine_slot::EngineSlot>>);

// SAFETY (Sync only): armed once at boot, read single-threaded per Raven's
// contract (the same argument as the module shell's cells).
unsafe impl Sync for SlotCell {}

impl SlotCell {
    /// The un-armed cell every slot starts as.
    const fn empty() -> SlotCell {
        SlotCell(std::cell::UnsafeCell::new(None))
    }

    /// Write the `(ctx, system_calls)` pair the load call site injects into
    /// `load_module` (LOAD-D8 injection).
    fn arm(&self, ctx: *mut core::ffi::c_void, syscall: super::engine_slot::SlotSyscall) {
        // SAFETY: single-threaded boot path; no syscall can race the arm.
        unsafe {
            *self.0.get() = Some(super::engine_slot::EngineSlot { ctx, syscall });
        }
    }

    /// Dispatch one unpacked frame through this cell's armed pair. `who` names
    /// the slot, so an un-armed dispatch reports which module called.
    ///
    /// # Safety
    /// `args` must point at a shim's 16-word frame.
    unsafe fn dispatch(&self, who: &str, args: *const isize) -> isize {
        let slot = (*self.0.get())
            .as_ref()
            .unwrap_or_else(|| panic!("{who} slot armed before any module syscall"));
        (slot.syscall)(slot.ctx, args)
    }
}

/// The game slot's armed pair.
static GAME_SLOT: SlotCell = SlotCell::empty();

/// The cgame slot's armed pair (DEC-55).
static CGAME_SLOT: SlotCell = SlotCell::empty();

/// The ui slot's armed pair (DEC-55).
static UI_SLOT: SlotCell = SlotCell::empty();

/// The typed forwarding target the game shim calls with the unpacked 16-word
/// frame (`args[0]` = syscall number) — our `currentVM->systemCall( args )`
/// (`vm.cpp:377`). Reads the game slot's injected `EngineSlot` (ctx + syscall,
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
    unsafe { GAME_SLOT.dispatch("game", args) }
}

/// The cgame shim's forwarding target, the exact twin of the game one.
///
/// Source: `oracle/codemp/qcommon/vm.cpp:363-377`
///
/// # Safety
/// Same contract as [`game_syscall_trampoline_words`], for the cgame slot.
#[no_mangle]
pub extern "C-unwind" fn cgame_syscall_trampoline_words(args: *const isize) -> isize {
    // SAFETY: the shim always passes its full 16-word frame (vm.cpp:366).
    unsafe { CGAME_SLOT.dispatch("cgame", args) }
}

/// The ui shim's forwarding target, the exact twin of the game one.
///
/// Source: `oracle/codemp/qcommon/vm.cpp:363-377`
///
/// # Safety
/// Same contract as [`game_syscall_trampoline_words`], for the ui slot.
#[no_mangle]
pub extern "C-unwind" fn ui_syscall_trampoline_words(args: *const isize) -> isize {
    // SAFETY: the shim always passes its full 16-word frame (vm.cpp:366).
    unsafe { UI_SLOT.dispatch("ui", args) }
}

/// Arm the game slot's trampoline cell with the same `(ctx, system_calls)`
/// pair the load call site injects into `load_module` (LOAD-D8 injection).
pub fn arm_game_slot(ctx: *mut core::ffi::c_void, syscall: super::engine_slot::SlotSyscall) {
    GAME_SLOT.arm(ctx, syscall);
}

/// Arm the cgame slot's trampoline cell (DEC-55). The client boot writes the
/// one leaked dispatch note here, and `cgame_syscall_trampoline` reads it back.
pub fn arm_cgame_slot(ctx: *mut core::ffi::c_void, syscall: super::engine_slot::SlotSyscall) {
    CGAME_SLOT.arm(ctx, syscall);
}

/// Arm the ui slot's trampoline cell (DEC-55), the twin of `arm_cgame_slot`.
pub fn arm_ui_slot(ctx: *mut core::ffi::c_void, syscall: super::engine_slot::SlotSyscall) {
    UI_SLOT.arm(ctx, syscall);
}
