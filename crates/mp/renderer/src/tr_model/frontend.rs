//! Raven `tr_model.cpp` client-rendering remainder (R3) — the server-subset
//! loaders live in this dir's other files and are NOT re-ported here.
//!
//! Source: `oracle/codemp/renderer/tr_model.cpp`

use core::ffi::c_void;

use mp_engine_qcommon::qfiles::md3_frame_s::md3Frame_t;
use mp_engine_qcommon::qfiles::md3_header_t::md3Header_t;
use mp_engine_qcommon::qfiles::md3_tag_s::md3Tag_t;
use mp_host_interface::mdx::mdxm::MdxmView;
use mp_qshared::shared::q_math::VectorClear;
use mp_qshared::shared::{orientation_t, qhandle_t, vec3_t};
use native_math::qmath::{AxisClear, VectorNormalize};

use super::render_models::RenderModels;
use super::server_load::read_qpath;
use crate::tr_local::model_s::model_t;

// PORT-NOTE: R3 wave-0 packet `tr_model.wave0.md` lists 13 fns; 12 of them
// are already live in this subsystem (`_PREAMBLE.md` "Never re-port an
// already-ported fn" — the wave-partition manifest predates that landed
// work, a wave-planning gap, not something to fork a second port over).
// Reconciled here rather than re-transcribed:
// - `CachedEndianedModelBinary_s` ctor -> `cached_model_binary.rs`'s
//   `impl Default for CachedEndianedModelBinary`.
// - `RE_RegisterModels_StoreShaderRequest` ->
//   `RenderModels::re_register_models_store_shader_request` (`cached_model_binary.rs`).
// - `RE_RegisterModels_GetDiskFile` ->
//   `RenderModels::re_register_models_get_disk_file` (`cached_model_binary.rs`).
// - `GetModelDataAllocSize` -> `RenderModels::get_model_data_alloc_size` (`cached_model_binary.rs`).
// - `RE_RegisterModels_DumpNonPure` ->
//   `RenderModels::re_register_models_dump_non_pure` (`cached_model_binary.rs`).
// - `RE_RegisterModels_Info_f` -> `RenderModels::models_info_f` (`cached_model_binary.rs`).
// - `RE_RegisterModels_DeleteAll` -> `RenderModels::re_register_models_delete_all` (`cached_model_binary.rs`).
// - `RE_RegisterMedia_GetLevel` -> `RenderModels::media_get_level` (`cached_model_binary.rs`).
// - `R_GetModelByHandle` -> `RenderModels::get_model` (`render_models.rs`).
// - `R_AllocModel` -> `RenderModels::r_alloc_model` (`render_models.rs`).
// - `generateHashValue` -> deliberately NOT reproduced; `render_models.rs`'s
//   `re_insert_model_into_hash` already documents the drop (`TRM-D3`/ruling 53:
//   the case-insensitive name->handle `HashMap` subsumes the bucket+node scheme).
// - `R_Modellist_f` -> `RenderModels::modellist_f` (`render_models.rs`).
//
// The 13th, `R_GetTag`, has no existing port — transcribed below.
//
// PORT-NOTE: R3 wave-1 packet `tr_model.wave1.md` lists 8 fns; 6 of them are
// already live in this subsystem (`_PREAMBLE.md` "Never re-port an
// already-ported fn" — same wave-planning gap as above). Reconciled here
// rather than re-transcribed:
// - `RE_RegisterServerModels_Malloc` ->
//   `RenderModels::re_register_server_models_malloc` (`cached_model_binary.rs`).
// - `RE_RegisterModels_LevelLoadEnd` ->
//   `RenderModels::models_level_load_end` (`cached_model_binary.rs`).
// - `RE_InsertModelIntoHash` ->
//   `RenderModels::re_insert_model_into_hash` (`render_models.rs`).
// - `R_ModelInit` -> `RenderModels::model_init` (`render_models.rs`).
// - `R_HunkClearCrap` -> `RenderModels::hunk_clear` (`render_models.rs`).
// - `R_ModelFree` -> `RenderModels::model_free` (`render_models.rs`).
//
// The remaining 2, `R_LerpTag` and `R_ModelBounds`, have no existing port —
// transcribed below.
//
// PORT-NOTE: R3 wave-2 packet `tr_model.wave2.md` lists 2 fns; BOTH are
// already live in this subsystem (`_PREAMBLE.md` "Never re-port an
// already-ported fn" — same wave-planning gap as wave-0/wave-1 above).
// Reconciled here, nothing new to transcribe:
// - `ServerLoadMDXA` -> `RenderModels::server_load_mdxa` (`server_load.rs`),
//   including the `#ifndef _M_IX86` skeletal/frame swap §20-drop
//   (`TRM-D3`/ruling 54: dead arm on the `_M_IX86` WinDed target) and the
//   `TRM-D4`/ruling 58 `AlignedBytes` cast discipline.
// - `R_SVModelInit` -> `RenderModels::model_init` (`render_models.rs`),
//   folded with `R_ModelInit` per §C10 (bare wrapper, always-compiled
//   dedicated-live entry).
//
// PORT-NOTE: R3 wave-3 packet `tr_model.wave3.md` lists 2 fns; BOTH are
// already reconciled in this subsystem (`_PREAMBLE.md` "Never re-port an
// already-ported fn" — same wave-planning gap as wave-0/wave-1/wave-2
// above). Nothing new to transcribe:
// - `RE_RegisterMedia_LevelLoadBegin` ->
//   `RenderModels::media_level_load_begin` (`cached_model_binary.rs`).
// - `RE_RegisterMedia_LevelLoadEnd` -> deliberately NOT ported: its sole
//   caller is the client `cl_cgame.cpp:1942`, zero dedicated callers
//   (`TRM-D5`/ruling 59b, already documented in `cached_model_binary.rs`'s
//   module doc "§20-dropped, no stub"); the live dedicated eviction path is
//   `RenderModels::models_level_load_end`.

/// The crate's single [`MdxmView`] handout over a `model_t`'s `.glm` block.
///
/// DEC-35's safe surface (`EngineHost::model_mdxm` -> `MdxmRef`) is keyed by
/// `qhandle_t`; Raven's `G2_FindSurface_BC`/`G2_ProcessSurfaceBolt` are handed
/// a bare `model_s *` with no host in reach, so the raw block read is
/// quarantined here — the one unsafe site behind which `tr_ghoul2.rs` stays
/// safe (porting-rules §D11).
///
/// # Safety invariant
/// `model.type == MOD_GL2M`, so `model.mdxm` is the non-null, endian-swap-
/// completed `.glm` block the model loader stored (`server_load_mdxm` ->
/// `CachedEndianedModelBinary::disk_image`), self-sized by its `ofsEnd` field
/// and immutable for as long as the model is registered. The returned view
/// borrows for `'a`, the lifetime of `model`, which the registry outlives.
///
/// Source: `oracle/codemp/renderer/tr_ghoul2.cpp:2952-2978` (the `mod->mdxm`
/// dereference this replaces)
pub fn mdxm_view_of<'a>(model: &'a model_t) -> MdxmView<'a> {
    debug_assert!(!model.mdxm.is_null());
    unsafe { MdxmView::from_block(model.mdxm as *const c_void) }
}

/// Raven `R_GetTag` — looks up `tag_name` at `frame` in an MD3 model's tag
/// table, clamping `frame` to the model's last valid frame.
///
/// Raven: it is possible to have a bad frame while changing models, so don't error
///
/// # Safety
/// `md3` must point at a fully loaded, in-bounds `md3Header_t` blob whose
/// `ofsTags`/`numFrames`/`numTags` describe `numFrames * numTags` valid
/// trailing `md3Tag_t` entries — the same raw-pointer contract `model_t::md3`
/// already carries on the live tier-2 path (`_PREAMBLE.md` Group 6: on-disk
/// MD3 layout is a frozen file format, not new interior state, so this reads
/// through the existing tier-2 shape rather than extending it).
///
/// Source: `oracle/codemp/renderer/tr_model.cpp:1744-1761`
pub unsafe fn r_get_tag(
    md3: *const md3Header_t,
    frame: i32,
    tag_name: &str,
) -> Option<*const md3Tag_t> {
    let mut frame = frame;
    if frame >= (*md3).numFrames {
        // it is possible to have a bad frame while changing models, so don't error
        frame = (*md3).numFrames - 1;
    }

    let tags_base = (md3 as *const u8).add((*md3).ofsTags as usize) as *const md3Tag_t;
    let mut tag = tags_base.add((frame * (*md3).numTags) as usize);

    for _ in 0..(*md3).numTags {
        if read_qpath(&(*tag).name) == tag_name {
            return Some(tag); // found it
        }
        tag = tag.add(1);
    }

    None
}

/// Raven `R_LerpTag` — looks up `tag_name` in an MD3 model's `start_frame`
/// and `end_frame` tag tables and linearly interpolates the origin/axis by
/// `frac`, writing the result into `tag`. Returns `false` (Raven `qfalse`)
/// if the model has no MD3 LOD 0, or `tag_name` isn't found in either frame
/// — both leave `tag` axis-cleared/origin-cleared, matching Raven.
///
/// Raven: it is possible to have a bad frame while changing models, so don't error
///
/// # Safety
/// `handle` must resolve (through [`RenderModels::get_model`]) to a
/// `model_t` whose `md3[0]`, if non-null, satisfies [`r_get_tag`]'s pointer
/// contract (tier-2 raw-pointer read, `_PREAMBLE.md` Group 6: on-disk MD3
/// layout is a frozen file format, not new interior state).
///
/// Source: `oracle/codemp/renderer/tr_model.cpp:1768-1803`
pub unsafe fn r_lerp_tag(
    rm: &RenderModels,
    tag: &mut orientation_t,
    handle: qhandle_t,
    start_frame: i32,
    end_frame: i32,
    frac: f32,
    tag_name: &str,
) -> bool {
    let model = rm.get_model(handle);
    if model.md3[0].is_null() {
        AxisClear(tag.axis.as_mut_ptr());
        VectorClear(&mut tag.origin);
        return false;
    }

    let start = r_get_tag(model.md3[0], start_frame, tag_name);
    let end = r_get_tag(model.md3[0], end_frame, tag_name);
    let (Some(start), Some(end)) = (start, end) else {
        AxisClear(tag.axis.as_mut_ptr());
        VectorClear(&mut tag.origin);
        return false;
    };

    let front_lerp = frac;
    let back_lerp = 1.0 - frac;

    for i in 0..3 {
        tag.origin[i] = (*start).origin[i] * back_lerp + (*end).origin[i] * front_lerp;
        tag.axis[0][i] = (*start).axis[0][i] * back_lerp + (*end).axis[0][i] * front_lerp;
        tag.axis[1][i] = (*start).axis[1][i] * back_lerp + (*end).axis[1][i] * front_lerp;
        tag.axis[2][i] = (*start).axis[2][i] * back_lerp + (*end).axis[2][i] * front_lerp;
    }
    VectorNormalize(&mut tag.axis[0]);
    VectorNormalize(&mut tag.axis[1]);
    VectorNormalize(&mut tag.axis[2]);

    true
}

/// Raven `R_ModelBounds` — a model's bounding box: an inline (brush)
/// model's own `bmodel_t::bounds`, else its MD3 LOD-0 first frame's bounds,
/// else a cleared box if the model has no MD3 LOD 0. Out-params `mins`/
/// `maxs` become the return tuple (porting-rules §C7).
///
/// # Safety
/// `handle` must resolve (through [`RenderModels::get_model`]) to a
/// `model_t` whose `bmodel`/`md3[0]`, if non-null, satisfy the tier-2
/// raw-pointer read contract those fields already carry (`_PREAMBLE.md`
/// Group 1/6).
///
/// Source: `oracle/codemp/renderer/tr_model.cpp:1811-1836`
pub unsafe fn r_model_bounds(rm: &RenderModels, handle: qhandle_t) -> (vec3_t, vec3_t) {
    let model = rm.get_model(handle);

    if !model.bmodel.is_null() {
        return ((*model.bmodel).bounds[0], (*model.bmodel).bounds[1]);
    }

    if model.md3[0].is_null() {
        return ([0.0; 3], [0.0; 3]);
    }

    let header = model.md3[0];
    let frame = (header as *const u8).add((*header).ofsFrames as usize) as *const md3Frame_t;

    ((*frame).bounds[0], (*frame).bounds[1])
}
