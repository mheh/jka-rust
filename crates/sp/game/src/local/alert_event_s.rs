#![allow(non_camel_case_types, non_snake_case)]

use sp_qshared::common::sp::gentity::gentity_t;
use sp_qshared::shared::{qboolean, vec3_t};

use super::alert_event_level_e::alertEventLevel_e;
use super::alert_event_type_e::alertEventType_e;

/// Raven `alertEvent_s` — an AI awareness alert event (sight/sound).
///
/// Type definition source: `oracle/code/game/g_local.h:125-137`
#[repr(C)]
pub struct alertEvent_t {
    /// Where the event is located
    pub position: vec3_t,
    /// Consideration radius
    pub radius: f32,
    /// Priority level of the event
    pub level: alertEventLevel_e,
    /// Event type (sound,sight)
    pub r#type: alertEventType_e,
    /// Who made the sound
    pub owner: *mut gentity_t,
    /// ambient light level at point
    pub light: f32,
    /// additional light- makes it more noticable, even in darkness
    pub addLight: f32,
    /// unique... if get a ridiculous number, this will repeat, but should not be a problem as it's just comparing it to your lastAlertID
    pub ID: i32,
    /// when it was created
    pub timestamp: i32,
    /// alert is on the ground (only used for sounds)
    pub onGround: qboolean,
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<alertEvent_t>() == 56);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(alertEvent_t, position) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(alertEvent_t, radius) == 12);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(alertEvent_t, level) == 16);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(alertEvent_t, r#type) == 20);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(alertEvent_t, owner) == 24);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(alertEvent_t, light) == 32);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(alertEvent_t, addLight) == 36);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(alertEvent_t, ID) == 40);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(alertEvent_t, timestamp) == 44);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(alertEvent_t, onGround) == 48);
