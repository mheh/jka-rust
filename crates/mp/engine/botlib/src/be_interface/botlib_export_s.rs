#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_int};

use crate::BotLib;
use mp_qshared::common::mp::botlib::bot_entitystate_s::bot_entitystate_t;
use mp_qshared::shared::{pc_token_t, vec3_t};

use super::aas_export_s::aas_export_t;
use super::ai_export_s::ai_export_t;
use super::ea_export_s::ea_export_t;

/// Raven `botlib_export_t` — top-level botlib export table (AAS/EA/AI function
/// tables plus setup/shutdown/frame/config entry points) the engine hands to
/// the game module.
///
/// Raven: (unnamed).
/// Type definition source: `oracle/codemp/game/botlib.h:388-422`
//
// Engine-internal per the 2026-07-11 ruling: statically linked in jampDed, no
// ABI crossing, layout free. Fn-pointer fields carry the ported `&mut BotLib`
// receiver (the stored fn's real signature is LAW).
pub struct botlib_export_s {
    //Area Awareness System functions
    pub aas: aas_export_t,
    //Elementary Action functions
    pub ea: ea_export_t,
    //AI functions
    pub ai: ai_export_t,
    //setup the bot library, returns BLERR_
    pub BotLibSetup: Option<fn(bot: &mut BotLib) -> c_int>,
    //shutdown the bot library, returns BLERR_
    pub BotLibShutdown: Option<fn(bot: &mut BotLib) -> c_int>,
    //sets a library variable returns BLERR_
    pub BotLibVarSet:
        Option<fn(bot: &mut BotLib, var_name: *mut c_char, value: *mut c_char) -> c_int>,
    //gets a library variable returns BLERR_
    pub BotLibVarGet: Option<
        fn(bot: &mut BotLib, var_name: *mut c_char, value: *mut c_char, size: c_int) -> c_int,
    >,

    //sets a C-like define returns BLERR_
    pub PC_AddGlobalDefine: Option<fn(string: *mut c_char) -> c_int>,
    pub PC_LoadSourceHandle: Option<fn(bot: &mut BotLib, filename: *const c_char) -> c_int>,
    pub PC_FreeSourceHandle: Option<fn(bot: &mut BotLib, handle: c_int) -> c_int>,
    pub PC_ReadTokenHandle:
        Option<fn(bot: &mut BotLib, handle: c_int, pc_token: *mut pc_token_t) -> c_int>,
    pub PC_SourceFileAndLine: Option<
        fn(bot: &mut BotLib, handle: c_int, filename: *mut c_char, line: *mut c_int) -> c_int,
    >,
    pub PC_LoadGlobalDefines: Option<fn(bot: &mut BotLib, filename: *const c_char) -> c_int>,
    pub PC_RemoveAllGlobalDefines: Option<fn(bot: &mut BotLib)>,

    //start a frame in the bot library
    pub BotLibStartFrame: Option<fn(bot: &mut BotLib, time: f32) -> c_int>,
    //load a new map in the bot library
    pub BotLibLoadMap: Option<fn(bot: &mut BotLib, mapname: *const c_char) -> c_int>,
    //entity updates
    pub BotLibUpdateEntity:
        Option<fn(bot: &mut BotLib, ent: c_int, state: *mut bot_entitystate_t) -> c_int>,
    //just for testing
    pub Test: Option<fn(parm0: c_int, parm1: *mut c_char, parm2: vec3_t, parm3: vec3_t) -> c_int>,
}

/// Raven `botlib_export_t` typedef alias.
pub type botlib_export_t = botlib_export_s;
