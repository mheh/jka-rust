//! `Server` (the SP `Engine.sv` island host) + the SP game-attach seam
//! (`SV_InitGameProgs`/`SV_ShutdownGameProgs` equivalents).
//!
//! SP has **no `VM_Create`**: `SV_InitGameProgs` fills a stack `game_import_t`
//! and attaches the game through `ge = Sys_GetGameAPI(&import)`
//! (`code/server/sv_game.cpp:478,669`), then version-checks
//! `ge->apiversion != GAME_API_VERSION` (`:682-684`) and calls `ge->Init`
//! (`:691`). There is therefore no MP `load_module`/`ModuleRegistry`/
//! `EngineSlot` dual here — no slots, no injected systemCalls, no trampoline
//! (the export table IS the entry surface; settled SP mapping 2026-07-03).
//! Our SP reaches `GetGameAPI` as a direct linked call — the `Sys_GetGameAPI`
//! DLL loader (`code/win32/win_main.cpp:483-547`) is dropped surface (LOAD-D1
//! round-3 / LOAD-D5 / DEC-07: our SP never exercises a loader; retail SP-DLL
//! hosting is outside DEC-05.3).

use core::ffi::c_int;

use sp_abi::game::public::game_export_t::game_export_t;
use sp_abi::game::public::saved_game_just_loaded_e::SavedGameJustLoaded_e;
use sp_qshared::shared::qboolean;

use crate::server::server_static_t::serverStatic_t;
use crate::server::server_t::server_t;
use crate::server::world_sector_s::{worldSector_t, AREA_NODES};

/// Raven `sv_worldSectors[AREA_NODES]` + `sv_numworldSectors` — the master
/// table's `Server.world_sectors` row, grouped under Raven names (Savegame
/// precedent: a Rust-side grouping colocated with its sole owner).
///
/// Source: `oracle/oracle/code/server/sv_world.cpp:82-83`
#[allow(non_snake_case)]
pub struct WorldSectors {
    pub sv_worldSectors: [worldSector_t; AREA_NODES],
    pub sv_numworldSectors: c_int,
}

/// SP save-game transition state — the master table's `Server.savegame` row
/// (SP ONLY; MP has no counterpart). Groups Raven's two file-scope savegame
/// globals under their Raven names. Defined inline here rather than in its own
/// file (user ruling 2026-07-05: a Rust-side grouping colocated with its sole
/// owner, not a ported Raven type).
///
/// Source: `oracle/oracle/code/server/sv_ccmds.cpp:22` (`qbLoadTransition`);
/// `oracle/oracle/code/server/server.h:316-317` (extern decls);
/// `oracle/oracle/code/game/g_public.h:54-59` (`SavedGameJustLoaded_e`).
#[allow(non_snake_case)] // Raven field names preserved (porting-rules §D12 spirit).
pub struct Savegame {
    /// Raven `qboolean qbLoadTransition = qfalse;` — set for cross-map load
    /// transitions (`sv_ccmds.cpp:288`), cleared after spawn (`:311`).
    ///
    /// Source: `oracle/oracle/code/server/sv_ccmds.cpp:22`
    pub qbLoadTransition: qboolean,
    /// Raven `SavedGameJustLoaded_e eSavedGameJustLoaded` — `eNO`/`eFULL`/
    /// `eAUTO`; consumed at client-enter-world (`sv_client.cpp:483`), reset to
    /// `eNO` (`sv_client.cpp:500`), fed to `ge->Init` (`sv_game.cpp:690`).
    ///
    /// Source: `oracle/oracle/code/server/server.h:316`;
    /// `oracle/oracle/code/game/g_public.h:54-59`
    pub eSavedGameJustLoaded: SavedGameJustLoaded_e,
}

/// The SP server-island state owned by `Engine.sv: Server` — always present,
/// NOT an `Option` (state-ownership § Seam definition, SP mirror per DEC-04):
/// liveness is `sv.state == SS_DEAD` (`serverState_t`, "no map loaded",
/// `code/server/server.h:42-49`), the direct dual of Raven's loader-zero-filled
/// `sv`/`svs` statics (whole-`Engine` zeroed-`Box` allocation). Reuses the
/// existing ported `server_t`/`serverStatic_t` types as fields
/// (`server/server_t.rs`, `server/server_static_t.rs`); `sv.state` (embedded in
/// `server_t`) IS the liveness flag — no island field here is ever `Option`
/// (the `ge` seam handle predates this fill and is a separate SEAM-D2
/// concern). SP deltas vs the MP twin: **adds** `savegame` (master table
/// SP-only row); **omits** MP's `bot` (`codemp/server/sv_bot.cpp:16-23` — SP
/// has no `sv_bot.cpp`) and `master_heartbeat`/`g_lastResolveTime`
/// (`codemp/server/sv_main.cpp:192` — no master-server block exists in SP's
/// `sv_main.cpp`).
///
/// Source: `oracle/oracle/code/server/sv_main.cpp:18-20`;
/// `oracle/oracle/code/server/server.h:42-49` (state/`SS_DEAD`).
pub struct Server {
    /// Raven SP `game_export_t *ge` (`sv_main.cpp:20`) — the engine-held game
    /// export handle (the SEAM-D2 table seam; state-ownership "SP seam
    /// handle"). `None` until the game attaches. NOTE: no frozen doc signature
    /// exists for this field's placement yet (checkpoint-3 finding).
    pub ge: Option<*const game_export_t>,
    /// Raven `sv` (`server_t`, "local server") — embeds the `SS_DEAD` liveness
    /// `state`, `svEntities`, `configstrings`, `models`.
    ///
    /// Source: `oracle/oracle/code/server/sv_main.cpp:19`
    pub sv: server_t,
    /// Raven `svs` (`serverStatic_t`, "persistant server info", persists
    /// across maps) — heap `clients[]`, snapshot ring.
    ///
    /// Source: `oracle/oracle/code/server/sv_main.cpp:18`
    pub svs: serverStatic_t,
    /// SP-only save-game transition state (master table `Server.savegame` row).
    ///
    /// Source: `oracle/oracle/code/server/sv_ccmds.cpp:22`;
    /// `oracle/oracle/code/server/server.h:316-317`
    pub savegame: Savegame,
    /// Raven `sv_worldSectors[AREA_NODES]` + `sv_numworldSectors`.
    ///
    /// Source: `oracle/oracle/code/server/sv_world.cpp:82-83`
    pub world_sectors: WorldSectors,
}

/// Raven SP `SV_InitGameProgs` (`sv_game.cpp:478`): fills the `game_import_t`
/// (`:495-667`), attaches via `ge = Sys_GetGameAPI(&import)` (`:669` — ours is
/// a direct linked `GetGameAPI` call, see module docs), `Com_Error(ERR_DROP,
/// "failed to load game DLL")` on null (`:671-672`), version check (`:682-684`),
/// then `ge->Init(...)` (`:691`). Signature is a skeleton placeholder — no doc
/// freezes the SP engine-side surface yet (checkpoint-3 finding).
///
/// Source: `oracle/oracle/code/server/sv_game.cpp:478`
pub fn sv_init_game_progs(server: &mut Server) {
    let _ = server;
    todo!("Port SV_InitGameProgs — oracle/oracle/code/server/sv_game.cpp:478")
}

/// Raven SP `SV_ShutdownGameProgs` (`sv_game.cpp:403`): `ge->Shutdown()`,
/// clear the handle (our SP unloads no DLL — static attach, DEC-07).
///
/// Source: `oracle/oracle/code/server/sv_game.cpp:403`
pub fn sv_shutdown_game_progs(server: &mut Server) {
    let _ = server;
    todo!("Port SV_ShutdownGameProgs — oracle/oracle/code/server/sv_game.cpp:403")
}
