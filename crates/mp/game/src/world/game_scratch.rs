//! `GameScratch` — game-tier function-local persistent scratch (safe-state
//! Stage 3, §B3: no `static mut`).
//!
//! Raven kept several rotating/persistent return buffers as function-local
//! `static` storage in the `g_*`/`w_*`/`NPC_*` `.c` files. This struct owns
//! them on `GameWorld`; the owning fns reach them as `ctx.world.scratch.*`.
//! The `tv` 8-slot ring's rotation index semantics are preserved exactly.
#![allow(non_snake_case, non_upper_case_globals)]

use core::ffi::c_int;

use mp_qshared::shared::vec3_t;

use crate::saber::saber_face_t::saberFace_t;

/// Game-tier persistent/rotating scratch, owned by `GameWorld`.
///
/// Each field cites the Raven function-local `static` it replaces.
pub struct GameScratch {
    /// Raven `NPC_AI_GalakMech.c` file-static `vec3_t impactPos4` — cached
    /// impact position shared across `GM_CheckFireState`/`NPC_BSGM_Attack`.
    /// Source: `oracle/codemp/game/NPC_AI_GalakMech.c`
    pub impact_pos_4: vec3_t,

    /// Raven `G_BuildSaberFaces`'s function-local `static saberFace_t faces[12]`
    /// — the per-call blade collision faces, returned to the caller by pointer.
    /// Source: `oracle/codemp/game/w_saber.c:2454-2570`
    pub faces: [saberFace_t; 12],

    /// Raven `tv`'s function-local `static int index` — 8-slot ring cursor.
    /// Source: `oracle/codemp/game/g_utils.c:627-642`
    pub tv_index: c_int,
    /// Raven `tv`'s function-local `static vec3_t vecs[8]` — rotating return
    /// buffer.
    /// Source: `oracle/codemp/game/g_utils.c:627-642`
    pub tv_vecs: [[f32; 3]; 8],
}

impl GameScratch {
    /// A freshly zeroed scratch (all rings at slot 0, all buffers cleared),
    /// matching Raven's zero-initialized function-local statics.
    pub fn zeroed() -> Self {
        Self {
            impact_pos_4: [0.0; 3],
            faces: [saberFace_t {
                v1: [0.0; 3],
                v2: [0.0; 3],
                v3: [0.0; 3],
            }; 12],
            tv_index: 0,
            tv_vecs: [[0.0; 3]; 8],
        }
    }
}

impl Default for GameScratch {
    fn default() -> Self {
        Self::zeroed()
    }
}
