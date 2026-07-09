#![allow(non_camel_case_types, non_snake_case)]

use sp_qshared::common::sp::gentity::gentity_t;
use sp_qshared::common::sp::qcommon::entity_state::entityState_t;
use sp_qshared::shared::{qboolean, vec3_t};

use super::player_entity_t::playerEntity_t;

// centity_t have a direct corespondence with gentity_t in the game, but
// only the entityState_t is directly communicated to the cgame

/// Raven `centity_t` — client-side representation of an entity, tracked
/// across snapshots for interpolation.
///
/// Type definition source: `oracle/code/cgame/cg_local.h:130-176`
#[repr(C)]
pub struct centity_t {
	/// from cg.frame
	pub currentState: entityState_t,
	/// from cg.nextFrame, if available
	pub nextState: *const entityState_t,
	/// true if next is valid to interpolate to
	pub interpolate: qboolean,
	/// true if cg.frame holds this entity
	pub currentValid: qboolean,

	/// move to playerEntity?
	pub muzzleFlashTime: i32,
	/// move to playerEntity?
	pub altFire: qboolean,

	pub previousEvent: i32,

	pub miscTime: i32,

	pub pe: playerEntity_t,

	// exact interpolated position of entity on this frame
	pub lerpOrigin: vec3_t,
	pub lerpAngles: vec3_t,
	/// for ET_PLAYERS, the actual angles it was rendered at- should be used by any getboltmatrix calls after CG_Player
	pub renderAngles: vec3_t,

	/// rotation increment for repeater effect
	pub rotValue: f32,

	pub snapShotTime: i32,

	/// Pointer to corresponding gentity
	pub gent: *mut gentity_t,
}

const _: () = assert!(core::mem::size_of::<centity_t>() == 488);
const _: () = assert!(core::mem::offset_of!(centity_t, currentState) == 0);
const _: () = assert!(core::mem::offset_of!(centity_t, nextState) == 272);
const _: () = assert!(core::mem::offset_of!(centity_t, interpolate) == 280);
const _: () = assert!(core::mem::offset_of!(centity_t, currentValid) == 284);
const _: () = assert!(core::mem::offset_of!(centity_t, muzzleFlashTime) == 288);
const _: () = assert!(core::mem::offset_of!(centity_t, altFire) == 292);
const _: () = assert!(core::mem::offset_of!(centity_t, previousEvent) == 296);
const _: () = assert!(core::mem::offset_of!(centity_t, miscTime) == 300);
const _: () = assert!(core::mem::offset_of!(centity_t, pe) == 304);
const _: () = assert!(core::mem::offset_of!(centity_t, lerpOrigin) == 432);
const _: () = assert!(core::mem::offset_of!(centity_t, lerpAngles) == 444);
const _: () = assert!(core::mem::offset_of!(centity_t, renderAngles) == 456);
const _: () = assert!(core::mem::offset_of!(centity_t, rotValue) == 468);
const _: () = assert!(core::mem::offset_of!(centity_t, snapShotTime) == 472);
const _: () = assert!(core::mem::offset_of!(centity_t, gent) == 480);
