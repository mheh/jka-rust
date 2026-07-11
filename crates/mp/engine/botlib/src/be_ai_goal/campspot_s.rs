#![allow(non_camel_case_types, non_snake_case)]

use crate::be_aas_bsp::be_aas_bsp_consts::MAX_EPAIRKEY;
use mp_qshared::shared::vec3_t;
use std::os::raw::c_char;

/// Raven `campspot_t` — a camp spot (`info_camp`).
///
/// Type definition source: `oracle/codemp/botlib/be_ai_goal.cpp:62-72`
#[repr(C)]
pub struct campspot_t {
    pub origin: vec3_t,
    pub areanum: i32,
    pub name: [c_char; MAX_EPAIRKEY as usize],
    pub range: f32,
    pub weight: f32,
    pub wait: f32,
    pub random: f32,
    pub next: *mut campspot_t,
}

pub type campspot_s = campspot_t;
