#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

use mp_qshared::common::mp::qcommon::player_state::MAX_PS_EVENTS;

/// Raven `boteventtracker_t` — tracks the last player-state events seen by a bot.
///
/// Type definition source: `oracle/oracle/codemp/game/ai_main.h:130-135`
#[repr(C)]
pub struct boteventtracker_t {
	pub eventSequence: c_int,
	pub events: [c_int; MAX_PS_EVENTS],
	pub eventTime: f32,
}

const _: () = assert!(core::mem::size_of::<boteventtracker_t>() == 16);
const _: () = assert!(core::mem::offset_of!(boteventtracker_t, eventSequence) == 0);
const _: () = assert!(core::mem::offset_of!(boteventtracker_t, events) == 4);
const _: () = assert!(core::mem::offset_of!(boteventtracker_t, eventTime) == 12);
