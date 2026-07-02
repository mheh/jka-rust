#![allow(non_camel_case_types, non_snake_case)]

/// Raven `sysEventType_t` — system event type enumeration.
///
/// Raven: .
/// Type definition source: `oracle/oracle/code/qcommon/qcommon.h:734-742`
#[repr(i32)]
pub enum sysEventType_t {
	SE_NONE,			// evTime is still valid
	SE_KEY,				// evValue is a key code, evValue2 is the down flag
	SE_CHAR,			// evValue is an ascii char
	SE_MOUSE,			// evValue and evValue2 are reletive signed x / y moves
	SE_JOYSTICK_AXIS,	// evValue is an axis number and evValue2 is the current state (-127 to 127)
	SE_CONSOLE,			// evPtr is a char*
	SE_PACKET,			// evPtr is a netadr_t followed by data bytes to evPtrLength
}
