#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::common::mp::cgame::poly_vert_t::polyVert_t;

use super::dlight_s::dlight_t;
use super::draw_surf_s::drawSurf_t;
use super::render_command_list_t::renderCommandList_t;
use super::srf_poly_s::srfPoly_t;
use super::tr_mini_ref_entity_t::trMiniRefEntity_t;
use super::tr_ref_entity_t::trRefEntity_t;

/// Raven `backEndData_t` — the per-frame scratch buffers the back end reads
/// while the front end fills the next frame's buffers (`drawSurfs[MAX_DRAWSURFS]`
/// = 65536, `dlights[MAX_DLIGHTS]` = 32, `entities[MAX_ENTITIES]` = 2048,
/// `miniEntities[MAX_MINI_ENTITIES]` = 1024; counts derived from the packet's
/// offset asserts, not re-explored in the oracle headers).
///
/// Raven: none.
/// Type definition source: `oracle/oracle/codemp/renderer/tr_local.h:2263-2273`
#[repr(C, align(8))]
pub struct backEndData_t {
    pub drawSurfs: [drawSurf_t; 65536],
    pub dlights: [dlight_t; 32],
    pub entities: [trRefEntity_t; 2048],
    pub miniEntities: [trMiniRefEntity_t; 1024],
    /// Raven: `srfPoly_t *polys;//[MAX_POLYS];`
    pub polys: *mut srfPoly_t,
    /// Raven: `polyVert_t *polyVerts;//[MAX_POLYVERTS];`
    pub polyVerts: *mut polyVert_t,
    pub commands: renderCommandList_t,
}

const _: () = assert!(core::mem::size_of::<backEndData_t>() == 1983128);
const _: () = assert!(core::mem::offset_of!(backEndData_t, drawSurfs) == 0);
const _: () = assert!(core::mem::offset_of!(backEndData_t, dlights) == 1048576);
const _: () = assert!(core::mem::offset_of!(backEndData_t, entities) == 1053312);
const _: () = assert!(core::mem::offset_of!(backEndData_t, miniEntities) == 1610368);
const _: () = assert!(core::mem::offset_of!(backEndData_t, polys) == 1720960);
const _: () = assert!(core::mem::offset_of!(backEndData_t, polyVerts) == 1720968);
const _: () = assert!(core::mem::offset_of!(backEndData_t, commands) == 1720976);
