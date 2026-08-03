//! The sim/VM thread (DEC-56.2): the com loop and the client frame.
//!
//! Raven ran this on the process's one thread, right after `WinMain` created
//! the window. macOS keeps the main thread for the event loop, so the whole
//! engine island moves here and reads window input through the platform bus.
//!
//! Source: `oracle/codemp/win32/win_main.cpp:1440-1604`

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::mpsc::{Receiver, SyncSender};
use std::sync::Arc;
use std::time::Duration;

use mp_engine_client::snd::sound_system::SoundSystem;
use mp_engine_client::Client;
use mp_engine_core::{
    com_frame, com_init, engine_host_view, install_engine_hooks, sys_milliseconds, Engine,
};
use mp_engine_qcommon::common::platform_events::PlatformEventSource;
use mp_engine_qcommon::sys_net::NET_Init;
use mp_renderer::render_state::frame_data::FrameData;
use mp_renderer::render_state::frame_package::FramePackage;
use mp_renderer::render_state::frame_sink::FrameSink;
use mp_renderer::renderer_frontend::RendererFrontend;

/// Boot the engine island and run the com loop forever.
///
/// A `Com_Error` that escapes the loop takes the whole process down, the way
/// Raven's `Sys_Error` did: there is no client left to draw with.
pub fn run(
    events: PlatformEventSource,
    command_line: String,
    packages: SyncSender<FramePackage>,
    recycled: Receiver<FrameData>,
) -> ! {
    let outcome = catch_unwind(AssertUnwindSafe(move || {
        boot_and_run(events, command_line, packages, recycled)
    }));
    if outcome.is_err() {
        eprintln!("jamp: the sim thread stopped, so the client is quitting");
    }
    std::process::exit(1);
}

fn boot_and_run(
    events: PlatformEventSource,
    command_line: String,
    packages: SyncSender<FramePackage>,
    recycled: Receiver<FrameData>,
) -> ! {
    // Construct first (captures the Instant base, LIFE-D4b).
    let mut engine: Box<Engine> = Engine::new();

    // Seat the three client-only islands BEFORE the hook install, because the
    // game dispatch note captures the `cl` and `re` addresses by value and a
    // module trap would otherwise reach a null one.
    engine.cl = Some(Client::default());

    // Block here until the pump reports a real drawable size.
    // This is the port's `GLimp_Init` stand-in.
    // Raven's `R_Init` did not return until `GLimp_Init` had created the window and measured it.
    // Source: `oracle/codemp/win32/win_glimp.cpp:713` (`GLimp_Init`, via `GLW_SetMode`'s `R_GetModeInfo`)
    let (drawable_width, drawable_height) = loop {
        let (width, height) = events.drawable_size();
        if width != 0 && height != 0 {
            break (width, height);
        }
        std::thread::sleep(Duration::from_millis(1));
    };

    let mut renderer = RendererFrontend {
        // Installing the sink is what turns `RE_EndFrame` from "clear the
        // stream" into "send the frame". Only a client build with a render
        // thread does it.
        frame_sink: Some(FrameSink { packages, recycled }),
        ..RendererFrontend::new()
    };
    // Seed `glconfig` once at boot from the measured window.
    // It stays boot-static after this, as in Raven.
    // A later change needs `vid_restart`.
    let assets = Arc::make_mut(&mut renderer.sim.published);
    assets.glconfig.vid_width = drawable_width;
    assets.glconfig.vid_height = drawable_height;
    engine.re = Some(renderer);

    engine.snd = Some(SoundSystem {
        // The platform shell is the arm that has a device (DEC-57.1).
        device_enabled: true,
        ..SoundSystem::default()
    });

    // The window pump's half of the event crossing (DEC-56.3).
    engine.common.platform_events = Some(events);

    install_engine_hooks(&mut engine);
    // Raven's warm-up read; base-relative, so `false`.
    let _ = sys_milliseconds(&engine, false);
    com_init(&mut engine, &command_line);

    // `NET_Init` is called from the entry point, not `Com_Init`.
    {
        let mut view = engine_host_view(&mut engine);
        NET_Init(&mut view);
    }

    // Raven sleeps only when minimized or dedicated; a live client spins and
    // `Com_Frame` holds the frame rate down through `com_maxfps`.
    loop {
        com_frame(&mut engine);
    }
}
