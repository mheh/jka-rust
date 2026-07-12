#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_int, c_void};

use mp_qshared::common::mp::cgame::ref_entity_t::refEntity_t;
use mp_qshared::shared::vec3_t;

/// Raven `TCGPositionOnBolt` — positions a render entity on a Ghoul2 model bolt.
///
/// Type definition source: `oracle/codemp/cgame/cg_public.h:528-537`
#[repr(C)]
pub struct TCGPositionOnBolt {
    pub ent: refEntity_t,    // output
    pub ghoul2: *mut c_void, // input
    pub modelIndex: c_int,   // input
    pub boltIndex: c_int,    // input
    pub origin: vec3_t,      // input
    pub angles: vec3_t,      // input
    pub modelScale: vec3_t,  // input
}

const _: () = assert!(core::mem::size_of::<TCGPositionOnBolt>() == 272);
const _: () = assert!(core::mem::offset_of!(TCGPositionOnBolt, ent) == 0);
const _: () = assert!(core::mem::offset_of!(TCGPositionOnBolt, ghoul2) == 216);
const _: () = assert!(core::mem::offset_of!(TCGPositionOnBolt, modelIndex) == 224);
const _: () = assert!(core::mem::offset_of!(TCGPositionOnBolt, boltIndex) == 228);
const _: () = assert!(core::mem::offset_of!(TCGPositionOnBolt, origin) == 232);
const _: () = assert!(core::mem::offset_of!(TCGPositionOnBolt, angles) == 244);
const _: () = assert!(core::mem::offset_of!(TCGPositionOnBolt, modelScale) == 256);
