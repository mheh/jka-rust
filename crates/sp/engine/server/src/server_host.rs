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

use sp_abi::game::public::game_export_t::game_export_t;

/// The SP server-island state owned by `Engine.sv: Option<Server>`
/// (state-ownership § Server, SP mirror per DEC-04). Placeheld so the SP engine
/// facade can name it; fields (sv/svs/savegame — `qbLoadTransition`/
/// `eSavedGameJustLoaded`, `sv_ccmds.cpp:22`) are subsystem detail.
///
/// Source: `oracle/oracle/code/server/sv_main.cpp:18-19`
pub struct Server {
    /// Raven SP `game_export_t *ge` (`sv_main.cpp:20`) — the engine-held game
    /// export handle (the SEAM-D2 table seam; state-ownership "SP seam
    /// handle"). `None` until the game attaches. NOTE: no frozen doc signature
    /// exists for this field's placement yet (checkpoint-3 finding).
    pub ge: Option<*const game_export_t>,
    //TODO: Port server_t/serverStatic_t fields (SP Server island)
    // Source: oracle/oracle/code/server/sv_main.cpp:18-19
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
