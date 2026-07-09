#![allow(non_camel_case_types, non_snake_case)]

use sp_qshared::common::sp::renderer::color4ub_t::color4ub_t;
use sp_qshared::shared::vec2_t;

use super::shader_stage_t::NUM_TEXTURE_BUNDLES;

/// `SHADER_MAX_VERTEXES`.
///
/// Source: `oracle/code/qcommon/qfiles.h:10`
pub const SHADER_MAX_VERTEXES: usize = 1000;

/// Raven `stageVars_t` — per-stage vertex colors and texture coordinates.
///
/// Type definition source: `oracle/code/renderer/tr_local.h:1630-1638`
#[repr(C)]
pub struct stageVars {
	pub colors: [color4ub_t; SHADER_MAX_VERTEXES],
	pub texcoords: [[vec2_t; SHADER_MAX_VERTEXES]; NUM_TEXTURE_BUNDLES],
}

const _: () = assert!(core::mem::size_of::<stageVars>() == 20000);
const _: () = assert!(core::mem::offset_of!(stageVars, colors) == 0);
const _: () = assert!(core::mem::offset_of!(stageVars, texcoords) == 4000);
