//! The SEAM-D11 inbound raw syscall trampoline — Rust half.
//!
//! The C-variadic definition lives in the committed shim
//! `game_syscall_trampoline.c` (built by this crate's `cc` build script; stable
//! Rust cannot define a C-variadic fn — skeleton-findings resolution 1,
//! 2026-07-03). The shim unpacks the va_list into a 16-word `intptr_t` frame
//! exactly as oracle `VM_DllSyscall` does (`vm.cpp:366-375`) and forwards here.

/// The raw C-variadic fn assigned to a hosted module's `syscall` slot — our
/// `VM_DllSyscall` equivalent, defined in `game_syscall_trampoline.c`.
/// Declared (not defined) in Rust so the loader can hand its address to the
/// module handshake (`dllEntry(syscall)`, `win_main.cpp:879-887`).
///
/// Source: `oracle/oracle/codemp/qcommon/vm.cpp:363-380`
extern "C-unwind" {
    pub fn game_syscall_trampoline(arg: isize, ...) -> isize;
}

/// The typed forwarding target the C shim calls with the unpacked 16-word frame
/// (`args[0]` = syscall number) — our `currentVM->systemCall( args )`
/// (`vm.cpp:377`). Reads the slot's injected `EngineSlot` (ctx + syscall,
/// engine_slot.rs) and dispatches through it. `extern "C-unwind"` so a
/// `com_error` panic unwinds back through the shim's live C frame (SEAM-D12).
///
/// Source: `oracle/oracle/codemp/qcommon/vm.cpp:363-377`
///
/// # Safety
/// `args` must point at the shim's 16-word frame; called only from
/// `game_syscall_trampoline` while an engine→module call into the owning slot
/// is on the stack (porting-rules §D11 engine-side seam exemption).
#[no_mangle]
pub extern "C-unwind" fn game_syscall_trampoline_words(args: *const isize) -> isize {
    let _ = args;
    todo!("Port VM_DllSyscall dispatch — oracle/oracle/codemp/qcommon/vm.cpp:363-377")
}
