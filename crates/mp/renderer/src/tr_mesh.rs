//! Raven `tr_mesh.cpp` logic (R3 frontend port).
//!
//! Source: `oracle/codemp/renderer/tr_mesh.cpp`

use core::ffi::c_char;

use mp_engine_qcommon::qfiles::md3_frame_s::md3Frame_t;
use mp_engine_qcommon::qfiles::md3_header_t::md3Header_t;
use mp_engine_qcommon::qfiles::md3_shader_t::md3Shader_t;
use mp_engine_qcommon::qfiles::md3_surface_t::md3Surface_t;
use mp_qshared::common::mp::cgame::tr_types::{RF_THIRD_PERSON, RF_WRAP_FRAMES};
use mp_qshared::shared::q_color::S_COLOR_RED;
use mp_qshared::shared::q_math::_DotProduct as DotProduct;
use mp_qshared::shared::{cplane_t, vec3_t};

use native_math::qmath::RadiusFromBounds;

use crate::render_state::frame_state::FrameState;
use crate::render_state::model_blocks::PublishedModel;
use crate::render_state::placeholders::RefEntity;
use crate::render_state::render_assets::RenderAssets;
use crate::render_state::render_cvar_snapshot::RenderCvarSnapshot;
use crate::render_state::shader_asset::ShaderHandle;
use crate::render_state::skin_asset::SkinAsset;
use crate::render_state::walk_warnings::WalkWarnings;
use crate::render_state::world_load_state::WorldLoadState;
use crate::tr_image::R_GetSkinByHandle;
use crate::tr_light::R_SetupEntityLighting;
use crate::tr_local::dlight_s::dlight_t;
use crate::tr_local::fog_t::fog_t;
use crate::tr_local::orientationr_t::orientationr_t;
use crate::tr_local::tr_ref_entity_t::trRefEntity_t;
use crate::tr_local::view_parms_t::viewParms_t;
use crate::tr_main::{
    ref_entity_from_tr, write_back_lighting, DrawSurf, Md3SurfaceRef, R_AddDrawSurf,
    R_CullLocalBox, R_CullLocalPointAndRadius, SurfaceGeometry, CULL_CLIP, CULL_IN, CULL_OUT,
};
use crate::tr_model::render_models::RenderModels;
use crate::tr_public::ref_flags::RDF_NOWORLDMODEL;
use crate::tr_shade_calc::myftol;
use crate::tr_shader::R_GetShaderByHandleQuiet;

/// Reads one on-disk `md3Frame_t` off `header` by frame index, returning the
/// bounds, local origin, and radius the cull and LOD math read. The oracle
/// walks the same `((byte *)header + ofsFrames) + index` frame array by raw
/// pointer.
///
/// SAFETY: `header` is the 16-byte-aligned `AlignedBytes` base a registered
/// `MOD_MESH` model owns, and `index` is range-checked by `R_AddMD3Surfaces`
/// against `numFrames` before any decode. The read stays inside the block the
/// file's `ofsEnd` sizes.
///
/// Source: `oracle/codemp/renderer/tr_mesh.cpp:64-65`
unsafe fn read_md3_frame(header: *const md3Header_t, index: i32) -> ([vec3_t; 2], vec3_t, f32) {
    let base = header as *const u8;
    let frames = base.add((*header).ofsFrames as usize) as *const md3Frame_t;
    let frame = &*frames.add(index as usize);
    (frame.bounds, frame.localOrigin, frame.radius)
}

/// Reads a NUL-terminated `md3` name field into an owned `String`. The bytes
/// are lower-cased Latin-1 at load, so a byte-to-char map decodes them.
fn md3_name(name: &[c_char]) -> String {
    let end = name.iter().position(|&c| c == 0).unwrap_or(name.len());
    name[..end].iter().map(|&c| (c as u8) as char).collect()
}

/// Wraps and checks an MD3 entity's frame indices. `RF_WRAP_FRAMES` mods both
/// by `num_frames` with the oracle's truncating `%`, so a negative frame keeps
/// its sign. The returned values are post-wrap and not clamped. The `bad` flag
/// is true when a frame is still outside `0..num_frames`. The caller then
/// prints the post-wrap values and resets both to 0.
///
/// Source: `oracle/codemp/renderer/tr_mesh.cpp:294-315`
fn wrap_and_check_md3_frames(
    frame: i32,
    oldframe: i32,
    num_frames: i32,
    wrap: bool,
) -> (i32, i32, bool) {
    let mut frame = frame;
    let mut oldframe = oldframe;
    if wrap {
        // The oracle wraps with truncating `%`, so `ent->e.frame %= numFrames`
        // leaves a negative frame negative for the range check below to catch.
        frame %= num_frames;
        oldframe %= num_frames;
    }

    let bad = frame >= num_frames || frame < 0 || oldframe >= num_frames || oldframe < 0;
    (frame, oldframe, bad)
}

/// Matches an MD3 surface name against a skin's per-surface list, returning the
/// skin surface's shader on a match or the default shader (slot 0) on none. The
/// names are lower-cased at load, so the compare is exact.
///
/// Source: `oracle/codemp/renderer/tr_mesh.cpp:365-372`
fn resolve_skin_shader(skin: &SkinAsset, surface_name: &str) -> ShaderHandle {
    for skin_surface in &skin.surfaces {
        if skin_surface.name == surface_name {
            return skin_surface.shader;
        }
    }
    ShaderHandle::slot_zero()
}

/// Raven `float ProjectRadius(float r, vec3_t location)` — projects a
/// world-space radius `r` at `location` into a screen-space fraction, for
/// LOD/culling decisions.
///
/// `view` is `tr.viewParms` (`.ori.axis[0]`/`.ori.origin`/`.projectionMatrix`).
/// The `_XBOX` sign flip is dropped - MP never builds `_XBOX`.
///
/// Source: `oracle/codemp/renderer/tr_mesh.cpp:12-50`
pub fn project_radius(r: f32, location: vec3_t, view: &viewParms_t) -> f32 {
    let c = DotProduct(view.ori.axis[0], view.ori.origin);
    let dist = DotProduct(view.ori.axis[0], location) - c;

    if dist <= 0.0 {
        return 0.0;
    }

    let p = [0.0f32, r.abs(), -dist];
    let m = &view.projectionMatrix;

    let width = p[0] * m[1] + p[1] * m[5] + p[2] * m[9] + m[13];
    let depth = p[0] * m[3] + p[1] * m[7] + p[2] * m[11] + m[15];

    let mut pr = width / depth;
    if pr > 1.0 {
        pr = 1.0;
    }
    pr
}

/// Raven `int R_ComputeFogNum(md3Header_t *header, trRefEntity_t *ent)` —
/// the fog volume `ent`'s MD3 frame bounds fall inside, if any.
///
/// `fogs` is `tr.world->fogs` (index 0 is the reserved "no fog" slot, matching
/// the oracle's `for (i=1; i<numfogs; i++)`); `refdef_rdflags` is
/// `tr.refdef.rdflags`. The `header`/frame walk reads the on-disk frame array
/// through [`read_md3_frame`].
///
/// Source: `oracle/codemp/renderer/tr_mesh.cpp:244-273`
fn r_compute_fog_num(
    header: *const md3Header_t,
    ent_frame: i32,
    ent_origin: vec3_t,
    fogs: &[fog_t],
    refdef_rdflags: i32,
) -> i32 {
    if refdef_rdflags & RDF_NOWORLDMODEL != 0 {
        return 0;
    }

    // SAFETY: `ent_frame` is clamped to `numFrames` by the caller before this
    // fog lookup; see [`read_md3_frame`].
    let (_, local_origin, radius) = unsafe { read_md3_frame(header, ent_frame) };
    let world_origin = [
        ent_origin[0] + local_origin[0],
        ent_origin[1] + local_origin[1],
        ent_origin[2] + local_origin[2],
    ];

    for i in 1..fogs.len() {
        let fog = &fogs[i];
        let mut j = 0usize;
        while j < 3 {
            if world_origin[j] - radius >= fog.bounds[1][j] {
                break;
            }
            if world_origin[j] + radius <= fog.bounds[0][j] {
                break;
            }
            j += 1;
        }
        if j == 3 {
            return i as i32;
        }
    }

    0
}

/// Raven `void RE_GetModelBounds(refEntity_t *refEnt, vec3_t bounds1, vec3_t
/// bounds2)` — the MD3 model's per-frame bounding box for `refEnt->hModel`
/// at `refEnt->frame`. Out-params fold into a returned pair (dictionary:
/// out-params→returns).
// DEFERRED: RE_GetModelBounds — no caller in this crate reaches it yet
// (rwwRMG-added, driven by the RMG terrain path, not the world/entity walk).
// The frame-array read now has a home ([`read_md3_frame`]); this fn stays
// deferred until its RMG caller lands, to avoid a dead public entry point.
// Source: oracle/codemp/renderer/tr_mesh.cpp:148-165
pub fn re_get_model_bounds(_ref_ent: &RefEntity, _models: &RenderModels) -> (vec3_t, vec3_t) {
    todo!("Port RE_GetModelBounds — oracle/codemp/renderer/tr_mesh.cpp:148-165")
}

/// Raven `int R_ComputeLOD(trRefEntity_t *ent)` — picks `ent`'s MD3 LOD level
/// from its projected screen-space bounding-sphere radius, biased by
/// `r_lodscale`/`r_autolodscalevalue`/`r_lodbias` and clamped to
/// `tr.currentModel->numLods`.
///
/// `current_model` is `tr.currentModel`, read as the published entry (DEC-65 ruling 3).
/// `view` is `tr.viewParms` (`ProjectRadius`), and the three lod cvars arrive on the frame's [`RenderCvarSnapshot`] (W2-F1).
/// The frame-array read for the projected radius runs through [`read_md3_frame`].
///
/// Source: `oracle/codemp/renderer/tr_mesh.cpp:173-236`
fn r_compute_lod(
    current_model: &PublishedModel,
    ent_frame: i32,
    ent_origin: vec3_t,
    view: &viewParms_t,
    cvars: RenderCvarSnapshot,
) -> i32 {
    // A registered multi-LOD model always publishes LOD 0, so an absent header here is unreachable and takes the single-LOD arm.
    let mut lod = match current_model.md3_ptr(0) {
        Some(header) if current_model.num_lods >= 2 => {
            // multiple LODs exist, so compute projected bounding sphere and use
            // that as a criteria for selecting LOD.
            // SAFETY: `ent_frame` is clamped to `numFrames` by the caller before the LOD read. See [`read_md3_frame`].
            let (bounds, _, _) = unsafe { read_md3_frame(header, ent_frame) };
            let radius = RadiusFromBounds(bounds[0], bounds[1]);

            let projected_radius = project_radius(radius, ent_origin, view);
            let flod;
            if projected_radius != 0.0 {
                let mut lodscale = cvars.lodscale + cvars.autolodscalevalue;
                if lodscale > 20.0 {
                    lodscale = 20.0;
                } else if lodscale < 0.0 {
                    lodscale = 0.0;
                }
                flod = 1.0f32 - projected_radius * lodscale;
            } else {
                // object intersects near view plane, e.g. view weapon
                flod = 0.0;
            }

            let flod = flod * current_model.num_lods as f32;
            let mut lod = myftol(flod);

            if lod < 0 {
                lod = 0;
            } else if lod >= current_model.num_lods {
                lod = current_model.num_lods - 1;
            }
            lod
        }
        // model has only 1 LOD level, skip computations and bias
        _ => 0,
    };

    lod += cvars.lodbias;

    if lod >= current_model.num_lods {
        lod = current_model.num_lods - 1;
    }
    if lod < 0 {
        lod = 0;
    }

    lod
}

/// Raven `static int R_CullModel(md3Header_t *header, trRefEntity_t *ent)` —
/// culls an MD3 model against the view frustum: first a bounding-sphere test
/// against the current (and, when animating, previous) frame, skipped for
/// non-normalized-axis (upscaled) entities, then a merged bounding-box test.
///
/// `ori` is the entity orientation `R_RotateForEntity` built.
/// `r_nocull_integer` is `r_nocull->integer`.
/// `frustum` is `tr.viewParms.frustum`.
///
// PORT-NOTE: the `tr.pc.c_sphere_cull_md3_*`/`c_box_cull_md3_*`
// (`frontEndCounters_t`) increments are dropped - `tr.pc` has no `FrameState`
// home yet, the same UNMAPPED finding `tr_cmds.rs`'s `R_PerformanceCounters`
// deferral records.
/// Source: `oracle/codemp/renderer/tr_mesh.cpp:58-137`
fn r_cull_model(
    header: *const md3Header_t,
    ent: &trRefEntity_t,
    ori: &orientationr_t,
    r_nocull_integer: i32,
    frustum: &[cplane_t; 4],
) -> i32 {
    // SAFETY: both frame indices are clamped to `numFrames` by the caller
    // before this cull; see [`read_md3_frame`].
    let (new_bounds, new_local, new_radius) = unsafe { read_md3_frame(header, ent.e.frame) };
    let (old_bounds, old_local, old_radius) = unsafe { read_md3_frame(header, ent.e.oldframe) };

    // cull bounding sphere ONLY if this is not an upscaled entity
    if ent.e.nonNormalizedAxes == 0 {
        if ent.e.frame == ent.e.oldframe {
            match R_CullLocalPointAndRadius(new_local, new_radius, ori, r_nocull_integer, frustum) {
                CULL_OUT => return CULL_OUT,
                CULL_IN => return CULL_IN,
                _ => {}
            }
        } else {
            let sphere_cull =
                R_CullLocalPointAndRadius(new_local, new_radius, ori, r_nocull_integer, frustum);
            let sphere_cull_b =
                R_CullLocalPointAndRadius(old_local, old_radius, ori, r_nocull_integer, frustum);

            if sphere_cull == sphere_cull_b {
                if sphere_cull == CULL_OUT {
                    return CULL_OUT;
                } else if sphere_cull == CULL_IN {
                    return CULL_IN;
                }
            }
        }
    }

    // calculate a bounding box in the current coordinate system
    let mut bounds = [[0.0f32; 3]; 2];
    for i in 0..3 {
        bounds[0][i] = old_bounds[0][i].min(new_bounds[0][i]);
        bounds[1][i] = old_bounds[1][i].max(new_bounds[1][i]);
    }

    match R_CullLocalBox(bounds, r_nocull_integer, ori, frustum) {
        CULL_IN => CULL_IN,
        CULL_CLIP => CULL_CLIP,
        _ => CULL_OUT,
    }
}

/// Raven `void R_AddMD3Surfaces(trRefEntity_t *ent)` — validates `ent`'s
/// current/old MD3 frame indices, computes LOD, culls the merged bounding
/// box, sets up lighting, resolves the fog volume, then walks every MD3
/// surface resolving its shader and pushing a draw surf.
///
/// `assets.models` resolves `tr.currentModel` (`R_GetModelByHandle`) as the published entry (DEC-65 ruling 3).
/// `view` is `tr.viewParms` (`.isPortal`/`.frustum`).
/// `ori` is the entity orientation `R_RotateForEntity` built.
/// The cvars arrive on the frame's [`RenderCvarSnapshot`] (W2-F1), and the
/// three `Com_DPrintf` diagnostics print once each through `eprintln!`, since
/// the render thread holds no `Common`.
/// `assets` holds the shader and skin registries and the world fog list.
/// `shifted_entity_num`/`rdf_nofog` feed `R_AddDrawSurf`'s sort key
/// (the shader travels through the sort key, not the surface ref, DEC-43.3).
///
/// PORT-NOTE: `ent->e.frame`/`oldframe` are written in place by the wrap and
/// clamp below, so a later per-frame read of `tr.refdef.entities[n]` sees the
/// validated values, matching the oracle's direct write into the entity.
///
/// Source: `oracle/codemp/renderer/tr_mesh.cpp:281-420`
#[allow(clippy::too_many_arguments)]
pub fn r_add_md3_surfaces<'a>(
    ent: &mut trRefEntity_t,
    view: &viewParms_t,
    ori: &orientationr_t,
    cvars: RenderCvarSnapshot,
    warnings: &mut WalkWarnings,
    assets: &RenderAssets,
    world_load: &WorldLoadState,
    frame: &FrameState,
    refdef_rdflags: i32,
    rdf_nofog: bool,
    shifted_entity_num: i32,
    fogs: &[fog_t],
    dlights: &[dlight_t],
    draw_surfs: &mut Vec<DrawSurf<SurfaceGeometry<'a>>>,
) {
    // don't add third_person objects if not in a portal
    let personal_model = (ent.e.renderfx & RF_THIRD_PERSON) != 0 && view.isPortal == 0;

    // The dispatch resolved this handle's `model_type` out of the same published registry this frame, so a `MOD_MESH` entry always exists here.
    // Both skips are the defined behavior where a stale reference could reach this arm.
    let Some(current_model) = assets.models.get(ent.e.hModel) else {
        return;
    };
    let Some(lod0_header) = current_model.md3_ptr(0) else {
        return;
    };
    let num_frames = {
        // SAFETY: `md3_ptr(0)` is the LOD-0 header of a registered `MOD_MESH` model, the aligned block the loader owns.
        unsafe { (*lod0_header).numFrames }
    };

    // Wrap and validate the frames so there is no chance of a crash. The wrap
    // writes into the entity first, so the surfaces render without a re-check.
    // The warning then prints the post-wrap values, and only after that does a
    // bad frame reset both to 0. This matches the oracle write order.
    let wrap = ent.e.renderfx & RF_WRAP_FRAMES != 0;
    let (wrapped_frame, wrapped_oldframe, bad_frame) =
        wrap_and_check_md3_frames(ent.e.frame, ent.e.oldframe, num_frames, wrap);
    ent.e.frame = wrapped_frame;
    ent.e.oldframe = wrapped_oldframe;
    if bad_frame {
        if !warnings.md3_bad_frame {
            warnings.md3_bad_frame = true;
            eprintln!(
                "{}R_AddMD3Surfaces: no such frame {} to {} for '{}'",
                S_COLOR_RED.to_str().expect("S_COLOR_RED is ASCII"),
                ent.e.oldframe,
                ent.e.frame,
                current_model.name,
            );
        }
        ent.e.frame = 0;
        ent.e.oldframe = 0;
    }

    // compute LOD
    let lod = r_compute_lod(current_model, ent.e.frame, ent.e.origin, view, cvars);

    // The LOD the computation picked is always loaded, because `r_compute_lod` clamps to `num_lods`.
    let Some(header) = current_model.md3_ptr(lod as usize) else {
        return;
    };

    // cull the whole model if the merged bounding box of both frames is
    // outside the view frustum
    let cull = r_cull_model(header, ent, ori, cvars.nocull, &view.frustum);
    if cull == CULL_OUT {
        return;
    }

    // set up lighting now that we know we aren't culled
    // PORT-NOTE: the MOD_BRUSH arm at `tr_main.rs` also folds the lighting onto
    // `frame.current_entity`, but `frame` arrives here shared, so this arm cannot.
    // No MD3 reader of `frame.current_entity` exists yet, so the fold is latent.
    // Source: oracle/codemp/renderer/tr_mesh.cpp:335-340
    if !personal_model || cvars.shadows > 1 {
        let mut re = ref_entity_from_tr(ent);
        R_SetupEntityLighting(cvars, assets, world_load, frame, refdef_rdflags, dlights, &mut re);
        write_back_lighting(ent, &re);
    }

    // see if we are in a fog volume
    let fog_num = r_compute_fog_num(header, ent.e.frame, ent.e.origin, fogs, refdef_rdflags);

    // draw all surfaces
    // SAFETY: every walk below stays inside the aligned block `ofsEnd` sizes.
    // Each surface advances by its own `ofsEnd`, matching the loader's walk.
    let num_surfaces = unsafe { (*header).numSurfaces };
    let mut surf =
        unsafe { (header as *const u8).add((*header).ofsSurfaces as usize) } as *const md3Surface_t;

    for i in 0..num_surfaces {
        let num_shaders = unsafe { (*surf).numShaders };
        let surface_name = md3_name(unsafe { &(*surf).name });

        let shader: ShaderHandle = if ent.e.customShader != 0 {
            R_GetShaderByHandleQuiet(assets, ent.e.customShader, warnings)
        } else if ent.e.customSkin > 0
            && assets
                .skins
                .handle_at_slot(ent.e.customSkin as u32)
                .is_some()
        {
            let skin_handle = R_GetSkinByHandle(assets, ent.e.customSkin);

            // match the surface name to something in the skin file. The names
            // have both been lowercased
            let mut resolved = ShaderHandle::slot_zero();
            let mut skin_name = String::new();
            if let Some(skin) = assets.skins.get(skin_handle) {
                skin_name = skin.name.clone();
                resolved = resolve_skin_shader(skin, &surface_name);
            }

            if resolved == ShaderHandle::slot_zero() {
                if !warnings.md3_skin_surface {
                    warnings.md3_skin_surface = true;
                    eprintln!(
                        "{}WARNING: no shader for surface {} in skin {}",
                        S_COLOR_RED.to_str().expect("S_COLOR_RED is ASCII"),
                        surface_name,
                        skin_name,
                    );
                }
            } else if assets
                .shaders
                .get(resolved)
                .map(|s| s.default_shader)
                .unwrap_or(false)
            {
                let shader_name = assets
                    .shaders
                    .get(resolved)
                    .map(|s| s.name.clone())
                    .unwrap_or_default();
                if !warnings.md3_skin_shader {
                    warnings.md3_skin_shader = true;
                    eprintln!(
                        "{}WARNING: shader {} in skin {} not found",
                        S_COLOR_RED.to_str().expect("S_COLOR_RED is ASCII"),
                        shader_name,
                        skin_name,
                    );
                }
            }
            resolved
        } else if num_shaders <= 0 {
            ShaderHandle::slot_zero()
        } else {
            // The oracle indexes `md3Shader[skinNum % numShaders]`. A negative
            // `skinNum` would index before the array (Raven UB, §19). Here
            // `rem_euclid` keeps the index in range, the one defined behavior.
            let sel = ent.e.skinNum.rem_euclid(num_shaders);
            let shaders_base = unsafe { (surf as *const u8).add((*surf).ofsShaders as usize) }
                as *const md3Shader_t;
            let shader_index = unsafe { (*shaders_base.add(sel as usize)).shaderIndex };
            R_GetShaderByHandleQuiet(assets, shader_index, warnings)
        };

        // DEFERRED: R_AddMD3Surfaces stencil-shadow and projection-shadow
        // pushes — the `tr.shadowShader`/`tr.projectionShadowShader` arms
        // (`r_shadows == 2`/`3`) need the shadow backend, which does not exist.
        // Source: oracle/codemp/renderer/tr_mesh.cpp:388-405

        // don't add third_person objects if not viewing through a portal
        if !personal_model {
            let sorted_index = assets
                .shaders
                .get(shader)
                .map(|s| s.sorted_index)
                .unwrap_or(0);
            R_AddDrawSurf(
                SurfaceGeometry::Md3(Md3SurfaceRef {
                    h_model: ent.e.hModel,
                    lod,
                    surface_index: i,
                    frame: ent.e.frame,
                    old_frame: ent.e.oldframe,
                    backlerp: ent.e.backlerp,
                }),
                sorted_index,
                shifted_entity_num,
                rdf_nofog,
                fog_num,
                0,
                draw_surfs,
            );
        }

        // find the next surface
        surf = unsafe { (surf as *const u8).add((*surf).ofsEnd as usize) } as *const md3Surface_t;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render_state::skin_asset::SkinSurface;

    // frame wrap and clamp

    #[test]
    fn frames_in_range_pass_through_unchanged() {
        // given three frames and both indices in range
        let (frame, oldframe, bad) = wrap_and_check_md3_frames(2, 1, 3, false);
        // then nothing changes and the frames are not flagged bad
        assert_eq!((frame, oldframe, bad), (2, 1, false));
    }

    #[test]
    fn wrap_mods_the_frames_by_the_count() {
        // given RF_WRAP_FRAMES and indices past the count
        let (frame, oldframe, bad) = wrap_and_check_md3_frames(7, 4, 3, true);
        // then both wrap into range and are not bad
        assert_eq!((frame, oldframe, bad), (1, 1, false));
    }

    #[test]
    fn truncating_wrap_keeps_a_negative_frame_negative_and_flags_bad() {
        // given RF_WRAP_FRAMES and a negative old frame
        let (frame, oldframe, bad) = wrap_and_check_md3_frames(1, -1, 3, true);
        // then the truncating mod leaves -1, unlike rem_euclid, and flags bad
        assert_eq!((frame, oldframe, bad), (1, -1, true));
    }

    #[test]
    fn out_of_range_frames_are_flagged_bad_without_clamp() {
        // given an out-of-range frame with no wrap
        let (frame, oldframe, bad) = wrap_and_check_md3_frames(5, 1, 3, false);
        // then the post-wrap values pass through and the frame is flagged bad
        assert_eq!((frame, oldframe, bad), (5, 1, true));

        // a negative old frame is bad too
        let (frame, oldframe, bad) = wrap_and_check_md3_frames(1, -1, 3, false);
        assert_eq!((frame, oldframe, bad), (1, -1, true));
    }

    // skin-name shader resolve

    #[test]
    fn skin_shader_resolves_by_lowercased_name() {
        // given a skin with two surfaces
        let skin = SkinAsset {
            name: "models/players/kyle/model_default.skin".to_owned(),
            surfaces: vec![
                SkinSurface {
                    name: "torso".to_owned(),
                    shader: ShaderHandle::new(7, 0),
                },
                SkinSurface {
                    name: "head".to_owned(),
                    shader: ShaderHandle::new(9, 0),
                },
            ],
        };
        // then a matching name returns that surface's shader (`ShaderHandle`
        // has no `Debug`, so the compare is `==`, not `assert_eq!`)
        assert!(resolve_skin_shader(&skin, "head") == ShaderHandle::new(9, 0));
        // and a non-matching name returns the default (slot 0)
        assert!(resolve_skin_shader(&skin, "legs") == ShaderHandle::slot_zero());
    }
}
