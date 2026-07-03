//! `jagame` — the SP game module cdylib shell (SEAM-D10 SP mirror; settled SP
//! mapping, `docs/handoffs/2026-07-03-skeleton-findings.md` § round-4 gate).
//!
//! SP enters via `GetGameAPI` returning a `game_export_t` of direct fn pointers
//! (`g_main.cpp:875`) — **no `vmMain`, no command decode, no `Dispatch<C>`
//! routing**. The shell hosts: the SP `ENGINE` handle (the stored
//! `game_import_t`, Raven's `gi = *import`), the SP `WORLD` cell, the static
//! `game_export_t` (Raven's `globals`, `g_main.cpp:48`), and the export shell
//! fns below. World lifetime: `ge->Init` writes the WORLD cell; `ge->Shutdown`
//! takes it — the direct analog of MP GAME_INIT-write/GAME_SHUTDOWN-take.

#![allow(non_snake_case, non_camel_case_types)]

use core::ffi::{c_char, c_int};
use std::cell::UnsafeCell;
use std::sync::OnceLock;

use sp_abi::game::public::game_export_t::game_export_t;
use sp_abi::game::public::game_import_t::game_import_t;
use sp_abi::game::public::saved_game_just_loaded_e::SavedGameJustLoaded_e;
use sp_abi::game::public::GAME_API_VERSION;
use sp_game::GameWorld;
use sp_qshared::common::sp::gentity::gentity_t;
use sp_qshared::common::sp::qcommon::usercmd::usercmd_t;
use sp_qshared::shared::qboolean;

/// The SP module-side engine handle: the by-value copy of the import table the
/// engine passes into `GetGameAPI` — Raven `gi = *import` (`g_main.cpp:878`).
/// OnceLock-style, mirroring the MP shell's `ENGINE: OnceLock<CEngine>`
/// (SEAM-D1 §B6 exception; the export fns take no context argument).
static ENGINE: EngineCell = EngineCell(OnceLock::new());

/// `game_import_t` carries a raw data-pointer member (`VoiceVolume: *mut
/// c_int`, `g_public.h`), so `OnceLock<game_import_t>` is not `Sync` on its
/// own — the same compile-forced wrapper the MP shell needed for `CEngine`
/// (skeleton finding 3; the unsafe stays local to the shell).
struct EngineCell(OnceLock<game_import_t>);

// SAFETY (Send+Sync): set once at GetGameAPI, read single-threaded per Raven's
// module contract; the pointer members are engine-owned statics.
unsafe impl Send for EngineCell {}
unsafe impl Sync for EngineCell {}

/// The SP module island's one owned `GameWorld` across export calls (STATE-D6
/// SP mirror). `None` until `ge->Init` builds it; `ge->Shutdown` takes it out.
static WORLD: WorldCell = WorldCell::new();

/// `UnsafeCell<Option<GameWorld>>` — reentrancy handled by per-export
/// raw-pointer derivation, not by this wrapper (STATE-D6 discipline).
struct WorldCell(UnsafeCell<Option<GameWorld>>);

impl WorldCell {
    const fn new() -> Self {
        WorldCell(UnsafeCell::new(None))
    }
}

// SAFETY (Sync only): the module runs single-threaded per Raven's contract;
// single-threaded *reentrant* aliasing is handled by each export deriving its
// own raw `*mut GameWorld` (STATE-D6).
unsafe impl Sync for WorldCell {}

/// Raven `game_export_t globals` (`g_main.cpp:48`) — the static export table
/// `GetGameAPI` fills and returns. `UnsafeCell` because `InitGame` later writes
/// the `gentities`/`num_entities` handoff fields through the returned pointer.
struct ExportsCell(UnsafeCell<game_export_t>);

// SAFETY (Sync only): single-threaded module contract, as WORLD above.
unsafe impl Sync for ExportsCell {}

static GLOBALS: ExportsCell = ExportsCell(UnsafeCell::new(game_export_t {
    apiversion: 0,
    Init: None,
    Shutdown: None,
    WriteLevel: None,
    ReadLevel: None,
    GameAllowedToSaveHere: None,
    ClientConnect: None,
    ClientBegin: None,
    ClientUserinfoChanged: None,
    ClientDisconnect: None,
    ClientCommand: None,
    ClientThink: None,
    RunFrame: None,
    ConnectNavs: None,
    ConsoleCommand: None,
    GameSpawnRMGEntity: None,
    gentities: core::ptr::null_mut(),
    gentitySize: 0,
    num_entities: 0,
}));

/// Raven `GetGameAPI` (`g_main.cpp:875-916`): copies the import table by value
/// (`gi = *import`, `:878`), fills every export member + `gentitySize =
/// sizeof(gentity_t)` (`:880-905`), and returns `&globals` (`:916`).
/// `extern "C-unwind"` per the frozen engine-seam export block (SEAM-D12).
///
/// # Safety
/// `import` must point at a live, fully-populated `game_import_t` for the
/// duration of the call (the engine's `SV_InitGameProgs` stack value).
#[no_mangle]
pub unsafe extern "C-unwind" fn GetGameAPI(import: *const game_import_t) -> *const game_export_t {
    // Raven `gi = *import` (g_main.cpp:878): the one by-value copy, stored once.
    ENGINE.0.set(core::ptr::read(import)).ok();

    // Table fill mirrors g_main.cpp:880-905 member-for-member (Raven's
    // commented-out PrintEntClassname/ValidateAnimRange members are dead and
    // omitted from the ported struct).
    *GLOBALS.0.get() = game_export_t {
        apiversion: GAME_API_VERSION,
        Init: Some(init),
        Shutdown: Some(shutdown),
        WriteLevel: Some(write_level),
        ReadLevel: Some(read_level),
        GameAllowedToSaveHere: Some(game_allowed_to_save_here),
        ClientConnect: Some(client_connect),
        ClientBegin: Some(client_begin),
        ClientUserinfoChanged: Some(client_userinfo_changed),
        ClientDisconnect: Some(client_disconnect),
        ClientCommand: Some(client_command),
        ClientThink: Some(client_think),
        RunFrame: Some(run_frame),
        ConnectNavs: Some(connect_navs),
        ConsoleCommand: Some(console_command),
        GameSpawnRMGEntity: Some(game_spawn_rmg_entity),
        gentities: core::ptr::null_mut(),
        gentitySize: core::mem::size_of::<gentity_t>() as c_int,
        num_entities: 0,
    };
    //TODO: Port GI_Init + gameinfo_import_t wiring
    // Source: oracle/oracle/code/game/g_main.cpp:906-914

    GLOBALS.0.get() as *const game_export_t
}

// ---------------------------------------------------------------------------
// Export shell fns (the game_export_t members). Per the settled SP mapping,
// EACH export derives its own `*mut GameWorld` from WORLD in its prologue and
// constructs the sp_game::GameContext itself (per-export construction — no
// shared router). Prologue shape every non-Init body opens with:
//
//     let world = unsafe { (*WORLD.0.get()).as_mut().expect("ge->Init built the world") }
//         as *mut GameWorld;
//     let ctx = sp_game::GameContext::new(world, ENGINE.get().expect("GetGameAPI set ENGINE"));
//
// `init` WRITES the cell first (GameWorld::zeroed, STATE-D9) then delegates;
// `shutdown` takes the world OUT after its delegation returns (drop = teardown).
// All bodies are skeleton todo!()s delegating to sp_game logic fns.
// ---------------------------------------------------------------------------

/// Raven `InitGame` (`g_main.cpp:696`). Writes the WORLD cell
/// (`GameWorld::zeroed`) then runs the init logic against it.
unsafe extern "C-unwind" fn init(
    mapname: *const c_char,
    spawntarget: *const c_char,
    checkSum: c_int,
    entstring: *const c_char,
    levelTime: c_int,
    randomSeed: c_int,
    globalTime: c_int,
    eSavedGameJustLoaded: SavedGameJustLoaded_e,
    qbLoadTransition: qboolean,
) {
    let _ = (
        mapname,
        spawntarget,
        checkSum,
        entstring,
        levelTime,
        randomSeed,
        globalTime,
        eSavedGameJustLoaded,
        qbLoadTransition,
    );
    let _ = (&ENGINE, WORLD.0.get());
    todo!("Port InitGame — oracle/oracle/code/game/g_main.cpp:696")
}

/// Raven `ShutdownGame` (`g_main.cpp:806`). Takes the world OUT of the cell
/// after its delegation returns (module-unload lifetime; drop runs teardown).
unsafe extern "C-unwind" fn shutdown() {
    todo!("Port ShutdownGame — oracle/oracle/code/game/g_main.cpp:806")
}

/// Raven `WriteLevel` (`g_savegame.cpp:1131`).
unsafe extern "C-unwind" fn write_level(qbAutosave: qboolean) {
    let _ = qbAutosave;
    todo!("Port WriteLevel — oracle/oracle/code/game/g_savegame.cpp:1131")
}

/// Raven `ReadLevel` (`g_savegame.cpp:1162`).
unsafe extern "C-unwind" fn read_level(qbAutosave: qboolean, qbLoadTransition: qboolean) {
    let _ = (qbAutosave, qbLoadTransition);
    todo!("Port ReadLevel — oracle/oracle/code/game/g_savegame.cpp:1162")
}

/// Raven `GameAllowedToSaveHere` (`g_savegame.cpp:1228`).
unsafe extern "C-unwind" fn game_allowed_to_save_here() -> qboolean {
    todo!("Port GameAllowedToSaveHere — oracle/oracle/code/game/g_savegame.cpp:1228")
}

/// Raven `ClientConnect` (`g_client.cpp:505`).
unsafe extern "C-unwind" fn client_connect(
    clientNum: c_int,
    firstTime: qboolean,
    eSavedGameJustLoaded: SavedGameJustLoaded_e,
) -> *mut c_char {
    let _ = (clientNum, firstTime, eSavedGameJustLoaded);
    todo!("Port ClientConnect — oracle/oracle/code/game/g_client.cpp:505")
}

/// Raven `ClientBegin` (`g_client.cpp:569`).
unsafe extern "C-unwind" fn client_begin(
    clientNum: c_int,
    cmd: *mut usercmd_t,
    eSavedGameJustLoaded: SavedGameJustLoaded_e,
) {
    let _ = (clientNum, cmd, eSavedGameJustLoaded);
    todo!("Port ClientBegin — oracle/oracle/code/game/g_client.cpp:569")
}

/// Raven `ClientUserinfoChanged` (`g_client.cpp:416`).
unsafe extern "C-unwind" fn client_userinfo_changed(clientNum: c_int) {
    let _ = clientNum;
    todo!("Port ClientUserinfoChanged — oracle/oracle/code/game/g_client.cpp:416")
}

/// Raven `ClientDisconnect` (`g_client.cpp:2449`).
unsafe extern "C-unwind" fn client_disconnect(clientNum: c_int) {
    let _ = clientNum;
    todo!("Port ClientDisconnect — oracle/oracle/code/game/g_client.cpp:2449")
}

/// Raven `ClientCommand` (`g_cmds.cpp:1368`).
unsafe extern "C-unwind" fn client_command(clientNum: c_int) {
    let _ = clientNum;
    todo!("Port ClientCommand — oracle/oracle/code/game/g_cmds.cpp:1368")
}

/// Raven `ClientThink` (`g_active.cpp:5615`).
unsafe extern "C-unwind" fn client_think(clientNum: c_int, cmd: *mut usercmd_t) {
    let _ = (clientNum, cmd);
    todo!("Port ClientThink — oracle/oracle/code/game/g_active.cpp:5615")
}

/// Raven `G_RunFrame` (`g_main.cpp:1895`).
unsafe extern "C-unwind" fn run_frame(levelTime: c_int) {
    let _ = levelTime;
    todo!("Port G_RunFrame — oracle/oracle/code/game/g_main.cpp:1895")
}

/// Raven `G_ConnectNavs` (`g_main.cpp:503`).
unsafe extern "C-unwind" fn connect_navs(mapname: *const c_char, checkSum: c_int) {
    let _ = (mapname, checkSum);
    todo!("Port G_ConnectNavs — oracle/oracle/code/game/g_main.cpp:503")
}

/// Raven `ConsoleCommand` (`g_svcmds.cpp:1005`).
unsafe extern "C-unwind" fn console_command() -> qboolean {
    todo!("Port ConsoleCommand — oracle/oracle/code/game/g_svcmds.cpp:1005")
}

/// Raven `G_GameSpawnRMGEntity` (`g_main.cpp:857`).
unsafe extern "C-unwind" fn game_spawn_rmg_entity(s: *mut c_char) {
    let _ = s;
    todo!("Port G_GameSpawnRMGEntity — oracle/oracle/code/game/g_main.cpp:857")
}
