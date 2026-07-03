//! `com_init`/`com_frame`/`com_shutdown` + `sys_error` + `sys_milliseconds`
//! (LIFE-D2/D3/D4b). Ported per-mode into `mp_engine_core` — `com_frame` must
//! call `SV_Frame`/`CL_Frame`, which qcommon cannot reach. No shared trait.
//!
//! `com_error`/`ComError`/`ErrorLevel` live one tier below in
//! `mp_engine_qcommon` (STATE-Q4) so leaf throw sites reach them.

use mp_engine_qcommon::common::ComError;

use crate::engine::Engine;

/// Raven `Com_Init` (MP `common.cpp:1216` / SP `:950`). Runs the boot contract;
/// a `ComError` panic during init is caught here and escalated to fatal
/// (mirrors `catch → Sys_Error`, MP `:1439`, LIFE-D3).
///
/// Source: `oracle/oracle/codemp/qcommon/common.cpp:1216`
pub fn com_init(engine: &mut Engine, command_line: &str) {
    use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};
    match catch_unwind(AssertUnwindSafe(|| com_init_body(&mut *engine, command_line))) {
        Ok(()) => {}
        Err(p) => match p.downcast::<ComError>() {
            // Init-time errors are always fatal: Raven's init catch →
            // `Sys_Error ("Error during initialization: %s", reason)`
            // (common.cpp:1439-1441, LIFE-D3).
            Ok(e) => sys_error(engine, &format!("Error during initialization: {}", e.msg)),
            Err(other) => resume_unwind(other),
        },
    }
}

/// The 42-step MP boot contract body (`common.cpp:1216-1442`), Slice-0 subset:
/// steps 3/5/7/12 are the LIFE-Q8 boot-success no-op stubs; unported steps
/// carry `//TODO: Port` markers in step order so the transcript diff (DEC-09.2)
/// can activate step-by-step as B1/B2 land.
fn com_init_body(engine: &mut Engine, command_line: &str) {
    use mp_engine_qcommon::common::{
        cbuf_init, cmd_init, com_printf, cvar_init, fs_init_filesystem,
    };
    let _ = command_line;

    // 1. Version banner (common.cpp:1219): Com_Printf("%s %s %s\n", Q3_VERSION,
    //    CPUSTRING, __DATE__); Q3_VERSION = "JAmp: v1.0.1.0" (game_version.h:9).
    //TODO: Port CPUSTRING/__DATE__ banner fields
    // Source: oracle/oracle/codemp/qcommon/common.cpp:1219
    com_printf(&mut engine.common, "JAmp: v1.0.1.0 (jka-rust slice 0)
");
    //TODO: Port Com_InitPushEvent — step 2
    // Source: oracle/oracle/codemp/qcommon/common.cpp:1224
    cvar_init(); // step 3 (LIFE-Q8 stub; common.cpp:1226)
    //TODO: Port Com_ParseCommandLine — step 4
    // Source: oracle/oracle/codemp/qcommon/common.cpp:1230
    cbuf_init(); // step 5 (LIFE-Q8 stub; common.cpp:1233)
    // step 6 Com_InitZoneMemory: dropped — Rust ownership replaces TheZone (§C9).
    cmd_init(); // step 7 (LIFE-Q8 stub; common.cpp:1242)
    //TODO: Port Com_StartupVariable/Rand_Init/CL_InitKeyCommands — steps 8-11
    // Source: oracle/oracle/codemp/qcommon/common.cpp:1245-1254
    fs_init_filesystem(); // step 12 (LIFE-Q8 stub; common.cpp:1266)
    //TODO: Port Com_InitJournaling + config execs + cvar block — steps 13-29
    // Source: oracle/oracle/codemp/qcommon/common.cpp:1268-1383
    // 30. VM_Init: the empty ModuleRegistry — already default-constructed into
    //     Engine.common.modules by Engine::new (LIFE-Q9); nothing to do here.
    //TODO: Port SV_Init + the dedicated/client-init tail — steps 31-39
    // Source: oracle/oracle/codemp/qcommon/common.cpp:1385-1431
    // 40. com_fullyInitialized = qtrue (common.cpp:1434).
    engine.common.fully_initialized = true;
    // 41. Completion banner (common.cpp:1435).
    com_printf(
        &mut engine.common,
        "--- Common Initialization Complete ---
",
    );
}

/// Raven `Com_Frame` (MP `common.cpp:1593` / SP `:1269`). One frame; the
/// `catch_unwind` boundary (DEC-08 / SEAM-D10) wraps the body. A caught
/// `ComError` runs the full per-level recovery catch-side then prints the level
/// literal; any non-`ComError` panic is re-raised as fatal (LIFE-D3).
///
/// Round-4 amendment (LIFE-D3, group-a-regate4): the catch arm wraps
/// `com_error_recover` in its OWN `catch_unwind`; if recovery itself panics
/// while the `errorEntered` guard is set, route to `sys_error("recursive error
/// after: {saved msg}")` — reproducing Raven's recursive-error banner + exit
/// (MP `common.cpp:288`, `Sys_Error("recursive error after: %s", …)`).
///
/// Source: `oracle/oracle/codemp/qcommon/common.cpp:1593`
pub fn com_frame(engine: &mut Engine) {
    use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};
    match catch_unwind(AssertUnwindSafe(|| com_frame_body(&mut *engine))) {
        Ok(()) => {}
        Err(p) => match p.downcast::<ComError>() {
            // ERR_DROP recovery point. `com_error` only panicked; the catch runs
            // ALL of Raven's pre-throw work in oracle print order (LIFE-D2).
            Ok(e) => {
                let saved = e.msg.clone();
                let err = *e;
                let recovered =
                    catch_unwind(AssertUnwindSafe(|| com_error_recover(&mut *engine, err)));
                if recovered.is_err() {
                    // Recovery panicked while `errorEntered` is set → recursive
                    // error fatal path (round-4 LIFE-D3; MP common.cpp:288).
                    sys_error(engine, &format!("recursive error after: {saved}"));
                }
            }
            // Real Rust bug → fatal (LIFE-D3).
            Err(other) => resume_unwind(other),
        },
    }
}

/// Raven `Com_Shutdown` + `Com_Quit_f` orchestration (MP `common.cpp:356,1785`).
///
/// Source: `oracle/oracle/codemp/qcommon/common.cpp:1785`
pub fn com_shutdown(engine: &mut Engine) {
    let _ = engine;
    todo!("Port Com_Shutdown — oracle/oracle/codemp/qcommon/common.cpp:1785")
}

/// Raven `Sys_Error` (`win32/win_main.cpp:350`; dedicated `null/win_main.cpp:324`).
/// Noreturn — the fatal escalation point for `com_init`'s init-catch and the
/// recursive-error path. Ported INTO `mp_engine_core` (LIFE-D3, LIFE-Q2 closed),
/// delegating print+exit to `native/platform` (a downhill call).
///
/// Source: `oracle/oracle/codemp/win32/win_main.cpp:350`
pub fn sys_error(engine: &mut Engine, msg: &str) -> ! {
    let _ = engine;
    //TODO: Port Sys_Error console teardown + IN_Shutdown (client-shell slice)
    // Source: oracle/oracle/codemp/win32/win_main.cpp:350-389
    native_platform::platform::sys_fatal_print_exit(msg)
}

/// Raven `Sys_Milliseconds` (`win_shared.cpp:22-34`) — the base-relative clock
/// read (LIFE-D4b). Reads the `Instant` base held in `Common`; `now − base` as
/// `u64` ms truncated `as i32` (reproducing `timeGetTime`'s 49.7-day wrap). The
/// `base_time=true` raw variant reads `SystemTime::now()` (unix-epoch ms → i32;
/// LIFE-Q3). Pure `std`, no platform shell.
///
/// Source: `oracle/oracle/codemp/win32/win_shared.cpp:22-34`
pub fn sys_milliseconds(engine: &Engine, base_time: bool) -> i32 {
    if base_time {
        // The raw absolute variant: SystemTime::now() unix-epoch ms → i32
        // (LIFE-Q3; sole use is the Rand_Init seed, common.cpp:1248).
        return std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0) as i32;
    }
    // Base-relative: now − base as u64 ms truncated `as i32` — reproducing
    // timeGetTime's practical 49.7-day wrap (LIFE-D4b).
    engine.common.time_base.elapsed().as_millis() as u64 as i32
}

/// The one-frame body wrapped by `com_frame`'s `catch_unwind` boundary. Private
/// (not part of the frozen surface); a mechanical §C port of `Com_Frame`'s body.
fn com_frame_body(engine: &mut Engine) {
    let _ = engine;
    todo!("Port Com_Frame body — oracle/oracle/codemp/qcommon/common.cpp:1593")
}

/// Catch-side recovery helper (private, `&mut Engine` in hand): the per-level
/// sequence of § Error recovery, run in Raven's pre-throw order, ending in the
/// level literal print (LIFE-D2) or, for `ERR_FATAL`/escalated, `sys_error`.
fn com_error_recover(engine: &mut Engine, err: ComError) {
    let _ = (engine, err);
    todo!("Port Com_Error catch-side recovery — oracle/oracle/codemp/qcommon/common.cpp:249-345")
}
