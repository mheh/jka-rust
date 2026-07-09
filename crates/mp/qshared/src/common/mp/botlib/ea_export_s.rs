#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_float, c_int};

use crate::shared::vec3_t;

use super::bot_input_s::bot_input_t;

/// Raven `ea_export_t` — elementary bot action function table.
///
/// Type definition source: `oracle/codemp/game/botlib.h:255-287`
#[repr(C)]
pub struct ea_export_t {
    // ClientCommand elementary actions
    pub EA_Command: Option<unsafe extern "C" fn(client: c_int, command: *mut c_char)>,
    pub EA_Say: Option<unsafe extern "C" fn(client: c_int, str_: *mut c_char)>,
    pub EA_SayTeam: Option<unsafe extern "C" fn(client: c_int, str_: *mut c_char)>,
    //
    pub EA_Action: Option<unsafe extern "C" fn(client: c_int, action: c_int)>,
    pub EA_Gesture: Option<unsafe extern "C" fn(client: c_int)>,
    pub EA_Talk: Option<unsafe extern "C" fn(client: c_int)>,
    pub EA_Attack: Option<unsafe extern "C" fn(client: c_int)>,
    pub EA_Use: Option<unsafe extern "C" fn(client: c_int)>,
    pub EA_Respawn: Option<unsafe extern "C" fn(client: c_int)>,
    pub EA_MoveUp: Option<unsafe extern "C" fn(client: c_int)>,
    pub EA_MoveDown: Option<unsafe extern "C" fn(client: c_int)>,
    pub EA_MoveForward: Option<unsafe extern "C" fn(client: c_int)>,
    pub EA_MoveBack: Option<unsafe extern "C" fn(client: c_int)>,
    pub EA_MoveLeft: Option<unsafe extern "C" fn(client: c_int)>,
    pub EA_MoveRight: Option<unsafe extern "C" fn(client: c_int)>,
    pub EA_Crouch: Option<unsafe extern "C" fn(client: c_int)>,
    pub EA_Alt_Attack: Option<unsafe extern "C" fn(client: c_int)>,
    pub EA_ForcePower: Option<unsafe extern "C" fn(client: c_int)>,

    pub EA_SelectWeapon: Option<unsafe extern "C" fn(client: c_int, weapon: c_int)>,
    pub EA_Jump: Option<unsafe extern "C" fn(client: c_int)>,
    pub EA_DelayedJump: Option<unsafe extern "C" fn(client: c_int)>,
    pub EA_Move: Option<unsafe extern "C" fn(client: c_int, dir: vec3_t, speed: c_float)>,
    pub EA_View: Option<unsafe extern "C" fn(client: c_int, viewangles: vec3_t)>,
    // send regular input to the server
    pub EA_EndRegular: Option<unsafe extern "C" fn(client: c_int, thinktime: c_float)>,
    pub EA_GetInput:
        Option<unsafe extern "C" fn(client: c_int, thinktime: c_float, input: *mut bot_input_t)>,
    pub EA_ResetInput: Option<unsafe extern "C" fn(client: c_int)>,
}

pub type ea_export_s = ea_export_t;

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<ea_export_t>() == 208);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ea_export_t, EA_Command) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ea_export_t, EA_Say) == 8);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ea_export_t, EA_SayTeam) == 16);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ea_export_t, EA_Action) == 24);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ea_export_t, EA_Gesture) == 32);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ea_export_t, EA_Talk) == 40);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ea_export_t, EA_Attack) == 48);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ea_export_t, EA_Use) == 56);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ea_export_t, EA_Respawn) == 64);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ea_export_t, EA_MoveUp) == 72);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ea_export_t, EA_MoveDown) == 80);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ea_export_t, EA_MoveForward) == 88);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ea_export_t, EA_MoveBack) == 96);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ea_export_t, EA_MoveLeft) == 104);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ea_export_t, EA_MoveRight) == 112);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ea_export_t, EA_Crouch) == 120);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ea_export_t, EA_Alt_Attack) == 128);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ea_export_t, EA_ForcePower) == 136);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ea_export_t, EA_SelectWeapon) == 144);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ea_export_t, EA_Jump) == 152);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ea_export_t, EA_DelayedJump) == 160);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ea_export_t, EA_Move) == 168);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ea_export_t, EA_View) == 176);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ea_export_t, EA_EndRegular) == 184);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ea_export_t, EA_GetInput) == 192);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ea_export_t, EA_ResetInput) == 200);
