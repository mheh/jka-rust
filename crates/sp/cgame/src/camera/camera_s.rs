#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;
use sp_qshared::shared::{qboolean, vec3_t, vec4_t, MAX_QPATH};

/// Raven `camera_s` — cinematic camera state (position, movement, FOV, fades,
/// tracking, shakes, and ROFF playback).
///
/// Type definition source: `oracle/code/cgame/cg_camera.h:30-118`
#[repr(C)]
pub struct camera_t {
	//Position / Facing information
	pub origin: vec3_t,
	pub angles: vec3_t,

	pub origin2: vec3_t,
	pub angles2: vec3_t,

	//Movement information
	pub move_duration: f32,
	pub move_time: f32,
	pub move_type: i32, //CMOVE_LINEAR, CMOVE_BEZIER

	//FOV information
	pub FOV: f32,
	pub FOV2: f32,
	pub FOV_duration: f32,
	pub FOV_time: f32,
	pub FOV_vel: f32,
	pub FOV_acc: f32,

	//Pan information
	pub pan_time: f32,
	pub pan_duration: f32,

	//Following information
	pub cameraGroup: [c_char; MAX_QPATH],
	pub cameraGroupZOfs: f32,
	pub cameraGroupTag: [c_char; MAX_QPATH],
	pub subjectPos: vec3_t,
	pub subjectSpeed: f32,
	pub followSpeed: f32,
	pub followInitLerp: qboolean,
	pub distance: f32,
	pub distanceInitLerp: qboolean,
	//int		aimEntNum;//FIXME: remove

	//Tracking information
	pub trackEntNum: i32,
	pub trackToOrg: vec3_t,
	pub moveDir: vec3_t,
	pub speed: f32,
	pub initSpeed: f32,
	pub trackInitLerp: f32,
	pub nextTrackEntUpdateTime: i32,

	//Cine-bar information
	pub bar_alpha: f32,
	pub bar_alpha_source: f32,
	pub bar_alpha_dest: f32,
	pub bar_time: f32,

	pub bar_height_source: f32,
	pub bar_height_dest: f32,
	pub bar_height: f32,

	pub fade_color: vec4_t,
	pub fade_source: vec4_t,
	pub fade_dest: vec4_t,
	pub fade_time: f32,
	pub fade_duration: f32,

	//State information
	pub info_state: i32,

	//Shake information
	pub shake_intensity: f32,
	pub shake_duration: i32,
	pub shake_start: i32,

	//Smooth information
	pub smooth_intensity: f32,
	pub smooth_duration: i32,
	pub smooth_start: i32,
	pub smooth_origin: vec3_t,
	pub smooth_active: bool, // means smooth_origin and angles are valid

	// ROFF information
	pub sRoff: [c_char; MAX_QPATH], // name of a cached roff
	pub roff_frame: i32,            // current frame in the roff data
	pub next_roff_time: i32,        // time when it's ok to apply the next roff frame
	                                 //#ifdef _XBOX
	                                 //	qboolean	widescreen;
	                                 //#endif
}

const _: () = assert!(core::mem::size_of::<camera_t>() == 500);
const _: () = assert!(core::mem::offset_of!(camera_t, origin) == 0);
const _: () = assert!(core::mem::offset_of!(camera_t, angles) == 12);
const _: () = assert!(core::mem::offset_of!(camera_t, origin2) == 24);
const _: () = assert!(core::mem::offset_of!(camera_t, angles2) == 36);
const _: () = assert!(core::mem::offset_of!(camera_t, move_duration) == 48);
const _: () = assert!(core::mem::offset_of!(camera_t, move_time) == 52);
const _: () = assert!(core::mem::offset_of!(camera_t, move_type) == 56);
const _: () = assert!(core::mem::offset_of!(camera_t, FOV) == 60);
const _: () = assert!(core::mem::offset_of!(camera_t, FOV2) == 64);
const _: () = assert!(core::mem::offset_of!(camera_t, FOV_duration) == 68);
const _: () = assert!(core::mem::offset_of!(camera_t, FOV_time) == 72);
const _: () = assert!(core::mem::offset_of!(camera_t, FOV_vel) == 76);
const _: () = assert!(core::mem::offset_of!(camera_t, FOV_acc) == 80);
const _: () = assert!(core::mem::offset_of!(camera_t, pan_time) == 84);
const _: () = assert!(core::mem::offset_of!(camera_t, pan_duration) == 88);
const _: () = assert!(core::mem::offset_of!(camera_t, cameraGroup) == 92);
const _: () = assert!(core::mem::offset_of!(camera_t, cameraGroupZOfs) == 156);
const _: () = assert!(core::mem::offset_of!(camera_t, cameraGroupTag) == 160);
const _: () = assert!(core::mem::offset_of!(camera_t, subjectPos) == 224);
const _: () = assert!(core::mem::offset_of!(camera_t, subjectSpeed) == 236);
const _: () = assert!(core::mem::offset_of!(camera_t, followSpeed) == 240);
const _: () = assert!(core::mem::offset_of!(camera_t, followInitLerp) == 244);
const _: () = assert!(core::mem::offset_of!(camera_t, distance) == 248);
const _: () = assert!(core::mem::offset_of!(camera_t, distanceInitLerp) == 252);
const _: () = assert!(core::mem::offset_of!(camera_t, trackEntNum) == 256);
const _: () = assert!(core::mem::offset_of!(camera_t, trackToOrg) == 260);
const _: () = assert!(core::mem::offset_of!(camera_t, moveDir) == 272);
const _: () = assert!(core::mem::offset_of!(camera_t, speed) == 284);
const _: () = assert!(core::mem::offset_of!(camera_t, initSpeed) == 288);
const _: () = assert!(core::mem::offset_of!(camera_t, trackInitLerp) == 292);
const _: () = assert!(core::mem::offset_of!(camera_t, nextTrackEntUpdateTime) == 296);
const _: () = assert!(core::mem::offset_of!(camera_t, bar_alpha) == 300);
const _: () = assert!(core::mem::offset_of!(camera_t, bar_alpha_source) == 304);
const _: () = assert!(core::mem::offset_of!(camera_t, bar_alpha_dest) == 308);
const _: () = assert!(core::mem::offset_of!(camera_t, bar_time) == 312);
const _: () = assert!(core::mem::offset_of!(camera_t, bar_height_source) == 316);
const _: () = assert!(core::mem::offset_of!(camera_t, bar_height_dest) == 320);
const _: () = assert!(core::mem::offset_of!(camera_t, bar_height) == 324);
const _: () = assert!(core::mem::offset_of!(camera_t, fade_color) == 328);
const _: () = assert!(core::mem::offset_of!(camera_t, fade_source) == 344);
const _: () = assert!(core::mem::offset_of!(camera_t, fade_dest) == 360);
const _: () = assert!(core::mem::offset_of!(camera_t, fade_time) == 376);
const _: () = assert!(core::mem::offset_of!(camera_t, fade_duration) == 380);
const _: () = assert!(core::mem::offset_of!(camera_t, info_state) == 384);
const _: () = assert!(core::mem::offset_of!(camera_t, shake_intensity) == 388);
const _: () = assert!(core::mem::offset_of!(camera_t, shake_duration) == 392);
const _: () = assert!(core::mem::offset_of!(camera_t, shake_start) == 396);
const _: () = assert!(core::mem::offset_of!(camera_t, smooth_intensity) == 400);
const _: () = assert!(core::mem::offset_of!(camera_t, smooth_duration) == 404);
const _: () = assert!(core::mem::offset_of!(camera_t, smooth_start) == 408);
const _: () = assert!(core::mem::offset_of!(camera_t, smooth_origin) == 412);
const _: () = assert!(core::mem::offset_of!(camera_t, smooth_active) == 424);
const _: () = assert!(core::mem::offset_of!(camera_t, sRoff) == 425);
const _: () = assert!(core::mem::offset_of!(camera_t, roff_frame) == 492);
const _: () = assert!(core::mem::offset_of!(camera_t, next_roff_time) == 496);
