//! `mp_engine_server` crate. //TODO: Port module mp_engine_server

// Raven-named functions/types (`SV_SendClientSnapshot`, `gameCallbacks`, …)
// keep their original casing across the ABI seam, matching `mp_game`'s policy.
#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

pub mod botlib_import;
pub mod gameCallbacks;
pub mod game_dispatch_ctx;
pub mod hook_install;
pub mod npcnav;
pub mod server;
pub mod server_host;
pub mod sv_bot;
pub mod sv_ccmds;
pub mod sv_client;
pub mod sv_game;
pub mod sv_init;
pub mod sv_main;
pub mod sv_net_chan;
pub mod sv_referee;
pub mod sv_renderer;
pub mod sv_snapshot;
pub mod sv_world;

pub use game_dispatch_ctx::GameDispatchCtx;
pub use server_host::{game_system_calls_shim, server_from_slot, server_slot, Server, ServerGame};

// Crate-root re-exports for the SV_* free functions that cross-module call sites
// reach as `crate::SV_*` (matching Raven's flat global namespace at the seam).
pub use sv_bot::{
    BotImport_DebugPolygonCreate, BotImport_DebugPolygonDelete, SV_BotAllocateClient,
    SV_BotCalculatePaths, SV_BotFreeClient, SV_BotGetConsoleMessage, SV_BotGetSnapshotEntity,
    SV_BotLibSetup, SV_BotLibShutdown, SV_BotWaypointReception,
};
pub use sv_ccmds::SV_GetStringEdString_str;
pub use sv_client::{SV_CloseDownload, SV_DropClient};
pub use sv_init::SV_SetUserinfo;
pub use sv_main::{SV_AddServerCommand, SV_SendServerCommand};
pub use sv_snapshot::{
    SV_SendClientSnapshot, SV_SendMessageToClient, SV_UpdateServerCommandsToClient,
};
