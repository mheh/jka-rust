#![allow(non_camel_case_types, non_snake_case)]

/// Raven `joystickAxis_t` — joystick axis enumeration.
///
/// Raven: .
/// Type definition source: `oracle/code/qcommon/qcommon.h:724-732`
#[repr(i32)]
pub enum joystickAxis_t {
    AXIS_SIDE,
    AXIS_FORWARD,
    AXIS_UP,
    AXIS_ROLL,
    AXIS_YAW,
    AXIS_PITCH,
    MAX_JOYSTICK_AXIS,
}
