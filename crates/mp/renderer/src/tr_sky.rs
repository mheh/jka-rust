//! Raven `tr_sky.cpp` logic (R3 frontend port).
//!
//! Source: `oracle/codemp/renderer/tr_sky.cpp`

#![allow(non_snake_case)]

// PORT-NOTE (wave 0, `tr_sky.wave0.md`): `MakeSkyVec` needs `backEnd
// .viewParms.zFar`, but `FrameState::view` is still the empty `ViewParms`
// landing placeholder (`render_state/placeholders.rs` — "fields land with
// the `tr_main` R3 wave") and this packet restricts a wave-0 transcriber to
// this one file. Per the interior-safety law's own carve-out ("Tier-2
// fields may be *read* through their existing shapes until their owning
// wave replaces them") and porting-rules §B4 ("state is threaded, not
// reached"), `MakeSkyVec` takes the already-ported tier-2 `viewParms_t`
// directly instead — the same pattern `tr_main.rs`'s wave-0 PORT-NOTE
// established. Flagged for the integrator: once `FrameState::view` lands a
// real shape, thread `&frame.view` instead.

use mp_qshared::shared::q_math::{_VectorAdd as VectorAdd, vec3_origin};
use mp_qshared::shared::vec3_t;

use crate::tr_local::view_parms_t::viewParms_t;

/// Per-subsystem carrier for `tr_sky.cpp`'s file-scope sky-box scratch
/// globals — NAMED BY THIS WAVE per DEC-37 A13.3 (kind-3 cross-call state:
/// `ClearSkyBox` resets it, `AddSkyPolygon` accumulates into it,
/// `MakeSkyVec` reads the clamp bounds).
pub struct SkyState {
    /// Raven `sky_mins[2][6]` — per-axis minimum projected sky-box texture
    /// coordinate (`[0]` = s, `[1]` = t), indexed by face axis.
    ///
    /// Source: `oracle/codemp/renderer/tr_sky.cpp` (file-scope static)
    pub sky_mins: [[f32; 6]; 2],
    /// Raven `sky_maxs[2][6]` — per-axis maximum projected sky-box texture
    /// coordinate.
    ///
    /// Source: `oracle/codemp/renderer/tr_sky.cpp` (file-scope static)
    pub sky_maxs: [[f32; 6]; 2],
    /// Raven `sky_min` — bilerp-seam clamp lower bound `MakeSkyVec` clamps
    /// projected `s`/`t` into. Definition is outside this packet's oracle
    /// slice (`tr_sky.cpp`, above line 276); threaded as a plain field per
    /// DEC-37 A13.3 rather than reached for.
    ///
    /// Source: `oracle/codemp/renderer/tr_sky.cpp` (file-scope static, not in
    /// this packet's slice)
    pub sky_min: f32,
    /// Raven `sky_max` — bilerp-seam clamp upper bound. Same provenance note
    /// as `sky_min`.
    ///
    /// Source: `oracle/codemp/renderer/tr_sky.cpp` (file-scope static, not in
    /// this packet's slice)
    pub sky_max: f32,
}

/// Raven `vec_to_st[6][3]` — `AddSkyPolygon`'s face-axis -> (s,t,dv) index
/// table. Const per the three-kind fn-scope-statics rule (kind 1: never
/// mutated).
///
/// Source: `oracle/codemp/renderer/tr_sky.cpp:49-62`
const VEC_TO_ST: [[i32; 3]; 6] = [
    [-2, 3, 1],
    [2, 3, -1],
    [1, 3, 2],
    [-1, 3, -2],
    [-2, -1, 3],
    [-2, 1, -3],
    // {-1,2,3},
    // {1,2,-3}
];

/// Raven `AddSkyPolygon`.
///
/// Raven: decide which face it maps to, then project new texture coords.
///
/// `nump` drops in favor of `vecs.len()` (the length collapses into the
/// slice, porting-rules §C7/§C10). `sky` is `sky_mins`/`sky_maxs`
/// (`SkyState`, DEC-37 A13.3).
///
/// Source: `oracle/codemp/renderer/tr_sky.cpp:41-125`
pub fn AddSkyPolygon(vecs: &[vec3_t], sky: &mut SkyState) {
    // decide which face it maps to
    let mut v: vec3_t = vec3_origin;
    for vp in vecs {
        VectorAdd(*vp, v, &mut v);
    }
    let av = [v[0].abs(), v[1].abs(), v[2].abs()];

    let axis: usize = if av[0] > av[1] && av[0] > av[2] {
        if v[0] < 0.0 {
            1
        } else {
            0
        }
    } else if av[1] > av[2] && av[1] > av[0] {
        if v[1] < 0.0 {
            3
        } else {
            2
        }
    } else if v[2] < 0.0 {
        5
    } else {
        4
    };

    // project new texture coords
    for vp in vecs {
        let j = VEC_TO_ST[axis][2];
        let dv = if j > 0 {
            vp[(j - 1) as usize]
        } else {
            -vp[(-j - 1) as usize]
        };
        if dv < 0.001 {
            continue; // don't divide by zero
        }
        let j = VEC_TO_ST[axis][0];
        let s = if j < 0 {
            -vp[(-j - 1) as usize] / dv
        } else {
            vp[(j - 1) as usize] / dv
        };
        let j = VEC_TO_ST[axis][1];
        let t = if j < 0 {
            -vp[(-j - 1) as usize] / dv
        } else {
            vp[(j - 1) as usize] / dv
        };

        if s < sky.sky_mins[0][axis] {
            sky.sky_mins[0][axis] = s;
        }
        if t < sky.sky_mins[1][axis] {
            sky.sky_mins[1][axis] = t;
        }
        if s > sky.sky_maxs[0][axis] {
            sky.sky_maxs[0][axis] = s;
        }
        if t > sky.sky_maxs[1][axis] {
            sky.sky_maxs[1][axis] = t;
        }
    }
}

/// Raven `ClearSkyBox`.
///
/// `sky` is `sky_mins`/`sky_maxs` (`SkyState`, DEC-37 A13.3).
///
/// Source: `oracle/codemp/renderer/tr_sky.cpp:230-237`
pub fn ClearSkyBox(sky: &mut SkyState) {
    for i in 0..6usize {
        sky.sky_mins[0][i] = 9999.0;
        sky.sky_mins[1][i] = 9999.0;
        sky.sky_maxs[0][i] = -9999.0;
        sky.sky_maxs[1][i] = -9999.0;
    }
}

/// Raven `st_to_vec[6][3]` — `MakeSkyVec`'s face-axis -> (x,y,z) index
/// table. Const per the three-kind fn-scope-statics rule (kind 1: never
/// mutated).
///
/// Source: `oracle/codemp/renderer/tr_sky.cpp:279-289`
const ST_TO_VEC: [[i32; 3]; 6] = [
    [3, -1, 2],
    [-3, 1, 2],
    [1, 3, 2],
    [-1, -3, 2],
    [-2, -1, 3], // 0 degrees yaw, look straight up
    [2, -1, -3], // look straight down
];

/// Raven `MakeSkyVec`. Out-params `outSt`/`outXYZ` -> return value
/// `(xyz, st)`.
///
/// `view` is `backEnd.viewParms` (tier-2 `viewParms_t`, see top-of-file
/// PORT-NOTE — `zFar` only). `sky` is `sky_min`/`sky_max` (`SkyState`,
/// DEC-37 A13.3).
///
/// PORT-NOTE: the oracle's `if ( outSt )` null-check exists only to let a
/// caller skip the texcoord write; the return-value translation computes
/// `st` unconditionally (cheap, pure) and callers that don't need it simply
/// ignore it (porting-rules §C10: preserve control-flow behavior, not
/// shape).
///
/// Source: `oracle/codemp/renderer/tr_sky.cpp:276-342`
pub fn MakeSkyVec(
    s: f32,
    t: f32,
    axis: usize,
    view: &viewParms_t,
    sky: &SkyState,
) -> (vec3_t, [f32; 2]) {
    // 1 = s, 2 = t, 3 = 2048
    let box_size = view.zFar / 1.75; // div sqrt(3)
    let b: vec3_t = [s * box_size, t * box_size, box_size];

    let mut out_xyz: vec3_t = [0.0; 3];
    for j in 0..3usize {
        let k = ST_TO_VEC[axis][j];
        out_xyz[j] = if k < 0 {
            -b[(-k - 1) as usize]
        } else {
            b[(k - 1) as usize]
        };
    }

    // avoid bilerp seam
    let mut s = (s + 1.0) * 0.5;
    let mut t = (t + 1.0) * 0.5;
    if s < sky.sky_min {
        s = sky.sky_min;
    } else if s > sky.sky_max {
        s = sky.sky_max;
    }

    if t < sky.sky_min {
        t = sky.sky_min;
    } else if t > sky.sky_max {
        t = sky.sky_max;
    }

    t = 1.0 - t;

    (out_xyz, [s, t])
}

/// Raven `FillCloudySkySide`.
///
/// DEFERRED: R4 — every write lives on `tess` (dissolved into R4's
/// tessellation/vertex-building pipeline; R2 `## State ownership` row
/// `tess`), including the `SHADER_MAX_VERTEXES` `Com_Error` bound check
/// (`tess.numVertexes`); no R3 carrier holds `tess.xyz`/`texCoords`/
/// `indexes`/`numVertexes`/`numIndexes` to read or write.
///
/// Source: `oracle/codemp/renderer/tr_sky.cpp:448-496`
pub fn FillCloudySkySide(_mins: [i32; 2], _maxs: [i32; 2], _add_indexes: bool) {
    // DEFERRED: R4 — FillCloudySkySide (see doc comment above)
    // Source: oracle/codemp/renderer/tr_sky.cpp:448-496
}
