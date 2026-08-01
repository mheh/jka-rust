#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_int};

use mp_qshared::shared::{qboolean, MAX_QPATH};

use mp_engine_qcommon::qfiles::md3_header_t::md3Header_t;

use super::modtype_t::modtype_t;
use crate::mdx_format::mdxa_header_t::mdxaHeader_t;
use crate::mdx_format::mdxm_header_t::mdxmHeader_t;

/// `MD3_MAX_LODS`.
/// Source: `oracle/codemp/qcommon/qfiles.h:96`
const MD3_MAX_LODS: usize = 3;

/// Raven `model_s` (typedef `model_t`) — a loaded renderable model (brush,
/// MD3 mesh, or Ghoul2 mesh/animation).
///
/// This type stays inside the renderer, so it takes the idiomatic latitude
/// porting-rules §D12 gives an internal type.
/// Raven's `bmodel` link is gone: a brush model finds its `WorldAsset::bmodels`
/// row through `RenderModels::bmodel_index`, so the layout no longer matches
/// the header and the layout asserts retire with the field.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:1117-1135`
#[repr(C)]
pub struct model_t {
    pub name: [c_char; MAX_QPATH as usize],
    pub r#type: modtype_t,
    /// model = tr.models[model->index]
    pub index: c_int,

    /// just for listing purposes
    pub dataSize: c_int,
    /// only if type == MOD_MESH
    pub md3: [*mut md3Header_t; MD3_MAX_LODS],
    /// only if type == MOD_GL2M which is a GHOUL II Mesh file NOT a GHOUL II animation file
    pub mdxm: *mut mdxmHeader_t,
    /// only if type == MOD_GL2A which is a GHOUL II Animation file
    pub mdxa: *mut mdxaHeader_t,
    pub numLods: c_int,
    pub bspInstance: qboolean,
}

pub type model_s = model_t;

// The header's leading fields keep their Raven offsets, so we assert them.
// Every field after the dropped `bmodel` sits at a new offset, and the size
// asserts retire with it (see the struct doc above).
const _: () = assert!(core::mem::offset_of!(model_t, name) == 0);
const _: () = assert!(core::mem::offset_of!(model_t, r#type) == 64);
const _: () = assert!(core::mem::offset_of!(model_t, index) == 68);
const _: () = assert!(core::mem::offset_of!(model_t, dataSize) == 72);
