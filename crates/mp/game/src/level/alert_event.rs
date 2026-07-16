//! MP `alertEvent_t` and its enums.
//!
//! Type definition source: `oracle/codemp/game/g_local.h:778-805`

#![allow(non_camel_case_types)]

use core::ffi::c_int;

use crate::entity::gentity_t;
use mp_qshared::shared::vec3_t;

/// Raven `MAX_ALERT_EVENTS`. Source: `oracle/codemp/game/g_local.h:777`
pub const MAX_ALERT_EVENTS: usize = 32;

/// Raven `alertEventType_e` (named `typedef enum`).
///
/// Type definition source: `oracle/codemp/game/g_local.h:778-783`
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum alertEventType_e {
    AET_SIGHT = 0,
    AET_SOUND,
}

/// Raven `alertEventLevel_e` (named `typedef enum`).
///
/// Type definition source: `oracle/codemp/game/g_local.h:785-792`
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum alertEventLevel_e {
    AEL_MINOR = 0,    // Enemy responds to the sound, but only by looking
    AEL_SUSPICIOUS,   // Enemy looks at the sound, and will also investigate it
    AEL_DISCOVERED,   // Enemy knows the player is around, and will actively hunt
    AEL_DANGER,       // Enemy should try to find cover
    AEL_DANGER_GREAT, // Enemy should run like hell!
}

/// Raven `alertEvent_t`. Pointer-bearing (`owner`) => arch-dependent.
///
/// Type definition source: `oracle/codemp/game/g_local.h:794-805`
#[repr(C)]
#[derive(Clone, Copy)]
pub struct alertEvent_t {
    pub position: vec3_t,         // Where the event is located
    pub radius: f32,              // Consideration radius
    pub level: alertEventLevel_e, // Priority level of the event
    pub r#type: alertEventType_e, // Event type (sound, sight)
    pub owner: *mut gentity_t,    // Who made the sound
    pub light: f32,               // ambient light level at point
    pub addLight: f32,            // additional light — more noticeable, even in darkness
    pub ID: c_int,                // unique id (wraps, but only used for comparison)
    pub timestamp: c_int,         // when it was created
}
// Manual `Default` (not `derive`) since `alertEventLevel_e`/`alertEventType_e`
// don't derive it; zero-valued first variants match Raven's zero-init idiom.
impl Default for alertEvent_t {
    fn default() -> Self {
        alertEvent_t {
            position: [0.0, 0.0, 0.0],
            radius: 0.0,
            level: alertEventLevel_e::AEL_MINOR,
            r#type: alertEventType_e::AET_SIGHT,
            owner: core::ptr::null_mut(),
            light: 0.0,
            addLight: 0.0,
            ID: 0,
            timestamp: 0,
        }
    }
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<alertEvent_t>() == 48);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(alertEvent_t, owner) == 24);
