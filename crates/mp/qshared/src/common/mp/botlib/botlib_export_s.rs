#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_float, c_int};

use crate::shared::{pc_token_t, vec3_t};

use super::aas_export_s::aas_export_t;
use super::ai_export_s::ai_export_t;
use super::bot_entitystate_s::bot_entitystate_t;
use super::ea_export_s::ea_export_t;

/// Raven `botlib_export_t` — top-level botlib export table (AAS/EA/AI function
/// tables plus setup/shutdown/frame/config entry points) the engine hands to
/// the game module.
///
/// Raven: (unnamed).
/// Type definition source: `oracle/oracle/codemp/game/botlib.h:388-422`
#[repr(C)]
pub struct botlib_export_s {
    //Area Awareness System functions
    pub aas: aas_export_t,
    //Elementary Action functions
    pub ea: ea_export_t,
    //AI functions
    pub ai: ai_export_t,
    //setup the bot library, returns BLERR_
    pub BotLibSetup: Option<unsafe extern "C" fn() -> c_int>,
    //shutdown the bot library, returns BLERR_
    pub BotLibShutdown: Option<unsafe extern "C" fn() -> c_int>,
    //sets a library variable returns BLERR_
    pub BotLibVarSet: Option<unsafe extern "C" fn(var_name: *mut c_char, value: *mut c_char) -> c_int>,
    //gets a library variable returns BLERR_
    pub BotLibVarGet:
        Option<unsafe extern "C" fn(var_name: *mut c_char, value: *mut c_char, size: c_int) -> c_int>,

    //sets a C-like define returns BLERR_
    pub PC_AddGlobalDefine: Option<unsafe extern "C" fn(string: *mut c_char) -> c_int>,
    pub PC_LoadSourceHandle: Option<unsafe extern "C" fn(filename: *const c_char) -> c_int>,
    pub PC_FreeSourceHandle: Option<unsafe extern "C" fn(handle: c_int) -> c_int>,
    pub PC_ReadTokenHandle: Option<unsafe extern "C" fn(handle: c_int, pc_token: *mut pc_token_t) -> c_int>,
    pub PC_SourceFileAndLine:
        Option<unsafe extern "C" fn(handle: c_int, filename: *mut c_char, line: *mut c_int) -> c_int>,
    pub PC_LoadGlobalDefines: Option<unsafe extern "C" fn(filename: *const c_char) -> c_int>,
    pub PC_RemoveAllGlobalDefines: Option<unsafe extern "C" fn()>,

    //start a frame in the bot library
    pub BotLibStartFrame: Option<unsafe extern "C" fn(time: c_float) -> c_int>,
    //load a new map in the bot library
    pub BotLibLoadMap: Option<unsafe extern "C" fn(mapname: *const c_char) -> c_int>,
    //entity updates
    pub BotLibUpdateEntity:
        Option<unsafe extern "C" fn(ent: c_int, state: *mut bot_entitystate_t) -> c_int>,
    //just for testing
    pub Test: Option<
        unsafe extern "C" fn(parm0: c_int, parm1: *mut c_char, parm2: *mut vec3_t, parm3: *mut vec3_t) -> c_int,
    >,
}

/// Raven `botlib_export_t` typedef alias.
pub type botlib_export_t = botlib_export_s;

const _: () = assert!(core::mem::size_of::<botlib_export_t>() == 1104);
const _: () = assert!(core::mem::offset_of!(botlib_export_t, aas) == 0);
const _: () = assert!(core::mem::offset_of!(botlib_export_t, ea) == 176);
const _: () = assert!(core::mem::offset_of!(botlib_export_t, ai) == 384);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(botlib_export_t, BotLibSetup) == 984);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(botlib_export_t, BotLibShutdown) == 992);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(botlib_export_t, BotLibVarSet) == 1000);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(botlib_export_t, BotLibVarGet) == 1008);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(botlib_export_t, PC_AddGlobalDefine) == 1016);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(botlib_export_t, PC_LoadSourceHandle) == 1024);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(botlib_export_t, PC_FreeSourceHandle) == 1032);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(botlib_export_t, PC_ReadTokenHandle) == 1040);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(botlib_export_t, PC_SourceFileAndLine) == 1048);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(botlib_export_t, PC_LoadGlobalDefines) == 1056);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(botlib_export_t, PC_RemoveAllGlobalDefines) == 1064);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(botlib_export_t, BotLibStartFrame) == 1072);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(botlib_export_t, BotLibLoadMap) == 1080);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(botlib_export_t, BotLibUpdateEntity) == 1088);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(botlib_export_t, Test) == 1096);
