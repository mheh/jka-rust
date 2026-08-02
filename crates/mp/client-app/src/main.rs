//! `mp_client_app` — the MP client host binary (the `jamp`-shaped thin bin
//! shell), the twin of `mp/app`'s dedicated server.
//!
//! Three threads, per DEC-56.2 and the DEC-37.2 topology:
//! - the main thread runs the winit event loop and nothing else, because macOS
//!   gives the event loop to the main thread;
//! - the sim/VM thread runs `Com_Init` and the com loop;
//! - the render thread owns every GPU object.
//!
//! Raven had one thread: `WinMain` created the window, called `Com_Init`, and
//! then looped `IN_Frame` + `Com_Frame` while the window procedure queued input
//! on the same stack. The split keeps that shape at the seam: the pump still
//! translates window messages into `sysEvent_t` values, and the sim thread
//! still drains them inside `Sys_GetEvent`.
//!
//! Source: `oracle/codemp/win32/win_main.cpp:1410-1604`

mod keymap;
mod pump;
mod render_thread;
mod sim;

use std::thread::Builder;

use mp_engine_qcommon::common::platform_events::platform_event_bus;
use mp_engine_qcommon::common::ComError;
use winit::event_loop::EventLoop;

use crate::pump::{Pump, PUMP_CONTROL_FLOW};

/// Join process argv into the single command string `Com_ParseCommandLine`
/// splits, Raven's merge-argv step.
///
/// Source: `oracle/codemp/win32/win_main.cpp:1425`
fn command_line() -> String {
    std::env::args().skip(1).collect::<Vec<_>>().join(" ")
}

fn main() {
    // `ComError` panics are Raven's `throw` — control flow, not bugs (DEC-08).
    let default_hook = std::panic::take_hook();
    let debug_comerror = std::env::var_os("JKA_DEBUG_COMERROR").is_some();
    std::panic::set_hook(Box::new(move |info| {
        match info.payload().downcast_ref::<ComError>() {
            Some(e) if debug_comerror => {
                eprintln!("[com_error debug] {:?}: {}", e.level, e.msg);
            }
            Some(_) => {}
            None => default_hook(info),
        }
    }));

    let (sink, source) = platform_event_bus();
    let arguments = command_line();
    Builder::new()
        .name("jamp-sim".to_string())
        .spawn(move || sim::run(source, arguments))
        .expect("spawn: the client could not start its sim thread");

    let event_loop = EventLoop::new().expect("EventLoop::new: the client has no window system");
    event_loop.set_control_flow(PUMP_CONTROL_FLOW);
    let mut pump = Pump::new(sink);
    event_loop
        .run_app(&mut pump)
        .expect("run_app: the client event loop failed");

    // Raven's `WM_CLOSE` ends the process outright, and the sim thread has no
    // engine left to shut down cleanly once the window is gone.
    std::process::exit(0);
}
