//! SP server-game public ABI surface (`g_public.h`).

use core::ffi::c_int;

pub mod game_export_t;
pub mod game_import_t;
pub mod saved_game_just_loaded_e;

/// Raven SP `GAME_API_VERSION` — the `game_export_t.apiversion` contract,
/// checked engine-side after `Sys_GetGameAPI` (`sv_game.cpp:682-684`).
///
/// Source: `oracle/oracle/code/game/g_public.h:5`
pub const GAME_API_VERSION: c_int = 8;
