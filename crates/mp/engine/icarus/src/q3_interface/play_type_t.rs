#![allow(non_camel_case_types, non_snake_case)]

/// Raven `playType_t` — Types of file to play.
///
/// Type definition source: `oracle/oracle/codemp/icarus/Q3_Interface.h:261-269`
#[repr(i32)]
pub enum playType_t {
	/// Play a ROFF file
	PLAY_ROFF = 0,

	PLAY_NUMBEROF,
}
