#![allow(non_camel_case_types, non_snake_case)]

/// Raven `joystickAxis_t` — joystick axis indices.
///
/// Type definition source: `oracle/oracle/codemp/qcommon/qcommon.h:913-921`
#[repr(i32)]
pub enum joystickAxis_t {
    AXIS_SIDE = 0,
    AXIS_FORWARD = 1,
    AXIS_UP = 2,
    AXIS_ROLL = 3,
    AXIS_YAW = 4,
    AXIS_PITCH = 5,
    MAX_JOYSTICK_AXIS = 6,
}
