//! Port of `oracle/codemp/cgame/cg_strap.c` — the bg tier's trap shims for the cgame link unit. Functions land via the C5
//! transcription waves.

#![allow(non_snake_case)]

use core::ffi::{c_int, c_void};

use mp_qshared::common::mp::qcommon::{
    sharedRagDollParams_t, sharedRagDollUpdateParams_t, sharedSetBoneIKStateParams_t,
};
use mp_qshared::shared::{mdxaBone_t, qhandle_t, sharedIKMoveParams_t, vec3_t};

use crate::trap;
use crate::world::CgContext;

/// Raven `strap_G2API_GetBoltMatrix` — forwards straight to the engine trap.
///
/// Source: `oracle/codemp/cgame/cg_strap.c:6-10`
#[allow(clippy::too_many_arguments)]
pub fn strap_G2API_GetBoltMatrix(
    ctx: &mut CgContext,
    ghoul2: *mut c_void,
    modelIndex: c_int,
    boltIndex: c_int,
    matrix: &mut mdxaBone_t,
    angles: &vec3_t,
    position: &vec3_t,
    frameNum: c_int,
    modelList: Option<&mut qhandle_t>,
    scale: &vec3_t,
) -> bool {
    trap::G2API_GetBoltMatrix(
        ctx.engine, ghoul2, modelIndex, boltIndex, matrix, angles, position, frameNum, modelList,
        scale,
    )
}

/// Raven `strap_G2API_GetBoltMatrix_NoReconstruct` — forwards straight to the engine trap.
///
/// Source: `oracle/codemp/cgame/cg_strap.c:12-16`
#[allow(clippy::too_many_arguments)]
pub fn strap_G2API_GetBoltMatrix_NoReconstruct(
    ctx: &mut CgContext,
    ghoul2: *mut c_void,
    modelIndex: c_int,
    boltIndex: c_int,
    matrix: &mut mdxaBone_t,
    angles: &vec3_t,
    position: &vec3_t,
    frameNum: c_int,
    modelList: Option<&mut qhandle_t>,
    scale: &vec3_t,
) -> bool {
    trap::G2API_GetBoltMatrix_NoReconstruct(
        ctx.engine, ghoul2, modelIndex, boltIndex, matrix, angles, position, frameNum, modelList,
        scale,
    )
}

/// Raven `strap_G2API_GetBoltMatrix_NoRecNoRot` — forwards straight to the engine trap.
///
/// Source: `oracle/codemp/cgame/cg_strap.c:18-22`
#[allow(clippy::too_many_arguments)]
pub fn strap_G2API_GetBoltMatrix_NoRecNoRot(
    ctx: &mut CgContext,
    ghoul2: *mut c_void,
    modelIndex: c_int,
    boltIndex: c_int,
    matrix: &mut mdxaBone_t,
    angles: &vec3_t,
    position: &vec3_t,
    frameNum: c_int,
    modelList: Option<&mut qhandle_t>,
    scale: &vec3_t,
) -> bool {
    trap::G2API_GetBoltMatrix_NoRecNoRot(
        ctx.engine, ghoul2, modelIndex, boltIndex, matrix, angles, position, frameNum, modelList,
        scale,
    )
}

/// Raven `strap_G2API_SetBoneAngles` — forwards straight to the engine trap.
///
/// Source: `oracle/codemp/cgame/cg_strap.c:24-29`
#[allow(clippy::too_many_arguments)]
pub fn strap_G2API_SetBoneAngles(
    ctx: &mut CgContext,
    ghoul2: *mut c_void,
    modelIndex: c_int,
    boneName: &str,
    angles: &vec3_t,
    flags: c_int,
    up: c_int,
    right: c_int,
    forward: c_int,
    modelList: Option<&mut qhandle_t>,
    blendTime: c_int,
    currentTime: c_int,
) -> bool {
    trap::G2API_SetBoneAngles(
        ctx.engine,
        ghoul2,
        modelIndex,
        boneName,
        angles,
        flags,
        up,
        right,
        forward,
        modelList,
        blendTime,
        currentTime,
    )
}

/// Raven `strap_G2API_SetBoneAnim` — forwards straight to the engine trap.
///
/// Source: `oracle/codemp/cgame/cg_strap.c:31-35`
#[allow(clippy::too_many_arguments)]
pub fn strap_G2API_SetBoneAnim(
    ctx: &mut CgContext,
    ghoul2: *mut c_void,
    modelIndex: c_int,
    boneName: &str,
    startFrame: c_int,
    endFrame: c_int,
    flags: c_int,
    animSpeed: f32,
    currentTime: c_int,
    setFrame: f32,
    blendTime: c_int,
) -> bool {
    trap::G2API_SetBoneAnim(
        ctx.engine,
        ghoul2,
        modelIndex,
        boneName,
        startFrame,
        endFrame,
        flags,
        animSpeed,
        currentTime,
        setFrame,
        blendTime,
    )
}

/// Raven `strap_G2API_GetBoneAnim` — forwards straight to the engine trap.
///
/// Source: `oracle/codemp/cgame/cg_strap.c:37-41`
#[allow(clippy::too_many_arguments)]
pub fn strap_G2API_GetBoneAnim(
    ctx: &mut CgContext,
    ghoul2: *mut c_void,
    boneName: &str,
    currentTime: c_int,
    currentFrame: &mut f32,
    startFrame: &mut c_int,
    endFrame: &mut c_int,
    flags: &mut c_int,
    animSpeed: &mut f32,
    modelList: Option<&mut c_int>,
    modelIndex: c_int,
) -> bool {
    trap::G2API_GetBoneAnim(
        ctx.engine,
        ghoul2,
        boneName,
        currentTime,
        currentFrame,
        startFrame,
        endFrame,
        flags,
        animSpeed,
        modelList,
        modelIndex,
    )
}

/// Raven `strap_G2API_SetRagDoll` — forwards straight to the engine trap.
///
/// Source: `oracle/codemp/cgame/cg_strap.c:43-46`
pub fn strap_G2API_SetRagDoll(
    ctx: &mut CgContext,
    ghoul2: *mut c_void,
    params: Option<&mut sharedRagDollParams_t>,
) {
    trap::G2API_SetRagDoll(ctx.engine, ghoul2, params)
}

/// Raven `strap_G2API_AnimateG2Models` — forwards straight to the engine trap.
///
/// Source: `oracle/codemp/cgame/cg_strap.c:48-51`
pub fn strap_G2API_AnimateG2Models(
    ctx: &mut CgContext,
    ghoul2: *mut c_void,
    time: c_int,
    params: &mut sharedRagDollUpdateParams_t,
) {
    trap::G2API_AnimateG2Models(ctx.engine, ghoul2, time, params)
}

/// Raven `strap_G2API_SetBoneIKState` — forwards straight to the engine trap.
///
/// Source: `oracle/codemp/cgame/cg_strap.c:53-56`
pub fn strap_G2API_SetBoneIKState(
    ctx: &mut CgContext,
    ghoul2: *mut c_void,
    time: c_int,
    boneName: Option<&str>,
    ikState: c_int,
    params: Option<&mut sharedSetBoneIKStateParams_t>,
) -> bool {
    trap::G2API_SetBoneIKState(ctx.engine, ghoul2, time, boneName, ikState, params)
}

/// Raven `strap_G2API_IKMove` — forwards straight to the engine trap.
///
/// Source: `oracle/codemp/cgame/cg_strap.c:58-61`
pub fn strap_G2API_IKMove(
    ctx: &mut CgContext,
    ghoul2: *mut c_void,
    time: c_int,
    params: &mut sharedIKMoveParams_t,
) -> bool {
    trap::G2API_IKMove(ctx.engine, ghoul2, time, params)
}

/// Raven `strap_TrueMalloc` — forwards straight to the engine trap.
///
/// Source: `oracle/codemp/cgame/cg_strap.c:63-66`
pub fn strap_TrueMalloc(ctx: &mut CgContext, ptr: *mut *mut c_void, size: c_int) {
    trap::TrueMalloc(ctx.engine, ptr, size)
}

/// Raven `strap_TrueFree` — forwards straight to the engine trap.
///
/// Source: `oracle/codemp/cgame/cg_strap.c:68-71`
pub fn strap_TrueFree(ctx: &mut CgContext, ptr: *mut *mut c_void) {
    trap::TrueFree(ctx.engine, ptr)
}
