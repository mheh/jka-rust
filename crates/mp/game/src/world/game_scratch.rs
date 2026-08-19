//! `GameScratch` holds game-tier scratch state that Raven kept as function-local statics
//! (porting-rules §B3: no `static mut`).
//!
//! Raven kept several rotating and persistent return buffers as function-local `static`
//! storage in the `g_*`, `w_*`, and `NPC_*` `.c` files.
//! This struct owns them on `GameWorld`.
//! The owning functions reach them as `ctx.world.scratch.*`.
//! A caller can hold up to 8 live results from repeat calls to `tv` before the ring overwrites the oldest slot.
#![allow(non_snake_case, non_upper_case_globals)]

use core::ffi::{c_char, c_int};

use mp_qshared::shared::vec3_t;

use crate::saber::saber_face_t::saberFace_t;

/// Game-tier persistent/rotating scratch, owned by `GameWorld`.
///
/// Each field cites the Raven function-local `static` it replaces.
pub struct GameScratch {
    /// Raven `NPC_AI_GalakMech.c` file-static `vec3_t impactPos4`.
    /// It caches the impact position shared across `GM_CheckFireState` and `NPC_BSGM_Attack`.
    /// Source: `oracle/codemp/game/NPC_AI_GalakMech.c`
    pub impact_pos_4: vec3_t,

    /// Raven `G_BuildSaberFaces`'s function-local `static saberFace_t faces[12]`.
    /// It holds the per-call blade collision faces, returned to the caller by pointer.
    /// Source: `oracle/codemp/game/w_saber.c:2454-2570`
    pub faces: [saberFace_t; 12],

    /// Raven `tv`'s function-local `static int index`, the 8-slot ring cursor.
    /// Source: `oracle/codemp/game/g_utils.c:627-642`
    pub tv_index: c_int,
    /// Raven `tv`'s function-local `static vec3_t vecs[8]`, the rotating return buffer.
    /// Source: `oracle/codemp/game/g_utils.c:627-642`
    pub tv_vecs: [[f32; 3]; 8],

    /// C-string staging for `Q3_GetString`'s `SET_TARGET` arm.
    /// `target` is now an owned `Option<String>`, so this field stages its NUL-terminated view, returned to the caller by pointer.
    /// The ICARUS dispatch copies the string out immediately, so the buffer only needs to outlive the call.
    /// Raven returned `ent->target` directly, and this field replaces that persistent pool pointer.
    /// The buffer is sized to the engine's 2048-byte `T_G_ICARUS_GETSTRING.value`.
    /// Source: `oracle/codemp/game/g_ICARUScb.c:1642-1854`
    pub icarus_get_string: [c_char; 2048],
}

impl GameScratch {
    /// Creates a scratch with all rings at slot 0 and all buffers cleared.
    /// Raven's function-local statics started zeroed the same way.
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
            icarus_get_string: [0; 2048],
        }
    }
}

impl Default for GameScratch {
    fn default() -> Self {
        Self::zeroed()
    }
}
