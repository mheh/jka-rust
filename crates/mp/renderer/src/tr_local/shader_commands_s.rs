#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::common::mp::cgame::color4ub_t::color4ub_t;
use mp_qshared::shared::{qboolean, vec2_t, vec4_t};

use super::gl_index_t::glIndex_t;
use super::shader_s::shader_t;
use super::shader_stage_t::shaderStage_t;
use super::stage_vars::{stageVars, SHADER_MAX_VERTEXES};

/// `SHADER_MAX_INDEXES`.
///
/// Source: `oracle/codemp/qcommon/qfiles.h:11`
pub const SHADER_MAX_INDEXES: usize = 6 * SHADER_MAX_VERTEXES;

/// `NUM_TEX_COORDS` (`MAXLIGHTMAPS+1`).
///
/// Source: `oracle/codemp/renderer/tr_local.h:1842`
pub const NUM_TEX_COORDS: usize = 5;

/// Raven `shaderCommands_s` — the tess buffer holding the current draw call's
/// vertex/index data as it's assembled and rendered.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:1844-1883`
#[repr(C)]
pub struct shaderCommands_s {
    pub indexes: [glIndex_t; SHADER_MAX_INDEXES],
    pub xyz: [vec4_t; SHADER_MAX_VERTEXES],
    pub normal: [vec4_t; SHADER_MAX_VERTEXES],
    pub texCoords: [[vec2_t; NUM_TEX_COORDS]; SHADER_MAX_VERTEXES],
    pub vertexColors: [color4ub_t; SHADER_MAX_VERTEXES],
    //rwwRMG - added support
    pub vertexAlphas: [[u8; 4]; SHADER_MAX_VERTEXES],
    pub vertexDlightBits: [i32; SHADER_MAX_VERTEXES],

    pub svars: stageVars,

    pub shader: *mut shader_t,
    pub shaderTime: f32,
    pub fogNum: i32,

    /// or together of all vertexDlightBits
    pub dlightBits: i32,

    pub numIndexes: i32,
    pub numVertexes: i32,

    // info extracted from current shader
    pub numPasses: i32,
    pub currentStageIteratorFunc: Option<unsafe extern "C" fn()>,
    pub xstages: *mut shaderStage_t,

    pub registration: i32,

    pub SSInitializedWind: qboolean,

    //rww - doing a fade, don't compute shader color/alpha overrides
    pub fading: bool,
}

const _: () = assert!(core::mem::offset_of!(shaderCommands_s, indexes) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<shaderCommands_s>() == 128064);
    assert!(core::mem::offset_of!(shaderCommands_s, xyz) == 24000);
    assert!(core::mem::offset_of!(shaderCommands_s, normal) == 40000);
    assert!(core::mem::offset_of!(shaderCommands_s, texCoords) == 56000);
    assert!(core::mem::offset_of!(shaderCommands_s, vertexColors) == 96000);
    assert!(core::mem::offset_of!(shaderCommands_s, vertexAlphas) == 100000);
    assert!(core::mem::offset_of!(shaderCommands_s, vertexDlightBits) == 104000);
    assert!(core::mem::offset_of!(shaderCommands_s, svars) == 108000);
    assert!(core::mem::offset_of!(shaderCommands_s, shader) == 128000);
    assert!(core::mem::offset_of!(shaderCommands_s, shaderTime) == 128008);
    assert!(core::mem::offset_of!(shaderCommands_s, fogNum) == 128012);
    assert!(core::mem::offset_of!(shaderCommands_s, dlightBits) == 128016);
    assert!(core::mem::offset_of!(shaderCommands_s, numIndexes) == 128020);
    assert!(core::mem::offset_of!(shaderCommands_s, numVertexes) == 128024);
    assert!(core::mem::offset_of!(shaderCommands_s, numPasses) == 128028);
    assert!(core::mem::offset_of!(shaderCommands_s, currentStageIteratorFunc) == 128032);
    assert!(core::mem::offset_of!(shaderCommands_s, xstages) == 128040);
    assert!(core::mem::offset_of!(shaderCommands_s, registration) == 128048);
    assert!(core::mem::offset_of!(shaderCommands_s, SSInitializedWind) == 128052);
    assert!(core::mem::offset_of!(shaderCommands_s, fading) == 128056);
};
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<shaderCommands_s>() == 128048);
    assert!(core::mem::offset_of!(shaderCommands_s, xyz) == 24000);
    assert!(core::mem::offset_of!(shaderCommands_s, normal) == 40000);
    assert!(core::mem::offset_of!(shaderCommands_s, texCoords) == 56000);
    assert!(core::mem::offset_of!(shaderCommands_s, vertexColors) == 96000);
    assert!(core::mem::offset_of!(shaderCommands_s, vertexAlphas) == 100000);
    assert!(core::mem::offset_of!(shaderCommands_s, vertexDlightBits) == 104000);
    assert!(core::mem::offset_of!(shaderCommands_s, svars) == 108000);
    assert!(core::mem::offset_of!(shaderCommands_s, shader) == 128000);
    assert!(core::mem::offset_of!(shaderCommands_s, shaderTime) == 128004);
    assert!(core::mem::offset_of!(shaderCommands_s, fogNum) == 128008);
    assert!(core::mem::offset_of!(shaderCommands_s, dlightBits) == 128012);
    assert!(core::mem::offset_of!(shaderCommands_s, numIndexes) == 128016);
    assert!(core::mem::offset_of!(shaderCommands_s, numVertexes) == 128020);
    assert!(core::mem::offset_of!(shaderCommands_s, numPasses) == 128024);
    assert!(core::mem::offset_of!(shaderCommands_s, currentStageIteratorFunc) == 128028);
    assert!(core::mem::offset_of!(shaderCommands_s, xstages) == 128032);
    assert!(core::mem::offset_of!(shaderCommands_s, registration) == 128036);
    assert!(core::mem::offset_of!(shaderCommands_s, SSInitializedWind) == 128040);
    assert!(core::mem::offset_of!(shaderCommands_s, fading) == 128044);
};
