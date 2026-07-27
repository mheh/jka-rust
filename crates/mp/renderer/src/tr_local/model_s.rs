#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_int};

use mp_qshared::shared::{qboolean, MAX_QPATH};

use mp_engine_qcommon::qfiles::md3_header_t::md3Header_t;

use super::bmodel_t::bmodel_t;
use super::modtype_t::modtype_t;
use crate::mdx_format::mdxa_header_t::mdxaHeader_t;
use crate::mdx_format::mdxm_header_t::mdxmHeader_t;

/// `MD3_MAX_LODS`.
/// Source: `oracle/codemp/qcommon/qfiles.h:96`
const MD3_MAX_LODS: usize = 3;

/// Raven `model_s` (typedef `model_t`) — a loaded renderable model (brush,
/// MD3 mesh, or Ghoul2 mesh/animation).
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
    /// only if type == MOD_BRUSH
    pub bmodel: *mut bmodel_t,
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

impl model_t {
    /// Raven's `model->bmodel` deref (`RE_GetBModelVerts`,
    /// `tr_world.cpp:663`) — the inline (brush) model's surface range. The
    /// raw deref is quarantined here (§D11) so the `tr_world.cpp` logic port
    /// stays entirely safe; it mirrors `tr_model::frontend::mdxm_view_of`'s
    /// established quarantine for the sibling `mdxm` pointer.
    ///
    /// # Safety invariant
    /// `bmodel` is set by `R_LoadBrushModel` (`tr_bsp.cpp`) for
    /// `MOD_BRUSH` models and points into the world's `Hunk_Alloc`'d block,
    /// valid while the world asset lives; the oracle dereferences it
    /// unchecked on the same path, so callers must only reach here for brush
    /// models.
    ///
    /// This accessor retires with the type at the #41 type pass.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:1128`
    pub fn bmodel(&self) -> &bmodel_t {
        debug_assert!(!self.bmodel.is_null());
        unsafe { &*self.bmodel }
    }
}

const _: () = assert!(core::mem::offset_of!(model_t, name) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<model_t>() == 136);
    assert!(core::mem::offset_of!(model_t, r#type) == 64);
    assert!(core::mem::offset_of!(model_t, index) == 68);
    assert!(core::mem::offset_of!(model_t, dataSize) == 72);
    assert!(core::mem::offset_of!(model_t, bmodel) == 80);
    assert!(core::mem::offset_of!(model_t, md3) == 88);
    assert!(core::mem::offset_of!(model_t, mdxm) == 112);
    assert!(core::mem::offset_of!(model_t, mdxa) == 120);
    assert!(core::mem::offset_of!(model_t, numLods) == 128);
    assert!(core::mem::offset_of!(model_t, bspInstance) == 132);
};
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<model_t>() == 108);
    assert!(core::mem::offset_of!(model_t, r#type) == 64);
    assert!(core::mem::offset_of!(model_t, index) == 68);
    assert!(core::mem::offset_of!(model_t, dataSize) == 72);
    assert!(core::mem::offset_of!(model_t, bmodel) == 76);
    assert!(core::mem::offset_of!(model_t, md3) == 80);
    assert!(core::mem::offset_of!(model_t, mdxm) == 92);
    assert!(core::mem::offset_of!(model_t, mdxa) == 96);
    assert!(core::mem::offset_of!(model_t, numLods) == 100);
    assert!(core::mem::offset_of!(model_t, bspInstance) == 104);
};
