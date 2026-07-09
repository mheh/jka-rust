#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::shared::vec3_t;

/// Raven `aas_settings_t` — AAS physics and reachability (rs_*) settings.
///
/// Type definition source: `oracle/codemp/botlib/be_aas_def.h:88-127`
#[repr(C)]
pub struct aas_settings_t {
	pub phys_gravitydirection: vec3_t,
	pub phys_friction: f32,
	pub phys_stopspeed: f32,
	pub phys_gravity: f32,
	pub phys_waterfriction: f32,
	pub phys_watergravity: f32,
	pub phys_maxvelocity: f32,
	pub phys_maxwalkvelocity: f32,
	pub phys_maxcrouchvelocity: f32,
	pub phys_maxswimvelocity: f32,
	pub phys_walkaccelerate: f32,
	pub phys_airaccelerate: f32,
	pub phys_swimaccelerate: f32,
	pub phys_maxstep: f32,
	pub phys_maxsteepness: f32,
	pub phys_maxwaterjump: f32,
	pub phys_maxbarrier: f32,
	pub phys_jumpvel: f32,
	pub phys_falldelta5: f32,
	pub phys_falldelta10: f32,
	pub rs_waterjump: f32,
	pub rs_teleport: f32,
	pub rs_barrierjump: f32,
	pub rs_startcrouch: f32,
	pub rs_startgrapple: f32,
	pub rs_startwalkoffledge: f32,
	pub rs_startjump: f32,
	pub rs_rocketjump: f32,
	pub rs_bfgjump: f32,
	pub rs_jumppad: f32,
	pub rs_aircontrolledjumppad: f32,
	pub rs_funcbob: f32,
	pub rs_startelevator: f32,
	pub rs_falldamage5: f32,
	pub rs_falldamage10: f32,
	pub rs_maxfallheight: f32,
	pub rs_maxjumpfallheight: f32,
}

/// Raven's C tag is `aas_settings_s`; the typedef name `aas_settings_t` is
/// house style for the struct itself.
pub type aas_settings_s = aas_settings_t;

const _: () = assert!(core::mem::size_of::<aas_settings_t>() == 156);
const _: () = assert!(core::mem::offset_of!(aas_settings_t, phys_gravitydirection) == 0);
const _: () = assert!(core::mem::offset_of!(aas_settings_t, phys_friction) == 12);
const _: () = assert!(core::mem::offset_of!(aas_settings_t, phys_stopspeed) == 16);
const _: () = assert!(core::mem::offset_of!(aas_settings_t, phys_gravity) == 20);
const _: () = assert!(core::mem::offset_of!(aas_settings_t, phys_waterfriction) == 24);
const _: () = assert!(core::mem::offset_of!(aas_settings_t, phys_watergravity) == 28);
const _: () = assert!(core::mem::offset_of!(aas_settings_t, phys_maxvelocity) == 32);
const _: () = assert!(core::mem::offset_of!(aas_settings_t, phys_maxwalkvelocity) == 36);
const _: () = assert!(core::mem::offset_of!(aas_settings_t, phys_maxcrouchvelocity) == 40);
const _: () = assert!(core::mem::offset_of!(aas_settings_t, phys_maxswimvelocity) == 44);
const _: () = assert!(core::mem::offset_of!(aas_settings_t, phys_walkaccelerate) == 48);
const _: () = assert!(core::mem::offset_of!(aas_settings_t, phys_airaccelerate) == 52);
const _: () = assert!(core::mem::offset_of!(aas_settings_t, phys_swimaccelerate) == 56);
const _: () = assert!(core::mem::offset_of!(aas_settings_t, phys_maxstep) == 60);
const _: () = assert!(core::mem::offset_of!(aas_settings_t, phys_maxsteepness) == 64);
const _: () = assert!(core::mem::offset_of!(aas_settings_t, phys_maxwaterjump) == 68);
const _: () = assert!(core::mem::offset_of!(aas_settings_t, phys_maxbarrier) == 72);
const _: () = assert!(core::mem::offset_of!(aas_settings_t, phys_jumpvel) == 76);
const _: () = assert!(core::mem::offset_of!(aas_settings_t, phys_falldelta5) == 80);
const _: () = assert!(core::mem::offset_of!(aas_settings_t, phys_falldelta10) == 84);
const _: () = assert!(core::mem::offset_of!(aas_settings_t, rs_waterjump) == 88);
const _: () = assert!(core::mem::offset_of!(aas_settings_t, rs_teleport) == 92);
const _: () = assert!(core::mem::offset_of!(aas_settings_t, rs_barrierjump) == 96);
const _: () = assert!(core::mem::offset_of!(aas_settings_t, rs_startcrouch) == 100);
const _: () = assert!(core::mem::offset_of!(aas_settings_t, rs_startgrapple) == 104);
const _: () = assert!(core::mem::offset_of!(aas_settings_t, rs_startwalkoffledge) == 108);
const _: () = assert!(core::mem::offset_of!(aas_settings_t, rs_startjump) == 112);
const _: () = assert!(core::mem::offset_of!(aas_settings_t, rs_rocketjump) == 116);
const _: () = assert!(core::mem::offset_of!(aas_settings_t, rs_bfgjump) == 120);
const _: () = assert!(core::mem::offset_of!(aas_settings_t, rs_jumppad) == 124);
const _: () = assert!(core::mem::offset_of!(aas_settings_t, rs_aircontrolledjumppad) == 128);
const _: () = assert!(core::mem::offset_of!(aas_settings_t, rs_funcbob) == 132);
const _: () = assert!(core::mem::offset_of!(aas_settings_t, rs_startelevator) == 136);
const _: () = assert!(core::mem::offset_of!(aas_settings_t, rs_falldamage5) == 140);
const _: () = assert!(core::mem::offset_of!(aas_settings_t, rs_falldamage10) == 144);
const _: () = assert!(core::mem::offset_of!(aas_settings_t, rs_maxfallheight) == 148);
const _: () = assert!(core::mem::offset_of!(aas_settings_t, rs_maxjumpfallheight) == 152);
