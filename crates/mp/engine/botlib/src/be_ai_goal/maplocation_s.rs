#![allow(non_camel_case_types, non_snake_case)]

use crate::be_aas_bsp::be_aas_bsp_consts::MAX_EPAIRKEY;
use mp_qshared::shared::vec3_t;
use std::os::raw::c_char;

/// Raven `maplocation_t` — a `target_location` in the map.
///
/// Type definition source: `oracle/codemp/botlib/be_ai_goal.cpp:52-59`
#[repr(C)]
pub struct maplocation_t {
    pub origin: vec3_t,
    pub areanum: i32,
    pub name: [c_char; MAX_EPAIRKEY as usize],
    pub next: *mut maplocation_t,
}

pub type maplocation_s = maplocation_t;
