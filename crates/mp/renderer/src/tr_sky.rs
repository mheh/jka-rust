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

use mp_engine_qcommon::common::{com_error, Common};
use mp_engine_qcommon::common_fns::Q_acos;
use mp_qshared::shared::error_parm::errorParm_t;
use mp_qshared::shared::q_math::{
    _DotProduct as DotProduct, _VectorAdd as VectorAdd, _VectorScale as VectorScale, vec3_origin,
    CrossProduct, PerpendicularVector,
};
use mp_qshared::shared::vec3_t;
use native_math::qmath::VectorNormalize;

use crate::render_state::frame_state::FrameState;
use crate::render_state::gpu_resources::GpuResources;
use crate::render_state::image_asset::ImageHandle;
use crate::render_state::render_assets::RenderAssets;
use crate::render_state::renderer_cvars::RendererCvars;
use crate::render_state::shader_asset::ShaderHandle;
use crate::tr_backend::GL_Bind;
use crate::tr_local::view_parms_t::viewParms_t;
use crate::tr_shade_calc::myftol;

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
    /// Raven `sky_clip[6]` — per-stage clip-plane normal `ClipSkyPolygon`
    /// clips the box polygon against. Definition (the six normals) is
    /// outside this packet's oracle slice (`tr_sky.cpp`, above line 134);
    /// threaded as a plain field per DEC-37 A13.3 rather than reached for —
    /// same provenance treatment as `sky_min`/`sky_max` above.
    ///
    /// Source: `oracle/codemp/renderer/tr_sky.cpp` (file-scope static, not in
    /// this packet's slice)
    pub sky_clip: [vec3_t; 6],
    /// Raven `s_skyPoints[SKY_SUBDIVISIONS+1][SKY_SUBDIVISIONS+1]` — the
    /// projected sky-box grid vertices `FillCloudBox`/`R_InitSkyTexCoords`
    /// write and `DrawSkySide` reads. `SKY_SUBDIVISIONS` (the fixed C array
    /// dimension) is not in this packet's oracle slice (wave 1 `tr_sky.cpp`
    /// top-of-file `#define` block) — sized as an owned `Vec<Vec<_>>`
    /// instead of a fixed array (interior free, DEC-37 ruling 1) so the
    /// missing constant blocks only the dimension, not the field's shape.
    ///
    /// Source: `oracle/codemp/renderer/tr_sky.cpp:344` (file-scope static)
    pub sky_points: Vec<Vec<vec3_t>>,
    /// Raven `s_skyTexCoords[SKY_SUBDIVISIONS+1][SKY_SUBDIVISIONS+1]` — the
    /// sky-box grid's texture coordinates. Same sizing note as
    /// `sky_points`.
    ///
    /// Source: `oracle/codemp/renderer/tr_sky.cpp:345` (file-scope static)
    pub sky_tex_coords: Vec<Vec<[f32; 2]>>,
    /// Raven `s_cloudTexCoords[6][SKY_SUBDIVISIONS+1][SKY_SUBDIVISIONS+1]` —
    /// per-face cloud-layer texture coordinates, precomputed by
    /// `R_InitSkyTexCoords` and consumed by `FillCloudBox`. Same sizing note
    /// as `sky_points`.
    ///
    /// Source: `oracle/codemp/renderer/tr_sky.cpp` (file-scope static, not in
    /// this packet's slice)
    pub cloud_tex_coords: Vec<Vec<Vec<[f32; 2]>>>,
    /// Raven `s_cloudTexP[6][SKY_SUBDIVISIONS+1][SKY_SUBDIVISIONS+1]` — the
    /// per-vertex cloud-layer intersection parameter `R_InitSkyTexCoords`
    /// precomputes. Same sizing note as `sky_points`.
    ///
    /// Source: `oracle/codemp/renderer/tr_sky.cpp` (file-scope static, not in
    /// this packet's slice)
    pub cloud_tex_p: Vec<Vec<Vec<f32>>>,
}

/// Raven `SKY_SUBDIVISIONS`.
///
/// Source: `oracle/codemp/renderer/tr_sky.cpp:7`
const SKY_SUBDIVISIONS: i32 = 8;

/// Raven `HALF_SKY_SUBDIVISIONS` (`SKY_SUBDIVISIONS/2`).
///
/// Source: `oracle/codemp/renderer/tr_sky.cpp:8`
const HALF_SKY_SUBDIVISIONS: i32 = SKY_SUBDIVISIONS / 2;

/// Raven `MAX_CLIP_VERTS`.
///
/// Source: `oracle/codemp/renderer/tr_sky.cpp:128`
const MAX_CLIP_VERTS: usize = 64;

/// Raven `ON_EPSILON` — `tr_sky.cpp`'s own file-local definition (this
/// codebase carries three other, disagreeing `ON_EPSILON` locals; none is a
/// substitute).
///
/// Raven: point on plane side epsilon.
///
/// Source: `oracle/codemp/renderer/tr_sky.cpp:127`
const ON_EPSILON: f32 = 0.1;

/// `SIDE_FRONT`/`SIDE_BACK`/`SIDE_ON` — planar-side classification codes.
///
/// Source: `oracle/codemp/renderer/tr_local.h:868-870`
const SIDE_FRONT: i32 = 0;
const SIDE_BACK: i32 = 1;
const SIDE_ON: i32 = 2;

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

/// Raven `ClipSkyPolygon`.
///
/// Raven: recursively clips a sky-box polygon against `sky_clip`'s planes;
/// once fully clipped (`stage == 6`) the fragment is handed off to
/// `AddSkyPolygon`.
///
/// `nump` drops in favor of `vecs.len()` (porting-rules §C7/§C10, same
/// treatment wave 0's `AddSkyPolygon` already applied); the C `dists`/
/// `sides`/`newv` fixed scratch arrays (sized by `MAX_CLIP_VERTS`) become
/// owned growable `Vec`s instead (interior free, DEC-37 ruling 1); the
/// overflow guard keeps the constant. `sky` carries `sky_clip`
/// (`SkyState`, DEC-37 A13.3, definition outside this packet's slice — same
/// treatment as wave 0's `sky_min`/`sky_max`).
///
/// Source: `oracle/codemp/renderer/tr_sky.cpp:134-223`
pub fn ClipSkyPolygon(vecs: &[vec3_t], stage: i32, sky: &mut SkyState) {
    let nump = vecs.len();

    if nump > MAX_CLIP_VERTS - 2 {
        com_error(
            errorParm_t::ERR_DROP,
            "ClipSkyPolygon: MAX_CLIP_VERTS".to_string(),
        );
    }
    if stage == 6 {
        // fully clipped, so draw it
        AddSkyPolygon(vecs, sky);
        return;
    }

    let mut front = false;
    let mut back = false;
    let norm = sky.sky_clip[stage as usize];
    let mut sides: Vec<i32> = Vec::with_capacity(nump + 1);
    let mut dists: Vec<f32> = Vec::with_capacity(nump + 1);
    for v in vecs {
        let d = DotProduct(*v, norm);
        if d > ON_EPSILON {
            front = true;
            sides.push(SIDE_FRONT);
        } else if d < -ON_EPSILON {
            back = true;
            sides.push(SIDE_BACK);
        } else {
            sides.push(SIDE_ON);
        }
        dists.push(d);
    }

    if !front || !back {
        // not clipped
        ClipSkyPolygon(vecs, stage + 1, sky);
        return;
    }

    // clip it — append vecs[0]/sides[0]/dists[0] as the closing sentinel
    // vertex (Raven: `sides[i] = sides[0]; dists[i] = dists[0]; VectorCopy
    // (vecs, vecs+(i*3));` with `i == nump` at this point).
    sides.push(sides[0]);
    dists.push(dists[0]);
    let mut vecs = vecs.to_vec();
    vecs.push(vecs[0]);

    let mut newv: [Vec<vec3_t>; 2] = [Vec::new(), Vec::new()];

    for i in 0..nump {
        let v = vecs[i];
        match sides[i] {
            SIDE_FRONT => newv[0].push(v),
            SIDE_BACK => newv[1].push(v),
            SIDE_ON => {
                newv[0].push(v);
                newv[1].push(v);
            }
            _ => {}
        }

        if sides[i] == SIDE_ON || sides[i + 1] == SIDE_ON || sides[i + 1] == sides[i] {
            continue;
        }

        let d = dists[i] / (dists[i] - dists[i + 1]);
        let mut e: vec3_t = [0.0; 3];
        for j in 0..3usize {
            e[j] = v[j] + d * (vecs[i + 1][j] - v[j]);
        }
        newv[0].push(e);
        newv[1].push(e);
    }

    // continue
    ClipSkyPolygon(&newv[0], stage + 1, sky);
    ClipSkyPolygon(&newv[1], stage + 1, sky);
}

/// Raven `DrawSkySide`.
///
/// GL/WGL surface (`qglBegin`/`qglTexCoord2fv`/`qglVertex3fv`/`qglEnd`): DEC-01/
/// DEC-37 — the backend is an idiomatic wgpu rewrite, not a GL
/// transcription, and R2 leaves these fixed-function entry points unhomed
/// (`GpuResources::gl_state` is a named placeholder until R4). Each is
/// DEFERRED at its call site below; the surrounding CPU logic (the
/// subdivision loop bounds and per-vertex texcoord/vertex lookups) is
/// ported in full — `GL_Bind` (already ported, wave 0 `tr_backend.cpp`) is
/// called for real.
///
/// `image` is the oracle's `struct image_s *image` → `Option<ImageHandle>`
/// per the interior-safety law (handle, not a raw pointer).
///
/// Source: `oracle/codemp/renderer/tr_sky.cpp:347-376`
pub fn DrawSkySide(
    gpu: &mut GpuResources,
    image: Option<ImageHandle>,
    mins: [i32; 2],
    maxs: [i32; 2],
    sky: &SkyState,
) {
    GL_Bind(gpu, image);

    for t in (mins[1] + HALF_SKY_SUBDIVISIONS)..(maxs[1] + HALF_SKY_SUBDIVISIONS) {
        // DEFERRED: R4 — qglBegin(GL_TRIANGLE_STRIP) (DEC-37 A13.2, unhomed
        // GL/WGL entry point)
        // Source: oracle/codemp/renderer/tr_sky.cpp:362

        for s in (mins[0] + HALF_SKY_SUBDIVISIONS)..=(maxs[0] + HALF_SKY_SUBDIVISIONS) {
            let _tex_coord_0 = sky.sky_tex_coords[t as usize][s as usize];
            let _vertex_0 = sky.sky_points[t as usize][s as usize];
            let _tex_coord_1 = sky.sky_tex_coords[(t + 1) as usize][s as usize];
            let _vertex_1 = sky.sky_points[(t + 1) as usize][s as usize];
            // DEFERRED: R4 — qglTexCoord2fv/qglVertex3fv ×2 (DEC-37 A13.2,
            // unhomed GL/WGL entry points)
            // Source: oracle/codemp/renderer/tr_sky.cpp:367-371
        }

        // DEFERRED: R4 — qglEnd() (DEC-37 A13.2, unhomed GL/WGL entry point)
        // Source: oracle/codemp/renderer/tr_sky.cpp:374
    }
}

/// Raven `FillCloudBox`.
///
/// `shader` is kept for signature parity only — the oracle's
/// `shader->sky->fullClouds` branch is hard-coded to `if ( 1 )` with a
/// `// FIXME?` left in place, so the field is never actually read
/// (porting-rules §A2: port the behavior as written, not a "cleaner"
/// reading of the dead branch). `view` threads `backEnd.viewParms` into
/// `MakeSkyVec` (same top-of-file PORT-NOTE precedent as wave 0). `sky`
/// carries `sky_mins`/`sky_maxs`/`sky_points`/`sky_tex_coords`/
/// `cloud_tex_coords` (`SkyState`, DEC-37 A13.3). The `floor`/`ceil` calls
/// are the C `<math.h>` double overloads (no `floorf`/`ceilf`), so their
/// operand promotes to `f64` and the result rounds back to `f32` once at
/// the assignment (wave-0 ruling 12).
///
/// Source: `oracle/codemp/renderer/tr_sky.cpp:498-591`
pub fn FillCloudBox(_shader: ShaderHandle, stage: i32, view: &viewParms_t, sky: &mut SkyState) {
    for i in 0..6usize {
        let min_t: f32;

        // if ( 1 ) // FIXME? shader->sky->fullClouds )
        {
            min_t = -(HALF_SKY_SUBDIVISIONS as f32);

            // still don't want to draw the bottom, even if fullClouds
            if i == 5 {
                continue;
            }
        }
        // (the oracle's `else` branch below `if ( 1 )` is dead code —
        // dropped per porting-rules §20, not transcribed)

        let half = HALF_SKY_SUBDIVISIONS as f64;
        sky.sky_mins[0][i] = ((sky.sky_mins[0][i] as f64 * half).floor() / half) as f32;
        sky.sky_mins[1][i] = ((sky.sky_mins[1][i] as f64 * half).floor() / half) as f32;
        sky.sky_maxs[0][i] = ((sky.sky_maxs[0][i] as f64 * half).ceil() / half) as f32;
        sky.sky_maxs[1][i] = ((sky.sky_maxs[1][i] as f64 * half).ceil() / half) as f32;

        if sky.sky_mins[0][i] >= sky.sky_maxs[0][i] || sky.sky_mins[1][i] >= sky.sky_maxs[1][i] {
            continue;
        }

        let mut sky_mins_subd = [
            myftol(sky.sky_mins[0][i] * HALF_SKY_SUBDIVISIONS as f32),
            myftol(sky.sky_mins[1][i] * HALF_SKY_SUBDIVISIONS as f32),
        ];
        let mut sky_maxs_subd = [
            myftol(sky.sky_maxs[0][i] * HALF_SKY_SUBDIVISIONS as f32),
            myftol(sky.sky_maxs[1][i] * HALF_SKY_SUBDIVISIONS as f32),
        ];

        if sky_mins_subd[0] < -HALF_SKY_SUBDIVISIONS {
            sky_mins_subd[0] = -HALF_SKY_SUBDIVISIONS;
        } else if sky_mins_subd[0] > HALF_SKY_SUBDIVISIONS {
            sky_mins_subd[0] = HALF_SKY_SUBDIVISIONS;
        }
        if (sky_mins_subd[1] as f32) < min_t {
            sky_mins_subd[1] = min_t as i32;
        } else if sky_mins_subd[1] > HALF_SKY_SUBDIVISIONS {
            sky_mins_subd[1] = HALF_SKY_SUBDIVISIONS;
        }

        if sky_maxs_subd[0] < -HALF_SKY_SUBDIVISIONS {
            sky_maxs_subd[0] = -HALF_SKY_SUBDIVISIONS;
        } else if sky_maxs_subd[0] > HALF_SKY_SUBDIVISIONS {
            sky_maxs_subd[0] = HALF_SKY_SUBDIVISIONS;
        }
        if (sky_maxs_subd[1] as f32) < min_t {
            sky_maxs_subd[1] = min_t as i32;
        } else if sky_maxs_subd[1] > HALF_SKY_SUBDIVISIONS {
            sky_maxs_subd[1] = HALF_SKY_SUBDIVISIONS;
        }

        // iterate through the subdivisions
        for t in
            (sky_mins_subd[1] + HALF_SKY_SUBDIVISIONS)..=(sky_maxs_subd[1] + HALF_SKY_SUBDIVISIONS)
        {
            for s in (sky_mins_subd[0] + HALF_SKY_SUBDIVISIONS)
                ..=(sky_maxs_subd[0] + HALF_SKY_SUBDIVISIONS)
            {
                let (xyz, _st) = MakeSkyVec(
                    (s - HALF_SKY_SUBDIVISIONS) as f32 / HALF_SKY_SUBDIVISIONS as f32,
                    (t - HALF_SKY_SUBDIVISIONS) as f32 / HALF_SKY_SUBDIVISIONS as f32,
                    i,
                    view,
                    sky,
                );
                sky.sky_points[t as usize][s as usize] = xyz;

                sky.sky_tex_coords[t as usize][s as usize] =
                    sky.cloud_tex_coords[i][t as usize][s as usize];
            }
        }

        // only add indexes for first stage
        FillCloudySkySide(sky_mins_subd, sky_maxs_subd, stage == 0);
    }
}

/// Raven `R_InitSkyTexCoords`.
///
/// `view` threads `backEnd.viewParms` — same top-of-file PORT-NOTE precedent
/// as wave 0's `MakeSkyVec` (`FrameState::view` is still the empty
/// `ViewParms` landing placeholder; rethread once it lands a real shape).
/// `sky` carries `s_cloudTexP`/`s_cloudTexCoords` (`SkyState`, DEC-37 A13.3).
///
/// Double-promotion (wave-0 ruling 12): the oracle's `p` expression mixes
/// float sub-terms with a `sqrt()` call (the `<math.h>` double overload, no
/// `sqrtf`) — `1.0f / (2 * DotProduct(...))` is pure `f32` arithmetic
/// (computed and implicitly rounded to that precision by the C compiler
/// before the outer multiply, since it's a self-contained parenthesized
/// float sub-expression), while `sqrt(...)`'s argument and the term it
/// feeds are `f64`; the final multiply and assignment to `float p` round to
/// `f32` once. Reproduced here as two explicitly separate precision
/// regions, not one blanket `f64` computation, to match that exactly.
///
/// Source: `oracle/codemp/renderer/tr_sky.cpp:626-680`
pub fn R_InitSkyTexCoords(height_cloud: f32, view: &mut viewParms_t, sky: &mut SkyState) {
    let radius_world: f32 = 4096.0;

    // Raven's `s_cloudTexP[6][SKY_SUBDIVISIONS+1][SKY_SUBDIVISIONS+1]` and
    // `s_cloudTexCoords[6][...][...]` are fixed-size static arrays; the port
    // holds them as jagged `Vec`s (`SkyState`), so this writer sizes them to
    // those dimensions before it indexes. The loop below fills every element,
    // so a fresh zeroed allocation each call matches the static array exactly.
    // Source: `oracle/codemp/renderer/tr_sky.cpp:39-40`
    let dim = (SKY_SUBDIVISIONS + 1) as usize;
    sky.cloud_tex_p = vec![vec![vec![0.0f32; dim]; dim]; 6];
    sky.cloud_tex_coords = vec![vec![vec![[0.0f32; 2]; dim]; dim]; 6];

    // init zfar so MakeSkyVec works even though
    // a world hasn't been bounded
    view.zFar = 1024.0;

    for i in 0..6usize {
        for t in 0..=SKY_SUBDIVISIONS {
            for s in 0..=SKY_SUBDIVISIONS {
                // compute vector from view origin to sky side integral point
                let (sky_vec, _st) = MakeSkyVec(
                    (s - HALF_SKY_SUBDIVISIONS) as f32 / HALF_SKY_SUBDIVISIONS as f32,
                    (t - HALF_SKY_SUBDIVISIONS) as f32 / HALF_SKY_SUBDIVISIONS as f32,
                    i,
                    view,
                    sky,
                );

                // compute parametric value 'p' that intersects with cloud layer
                let (sx, sy, sz) = (sky_vec[0], sky_vec[1], sky_vec[2]);
                // `1.0f / ( 2 * DotProduct(...) )` — self-contained f32 term.
                let inv_factor: f32 = 1.0 / (2.0 * DotProduct(sky_vec, sky_vec));
                // `SQR(...)` chain feeding `sqrt()` — still pure float terms
                // until the `sqrt()` call itself promotes to f64.
                // `SQR(a)` is `((a)*(a))`; each term keeps the macro's own
                // parenthesisation so the f32 multiply order matches exactly.
                let sqr_sum: f32 = (sz * sz) * (radius_world * radius_world)
                    + 2.0 * (sx * sx) * radius_world * height_cloud
                    + (sx * sx) * (height_cloud * height_cloud)
                    + 2.0 * (sy * sy) * radius_world * height_cloud
                    + (sy * sy) * (height_cloud * height_cloud)
                    + 2.0 * (sz * sz) * radius_world * height_cloud
                    + (sz * sz) * (height_cloud * height_cloud);
                let sqrt_val: f64 = (sqr_sum as f64).sqrt();
                let inner: f64 = (-2.0 * sz * radius_world) as f64 + 2.0 * sqrt_val;
                let p: f32 = (inv_factor as f64 * inner) as f32;

                sky.cloud_tex_p[i][t as usize][s as usize] = p;

                // compute intersection point based on p
                let mut v: vec3_t = [0.0; 3];
                VectorScale(sky_vec, p, &mut v);
                v[2] += radius_world;

                // compute vector from world origin to intersection point 'v'
                VectorNormalize(&mut v);

                let s_rad = Q_acos(v[0]);
                let t_rad = Q_acos(v[1]);

                sky.cloud_tex_coords[i][t as usize][s as usize] = [s_rad, t_rad];
            }
        }
    }
}

/// Raven `RB_ClipSkyPolygons`.
///
/// DEFERRED: R4 — the clip loop's every read comes from `input`
/// (`shaderCommands_t *`, the same dissolved type as the global `tess` — R2
/// `## State ownership` row `tess`: "dissolved into R4's
/// tessellation/vertex-building pipeline ... no single global scratch
/// buffer survives the new topology"; no R3 type exists for
/// `input->numIndexes`/`->xyz`/`->indexes` to be read from). No computation
/// survives once `input` is removed — the loop's only output (`p[j]`, via
/// `VectorSubtract` against `backEnd.viewParms.ori.origin`) feeds straight
/// into the already-ported `ClipSkyPolygon`. `ClearSkyBox`'s reset at the top
/// of the fn is unconditional and independent of `input`, so it is
/// transcribed for real (porting-rules: port the surrounding CPU logic,
/// defer only the blocked piece). `_view` threads `backEnd.viewParms` for
/// the deferred leg's future completion (top-of-file PORT-NOTE precedent).
///
/// Source: `oracle/codemp/renderer/tr_sky.cpp:244-261`
pub fn RB_ClipSkyPolygons(_view: &viewParms_t, sky: &mut SkyState) {
    ClearSkyBox(sky);

    // DEFERRED: R4 — the `for (i = 0; i < input->numIndexes; i += 3)` clip
    // loop (see doc comment above)
    // Source: oracle/codemp/renderer/tr_sky.cpp:251-260
}

/// Raven `DrawSkyBox`.
///
/// The `HALF_SKY_SUBDIVISIONS`-scaled bounds keep the oracle's plain C
/// float->int truncating cast (`porting-rules §C10` — no `myftol` call in
/// this body, unlike sibling `FillCloudBox`/`R_BuildCloudData`'s
/// `FillCloudBox`, which does call it). The `floor`/`ceil` calls are the
/// `<math.h>` double overloads (no `floorf`/`ceilf`), so wave-0 ruling 12
/// applies the same f64-intermediate, round-once treatment `FillCloudBox`
/// already used for its identical expression shape.
/// `Com_Memset( s_skyTexCoords, 0, sizeof( s_skyTexCoords ) )` translates to
/// a direct zero-fill: the target is an owned `Vec<Vec<_>>`, not a raw
/// buffer, and this file bans `unsafe` even for tier-2 raw-pointer reads, so
/// the already-ported `Com_Memset(dest: *mut (), ...)` cannot be called here.
///
/// `sky` carries `sky_min`/`sky_max` (write)/`sky_mins`/`sky_maxs`
/// (write)/`sky_points`/`sky_tex_coords` (`SkyState`, DEC-37 A13.3 — already
/// landed by wave 0/1, not newly named here). `view` threads
/// `backEnd.viewParms` into `MakeSkyVec` (top-of-file PORT-NOTE precedent).
///
/// `shader->sky->outerbox[i]` feeds `DrawSkySide`'s `image` argument, but
/// `ShaderAsset::sky` is still the empty `SkyParms` placeholder
/// (`render_state/placeholders.rs` — untouched by wave 0, fields land with
/// the `tr_shader` wave that ports `skyParms_t`) — `outerbox` has no landed
/// field, and this file cannot extend `placeholders.rs` (out of scope):
/// DEFERRED, `todo!()`, not a guess. `gpu`/`shader_handle` are kept unread
/// for call-site signature parity (same "kept for signature parity only"
/// precedent this file's own `FillCloudBox` already established for its
/// `_shader` param) — nothing downstream of the `outerbox` gap is reachable
/// to read them.
///
/// Source: `oracle/codemp/renderer/tr_sky.cpp:378-446`
pub fn DrawSkyBox(
    _gpu: &mut GpuResources,
    _shader_handle: ShaderHandle,
    view: &viewParms_t,
    sky: &mut SkyState,
) {
    sky.sky_min = 0.0;
    sky.sky_max = 1.0;

    for row in sky.sky_tex_coords.iter_mut() {
        for cell in row.iter_mut() {
            *cell = [0.0, 0.0];
        }
    }

    for i in 0..6usize {
        let half = HALF_SKY_SUBDIVISIONS as f64;
        sky.sky_mins[0][i] = ((sky.sky_mins[0][i] as f64 * half).floor() / half) as f32;
        sky.sky_mins[1][i] = ((sky.sky_mins[1][i] as f64 * half).floor() / half) as f32;
        sky.sky_maxs[0][i] = ((sky.sky_maxs[0][i] as f64 * half).ceil() / half) as f32;
        sky.sky_maxs[1][i] = ((sky.sky_maxs[1][i] as f64 * half).ceil() / half) as f32;

        if sky.sky_mins[0][i] >= sky.sky_maxs[0][i] || sky.sky_mins[1][i] >= sky.sky_maxs[1][i] {
            continue;
        }

        let mut sky_mins_subd = [
            (sky.sky_mins[0][i] * HALF_SKY_SUBDIVISIONS as f32) as i32,
            (sky.sky_mins[1][i] * HALF_SKY_SUBDIVISIONS as f32) as i32,
        ];
        let mut sky_maxs_subd = [
            (sky.sky_maxs[0][i] * HALF_SKY_SUBDIVISIONS as f32) as i32,
            (sky.sky_maxs[1][i] * HALF_SKY_SUBDIVISIONS as f32) as i32,
        ];

        if sky_mins_subd[0] < -HALF_SKY_SUBDIVISIONS {
            sky_mins_subd[0] = -HALF_SKY_SUBDIVISIONS;
        } else if sky_mins_subd[0] > HALF_SKY_SUBDIVISIONS {
            sky_mins_subd[0] = HALF_SKY_SUBDIVISIONS;
        }
        if sky_mins_subd[1] < -HALF_SKY_SUBDIVISIONS {
            sky_mins_subd[1] = -HALF_SKY_SUBDIVISIONS;
        } else if sky_mins_subd[1] > HALF_SKY_SUBDIVISIONS {
            sky_mins_subd[1] = HALF_SKY_SUBDIVISIONS;
        }

        if sky_maxs_subd[0] < -HALF_SKY_SUBDIVISIONS {
            sky_maxs_subd[0] = -HALF_SKY_SUBDIVISIONS;
        } else if sky_maxs_subd[0] > HALF_SKY_SUBDIVISIONS {
            sky_maxs_subd[0] = HALF_SKY_SUBDIVISIONS;
        }
        if sky_maxs_subd[1] < -HALF_SKY_SUBDIVISIONS {
            sky_maxs_subd[1] = -HALF_SKY_SUBDIVISIONS;
        } else if sky_maxs_subd[1] > HALF_SKY_SUBDIVISIONS {
            sky_maxs_subd[1] = HALF_SKY_SUBDIVISIONS;
        }

        // iterate through the subdivisions
        for t in
            (sky_mins_subd[1] + HALF_SKY_SUBDIVISIONS)..=(sky_maxs_subd[1] + HALF_SKY_SUBDIVISIONS)
        {
            for s in (sky_mins_subd[0] + HALF_SKY_SUBDIVISIONS)
                ..=(sky_maxs_subd[0] + HALF_SKY_SUBDIVISIONS)
            {
                let (xyz, st) = MakeSkyVec(
                    (s - HALF_SKY_SUBDIVISIONS) as f32 / HALF_SKY_SUBDIVISIONS as f32,
                    (t - HALF_SKY_SUBDIVISIONS) as f32 / HALF_SKY_SUBDIVISIONS as f32,
                    i,
                    view,
                    sky,
                );
                sky.sky_tex_coords[t as usize][s as usize] = st;
                sky.sky_points[t as usize][s as usize] = xyz;
            }
        }

        // DEFERRED: skyParms_t.outerbox — SkyParms interior not yet landed
        // (see doc comment above); lands with a later tr_shader wave
        // Source: oracle/codemp/renderer/tr_local.h:449-452
        todo!("Port skyParms_t.outerbox — oracle/codemp/renderer/tr_local.h:449-452");
    }
}

/// Raven `R_BuildCloudData`.
///
/// `input->shader` resolves through `RenderAssets::shaders` (same
/// handle-threading translation as sibling `DrawSkyBox`, and the same
/// `if let Some(...)` guard convention this crate already uses at
/// `assets.shaders.get(...)` call sites). `assert( shader->sky )` becomes
/// `debug_assert!` (existing precedent, e.g. `tr_scene.rs`'s
/// `debug_assert!(ent.renderfx >= 0)`) — `ShaderAsset::sky` is a real,
/// landed `Option<SkyParms>` field, so the assert itself is checkable even
/// though `SkyParms`'s interior is still the empty wave-0 placeholder.
///
/// `sky_min`/`sky_max`'s RHS mixes an unsuffixed (`double`) literal with an
/// `f`-suffixed one (`1.0 / 256.0f`), so the divide promotes to `f64` and
/// rounds once at the assignment (wave-0 ruling 12) — Raven's own
/// `// FIXME: not correct?` comment is kept verbatim. `tess.numIndexes = 0;
/// tess.numVertexes = 0;` write the dissolved `tess`/`shaderCommands_t` (R2
/// `## State ownership` row `tess`, no R3 carrier — DEFERRED).
///
/// `input->shader->sky->cloudHeight` gates the `FillCloudBox` loop, but
/// `ShaderAsset::sky`'s interior (`SkyParms`) has no landed `cloudHeight`
/// field yet (same placeholder as `DrawSkyBox`'s `outerbox` gap, and this
/// file cannot extend `placeholders.rs`): DEFERRED, `todo!()`, not a guess.
/// `input->shader->numUnfoggedPasses` is real
/// (`ShaderAsset::num_unfogged_passes`) and threads straight into the
/// already-ported `FillCloudBox`.
///
/// Source: `oracle/codemp/renderer/tr_sky.cpp:596-619`
#[allow(unreachable_code, unused_variables)]
pub fn R_BuildCloudData(
    assets: &RenderAssets,
    shader_handle: ShaderHandle,
    view: &viewParms_t,
    sky: &mut SkyState,
) {
    let Some(shader) = assets.shaders.get(shader_handle) else {
        return;
    };

    debug_assert!(shader.sky.is_some(), "R_BuildCloudData: shader->sky");

    // FIXME: not correct?
    sky.sky_min = (1.0_f64 / 256.0_f64) as f32;
    sky.sky_max = (255.0_f64 / 256.0_f64) as f32;

    // DEFERRED: R4 — tess.numIndexes = 0; tess.numVertexes = 0; (dissolved
    // `tess`/`shaderCommands_t`, R2 `## State ownership` row `tess`, no R3
    // carrier)
    // Source: oracle/codemp/renderer/tr_sky.cpp:609-610

    // DEFERRED: skyParms_t.cloudHeight — SkyParms interior not yet landed
    // (see doc comment above); lands with a later tr_shader wave
    // Source: oracle/codemp/renderer/tr_local.h:449-452
    if todo!("Port skyParms_t.cloudHeight — oracle/codemp/renderer/tr_local.h:449-452") {
        for i in 0..shader.num_unfogged_passes {
            FillCloudBox(shader_handle, i, view, sky);
        }
    }
}

/// Raven `RB_DrawSun` — wave 4.
///
/// `frame` carries `backEnd.skyRenderedThisView`
/// (`FrameState::sky_rendered_this_view`) and `tr.sunDirection`
/// (`FrameState::sun_direction`, landed by an earlier wave — R2
/// `## State ownership` row "`tr` frontend scratch/counters"). `common`/
/// `cvars` read `r_drawSun` through the live engine cvar table (DEC-37
/// A13.1 — the established `common.cvar(cvars.r_x).integer` idiom this
/// crate already uses, e.g. `tr_light.rs`'s `R_SetupEntityLightingGrid`).
/// `view` is `backEnd.viewParms` (tier-2 `viewParms_t`, `zFar` only — the
/// same top-of-file PORT-NOTE precedent this file's `DrawSkyBox`/
/// `FillCloudBox` already established for the identical `backEnd.viewParms`
/// object).
///
/// `dist`/`size`: Raven's `zFar / 1.75` and `dist * 0.4` both mix an
/// unsuffixed (`double`) literal with a `float` operand, so each divide/
/// multiply promotes to `f64` and rounds back to `f32` once at the
/// assignment (wave-0 ruling 12).
///
/// `qglLoadMatrixf`/`qglTranslatef`/`qglDepthRange` are unhomed GL/WGL entry
/// points (DEC-01/DEC-37, `GpuResources::gl_state` a named placeholder until
/// R4) — DEFERRED at their call sites; the surrounding CPU math (`dist`/
/// `size`/`origin`/`vec1`/`vec2`) is transcribed for real, since none of it
/// depends on a GL call's result.
///
/// `RB_BeginSurface( tr.sunShader, tess.fogNum )` through `RB_EndSurface()`
/// (the whole quad-stamp body building the sun's four tess vertices) is
/// DEFERRED whole: every write inside targets the dissolved `tess`/
/// `shaderCommands_t` global (R2 `## State ownership` row `tess` — "no
/// single global scratch buffer survives the new topology"), and
/// `tr.sunShader` itself has no R3 carrier — R2's `## State ownership` names
/// `tr.sunDirection`'s home on `FrameState` but no home for `sunShader`, so
/// even `RB_BeginSurface`'s `shader: ShaderHandle` argument cannot be
/// produced without inventing a handle. Escalate: a `tr.sunShader` carrier
/// row is needed before this leg can port for real.
///
/// Source: `oracle/codemp/renderer/tr_sky.cpp:687-772`
pub fn RB_DrawSun(frame: &FrameState, common: &Common, cvars: &RendererCvars, view: &viewParms_t) {
    if !frame.sky_rendered_this_view {
        return;
    }
    if common.cvar(cvars.r_drawSun).integer == 0 {
        return;
    }

    // DEFERRED: R4 — qglLoadMatrixf(backEnd.viewParms.world.modelMatrix);
    // qglTranslatef(backEnd.viewParms.ori.origin[0..2]) (DEC-37 A13.2,
    // unhomed GL/WGL entry points)
    // Source: oracle/codemp/renderer/tr_sky.cpp:699-700

    let dist = (view.zFar as f64 / 1.75_f64) as f32; // div sqrt(3)
    let size = (dist as f64 * 0.4_f64) as f32;

    let mut origin: vec3_t = [0.0; 3];
    VectorScale(frame.sun_direction, dist, &mut origin);
    let mut vec1: vec3_t = [0.0; 3];
    PerpendicularVector(&mut vec1, frame.sun_direction);
    let mut vec2: vec3_t = [0.0; 3];
    CrossProduct(frame.sun_direction, vec1, &mut vec2);

    VectorScale(vec1, size, &mut vec1);
    VectorScale(vec2, size, &mut vec2);

    // farthest depth range
    // DEFERRED: R4 — qglDepthRange(1.0, 1.0);
    // Source: oracle/codemp/renderer/tr_sky.cpp:713

    // FIXME: use quad stamp
    // DEFERRED: R4 — RB_BeginSurface(tr.sunShader, tess.fogNum) through
    // RB_EndSurface() (see doc comment above: dissolved `tess`, unhomed
    // `tr.sunShader`)
    // Source: oracle/codemp/renderer/tr_sky.cpp:716-768
    let _ = origin; // consumed only by the deferred tess-quad block above

    // back to normal depth range
    // DEFERRED: R4 — qglDepthRange(0.0, 1.0);
    // Source: oracle/codemp/renderer/tr_sky.cpp:771
}

/// Raven `RB_StageIteratorSky` — wave 10, final tail wave for this file.
///
/// Raven: go through all the polygons and project them onto the sky box to
/// see which blocks on each side need to be drawn; `r_showsky` will let all
/// the sky blocks be drawn in front of everything to allow developers to see
/// how much sky is getting sucked in; note that sky was drawn so we will
/// draw a sun later.
///
/// Whole-fn deferral, not a partial body: the function's very first
/// statement, `if ( g_bRenderGlowingObjects ) return;`, gates every single
/// statement that follows it — including the trailing unconditional
/// `backEnd.skyRenderedThisView = qtrue;` write. That gate is no longer the
/// blocker: campaign #41 batch 1 homed `g_bRenderGlowingObjects` and
/// `skyboxportal` on `FrameState` (`render_glowing_objects`/`skyboxportal`,
/// DEC-37 A13.3) and landed `RDF_SKYBOXPORTAL` in the crate's canonical flag
/// home (`tr_public::ref_flags`), so the two early-out guards
/// (`if (g_bRenderGlowingObjects) return;` and
/// `if (skyboxportal && !(backEnd.refdef.rdflags & RDF_SKYBOXPORTAL))
/// return;`) are a follow-up rewire once this fn takes `FrameState` and the
/// `refdef_rdflags: i32` parameter this crate threads elsewhere
/// (`tr_backend.rs`, `tr_light.rs`, `tr_world.rs`).
///
/// What still forces the whole-fn deferral is the body itself:
/// - `RB_ClipSkyPolygons( &tess )`, `tess.shader->sky->outerbox[...]`,
///   `DrawSkyBox( tess.shader )`, `R_BuildCloudData( &tess )`, and
///   `if (tess.numIndexes && tess.numVertexes) RB_StageIteratorGeneric()`
///   all key off the dissolved `tess`/`shaderCommands_t` global (R2
///   `## State ownership` row `tess`: "no single global scratch buffer
///   survives the new topology") — there is no R3 carrier from which to read
///   `tess.shader` (the `ShaderHandle` every one of those already-ported
///   in-module callees needs) or `tess.numIndexes`/`numVertexes`.
///
/// `r_fastsky`/`r_showsky` genuinely do have real carriers (`RendererCvars`,
/// DEC-37 A13.1 — see `RB_ClipSkyPolygons`'s sibling wave-9 fn
/// `RB_StageIteratorGeneric` for the established `common.cvar(cvars.r_x)
/// .integer` idiom) but sit behind the first unresolved gate above, so
/// reading them here would not shorten the deferral — the fn can still
/// return before ever reaching them.
///
/// Every `qgl*` call (`qglDepthRange`/`qglColor3f`/`qglPushMatrix`/
/// `qglPopMatrix`/`qglTranslatef`) is additionally unhomed GL/WGL surface
/// (DEC-01/DEC-37, `GpuResources::gl_state` a named placeholder until R4).
///
/// Loud `todo!()` per the whole-fn-deferral convention (partial-body fns
/// keep `DEFERRED:` comments instead of panicking) — same convention this
/// crate already applied at `tr_ghoul2.rs`'s `rb_surface_ghoul`
/// (`RB_SurfaceGhoul`).
///
/// Source: `oracle/codemp/renderer/tr_sky.cpp:786-848`
pub fn RB_StageIteratorSky(
    frame: &mut FrameState,
    common: &Common,
    cvars: &RendererCvars,
    gpu: &mut GpuResources,
    assets: &RenderAssets,
    sky: &mut SkyState,
    view: &viewParms_t,
) {
    let _ = (frame, common, cvars, gpu, assets, sky, view);
    todo!("Port RB_StageIteratorSky — oracle/codemp/renderer/tr_sky.cpp:786-848")
}
