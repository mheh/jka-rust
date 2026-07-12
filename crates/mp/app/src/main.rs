//! `mp_app` — the MP dedicated host binary (the `jampded`-shaped thin bin
//! shell, workspace-architecture `mp/app`): `Engine::new()` → warm-up read →
//! `com_init` → (Slice-0 acceptance driver) → exit.
//!
//! The frozen lifecycle skeleton's dedicated OS loop
//! (`sleep_ms(5)`/`console_poll`/`net_poll`/`com_frame`) is NOT yet run — its
//! `com_frame` body is unported:
//! //TODO: Port Com_Frame dedicated loop wiring
//! // Source: oracle/codemp/null/win_main.cpp:1478-1493

use mp_abi::game::exports::MpGameExport;
use mp_engine_core::{com_init, sv_init_game_progs, sys_milliseconds, Engine};
use native_platform::module_loader::{ModuleNaming, ModuleSearchPolicy, SearchStep};

/// App-bin helper joining process argv into the single command string
/// `Com_ParseCommandLine` splits — Raven's merge-argv step (jampded
/// `null/win_main.cpp:1425`). New-code glue, app-crate mechanical (lifecycle
/// § Slice hooks).
fn command_line() -> String {
    std::env::args().skip(1).collect::<Vec<_>>().join(" ")
}

/// Slice-0 DEV-GLUE module search policy for the acceptance run on the dev
/// host (macOS): resolves the freshly built `libjampgame.dylib` beside the
/// executable. This is NOT the frozen Win32 (`Some("x86.dll")`) / Unix
/// (`Some("i386.so")`) policy value and does NOT resolve LOAD-Q1 (the
/// canonical macOS suffix for a Raven-parity host stays open) — the cargo
/// artifact is staged under the bare `jampgame` + `.dylib` synthesis so the
/// frozen naming mechanism itself is exercised unchanged.
//TODO: Port <macOS module suffix> (LOAD-Q1) — this dev-glue value is not it
// Source: docs/architecture/module-loading.md § Open questions (LOAD-Q1)
fn slice0_dev_policy() -> ModuleSearchPolicy {
    let exe_dir = std::env::current_exe()
        .expect("current_exe")
        .parent()
        .expect("exe dir")
        .to_path_buf();
    // Stage the cargo cdylib (`libjampgame.dylib`) under the loader's
    // synthesized name (`jampgame` + `.dylib`) so name synthesis stays frozen.
    let built = exe_dir.join("libjampgame.dylib");
    let staged = exe_dir.join("jampgame.dylib");
    if built.exists() {
        let _ = std::fs::copy(&built, &staged);
    }
    ModuleSearchPolicy {
        naming: ModuleNaming {
            suffix: Some(".dylib"),
        },
        // Unix-shaped: no direct probe (unix_main.c:361-373 `#if 0`).
        direct_first: false,
        steps: vec![SearchStep::FsPath {
            base: exe_dir,
            gamedir: String::new(),
        }],
    }
}

fn main() {
    // The frozen jampded main skeleton (lifecycle.md § Slice hooks): construct
    // first (captures the Instant base), warm-up read, then com_init.
    let mut engine: Box<Engine> = Engine::new();
    // Raven's warm-up read (base already captured); base-relative → `false`
    // (null/win_main.cpp:1447 → qcommon.h:978).
    let _ = sys_milliseconds(&engine, false);
    com_init(&mut engine, &command_line());

    // ---- Slice-0 acceptance driver (provisional) ----
    // The settled load trigger is map spawn (SV_SpawnServer → SV_InitGameProgs,
    // post-Slice-0; LOAD-Q12); this driver invokes the equiv directly so the
    // GAME_INIT round-trip is exercised end-to-end (porting-rules §E16).
    let policy = slice0_dev_policy();
    let slot = sv_init_game_progs(&mut engine, &policy);

    // GAME_INIT round-trip: `VM_Call( gvm, GAME_INIT, svs.time,
    // Com_Milliseconds(), restart )` (sv_game.cpp:1690). svs.time and the
    // journaled Com_Milliseconds reader are unported — zeros stand in:
    //TODO: Port SV_InitGameVM GAME_INIT args (svs.time, Com_Milliseconds)
    // Source: oracle/codemp/server/sv_game.cpp:1680-1691
    let _ = engine
        .common
        .modules
        .vm_call(&slot, MpGameExport::GAME_INIT as i32, [0; 12]);

    // SV_ShutdownGameProgs dual: `VM_Call( gvm, GAME_SHUTDOWN, qfalse );
    // VM_Free( gvm )` (sv_game.cpp:1666-1673).
    let _ = engine
        .common
        .modules
        .vm_call(&slot, MpGameExport::GAME_SHUTDOWN as i32, [0; 12]);
    engine.common.modules.unload(slot);

    //TODO: Port the dedicated OS loop (sleep/console_poll/net_poll/com_frame)
    // Source: oracle/codemp/null/win_main.cpp:1478-1493 (com_frame body pending)
}
