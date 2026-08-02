//! `mp_engine_client` crate. //TODO: Port module mp_engine_client

// Raven-named functions/types (`CL_Init`, `Con_DrawConsole`, `cl_paused`, …) keep
// their original casing across the ABI seam, matching `mp_game`'s crate policy.
#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

pub mod cl_cgame;
pub mod cl_cin;
pub mod cl_console;
pub mod cl_input;
pub mod cl_keys;
pub mod cl_main;
pub mod cl_net_chan;
pub mod cl_parse;
pub mod cl_scrn;
pub mod cl_ui;
pub mod client;
pub mod client_host;
pub mod fffx;
pub mod fx;
pub mod keys;
pub mod mp3;
pub mod null;
pub mod snd;
pub mod snd_ambient;

pub use client_host::{Client, SoundSystem};
