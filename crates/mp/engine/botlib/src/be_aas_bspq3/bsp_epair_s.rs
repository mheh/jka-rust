//! Raven `bsp_epair_s`/`bsp_entity_s` — the BSP-entity epair store backing
//! `bspworld` (internal botlib scratch state, not ABI-crossing).
//!
//! Source: `oracle/codemp/botlib/be_aas_bspq3.cpp:42-54`

use core::ffi::c_char;

/// Raven `bsp_epair_t` — one entity key/value pair, linked into a list.
///
/// Raven: "bsp entity epair".
/// Source: `oracle/codemp/botlib/be_aas_bspq3.cpp:43-48`
#[derive(Clone, Copy)]
pub struct bsp_epair_t {
    pub key: *mut c_char,
    pub value: *mut c_char,
    pub next: *mut bsp_epair_t,
}

/// Raven `bsp_entity_t` — one BSP data entity (its epair list head).
///
/// Raven: "bsp data entity".
/// Source: `oracle/codemp/botlib/be_aas_bspq3.cpp:51-54`
#[derive(Clone, Copy)]
pub struct bsp_entity_t {
    pub epairs: *mut bsp_epair_t,
}
