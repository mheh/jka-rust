#![allow(non_camel_case_types, non_snake_case)]

/// Raven `sysEventType_t` — system event types.
///
/// Raven: bk001129 - make sure SE_NONE is zero.
/// Type definition source: `oracle/codemp/qcommon/qcommon.h:923-932`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum sysEventType_t {
    SE_NONE = 0,          // evTime is still valid
    SE_KEY = 1,           // evValue is a key code, evValue2 is the down flag
    SE_CHAR = 2,          // evValue is an ascii char
    SE_MOUSE = 3,         // evValue and evValue2 are reletive signed x / y moves
    SE_JOYSTICK_AXIS = 4, // evValue is an axis number and evValue2 is the current state (-127 to 127)
    SE_CONSOLE = 5,       // evPtr is a char*
    SE_PACKET = 6,        // evPtr is a netadr_t followed by data bytes to evPtrLength
}
