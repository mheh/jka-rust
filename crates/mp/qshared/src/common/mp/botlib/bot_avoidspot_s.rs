#![allow(non_camel_case_types, non_snake_case)]

use crate::shared::vec3_t;

/// Raven `bot_avoidspot_t` — a spot the bot movement code should avoid.
///
/// Type definition source: `oracle/codemp/game/be_ai_move.h:89-94`
#[repr(C)]
pub struct bot_avoidspot_t {
	pub origin: vec3_t,
	pub radius: f32,
	pub r#type: i32,
}

pub type bot_avoidspot_s = bot_avoidspot_t;

const _: () = assert!(core::mem::size_of::<bot_avoidspot_t>() == 20);
const _: () = assert!(core::mem::offset_of!(bot_avoidspot_t, origin) == 0);
const _: () = assert!(core::mem::offset_of!(bot_avoidspot_t, radius) == 12);
const _: () = assert!(core::mem::offset_of!(bot_avoidspot_t, r#type) == 16);
