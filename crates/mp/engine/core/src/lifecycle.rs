//! The `com_*` lifecycle surface (LIFE-D2): thin `&mut Engine` wrappers over
//! the qcommon-transcribed `Com_Init`/`Com_Frame`/`Com_Quit_f` bodies
//! (`common_fns.rs`), plus `sys_error`/`sys_milliseconds` (LIFE-D3/D4b).
//!
//! Post-DEC-23 shape: the engine world crosses into qcommon as the
//! `EngineHostView` bundle, so each wrapper split-borrows the view
//! (`host_view::engine_host_view`) and delegates. The `catch_unwind` error
//! boundary and the catch-side `com_error_recover` per-level recovery live
//! WITH the transcribed bodies in `mp_engine_qcommon::common_fns` (they need
//! only the view); nothing below duplicates them.

use mp_engine_qcommon::common_fns::{Com_Frame, Com_Init, Com_Quit_f};

use crate::engine::Engine;
use crate::host_view::engine_host_view;

/// Raven `Com_Init` (MP `common.cpp:1216`) — the 42-step boot contract,
/// delegated to the qcommon transcription (its own `catch_unwind` escalates a
/// boot-time `ComError` to `Sys_Error("Error during initialization: …")`).
///
/// Raven leaks `WinMain`'s `lpCmdLine` for process lifetime so the
/// `com_consoleLines[]` pointers stay live (`win_main.cpp:1524`);
/// `Com_ParseCommandLine` owns its line copies now, so the borrow ends here.
///
/// Source: `oracle/codemp/qcommon/common.cpp:1216`
pub fn com_init(engine: &mut Engine, command_line: &str) {
    let mut view = engine_host_view(engine);
    Com_Init(&mut view, command_line);
}

/// Raven `Com_Frame` (MP `common.cpp:1593`) — one frame, delegated to the
/// qcommon transcription (the `catch_unwind` ERR_DROP recovery point lives
/// there, DEC-08/LIFE-D3).
///
/// Source: `oracle/codemp/qcommon/common.cpp:1593`
pub fn com_frame(engine: &mut Engine) {
    let mut view = engine_host_view(engine);
    Com_Frame(&mut view);
}

/// Raven `Com_Quit_f` orchestration (MP `common.cpp:356`): `SV_Shutdown` →
/// `CL_Shutdown` → `Com_Shutdown` → `FS_Shutdown` → `Sys_Quit`. Never returns
/// (the quit path exits the process, as Raven's does).
///
/// Source: `oracle/codemp/qcommon/common.cpp:356`
pub fn com_shutdown(engine: &mut Engine) -> ! {
    let mut view = engine_host_view(engine);
    Com_Quit_f(&mut view);
    unreachable!("Com_Quit_f exits the process (Sys_Quit)");
}

/// Raven `Sys_Error` (`win32/win_main.cpp:350`; dedicated `null/win_main.cpp:324`).
/// Noreturn — the fatal escalation point. Ported INTO `mp_engine_core`
/// (LIFE-D3, LIFE-Q2 closed), delegating print+exit to `native/platform`.
///
/// Source: `oracle/codemp/win32/win_main.cpp:350`
pub fn sys_error(engine: &mut Engine, msg: &str) -> ! {
    let _ = engine;
    //TODO: Port Sys_Error console teardown + IN_Shutdown (client-shell slice)
    // Source: oracle/codemp/win32/win_main.cpp:350-389
    native_platform::platform::sys_fatal_print_exit(msg)
}

/// Raven `Sys_Milliseconds` (`win_shared.cpp:22-34`) — the base-relative clock
/// read (LIFE-D4b). Reads the `Instant` base held in `Common`; `now − base` as
/// `u64` ms truncated `as i32` (reproducing `timeGetTime`'s 49.7-day wrap). The
/// `base_time=true` raw variant reads `SystemTime::now()` (unix-epoch ms → i32;
/// LIFE-Q3). Pure `std`, no platform shell.
///
/// Source: `oracle/codemp/win32/win_shared.cpp:22-34`
pub fn sys_milliseconds(engine: &Engine, base_time: bool) -> i32 {
    if base_time {
        // The raw absolute variant: SystemTime::now() unix-epoch ms → i32
        // (LIFE-Q3; sole use is the Rand_Init seed, common.cpp:1248).
        return std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0) as i32;
    }
    // Base-relative: the one implementation lives in qcommon (Common owns the
    // time base; timing is not a host service) — delegate to it (LIFE-D4b).
    mp_engine_qcommon::timing::sys_milliseconds(&engine.common)
}
