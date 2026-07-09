#![allow(non_camel_case_types, non_snake_case)]

use sp_qshared::shared::vec3_t;

/// Raven `Muzzle` — per-muzzle runtime firing position/direction/timing state.
///
/// Type definition source: `oracle/code/game/G_Vehicles.h:443-456`
#[repr(C)]
pub struct Muzzle {
	// These are updated every frame and represent the current position and direction for the specific muzzle.
	pub m_vMuzzlePos: vec3_t,
	pub m_vMuzzleDir: vec3_t,

	// This is how long to wait before being able to fire a specific muzzle again. This is based on the firing rate
	// so that a firing rate of 10 rounds/sec would make this value initially 100 miliseconds.
	pub m_iMuzzleWait: i32,

	// whether this Muzzle was just fired or not (reset at muzzle flash code).
	pub m_bFired: bool,
}

const _: () = assert!(core::mem::size_of::<Muzzle>() == 32);
const _: () = assert!(core::mem::offset_of!(Muzzle, m_vMuzzlePos) == 0);
const _: () = assert!(core::mem::offset_of!(Muzzle, m_vMuzzleDir) == 12);
const _: () = assert!(core::mem::offset_of!(Muzzle, m_iMuzzleWait) == 24);
const _: () = assert!(core::mem::offset_of!(Muzzle, m_bFired) == 28);
