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
use mp_engine_qcommon::qfiles::shader_limits::SHADER_MAX_VERTEXES;
use mp_qshared::shared::error_parm::errorParm_t;
use mp_qshared::shared::q_math::{
    _DotProduct as DotProduct, _VectorAdd as VectorAdd, _VectorScale as VectorScale,
    _VectorSubtract as VectorSubtract, vec3_origin, CrossProduct, PerpendicularVector,
};
use mp_qshared::shared::vec3_t;
use native_math::qmath::VectorNormalize;

use crate::render_state::frame_state::FrameState;
use crate::render_state::image_asset::ImageHandle;
use crate::render_state::placeholders::SkyParms;
use crate::render_state::renderer_cvars::RendererCvars;
use crate::tr_local::view_parms_t::viewParms_t;
use crate::tr_public::ref_flags::RDF_SKYBOXPORTAL;
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

/// One outer-box face's draw list — the render-side product of one iteration
/// of Raven `DrawSkyBox`'s per-face loop, in the DEC-50 shape: the frontend
/// returns geometry data, and the backend binds the image and emits the
/// triangle strips (Raven `DrawSkySide`'s GL body, now the backend's
/// `build_sky_face_block`).
///
/// `points`/`tex_coords` snapshot the whole `s_skyPoints`/`s_skyTexCoords`
/// grid at the moment the face was projected, so the backend indexes them by
/// the same `[t][s]` scheme `DrawSkySide` uses over `mins`/`maxs`. The
/// grid is shared scratch that the next face overwrites, so each face keeps
/// its own copy.
///
/// Source: `oracle/codemp/renderer/tr_sky.cpp:347-444`
pub struct SkyBoxFace {
    /// `shader->sky->outerbox[i]` — the face image the backend's `GL_Bind`
    /// binds. `None` for a face `ParseSkyParms` left unset.
    ///
    /// Source: `oracle/codemp/renderer/tr_sky.cpp:441`
    pub image: Option<ImageHandle>,
    /// `sky_mins_subd` — the face's lower subdivision bound, `[s, t]`.
    ///
    /// Source: `oracle/codemp/renderer/tr_sky.cpp:442`
    pub mins: [i32; 2],
    /// `sky_maxs_subd` — the face's upper subdivision bound, `[s, t]`.
    ///
    /// Source: `oracle/codemp/renderer/tr_sky.cpp:443`
    pub maxs: [i32; 2],
    /// `s_skyPoints` — the projected sky-box grid vertex positions, relative
    /// to the view origin. The backend applies the view-origin translate.
    ///
    /// Source: `oracle/codemp/renderer/tr_sky.cpp:344`
    pub points: Vec<Vec<vec3_t>>,
    /// `s_skyTexCoords` — the grid's texture coordinates.
    ///
    /// Source: `oracle/codemp/renderer/tr_sky.cpp:345`
    pub tex_coords: Vec<Vec<[f32; 2]>>,
}

/// The cloud layer's world-space geometry — the render-side product of
/// Raven `R_BuildCloudData`, which the oracle stamps into the global `tess`
/// and draws through `RB_StageIteratorGeneric`. Under DEC-50 the frontend
/// returns it as owned buffers and the backend feeds the same generic stage
/// machinery.
///
/// Source: `oracle/codemp/renderer/tr_sky.cpp:448-496,596-619`
#[derive(Default)]
pub struct SkyCloudData {
    /// `tess.xyz` — the cloud vertices in world space (`s_skyPoints` plus the
    /// view origin).
    ///
    /// Source: `oracle/codemp/renderer/tr_sky.cpp:461`
    pub xyz: Vec<vec3_t>,
    /// `tess.texCoords[..][0]` — the per-vertex cloud texture coordinates.
    ///
    /// Source: `oracle/codemp/renderer/tr_sky.cpp:462-463`
    pub tex_coords: Vec<[f32; 2]>,
    /// `tess.indexes` — the cloud triangle indexes into [`Self::xyz`].
    ///
    /// Source: `oracle/codemp/renderer/tr_sky.cpp:480-492`
    pub indexes: Vec<u32>,
}

/// The full render-side product of Raven `RB_StageIteratorSky` — the six
/// outer-box faces plus the cloud layer, for the backend to draw. Sky is
/// drawn inline in the sorted draw-surf loop, one call per sky-shader surface
/// batch (DEC-50).
///
/// Source: `oracle/codemp/renderer/tr_sky.cpp:786-848`
pub struct SkyBoxDrawData {
    /// The six outer-box faces (`rt`/`lf`/`bk`/`ft`/`up`/`dn`). A face is
    /// `None` when it projects to nothing or when the shader draws no outer
    /// box.
    pub faces: [Option<SkyBoxFace>; 6],
    /// The cloud layer geometry.
    pub cloud: SkyCloudData,
}

/// Raven `SKY_SUBDIVISIONS`.
///
/// Source: `oracle/codemp/renderer/tr_sky.cpp:7`
const SKY_SUBDIVISIONS: i32 = 8;

/// Raven `HALF_SKY_SUBDIVISIONS` (`SKY_SUBDIVISIONS/2`). The GPU backend
/// imports this to index the sky-box grid, so the loop bounds have one home.
///
/// Source: `oracle/codemp/renderer/tr_sky.cpp:8`
pub const HALF_SKY_SUBDIVISIONS: i32 = SKY_SUBDIVISIONS / 2;

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

/// Raven `sky_clip[6]` — the six clip-plane normals `ClipSkyPolygon` clips
/// the box polygon against. The port carries the live values in
/// `SkyState::sky_clip` because the earlier wave's oracle slice omitted this
/// definition. This wave has the definition, so `RB_ClipSkyPolygons` copies
/// it in before the clip pass. The oracle keeps them as an immutable
/// file-scope const, so the copy never changes them.
///
/// Source: `oracle/codemp/renderer/tr_sky.cpp:23-31`
const SKY_CLIP: [vec3_t; 6] = [
    [1.0, 1.0, 0.0],
    [1.0, -1.0, 0.0],
    [0.0, -1.0, 1.0],
    [0.0, 1.0, 1.0],
    [1.0, 0.0, 1.0],
    [-1.0, 0.0, 1.0],
];

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
/// The oracle stamps the cloud grid into the global `tess`; this port appends
/// into an owned `SkyCloudData` accumulator (DEC-50). `tess.numVertexes` at
/// entry becomes `cloud.xyz.len()` (`vertex_start`), so the index offsets stay
/// correct as later faces append. `view_origin` is `backEnd.viewParms.ori
/// .origin`. The `SHADER_MAX_VERTEXES` `Com_Error` bound check is kept against
/// the accumulator length to reproduce the oracle overflow behavior.
///
/// Source: `oracle/codemp/renderer/tr_sky.cpp:448-496`
pub fn FillCloudySkySide(
    mins: [i32; 2],
    maxs: [i32; 2],
    add_indexes: bool,
    view_origin: vec3_t,
    sky: &SkyState,
    cloud: &mut SkyCloudData,
) {
    let vertex_start = cloud.xyz.len() as i32;
    let t_height = maxs[1] - mins[1] + 1;
    let s_width = maxs[0] - mins[0] + 1;

    for t in (mins[1] + HALF_SKY_SUBDIVISIONS)..=(maxs[1] + HALF_SKY_SUBDIVISIONS) {
        for s in (mins[0] + HALF_SKY_SUBDIVISIONS)..=(maxs[0] + HALF_SKY_SUBDIVISIONS) {
            let mut xyz: vec3_t = [0.0; 3];
            VectorAdd(sky.sky_points[t as usize][s as usize], view_origin, &mut xyz);
            cloud.xyz.push(xyz);
            cloud
                .tex_coords
                .push(sky.sky_tex_coords[t as usize][s as usize]);

            if cloud.xyz.len() >= SHADER_MAX_VERTEXES as usize {
                com_error(
                    errorParm_t::ERR_DROP,
                    "SHADER_MAX_VERTEXES hit in FillCloudySkySide()\n".to_string(),
                );
            }
        }
    }

    // only add indexes for one pass, otherwise it would draw multiple times
    // for each pass
    if add_indexes {
        for t in 0..(t_height - 1) {
            for s in 0..(s_width - 1) {
                cloud.indexes.push((vertex_start + s + t * s_width) as u32);
                cloud
                    .indexes
                    .push((vertex_start + s + (t + 1) * s_width) as u32);
                cloud
                    .indexes
                    .push((vertex_start + s + 1 + t * s_width) as u32);

                cloud
                    .indexes
                    .push((vertex_start + s + (t + 1) * s_width) as u32);
                cloud
                    .indexes
                    .push((vertex_start + s + 1 + (t + 1) * s_width) as u32);
                cloud
                    .indexes
                    .push((vertex_start + s + 1 + t * s_width) as u32);
            }
        }
    }
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

/// Raven `FillCloudBox`.
///
/// The oracle's `const shader_t *shader` argument drops: its only read is the
/// `shader->sky->fullClouds` branch, which is hard-coded to `if ( 1 )` with a
/// `// FIXME?` left in place, so the shader is never actually read
/// (porting-rules §A2: port the behavior as written, not a "cleaner" reading
/// of the dead branch). `view` threads `backEnd.viewParms` into `MakeSkyVec`
/// (same top-of-file PORT-NOTE precedent as wave 0) and supplies the view
/// origin for `FillCloudySkySide`. `sky` carries `sky_mins`/`sky_maxs`/
/// `sky_points`/`sky_tex_coords`/`cloud_tex_coords` (`SkyState`, DEC-37
/// A13.3). `cloud` accumulates the cloud geometry (DEC-50). The `floor`/`ceil`
/// calls are the C `<math.h>` double overloads (no `floorf`/`ceilf`), so their
/// operand promotes to `f64` and the result rounds back to `f32` once at the
/// assignment (wave-0 ruling 12).
///
/// Source: `oracle/codemp/renderer/tr_sky.cpp:498-591`
pub fn FillCloudBox(
    stage: i32,
    view: &viewParms_t,
    sky: &mut SkyState,
    cloud: &mut SkyCloudData,
) {
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
        FillCloudySkySide(
            sky_mins_subd,
            sky_maxs_subd,
            stage == 0,
            view.ori.origin,
            sky,
            cloud,
        );
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
/// The oracle reads the surface triangles from the global `tess`
/// (`input->numIndexes`/`->xyz`/`->indexes`); DEC-50 hands them in as
/// `verts`/`indexes` (the sky-shader surface batch, world space). Each
/// triangle vertex is offset by the view origin (`view.ori.origin`) into the
/// clip-space direction vector, then clipped by `ClipSkyPolygon`. The port
/// copies the six clip-plane normals into `sky.sky_clip` first (see
/// [`SKY_CLIP`]).
///
/// Raven's `vec3_t p[5]` scratch keeps one spare for clipping but fills only
/// `p[0..3]` and passes `nump = 3`; the port uses a plain `[vec3_t; 3]`.
///
/// Source: `oracle/codemp/renderer/tr_sky.cpp:244-261`
pub fn RB_ClipSkyPolygons(
    verts: &[vec3_t],
    indexes: &[u32],
    view: &viewParms_t,
    sky: &mut SkyState,
) {
    sky.sky_clip = SKY_CLIP;
    ClearSkyBox(sky);

    let mut i = 0usize;
    while i < indexes.len() {
        let mut p: [vec3_t; 3] = [[0.0; 3]; 3];
        for j in 0..3usize {
            VectorSubtract(
                verts[indexes[i + j] as usize],
                view.ori.origin,
                &mut p[j],
            );
        }
        ClipSkyPolygon(&p, 0, sky);
        i += 3;
    }
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
/// (write)/`sky_points`/`sky_tex_coords` (`SkyState`, DEC-37 A13.3). `view`
/// threads `backEnd.viewParms` into `MakeSkyVec` (top-of-file PORT-NOTE
/// precedent).
///
/// The oracle draws each face inline (`DrawSkySide`); DEC-50 returns the six
/// faces as data instead. `sky_parms.outerbox[i]` is the face image the
/// backend binds. After a face fills the shared grid, this function snapshots
/// the grid into the `SkyBoxFace`, because the next face overwrites it. A face
/// that projects to nothing (the `sky_mins >= sky_maxs` skip) stays `None`.
///
/// Source: `oracle/codemp/renderer/tr_sky.cpp:378-446`
pub fn DrawSkyBox(
    sky_parms: &SkyParms,
    view: &viewParms_t,
    sky: &mut SkyState,
) -> [Option<SkyBoxFace>; 6] {
    let mut faces: [Option<SkyBoxFace>; 6] = core::array::from_fn(|_| None);

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

        // The oracle calls DrawSkySide( shader->sky->outerbox[i], ... ) here.
        // DEC-50 snapshots the grid into the face draw data instead.
        faces[i] = Some(SkyBoxFace {
            image: sky_parms.outerbox[i],
            mins: sky_mins_subd,
            maxs: sky_maxs_subd,
            points: sky.sky_points.clone(),
            tex_coords: sky.sky_tex_coords.clone(),
        });
    }

    faces
}

/// Raven `R_BuildCloudData`.
///
/// The oracle reads `input->shader`; DEC-50 hands in the shader's `sky_parms`
/// and `num_unfogged_passes` directly. `assert( shader->sky )` drops: the
/// backend only calls this for a sky shader, so `sky_parms` is always present.
///
/// `sky_min`/`sky_max`'s RHS mixes an unsuffixed (`double`) literal with an
/// `f`-suffixed one (`1.0 / 256.0f`), so the divide promotes to `f64` and
/// rounds once at the assignment (wave-0 ruling 12) — Raven's own
/// `// FIXME: not correct?` comment is kept verbatim. `tess.numIndexes = 0;
/// tess.numVertexes = 0;` reset the cloud accumulator (`cloud`) instead of the
/// dissolved `tess` (DEC-50).
///
/// `sky_parms.cloud_height` gates the `FillCloudBox` loop, and
/// `num_unfogged_passes` (`shader->numUnfoggedPasses`) bounds it.
///
/// Source: `oracle/codemp/renderer/tr_sky.cpp:596-619`
pub fn R_BuildCloudData(
    sky_parms: &SkyParms,
    num_unfogged_passes: i32,
    view: &viewParms_t,
    sky: &mut SkyState,
    cloud: &mut SkyCloudData,
) {
    // FIXME: not correct?
    sky.sky_min = (1.0_f64 / 256.0_f64) as f32;
    sky.sky_max = (255.0_f64 / 256.0_f64) as f32;

    // set up for drawing
    cloud.xyz.clear();
    cloud.tex_coords.clear();
    cloud.indexes.clear();

    if sky_parms.cloud_height != 0.0 {
        for i in 0..num_unfogged_passes {
            FillCloudBox(i, view, sky, cloud);
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
/// (the whole quad-stamp body building the sun's four tess vertices) draws
/// nothing here, to match the oracle. The oracle's only `RB_DrawSun` call
/// site sits inside an `#if 0` block (`tr_backend.cpp:1222-1224`), so retail
/// Jedi Academy never draws the sun. Parity is no live draw. The carriers
/// both exist now (`RenderAssets::sun_shader`, `FrameState::sun_direction`),
/// so the quad body can port whenever the `#if 0` guard is lifted.
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
    // RB_EndSurface(). The oracle's only RB_DrawSun call site is inside an
    // `#if 0` block (tr_backend.cpp:1222-1224), so parity is no live draw.
    // Source: oracle/codemp/renderer/tr_sky.cpp:716-768
    let _ = origin; // consumed only by the deferred tess-quad block above

    // back to normal depth range
    // DEFERRED: R4 — qglDepthRange(0.0, 1.0);
    // Source: oracle/codemp/renderer/tr_sky.cpp:771
}

/// Raven `RB_StageIteratorSky`.
///
/// Raven: go through all the polygons and project them onto the sky box to
/// see which blocks on each side need to be drawn; note that sky was drawn so
/// we will draw a sun later.
///
/// DEC-50: the backend calls this once per sky-shader surface batch with the
/// batch's world-space triangles (`verts`/`indexes`), and this returns the
/// six outer-box faces plus the cloud geometry for the backend to draw. The
/// oracle reads all of it from the global `tess`; the port hands the pieces
/// in and returns owned data.
///
/// The two early-out guards port: `g_bRenderGlowingObjects` reads
/// `frame.render_glowing_objects`, and the portal guard reads
/// `frame.skyboxportal`/`frame.refdef.rdflags` against `RDF_SKYBOXPORTAL`
/// (campaign #41 batch 1 homed these). A guard that trips returns `None` and
/// leaves `sky_rendered_this_view` unset, matching the oracle's early return
/// before the trailing write.
///
/// The oracle's `r_fastsky` guard, the `r_showsky`/`qglDepthRange` depth-range
/// trick, and the actual draw calls are GPU/executor concerns the backend
/// owns (same as `DrawSkySide`'s deferred GL body). This function returns
/// geometry data only, no GPU state.
///
/// `sky_parms` is `tess.shader->sky`, and `num_unfogged_passes` is
/// `tess.shader->numUnfoggedPasses` (read by `R_BuildCloudData`).
/// `default_image` is `tr.defaultImage`, the value the outer-box gate compares
/// `outerbox[0]` against. `view` threads `backEnd.viewParms` (the view origin
/// and `zFar`, top-of-file PORT-NOTE precedent).
///
/// Source: `oracle/codemp/renderer/tr_sky.cpp:786-848`
pub fn RB_StageIteratorSky(
    frame: &mut FrameState,
    sky: &mut SkyState,
    sky_parms: &SkyParms,
    num_unfogged_passes: i32,
    default_image: Option<ImageHandle>,
    verts: &[vec3_t],
    indexes: &[u32],
    view: &viewParms_t,
) -> Option<SkyBoxDrawData> {
    if frame.render_glowing_objects {
        return None;
    }

    // The `r_fastsky` early-out is an executor concern (the backend owns
    // whether the sky draws at all, same as the `r_showsky` depth-range trick
    // below). The executor gate lives in `Pipeline3d::collect_sky_surface`
    // (`mp_renderer_gpu`), which returns before this fn runs, so
    // `skyRenderedThisView` stays unwritten under `r_fastsky`.
    // Source: oracle/codemp/renderer/tr_sky.cpp:791-793

    if frame.skyboxportal != 0 && (frame.refdef.rdflags & RDF_SKYBOXPORTAL) == 0 {
        return None;
    }

    // The oracle's `s_skyPoints`/`s_skyTexCoords` are fixed static arrays; the
    // port holds them as jagged `Vec`s, so size the grid scratch before
    // `DrawSkyBox`/`FillCloudBox` index it (a fresh zeroed grid each pass
    // matches the static array, which `DrawSkyBox` also `Com_Memset`s).
    let dim = (SKY_SUBDIVISIONS + 1) as usize;
    sky.sky_points = vec![vec![[0.0f32; 3]; dim]; dim];
    sky.sky_tex_coords = vec![vec![[0.0f32; 2]; dim]; dim];

    // go through all the polygons and project them onto the sky box to see
    // which blocks on each side need to be drawn
    RB_ClipSkyPolygons(verts, indexes, view, sky);

    // DEFERRED: R4 — the `r_showsky`/`qglDepthRange` depth-range trick is GPU
    // state the backend owns.
    // Source: oracle/codemp/renderer/tr_sky.cpp:808-816

    // draw the outer skybox, only when outerbox[0] is a real image (not the
    // default image and not the "-" no-outer-box shader)
    let draw_outerbox =
        sky_parms.outerbox[0].is_some() && sky_parms.outerbox[0] != default_image;
    let faces = if draw_outerbox {
        DrawSkyBox(sky_parms, view, sky)
    } else {
        core::array::from_fn(|_| None)
    };

    // generate the vertexes for all the clouds, which the backend draws
    // through its generic stage machinery (RB_StageIteratorGeneric)
    let mut cloud = SkyCloudData::default();
    R_BuildCloudData(sky_parms, num_unfogged_passes, view, sky, &mut cloud);

    // note that sky was drawn so we will draw a sun later
    frame.sky_rendered_this_view = true;

    Some(SkyBoxDrawData { faces, cloud })
}
