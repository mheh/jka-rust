// PORT-COMPLETE: g_strap.c 12/12 (ruling 30)
//! `g_strap.c` — ctx-less bg-boundary wrappers over the `trap_G2API_*`/`trap_True*`
//! seam (ruling 30, BLESSED 2026-07-05).
//!
//! Raven exposes these `strap_*` functions with fixed C signatures
//! (`bg_strap.h`); bg logic (`bg_pmove.c`) calls them WITHOUT a `GameContext`.
//! They mirror Raven's global syscall pointer: they reach the engine through the
//! seam-scoped [`STRAP_ENGINE`] cell, armed once by the GAME_INIT ABI entrypoint
//! (`g_init_game`). §D11 seam confinement applies — the raw-pointer cell + manual
//! `Send`/`Sync` mirror the `CEngine` seam static
//! (`crates/abi-transport/src/generic/engine.rs`). All ctx-taking game code keeps
//! using `ctx.engine`; this cell is ONLY for the ctx-less boundary fn-ptrs.
//!
//! Source: `oracle/oracle/codemp/game/g_strap.c`
#![allow(non_snake_case, unused, clippy::all)]

use std::ffi::{CStr, CString};
use std::sync::OnceLock;

use mp_engine_select::Engine;

use crate::prelude::*;

/// Seam engine handle for the ctx-less `strap_*` wrappers (ruling 30). Holds a
/// raw `*const Engine` because the engine outlives the module (mirrors Raven's
/// global syscall pointer); set once at GAME_INIT, read single-threaded from bg
/// logic.
struct StrapEngine(*const Engine);
// SAFETY: same soundness argument as `CEngine`'s manual `Send`/`Sync`
// (`abi-transport/src/generic/engine.rs:25-26`): the pointer is set once at
// GAME_INIT and read single-threaded from bg logic; §D11 seam confinement.
unsafe impl Send for StrapEngine {}
unsafe impl Sync for StrapEngine {}

/// The write-once seam cell (SEAM-D1-style `OnceLock`).
static STRAP_ENGINE: OnceLock<StrapEngine> = OnceLock::new();

/// Arm the seam engine cell (ruling 30). Called exactly once from `g_init_game`
/// (GAME_INIT) with the entrypoint-owned engine handle.
/// Source: `oracle/oracle/codemp/game/g_main.c:897` (`G_InitGame`).
pub fn init_strap_engine(engine: &Engine) {
    let _ = STRAP_ENGINE.set(StrapEngine(engine as *const Engine));
}

/// Read the seam engine handle; panics loudly (house stub style) if a `strap_*`
/// wrapper runs before GAME_INIT armed the cell.
fn strap_engine() -> &'static Engine {
    match STRAP_ENGINE.get() {
        // SAFETY: the pointer was taken from a live `&Engine` at GAME_INIT and the
        // engine outlives the module (mirrors Raven's global syscall pointer).
        Some(e) => unsafe { &*e.0 },
        None => panic!(
            "strap_* bg-boundary wrapper called before init_strap_engine (GAME_INIT) \
             armed the ruling-30 seam cell"
        ),
    }
}

/// Raven `strap_G2API_GetBoltMatrix`.
///
/// Source: `oracle/oracle/codemp/game/g_strap.c:6-10`
pub fn strap_G2API_GetBoltMatrix(
    ghoul2: *mut c_void,
    modelIndex: c_int,
    boltIndex: c_int,
    matrix: *mut mdxaBone_t,
    angles: vec3_t,
    position: vec3_t,
    frameNum: c_int,
    modelList: *mut qhandle_t,
    scale: vec3_t,
) -> qboolean {
    // ruling 30: ctx-less bg-boundary wrapper; engine via the seam cell.
    crate::trap::G2API_GetBoltMatrix(
        strap_engine(),
        mp_abi::game::syscalls::G_G2_GETBOLT::GG2GetboltArgs::new(
            ghoul2, modelIndex, boltIndex, matrix, &angles, &position, frameNum, modelList, &scale,
        ),
    )
}

/// Raven `strap_G2API_GetBoltMatrix_NoReconstruct`.
///
/// Source: `oracle/oracle/codemp/game/g_strap.c:12-16`
pub fn strap_G2API_GetBoltMatrix_NoReconstruct(
    ghoul2: *mut c_void,
    modelIndex: c_int,
    boltIndex: c_int,
    matrix: *mut mdxaBone_t,
    angles: vec3_t,
    position: vec3_t,
    frameNum: c_int,
    modelList: *mut qhandle_t,
    mut scale: vec3_t,
) -> qboolean {
    // ruling 30: ctx-less bg-boundary wrapper; engine via the seam cell.
    crate::trap::G2API_GetBoltMatrix_NoReconstruct(
        strap_engine(),
        mp_abi::game::syscalls::G_G2_GETBOLT_NOREC::GG2GetboltNorecArgs::new(
            ghoul2,
            modelIndex,
            boltIndex,
            matrix,
            &angles,
            &position,
            frameNum,
            modelList,
            &mut scale,
        ),
    )
}

/// Raven `strap_G2API_GetBoltMatrix_NoRecNoRot`.
///
/// Source: `oracle/oracle/codemp/game/g_strap.c:18-22`
pub fn strap_G2API_GetBoltMatrix_NoRecNoRot(
    ghoul2: *mut c_void,
    modelIndex: c_int,
    boltIndex: c_int,
    matrix: *mut mdxaBone_t,
    angles: vec3_t,
    position: vec3_t,
    frameNum: c_int,
    modelList: *mut qhandle_t,
    scale: vec3_t,
) -> qboolean {
    // ruling 30: ctx-less bg-boundary wrapper; engine via the seam cell.
    crate::trap::G2API_GetBoltMatrix_NoRecNoRot(
        strap_engine(),
        mp_abi::game::syscalls::G_G2_GETBOLT_NOREC_NOROT::GG2GetboltNorecNorotArgs::new(
            ghoul2, modelIndex, boltIndex, matrix, &angles, &position, frameNum, modelList, &scale,
        ),
    )
}

/// Raven `strap_G2API_SetBoneAngles`.
///
/// Source: `oracle/oracle/codemp/game/g_strap.c:24-29`
pub fn strap_G2API_SetBoneAngles(
    ghoul2: *mut c_void,
    modelIndex: c_int,
    boneName: *const c_char,
    angles: vec3_t,
    flags: c_int,
    up: c_int,
    right: c_int,
    forward: c_int,
    modelList: *mut qhandle_t,
    blendTime: c_int,
    currentTime: c_int,
) -> qboolean {
    // ruling 30: ctx-less bg-boundary wrapper; engine via the seam cell.
    crate::trap::G2API_SetBoneAngles(
        strap_engine(),
        mp_abi::game::syscalls::G_G2_ANGLEOVERRIDE::GG2AngleoverrideArgs::new(
            ghoul2,
            modelIndex,
            unsafe { CStr::from_ptr(boneName) }.to_owned(),
            &angles,
            flags,
            up,
            right,
            forward,
            modelList,
            blendTime,
            currentTime,
        ),
    )
}

/// Raven `strap_G2API_SetBoneAnim`.
///
/// Source: `oracle/oracle/codemp/game/g_strap.c:31-35`
pub fn strap_G2API_SetBoneAnim(
    ghoul2: *mut c_void,
    modelIndex: c_int,
    boneName: *const c_char,
    startFrame: c_int,
    endFrame: c_int,
    flags: c_int,
    animSpeed: f32,
    currentTime: c_int,
    setFrame: f32,
    blendTime: c_int,
) -> qboolean {
    // ruling 30: ctx-less bg-boundary wrapper; engine via the seam cell.
    crate::trap::G2API_SetBoneAnim(
        strap_engine(),
        mp_abi::game::syscalls::G_G2_PLAYANIM::GG2PlayanimArgs::new(
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
        ),
    )
}

/// Raven `strap_G2API_GetBoneAnim`.
///
/// Source: `oracle/oracle/codemp/game/g_strap.c:37-41`
pub fn strap_G2API_GetBoneAnim(
    ghoul2: *mut c_void,
    boneName: *const c_char,
    currentTime: c_int,
    currentFrame: *mut f32,
    startFrame: *mut c_int,
    endFrame: *mut c_int,
    flags: *mut c_int,
    animSpeed: *mut f32,
    modelList: *mut c_int,
    modelIndex: c_int,
) -> qboolean {
    // ruling 30: ctx-less bg-boundary wrapper; engine via the seam cell.
    crate::trap::G2API_GetBoneAnim(
        strap_engine(),
        mp_abi::game::syscalls::G_G2_GETBONEANIM::GG2GetboneanimArgs::new(
            ghoul2,
            unsafe { CStr::from_ptr(boneName) }.to_owned(),
            currentTime,
            currentFrame,
            startFrame,
            endFrame,
            flags,
            animSpeed,
            modelList,
            modelIndex,
        ),
    )
}

/// Raven `strap_G2API_SetRagDoll`.
///
/// Source: `oracle/oracle/codemp/game/g_strap.c:43-46`
pub fn strap_G2API_SetRagDoll(ghoul2: *mut c_void, params: *mut sharedRagDollParams_t) {
    // ruling 30: ctx-less bg-boundary wrapper; engine via the seam cell.
    crate::trap::G2API_SetRagDoll(
        strap_engine(),
        mp_abi::game::syscalls::G_G2_SETRAGDOLL::GG2SetragdollArgs::new(ghoul2, params),
    );
}

/// Raven `strap_G2API_AnimateG2Models`.
///
/// Source: `oracle/oracle/codemp/game/g_strap.c:48-51`
pub fn strap_G2API_AnimateG2Models(
    ghoul2: *mut c_void,
    time: c_int,
    params: *mut sharedRagDollUpdateParams_t,
) {
    // ruling 30: ctx-less bg-boundary wrapper; engine via the seam cell.
    crate::trap::G2API_AnimateG2Models(
        strap_engine(),
        mp_abi::game::syscalls::G_G2_ANIMATEG2MODELS::GG2Animateg2ModelsArgs::new(
            ghoul2, time, params,
        ),
    );
}

/// Raven `strap_G2API_SetBoneIKState`.
///
/// Source: `oracle/oracle/codemp/game/g_strap.c:53-56`
pub fn strap_G2API_SetBoneIKState(
    ghoul2: *mut c_void,
    time: c_int,
    boneName: *const c_char,
    ikState: c_int,
    params: *mut sharedSetBoneIKStateParams_t,
) -> qboolean {
    // ruling 30: ctx-less bg-boundary wrapper; engine via the seam cell.
    crate::trap::G2API_SetBoneIKState(
        strap_engine(),
        mp_abi::game::syscalls::G_G2_SETBONEIKSTATE::GG2SetboneikstateArgs::new(
            ghoul2,
            time,
            unsafe { CStr::from_ptr(boneName) }.to_owned(),
            ikState,
            params,
        ),
    )
}

/// Raven `strap_G2API_IKMove`.
///
/// Source: `oracle/oracle/codemp/game/g_strap.c:58-61`
pub fn strap_G2API_IKMove(
    ghoul2: *mut c_void,
    time: c_int,
    params: *mut sharedIKMoveParams_t,
) -> qboolean {
    // ruling 30: ctx-less bg-boundary wrapper; engine via the seam cell.
    crate::trap::G2API_IKMove(
        strap_engine(),
        mp_abi::game::syscalls::G_G2_IKMOVE::GG2IkmoveArgs::new(ghoul2, time, params),
    )
}

/// Raven `strap_TrueMalloc`.
///
/// Source: `oracle/oracle/codemp/game/g_strap.c:63-66`
pub fn strap_TrueMalloc(ptr: *mut *mut c_void, size: c_int) {
    // ruling 30: ctx-less bg-boundary wrapper; engine via the seam cell.
    crate::trap::TrueMalloc(
        strap_engine(),
        mp_abi::game::syscalls::G_TRUEMALLOC::GTruemallocArgs::new(ptr, size),
    );
}

/// Raven `strap_TrueFree`.
///
/// Source: `oracle/oracle/codemp/game/g_strap.c:68-71`
pub fn strap_TrueFree(ptr: *mut *mut c_void) {
    // ruling 30: ctx-less bg-boundary wrapper; engine via the seam cell.
    crate::trap::TrueFree(
        strap_engine(),
        mp_abi::game::syscalls::G_TRUEFREE::GTruefreeArgs::new(ptr),
    );
}
