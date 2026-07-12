//! `mp_app` — the MP dedicated host binary (the `jampded`-shaped thin bin
//! shell, workspace-architecture `mp/app`): `Engine::new()` → hook install →
//! warm-up read → `com_init` → `NET_Init` → the dedicated OS loop.
//!
//! Mirrors Raven's dedicated `main` (`oracle/codemp/null/win_main.cpp:1410`):
//! merge argv (`:1425`), timer warm-up (`:1447`), `Com_Init` (`:1457`),
//! `NET_Init` (`:1459` — the entry point calls it, NOT `Com_Init`), then
//! `while(1)`: `Sleep(5)` every iteration (`:1478`), `IN_Frame` (null input
//! stub, `:1490` — no-op here), `Com_Frame` (`:1493`). Never returns; exit is
//! via `Sys_Quit`/`Sys_Error` inside the frame (the `quit` command path).

use std::thread::sleep;
use std::time::Duration;

use mp_engine_core::{
    com_frame, com_init, engine_host_view, install_engine_hooks, sys_milliseconds, Engine,
};
use mp_engine_qcommon::sys_net::NET_Init;

/// App-bin helper joining process argv into the single command string
/// `Com_ParseCommandLine` splits — Raven's merge-argv step (jampded
/// `null/win_main.cpp:1425`). New-code glue, app-crate mechanical (lifecycle
/// § Slice hooks).
fn command_line() -> String {
    std::env::args().skip(1).collect::<Vec<_>>().join(" ")
}

fn main() {
    // Construct first (captures the Instant base, LIFE-D4b), install the
    // SV_*/renderer hook tables (Raven's link-time symbol resolution, DEC-23),
    // then the warm-up read and the boot contract.
    let mut engine: Box<Engine> = Engine::new();
    install_engine_hooks(&mut engine);
    // Raven's warm-up read (base already captured); base-relative → `false`
    // (null/win_main.cpp:1447 → qcommon.h:978).
    let _ = sys_milliseconds(&engine, false);
    com_init(&mut engine, &command_line());

    // NET_Init is called from the entry point, not Com_Init (lifecycle.md
    // Raven-ground-truth #2; null/win_main.cpp:1459).
    {
        let mut view = engine_host_view(&mut engine);
        NET_Init(&mut view);
    }

    loop {
        // run the game: Sleep(5) every dedicated iteration
        // (null/win_main.cpp:1478).
        sleep(Duration::from_millis(5));
        // IN_Frame (null/win_main.cpp:1490) is the dedicated null-input stub —
        // nothing to pump here.
        com_frame(&mut engine);
    }
}
