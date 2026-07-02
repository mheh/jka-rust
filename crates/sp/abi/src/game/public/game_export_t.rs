#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_int};

use sp_qshared::common::sp::gentity::gentity_t;
use sp_qshared::common::sp::qcommon::usercmd::usercmd_t;
use sp_qshared::shared::qboolean;

use super::saved_game_just_loaded_e::SavedGameJustLoaded_e;

/// Raven `game_export_t` — function table exported by the SP game DLL to the engine.
///
/// Raven: global variables shared between game and server. The gentities array is
/// allocated in the game dll so it can vary in size from one game to another. The
/// size will be fixed when ge->Init() is called; the server can't just use pointer
/// arithmetic on gentities, because the server's sizeof(struct gentity_s) doesn't
/// equal gentitySize.
/// Type definition source: `oracle/oracle/code/game/g_public.h:476-527`
#[repr(C)]
pub struct game_export_t {
    pub apiversion: c_int,

    // init and shutdown will be called every single level
    // levelTime will be near zero, while globalTime will be a large number
    // that can be used to track spectator entry times across restarts
    //TODO: Port Init variadic-free but multi-arg pointer signature
    // Source: oracle/oracle/code/game/g_public.h:482
    pub Init: Option<
        unsafe extern "C" fn(
            mapname: *const c_char,
            spawntarget: *const c_char,
            checkSum: c_int,
            entstring: *const c_char,
            levelTime: c_int,
            randomSeed: c_int,
            globalTime: c_int,
            eSavedGameJustLoaded: SavedGameJustLoaded_e,
            qbLoadTransition: qboolean,
        ),
    >,
    pub Shutdown: Option<unsafe extern "C" fn()>,

    // ReadLevel is called after the default map information has been
    // loaded with SpawnEntities
    pub WriteLevel: Option<unsafe extern "C" fn(qbAutosave: qboolean)>,
    pub ReadLevel: Option<unsafe extern "C" fn(qbAutosave: qboolean, qbLoadTransition: qboolean)>,
    pub GameAllowedToSaveHere: Option<unsafe extern "C" fn() -> qboolean>,

    // return NULL if the client is allowed to connect, otherwise return
    // a text string with the reason for denial
    pub ClientConnect: Option<
        unsafe extern "C" fn(
            clientNum: c_int,
            firstTime: qboolean,
            eSavedGameJustLoaded: SavedGameJustLoaded_e,
        ) -> *mut c_char,
    >,

    pub ClientBegin: Option<
        unsafe extern "C" fn(
            clientNum: c_int,
            cmd: *mut usercmd_t,
            eSavedGameJustLoaded: SavedGameJustLoaded_e,
        ),
    >,
    pub ClientUserinfoChanged: Option<unsafe extern "C" fn(clientNum: c_int)>,
    pub ClientDisconnect: Option<unsafe extern "C" fn(clientNum: c_int)>,
    pub ClientCommand: Option<unsafe extern "C" fn(clientNum: c_int)>,
    pub ClientThink: Option<unsafe extern "C" fn(clientNum: c_int, cmd: *mut usercmd_t)>,

    pub RunFrame: Option<unsafe extern "C" fn(levelTime: c_int)>,
    pub ConnectNavs: Option<unsafe extern "C" fn(mapname: *const c_char, checkSum: c_int)>,

    // ConsoleCommand will be called when a command has been issued
    // that is not recognized as a builtin function.
    // The game can issue gi.argc() / gi.argv() commands to get the command
    // and parameters.  Return qfalse if the game doesn't recognize it as a command.
    pub ConsoleCommand: Option<unsafe extern "C" fn() -> qboolean>,

    // Raven's commented-out `PrintEntClassname`/`ValidateAnimRange` members are dead
    // code (`//void (*PrintEntClassname)(...)`, `//int (*ValidateAnimRange)(...)`)
    // and contribute no layout; omitted here.
    pub GameSpawnRMGEntity: Option<unsafe extern "C" fn(s: *mut c_char)>,

    //
    // global variables shared between game and server
    //

    // The gentities array is allocated in the game dll so it
    // can vary in size from one game to another.
    //
    // The size will be fixed when ge->Init() is called
    // the server can't just use pointer arithmetic on gentities, because the
    // server's sizeof(struct gentity_s) doesn't equal gentitySize
    pub gentities: *mut gentity_t,
    pub gentitySize: c_int,
    pub num_entities: c_int, // current number, <= MAX_GENTITIES
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<game_export_t>() == 144);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_export_t, apiversion) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_export_t, Init) == 8);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_export_t, Shutdown) == 16);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_export_t, WriteLevel) == 24);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_export_t, ReadLevel) == 32);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_export_t, GameAllowedToSaveHere) == 40);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_export_t, ClientConnect) == 48);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_export_t, ClientBegin) == 56);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_export_t, ClientUserinfoChanged) == 64);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_export_t, ClientDisconnect) == 72);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_export_t, ClientCommand) == 80);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_export_t, ClientThink) == 88);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_export_t, RunFrame) == 96);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_export_t, ConnectNavs) == 104);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_export_t, ConsoleCommand) == 112);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_export_t, GameSpawnRMGEntity) == 120);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_export_t, gentities) == 128);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_export_t, gentitySize) == 136);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_export_t, num_entities) == 140);
