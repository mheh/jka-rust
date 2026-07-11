//! `mp_engine_server` crate. //TODO: Port module mp_engine_server

// Raven-named functions/types (`SV_SendClientSnapshot`, `gameCallbacks`, …)
// keep their original casing across the ABI seam, matching `mp_game`'s policy.
#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

pub mod gameCallbacks;
pub mod npcnav;
pub mod server;
pub mod server_host;
pub mod sv_bot;
pub mod sv_client;
pub mod sv_game;
pub mod sv_main;
pub mod sv_net_chan;
pub mod sv_snapshot;

pub use server_host::{game_system_calls_shim, sv_game_system_calls, Server, ServerGame};
