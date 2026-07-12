#![allow(non_camel_case_types, non_snake_case)]

use super::sex_type_t::sexType_t;

/// Raven `gNPCstats_t` — NPC AI/movement stats, loaded in and settable by scripts.
///
/// Raven: !!!!!!!!!! LOADSAVE-affecting structure !!!!!!!!!!
/// Type definition source: `oracle/code/game/b_public.h:115-138`
#[repr(C)]
pub struct gNPCstats_t {
    //AI
    pub aggression: i32,    //			"
    pub aim: i32,           //			"
    pub earshot: f32,       //			"
    pub evasion: i32,       //			"
    pub hfov: i32,          // horizontal field of view
    pub intelligence: i32,  //			"
    pub r#move: i32,        //			"
    pub reactions: i32,     // 1-5, higher is better
    pub shootDistance: f32, //Maximum range- overrides range set for weapon if nonzero
    pub vfov: i32,          // vertical field of view
    pub vigilance: f32,     //			"
    pub visrange: f32,      //			"
    //Movement
    pub runSpeed: i32,
    pub walkSpeed: i32,
    pub yawSpeed: f32, // 1 - whatever, default is 50
    pub health: i32,
    pub acceleration: i32,
    //sex
    pub sex: sexType_t, //male, female, etc.
}

const _: () = assert!(core::mem::size_of::<gNPCstats_t>() == 72);
const _: () = assert!(core::mem::offset_of!(gNPCstats_t, aggression) == 0);
const _: () = assert!(core::mem::offset_of!(gNPCstats_t, aim) == 4);
const _: () = assert!(core::mem::offset_of!(gNPCstats_t, earshot) == 8);
const _: () = assert!(core::mem::offset_of!(gNPCstats_t, evasion) == 12);
const _: () = assert!(core::mem::offset_of!(gNPCstats_t, hfov) == 16);
const _: () = assert!(core::mem::offset_of!(gNPCstats_t, intelligence) == 20);
const _: () = assert!(core::mem::offset_of!(gNPCstats_t, r#move) == 24);
const _: () = assert!(core::mem::offset_of!(gNPCstats_t, reactions) == 28);
const _: () = assert!(core::mem::offset_of!(gNPCstats_t, shootDistance) == 32);
const _: () = assert!(core::mem::offset_of!(gNPCstats_t, vfov) == 36);
const _: () = assert!(core::mem::offset_of!(gNPCstats_t, vigilance) == 40);
const _: () = assert!(core::mem::offset_of!(gNPCstats_t, visrange) == 44);
const _: () = assert!(core::mem::offset_of!(gNPCstats_t, runSpeed) == 48);
const _: () = assert!(core::mem::offset_of!(gNPCstats_t, walkSpeed) == 52);
const _: () = assert!(core::mem::offset_of!(gNPCstats_t, yawSpeed) == 56);
const _: () = assert!(core::mem::offset_of!(gNPCstats_t, health) == 60);
const _: () = assert!(core::mem::offset_of!(gNPCstats_t, acceleration) == 64);
const _: () = assert!(core::mem::offset_of!(gNPCstats_t, sex) == 68);
