#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_int};

use sp_qshared::shared::MAX_QPATH;

use sp_engine_qcommon::qfiles::md3_header_t::md3Header_t;

use super::bmodel_t::bmodel_t;
use super::modtype_t::modtype_t;
use crate::mdx_format::mdxa_header_t::mdxaHeader_t;
use crate::mdx_format::mdxm_header_t::mdxmHeader_t;

/// `MD3_MAX_LODS`.
/// Source: `oracle/code/qcommon/qfiles.h:99`
const MD3_MAX_LODS: usize = 3;

/// Raven `model_s` (typedef `model_t`) — a loaded renderable model (brush,
/// MD3 mesh, or Ghoul2 mesh/animation).
///
/// Type definition source: `oracle/code/renderer/tr_local.h:970-988`
#[repr(C)]
pub struct model_t {
    pub name: [c_char; MAX_QPATH as usize],
    pub r#type: modtype_t,
    /// model = tr.models[model->mod_index]
    pub index: c_int,

    /// just for listing purposes
    pub dataSize: c_int,
    /// only if type == MOD_BRUSH
    pub bmodel: *mut bmodel_t,
    /// only if type == MOD_MESH
    pub md3: [*mut md3Header_t; MD3_MAX_LODS],
    /// only if type == MOD_GL2M which is a GHOUL II Mesh file NOT a GHOUL II animation file
    pub mdxm: *mut mdxmHeader_t,
    /// only if type == MOD_GL2A which is a GHOUL II Animation file
    pub mdxa: *mut mdxaHeader_t,
    pub numLods: u8,
    /// model is a bsp instance
    pub bspInstance: bool,
}

pub type model_s = model_t;

const _: () = assert!(core::mem::size_of::<model_t>() == 136);
const _: () = assert!(core::mem::offset_of!(model_t, name) == 0);
const _: () = assert!(core::mem::offset_of!(model_t, r#type) == 64);
const _: () = assert!(core::mem::offset_of!(model_t, index) == 68);
const _: () = assert!(core::mem::offset_of!(model_t, dataSize) == 72);
const _: () = assert!(core::mem::offset_of!(model_t, bmodel) == 80);
const _: () = assert!(core::mem::offset_of!(model_t, md3) == 88);
const _: () = assert!(core::mem::offset_of!(model_t, mdxm) == 112);
const _: () = assert!(core::mem::offset_of!(model_t, mdxa) == 120);
const _: () = assert!(core::mem::offset_of!(model_t, numLods) == 128);
const _: () = assert!(core::mem::offset_of!(model_t, bspInstance) == 129);
