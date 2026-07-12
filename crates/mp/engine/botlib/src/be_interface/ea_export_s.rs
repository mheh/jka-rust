#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_int};

use crate::BotLib;
use mp_qshared::common::mp::botlib::bot_input_s::bot_input_t;
use mp_qshared::shared::vec3_t;

/// Raven `ea_export_t` — elementary bot action function table.
///
/// Type definition source: `oracle/codemp/game/botlib.h:255-287`
//
// Engine-internal per the 2026-07-11 ruling: statically linked in jampDed, no
// ABI crossing, layout free. Fn-pointer fields carry the ported `&mut BotLib`
// receiver (the stored fn's real signature is LAW).
pub struct ea_export_t {
    // ClientCommand elementary actions
    pub EA_Command: Option<fn(bot: &mut BotLib, client: c_int, command: *mut c_char)>,
    pub EA_Say: Option<fn(bot: &mut BotLib, client: c_int, str: *mut c_char)>,
    pub EA_SayTeam: Option<fn(bot: &mut BotLib, client: c_int, str: *mut c_char)>,
    //
    pub EA_Action: Option<fn(bot: &mut BotLib, client: c_int, action: c_int)>,
    pub EA_Gesture: Option<fn(bot: &mut BotLib, client: c_int)>,
    pub EA_Talk: Option<fn(bot: &mut BotLib, client: c_int)>,
    pub EA_Attack: Option<fn(bot: &mut BotLib, client: c_int)>,
    pub EA_Use: Option<fn(bot: &mut BotLib, client: c_int)>,
    pub EA_Respawn: Option<fn(bot: &mut BotLib, client: c_int)>,
    pub EA_MoveUp: Option<fn(bot: &mut BotLib, client: c_int)>,
    pub EA_MoveDown: Option<fn(bot: &mut BotLib, client: c_int)>,
    pub EA_MoveForward: Option<fn(bot: &mut BotLib, client: c_int)>,
    pub EA_MoveBack: Option<fn(bot: &mut BotLib, client: c_int)>,
    pub EA_MoveLeft: Option<fn(bot: &mut BotLib, client: c_int)>,
    pub EA_MoveRight: Option<fn(bot: &mut BotLib, client: c_int)>,
    pub EA_Crouch: Option<fn(bot: &mut BotLib, client: c_int)>,
    pub EA_Alt_Attack: Option<fn(bot: &mut BotLib, client: c_int)>,
    pub EA_ForcePower: Option<fn(bot: &mut BotLib, client: c_int)>,

    pub EA_SelectWeapon: Option<fn(bot: &mut BotLib, client: c_int, weapon: c_int)>,
    pub EA_Jump: Option<fn(bot: &mut BotLib, client: c_int)>,
    pub EA_DelayedJump: Option<fn(bot: &mut BotLib, client: c_int)>,
    pub EA_Move: Option<fn(bot: &mut BotLib, client: c_int, dir: vec3_t, speed: f32)>,
    pub EA_View: Option<fn(bot: &mut BotLib, client: c_int, viewangles: vec3_t)>,
    // send regular input to the server
    pub EA_EndRegular: Option<fn(client: c_int, thinktime: f32)>,
    pub EA_GetInput:
        Option<fn(bot: &mut BotLib, client: c_int, thinktime: f32, input: *mut bot_input_t)>,
    pub EA_ResetInput: Option<fn(bot: &mut BotLib, client: c_int)>,
}

pub type ea_export_s = ea_export_t;
