#![allow(non_camel_case_types, non_snake_case)]

use super::draw_surf_s::drawSurf_t;
use super::tr_refdef_t::trRefdef_t;
use super::view_parms_t::viewParms_t;

/// Raven `drawSurfsCommand_t` — a render command holding the list of
/// surfaces to draw for a view, along with the view's refdef and viewParms.
///
/// Raven: none.
/// Type definition source: `oracle/oracle/codemp/renderer/tr_local.h:2231-2237`
#[repr(C)]
pub struct drawSurfsCommand_t {
    pub commandId: i32,
    pub refdef: trRefdef_t,
    pub viewParms: viewParms_t,
    pub drawSurfs: *mut drawSurf_t,
    pub numDrawSurfs: i32,
}

const _: () = assert!(core::mem::size_of::<drawSurfsCommand_t>() == 968);
const _: () = assert!(core::mem::offset_of!(drawSurfsCommand_t, commandId) == 0);
const _: () = assert!(core::mem::offset_of!(drawSurfsCommand_t, refdef) == 8);
const _: () = assert!(core::mem::offset_of!(drawSurfsCommand_t, viewParms) == 456);
const _: () = assert!(core::mem::offset_of!(drawSurfsCommand_t, drawSurfs) == 952);
const _: () = assert!(core::mem::offset_of!(drawSurfsCommand_t, numDrawSurfs) == 960);
