//! `mp_engine_client` crate — the client island: `cl_main`, the two module
//! dispatchers, input, keys, console, screen, cinematics, and the carriers.

// Raven-named functions/types (`CL_Init`, `Con_DrawConsole`, `cl_paused`, …) keep
// their original casing across the ABI seam, matching `mp_game`'s crate policy.
#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

pub mod cin;
pub mod cl_cgame;
pub mod cl_cin;
pub mod cl_console;
pub mod cl_input;
pub mod cl_keys;
pub mod cl_main;
pub mod cl_net_chan;
pub mod cl_parse;
pub mod cl_referee;
pub mod cl_scrn;
pub mod cl_ui;
pub mod client;
pub mod client_dispatch_ctx;
pub mod client_host;
pub mod fffx;
pub mod fx;
pub mod keys;
pub mod mp3;
pub mod null;
pub mod snd;
pub mod snd_ambient;
pub mod snd_stubs;
pub mod svc_strings;

pub use client_dispatch_ctx::ClientDispatchCtx;
pub use client_host::{
    cgame_system_calls_shim, cl_from_view, client_legacy_syscall, g2_from_view,
    ui_system_calls_shim, Client, SoundSystem,
};
