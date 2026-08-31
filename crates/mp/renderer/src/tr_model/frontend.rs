//! Raven `tr_model.cpp` client-rendering remainder (R3) — the server-subset
//! loaders live in this dir's other files and are NOT re-ported here.
//!
//! # The client model family's carrier shape (DEC-42.3)
//!
//! Every client-only registration entry point below — [`RE_RegisterModel`],
//! `RE_RegisterModel_Actual`, [`r_load_md3`], [`re_register_models_malloc`],
//! and their `tr_ghoul2.rs` siblings `RenderModels::r_load_mdxa`/`r_load_mdxm`
//! — takes `view: &mut EngineHostView` as its engine carrier and threads the
//! renderer-state bundle `R_FindShader`/`RE_LoadWorldMap_Actual` need beside
//! it, in those two fns' own parameter order (`qs, frame, assets, view,
//! cvars, models, img_state, sky_view, sky[, world_effects]`).
//! `common` is reached as `view.common` by sequential reborrow; server-shared
//! helpers that take `host: &mut impl EngineHost`
//! (`re_register_models_get_disk_file`, `re_register_server_models_malloc`)
//! are handed `view` itself, which implements the trait.
//!
//! DEC-42.3 words `RenderModels` as reached "via the ruled scoped slot-cast".
//! It is threaded as an explicit `rm: &mut RenderModels` parameter instead —
//! the shape the already-landed `R_FindShader`/`RE_LoadWorldMap_Actual` chose
//! for the same state (`models: &RenderModels` beside `view`), so the family
//! composes with them without a cast at every call site. `view.rm` names the
//! same object; no `EngineHost` service this family calls reaches it (only
//! `print`/`fs_read_file`/`fs_free_file`/`fs_file_is_in_pak`, all
//! `Common`-side), so the two never alias live.
//!
//! Source: `oracle/codemp/renderer/tr_model.cpp`

// Raven fn names keep their casing (house convention, same as the sibling
// wave TUs).
#![allow(non_snake_case)]

use core::ffi::{c_char, c_void};
use std::sync::Arc;

use mp_engine_qcommon::common::{com_error, com_printf, EngineHostView};
use mp_engine_qcommon::common_fns::Com_DPrintf;
use mp_engine_qcommon::qfiles::md3_frame_s::md3Frame_t;
use mp_engine_qcommon::qfiles::md3_header_t::md3Header_t;
use mp_engine_qcommon::qfiles::md3_limits::{MD3_IDENT, MD3_VERSION};
use mp_engine_qcommon::qfiles::md3_shader_t::md3Shader_t;
use mp_engine_qcommon::qfiles::md3_surface_t::md3Surface_t;
use mp_engine_qcommon::qfiles::md3_tag_s::md3Tag_t;
use mp_engine_qcommon::qfiles::shader_limits::{SHADER_MAX_INDEXES, SHADER_MAX_VERTEXES};
use mp_host_interface::mdx::mdxm::MdxmView;
use mp_host_interface::EngineHost;
use mp_qshared::common::mp::qcommon::tags::memtag_t;
use mp_qshared::shared::com_parse::QSharedScratch;
use mp_qshared::shared::q_color::{S_COLOR_RED, S_COLOR_YELLOW};
use mp_qshared::shared::q_math::VectorClear;
use mp_qshared::shared::{errorParm_t, orientation_t, qhandle_t, vec3_t, MAX_QPATH};
use native_math::qmath::{AxisClear, VectorNormalize};
use native_math::rng::Rng;
use native_string::q_string::Q_strlwr;

use super::render_models::RenderModels;
use super::server_load::read_qpath;
use crate::render_state::frame_data::FrameData;
use crate::render_state::frame_state::FrameState;
use crate::render_state::world_load_state::WorldLoadState;
use crate::render_state::placeholders::{GlConfig, WorldAsset};
use crate::render_state::render_assets::RenderAssets;
use crate::render_state::render_assets_sim::RenderAssetsSim;
use crate::render_state::renderer_cvars::RendererCvars;
use crate::tr_bsp::RE_LoadWorldMap_Actual;
use crate::tr_cmds::{RE_StretchPic, R_SyncRenderThread};
use crate::tr_font::FontState;
use crate::tr_image::TrImageState;
use crate::tr_init::R_Init;
use crate::tr_local::model_s::model_t;
use crate::tr_local::modtype_t::modtype_t;
use crate::tr_local::surface_type_t::surfaceType_t;
use crate::tr_local::view_parms_t::viewParms_t;
use crate::tr_noise::NoiseState;
use crate::tr_scene::{RE_ClearScene, SceneState};
use crate::tr_shader::{lightmapsNone, stylesDefault, R_FindShader};
use crate::tr_worldeffects::world_effects::WorldEffectsState;

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
//
// PORT-NOTE: R3 wave-8 packet `tr_model.wave8.md`'s sole fn,
// `RE_RegisterModels_Malloc`, was earlier dispositioned "§20-dropped, no
// stub" in `cached_model_binary.rs`'s module doc under the dedicated-only-
// scope FROZEN design (`docs/subsystems/tr-model.md` `TRM-D3`/ruling 54:
// "client model-load path, no live dedicated caller"). That ruling predates
// this file's R3 client-rendering-remainder track, which already ports the
// SAME disposition class alongside it (`R_LerpTag`/`R_ModelBounds`/
// `R_GetTag`, `tr-model.md:818`) — reconciled here as a live client-track
// transcription, not a fork of an already-ported fn.
//
// PORT-NOTE: R3 wave-9 packet `tr_model.wave9.md` lists 3 fns; 2 of them are
// already live in this subsystem (`_PREAMBLE.md` "Never re-port an
// already-ported fn" — same wave-planning gap as waves 0/1/2/3/8 above).
// Reconciled here, nothing new to transcribe:
// - `ServerLoadMDXM` -> `RenderModels::server_load_mdxm` (`server_load.rs`).
// - `RE_RegisterServerModel` -> `RenderModels::register_server_model`
//   (`server_load.rs`).
// Both landed in the §F dedicated-server model-loader slice
// (`fd18c1b9`, `docs/subsystems/tr-model.md`) before this wave-partition
// manifest existed.
//
// The remaining fn, `R_LoadMD3`, has no existing port — transcribed below.
// Unlike its two SCC-479 siblings it is a genuinely client-track loader
// (`RE_RegisterServerModel`'s dispatch only recognizes `MDXA_IDENT`/
// `MDXM_IDENT`; an `MD3_IDENT` file hits its `default: goto fail` arm, so
// the dedicated server never calls `R_LoadMD3`), which is why it belongs in
// this file rather than `server_load.rs`.

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
/// model's own `BModel::bounds`, else its MD3 LOD-0 first frame's bounds,
/// else a cleared box if the model has no MD3 LOD 0. Out-params `mins`/
/// `maxs` become the return tuple (porting-rules §C7).
///
/// The brush arm reads the owned world: `RenderModels::bmodel_index` maps the
/// handle to a `WorldAsset::bmodels` row, which replaces `model_t::bmodel`.
///
/// # Safety
/// The MD3 arm reads `model_t::md3[0]` raw. `handle` must resolve (through
/// [`RenderModels::get_model`]) to a `model_t` whose `md3[0]`, if non-null,
/// satisfies the tier-2 raw-pointer read contract that field already carries
/// (`_PREAMBLE.md` Group 6).
///
/// Source: `oracle/codemp/renderer/tr_model.cpp:1811-1836`
pub unsafe fn r_model_bounds(
    rm: &RenderModels,
    assets: &RenderAssets,
    handle: qhandle_t,
) -> (vec3_t, vec3_t) {
    if let Some((world_index, idx)) = rm.bmodel_location(handle) {
        // A sub-BSP instance's submodels live in their own world's table, Raven's `tr.bspModels[index - 1]`.
        let world: &WorldAsset = if world_index == 0 {
            assets
                .world
                .as_deref()
                .expect("r_model_bounds needs the loaded world for a brush model")
        } else {
            &assets.bsp_models[world_index - 1]
        };
        let bmodel = &world.bmodels[idx];
        return (bmodel.bounds[0], bmodel.bounds[1]);
    }

    let model = rm.get_model(handle);

    if model.md3[0].is_null() {
        return ([0.0; 3], [0.0; 3]);
    }

    let header = model.md3[0];
    let frame = (header as *const u8).add((*header).ofsFrames as usize) as *const md3Frame_t;

    ((*frame).bounds[0], (*frame).bounds[1])
}

/// Raven `RE_RegisterModels_Malloc` — the client model-load cache ingest.
/// Structurally the client twin of
/// [`RenderModels::re_register_server_models_malloc`] (`cached_model_binary.rs`):
/// the same fresh-vs-repeat `CachedModels` entry lifecycle (morph/allocate
/// the disk buffer, stamp `alloc_size`/`pak_file_checksum`/
/// `last_level_used_on`), keyed by the same `disk_image.is_none()` check —
/// the oracle's fresh-entry and checksum-stamp code is *the same code* in
/// both fns, so this delegates rather than duplicating it (porting-rules
/// §C10: preserve behavior, not shape). Raven's `assert(CachedModels)`
/// (`:183`) needs no Rust equivalent — `RenderModels::cached` is an owned,
/// never-null map.
///
/// On a repeat registration the oracle additionally runs an `#ifndef
/// DEDICATED` shader-poke replay (`:221-242`), transcribed live here (DEC-40
/// client leg): for each `(nameOffset, pokeOffset)` pair this model recorded,
/// it re-resolves the shader via `R_FindShader(psShaderName, lightmapsNone,
/// stylesDefault, qtrue)` and pokes the resolved registry index (or `0` for a
/// fallback default shader) back into the disk image at `pokeOffset`. The
/// poked `int` is `handle.index() as i32` — the arena slot number IS Raven's
/// `shader_t::index` (DEC-42.2).
///
/// Raven walks `ModelBin` in place, resolving and poking one entry at a time;
/// this reads the whole request list out first
/// ([`RenderModels::shader_register_requests`]) and pokes back through
/// [`RenderModels::poke_shader_index`], because `R_FindShader` borrows the
/// same `RenderModels` this fn holds mutably. Behaviorally identical:
/// `R_FindShader` takes `models: &RenderModels`, so nothing it does can
/// change the request list or the disk image mid-walk.
/// Source: `oracle/codemp/renderer/tr_model.cpp:221-242`
///
/// Raven's `Z_Malloc`/`Z_MorphMallocTag` zone-allocator calls are not
/// reproduced — same `TRM-D4`(a)/`TRM-D3` ruling-54 divergence the delegated
/// twin already carries (an owned [`AlignedBytes`](super::aligned_bytes::AlignedBytes)
/// ingest copy, no Zone-allocator seam).
///
/// Out-param `qboolean *pqbAlreadyFound` collapses to the return tuple's
/// second field (porting-rules §C7); the `void *` return is the entry's
/// disk-image base pointer, matching the delegated twin's `*mut u8`.
/// The returned pointer is re-read after the replay, because DEC-65 ruling B made the poke copy-on-write and it
/// can leave the entry naming a different allocation.
///
/// Source: `oracle/codemp/renderer/tr_model.cpp:179-249`
#[allow(clippy::too_many_arguments)]
pub(crate) fn re_register_models_malloc(
    qs: &mut QSharedScratch,
    world_load: &mut WorldLoadState,
    assets: &mut RenderAssets,
    view: &mut EngineHostView,
    cvars: &RendererCvars,
    rm: &mut RenderModels,
    img_state: &mut TrImageState,
    sky_view: &mut viewParms_t,
    size: i32,
    disk_buffer_if_just_loaded: Option<&[u8]>,
    model_file_name: &str,
    tag: memtag_t,
) -> (*mut u8, bool) {
    // The fresh/repeat-entry ingest logic is identical to the already-ported
    // server twin, so this delegates to it rather than duplicating it
    // (porting-rules §C10).
    let (mut ptr, already_found) = rm.re_register_server_models_malloc(
        view,
        size,
        disk_buffer_if_just_loaded,
        model_file_name,
        tag,
    );

    if already_found {
        // if we already had this model entry, then re-register all the shaders it wanted...
        for (shader_name, poke_offset) in rm.shader_register_requests(model_file_name) {
            let sh = R_FindShader(
                &shader_name,
                &lightmapsNone,
                &stylesDefault,
                true,
                qs,
                world_load,
                assets,
                view,
                cvars,
                rm,
                img_state,
                sky_view,
            );
            let is_default_shader = assets
                .shaders
                .get(sh)
                .map(|shader| shader.default_shader)
                .unwrap_or(false);
            let poked = if is_default_shader {
                0
            } else {
                sh.index() as i32
            };
            rm.poke_shader_index(model_file_name, poke_offset, poked);
        }

        // A copy-on-write poke can leave the entry naming a different allocation than the one the malloc above
        // returned. All three callers store the returned pointer into `model_t`, so it must name the live block.
        if let Some(current) = rm.block_base_ptr(model_file_name) {
            ptr = current;
        }
    }

    (ptr, already_found)
}

/// Raven's `LL(x)` macro (`tr_model.cpp:20`) — identity on the LE hosts this
/// port targets (`TRM-D3`/ruling 54). `server_load.rs` already carries this
/// same tiny helper for its own swap sites, but that copy is private to its
/// module and this wave's scope is this file only (touch nothing else), so
/// this is a second small definition rather than a cross-module promotion.
///
/// Source: `oracle/codemp/renderer/tr_model.cpp:20`
#[inline]
fn ll(x: i32) -> i32 {
    x.to_le()
}

/// Raven `R_LoadMD3` — loads an MD3 mesh file at LOD `lod` into
/// `model.md3[lod]`, the client-track twin of
/// [`RenderModels::server_load_mdxa`]/`server_load_mdxm`'s cache/malloc/
/// endian-swap handshake (`server_load.rs`) applied to the MD3 format.
/// [`RE_RegisterModel`]'s LOD loop is this fn's sole caller; the dedicated
/// server never reaches it (`RE_RegisterServerModel`'s dispatch only
/// recognizes `MDXA_IDENT`/`MDXM_IDENT`, module-doc PORT-NOTE above).
///
/// Peeks `version`/`ofsEnd` out of `buffer`, `LittleLong`'d only when
/// `already_cached` is false; rejects a version mismatch with a
/// `Com_Printf` warning (routed through `host.print`, this subsystem's
/// established `Com_Printf` sink — `render_models.rs`'s `modellist_f`).
/// Sets `model.type = MOD_MESH` and bumps `model.dataSize`, then hands
/// `buffer` to `RE_RegisterModels_Malloc` (`TAG_MODEL_MD3`) which returns
/// the owning block as `model.md3[lod]`; that call's own `already_found`
/// out-param must equal the incoming `already_cached` (Raven's own
/// `assert`). A first-time load flips `already_cached` to `true` and runs
/// the header `LL()` swaps (`:1483-1491`); an already-found block skips
/// straight to the `numFrames < 1` reject, which both paths share
/// (`:1494-1502`) before an already-found block's early `qtrue` return.
///
/// The `#ifndef _M_IX86` frame/tag swaps (`:1509-1529`) are §20-dropped —
/// dead arm on the `_M_IX86` WinDed target (`TRM-D3`/ruling 54), same as the
/// analogous skeletal/frame swaps `server_load.rs`'s `server_load_mdxa`
/// already drops.
///
/// The surface loop (`:1533-1618`) runs on intel too: header-field `LL()`
/// swaps, `SHADER_MAX_VERTEXES`/`SHADER_MAX_INDEXES` bounds checks that
/// `Com_Error(ERR_DROP, ...)` on overflow (never a `return qfalse` here,
/// unlike the sibling GLM/GLA loaders — faithfully kept, not normalized,
/// §A2), forced `surf.ident = SF_MD3`, and the lowercase-name +
/// trailing-`_1`/`_2`-strip (`Q_strlwr`, "a crutch for q3data being a mess"
/// per Raven's comment), and the nested `#ifndef DEDICATED` shader
/// registration (`:1567-1582`) — live per DEC-40, resolving each
/// `md3Shader_t::name` through `R_FindShader(name, lightmapsNone,
/// stylesDefault, qtrue)` into `md3Shader_t::shaderIndex` (`0` for a default
/// shader, else `handle.index() as i32` — the arena slot number IS Raven's
/// `shader_t::index`, DEC-42.2) and recording the request for
/// [`re_register_models_malloc`]'s repeat replay. Only the nested
/// `#ifndef _M_IX86` triangle/ST/XyzNormal swaps are dropped, under the same
/// ruling-54 identity-on-LE disposition as the frame/tag swaps above.
///
/// The `*mut md3Header_t`/`*mut md3Surface_t` casts operate on the
/// 16-byte-aligned `AlignedBytes` base the cache entry owns (`TRM-D4`/ruling
/// 58); `unsafe`-confined at this seam (§D11) with a debug alignment assert
/// at the cast site.
///
/// Source: `oracle/codemp/renderer/tr_model.cpp:1427-1621`
#[allow(clippy::too_many_arguments)]
pub(crate) fn r_load_md3(
    qs: &mut QSharedScratch,
    world_load: &mut WorldLoadState,
    assets: &mut RenderAssets,
    view: &mut EngineHostView,
    cvars: &RendererCvars,
    rm: &mut RenderModels,
    img_state: &mut TrImageState,
    sky_view: &mut viewParms_t,
    model: qhandle_t,
    lod: i32,
    buffer: &[u8],
    mod_name: &str,
    already_cached: &mut bool,
) -> bool {
    let mut version = i32::from_le_bytes(buffer[4..8].try_into().unwrap());
    let mut size = i32::from_le_bytes(buffer[104..108].try_into().unwrap()); // md3Header_t::ofsEnd

    if !*already_cached {
        version = ll(version);
        size = ll(size);
    }

    if version != MD3_VERSION {
        view.print(&format!(
            "{}R_LoadMD3: {} has wrong version ({} should be {})\n",
            S_COLOR_YELLOW.to_str().expect("S_COLOR_YELLOW is ASCII"),
            mod_name,
            version,
            MD3_VERSION
        ));
        return false;
    }

    let idx = model as usize;
    let lod_idx = lod as usize;
    rm.models.slot_mut(idx).r#type = modtype_t::MOD_MESH;
    rm.models.slot_mut(idx).dataSize += size;

    let (ptr, already_found) = re_register_models_malloc(
        qs,
        world_load,
        assets,
        view,
        cvars,
        rm,
        img_state,
        sky_view,
        size,
        Some(buffer),
        mod_name,
        memtag_t::TAG_MODEL_MD3,
    );
    debug_assert_eq!(
        *already_cached, already_found,
        "bAlreadyCached == bAlreadyFound"
    );
    debug_assert_eq!(
        ptr as usize % 16,
        0,
        "AlignedBytes base must be 16-byte aligned"
    );

    let header = ptr as *mut md3Header_t;
    rm.models.slot_mut(idx).md3[lod_idx] = header;

    if !already_found {
        // "we've just done a tag-morph" (Raven) — the one-time ingest copy
        // (`TRM-D4`(a)); `assert(mod->md3[lod] == buffer)` doesn't hold
        // under that divergence and is dropped, not ported (§19).
        *already_cached = true;

        // SAFETY: `header` is the just-copy-constructed 16-byte-aligned
        // `AlignedBytes` base (`TRM-D4`/ruling 58); the debug alignment
        // assert above covers the cast, per §D11.
        unsafe {
            (*header).ident = ll((*header).ident);
            (*header).version = ll((*header).version);
            (*header).numFrames = ll((*header).numFrames);
            (*header).numTags = ll((*header).numTags);
            (*header).numSurfaces = ll((*header).numSurfaces);
            (*header).ofsFrames = ll((*header).ofsFrames);
            (*header).ofsTags = ll((*header).ofsTags);
            (*header).ofsSurfaces = ll((*header).ofsSurfaces);
            (*header).ofsEnd = ll((*header).ofsEnd);
        }
    }

    // SAFETY: see above — `header` is the aligned, live block either way.
    let num_frames = unsafe { (*header).numFrames };
    if num_frames < 1 {
        view.print(&format!(
            "{}R_LoadMD3: {} has no frames\n",
            S_COLOR_YELLOW.to_str().expect("S_COLOR_YELLOW is ASCII"),
            mod_name
        ));
        return false;
    }

    if already_found {
        // "All done. Stop, go no further, do not pass Go..."
        return true;
    }

    // `#ifndef _M_IX86` frame/tag swaps (`:1509-1529`) — §20-dropped, dead
    // arm on the `_M_IX86` WinDed target (`TRM-D3`/ruling 54).

    // SAFETY: every pointer walk below stays inside the `AlignedBytes` block
    // the cache entry owns (its size is the file's `ofsEnd`), off the
    // 16-byte-aligned base asserted above (§D11).
    unsafe {
        let num_surfaces = (*header).numSurfaces;
        let mut surf = ptr.add((*header).ofsSurfaces as usize) as *mut md3Surface_t;

        for _ in 0..num_surfaces {
            (*surf).flags = ll((*surf).flags);
            (*surf).numFrames = ll((*surf).numFrames);
            (*surf).numShaders = ll((*surf).numShaders);
            (*surf).numTriangles = ll((*surf).numTriangles);
            (*surf).ofsTriangles = ll((*surf).ofsTriangles);
            (*surf).numVerts = ll((*surf).numVerts);
            (*surf).ofsShaders = ll((*surf).ofsShaders);
            (*surf).ofsSt = ll((*surf).ofsSt);
            (*surf).ofsXyzNormals = ll((*surf).ofsXyzNormals);
            (*surf).ofsEnd = ll((*surf).ofsEnd);

            if (*surf).numVerts > SHADER_MAX_VERTEXES as i32 {
                com_error(
                    errorParm_t::ERR_DROP,
                    format!(
                        "R_LoadMD3: {} has more than {} verts on a surface ({})",
                        mod_name,
                        SHADER_MAX_VERTEXES,
                        (*surf).numVerts
                    ),
                );
            }
            if (*surf).numTriangles * 3 > SHADER_MAX_INDEXES as i32 {
                com_error(
                    errorParm_t::ERR_DROP,
                    format!(
                        "R_LoadMD3: {} has more than {} triangles on a surface ({})",
                        mod_name,
                        SHADER_MAX_INDEXES / 3,
                        (*surf).numTriangles
                    ),
                );
            }

            // change to surface identifier
            (*surf).ident = surfaceType_t::SF_MD3 as i32;

            // lowercase the surface name so skin compares are faster
            Q_strlwr(&mut (*surf).name);

            // strip off a trailing _1 or _2
            // this is a crutch for q3data being a mess
            //
            // `name_len` is C `strlen` — bytes before the NUL — not the
            // decoded `String`'s `.len()`: `read_qpath` widens every byte
            // >= 0x80 to a 2-byte UTF-8 char, so the two diverge on a
            // non-ASCII surface name.
            let name_len = {
                let name = &(*surf).name;
                name.iter().position(|&c| c == 0).unwrap_or(name.len())
            };
            if name_len > 2 && (*surf).name[name_len - 2] == b'_' as c_char {
                (*surf).name[name_len - 2] = 0;
            }

            // register the shaders
            // Source: oracle/codemp/renderer/tr_model.cpp:1567-1582
            let mut shader = (surf as *mut u8).add((*surf).ofsShaders as usize) as *mut md3Shader_t;
            for _ in 0..(*surf).numShaders {
                let shader_name = read_qpath(&(*shader).name);
                let sh = R_FindShader(
                    &shader_name,
                    &lightmapsNone,
                    &stylesDefault,
                    true,
                    qs,
                    world_load,
                    assets,
                    view,
                    cvars,
                    rm,
                    img_state,
                    sky_view,
                );
                let is_default_shader = assets
                    .shaders
                    .get(sh)
                    .map(|s| s.default_shader)
                    .unwrap_or(false);
                (*shader).shaderIndex = if is_default_shader {
                    0
                } else {
                    // DEC-42.2: the arena slot number IS `shader_t::index`.
                    sh.index() as i32
                };
                // Raven passes the two `char*`/`int*` addresses; the port's
                // store takes them as block-relative byte offsets (that
                // method's own doc comment).
                let name_offset =
                    (core::ptr::addr_of!((*shader).name) as usize - ptr as usize) as i32;
                let poke_offset =
                    (core::ptr::addr_of!((*shader).shaderIndex) as usize - ptr as usize) as i32;
                rm.re_register_models_store_shader_request(mod_name, name_offset, poke_offset);

                shader = shader.add(1);
            }

            // `#ifndef _M_IX86` triangle/ST/XyzNormal swaps (`:1589-1613`) —
            // §20-dropped, same ruling-54 identity-on-LE disposition as the
            // frame/tag swaps above.

            // find the next surface
            surf = (surf as *mut u8).add((*surf).ofsEnd as usize) as *mut md3Surface_t;
        }
    }

    true
}

/// Raven `RE_BeginRegistration` — the client's entry into a fresh (level)
/// registration cycle: reinitializes the renderer (`R_Init`), snapshots the
/// current `glConfig` for the caller, flushes any in-flight renderer
/// commands, clears the scene state, marks the renderer registered, then
/// issues a throwaway zero-size stretch pic — Raven's own comment explains
/// why: without it, the very first level-shot stretch pic is silently
/// dropped and the player sees a white flash on load.
///
/// Out-param `glconfig_t *glconfigOut` becomes the return value (porting-
/// rules §C7). `RenderAssets::glconfig` is the idiomatic `GlConfig` R2
/// already assigns Raven's `glConfig` to (`_PREAMBLE.md` `## State
/// ownership` row `glConfig`) — not the tier-1 `glconfig_t` — so the return
/// type is the owned `GlConfig`, cloned at the same point Raven copies the
/// struct by value.
///
/// `tr.viewCluster = -1;` — landed: `FrameState::view_cluster` is the field
/// home (campaign #41 batch 1).
/// Source: `oracle/codemp/renderer/tr_model.cpp:1637`
///
/// `// rww - 9-13-01 [1-26-01-sof2]` / `//R_ClearFlares();` — already
/// commented out in the oracle itself, nothing to transcribe.
///
/// Source: `oracle/codemp/renderer/tr_model.cpp:1629-1650`
#[allow(clippy::too_many_arguments)]
pub fn RE_BeginRegistration(
    view: &mut EngineHostView,
    cvars: &mut RendererCvars,
    sim: &mut RenderAssetsSim,
    state: &mut TrImageState,
    models: &mut RenderModels,
    frame: &mut FrameState,
    world_load: &mut WorldLoadState,
    scene: &mut SceneState,
    frame_data: &mut FrameData,
    noise: &mut NoiseState,
    rng: &mut Rng,
    font: &mut FontState,
    world_effects: &mut WorldEffectsState,
    qs: &mut QSharedScratch,
    sky_view: &mut viewParms_t,
) -> GlConfig {
    R_Init(
        view,
        cvars,
        sim,
        state,
        models,
        frame,
        world_load,
        scene,
        frame_data,
        noise,
        rng,
        font,
        world_effects,
        qs,
        sky_view,
    );

    // `R_Init` returned the registry, so this scope reaches it again.
    let assets = Arc::make_mut(&mut sim.published);

    let glconfig_out = assets.glconfig.clone();

    R_SyncRenderThread(assets, view.common, cvars);

    // tr.viewCluster = -1;
    frame.view_cluster = -1;

    RE_ClearScene(frame_data, scene);

    assets.registered = true;

    // NOTE: this sucks, for some reason the first stretch pic is never drawn
    // without this we'd see a white flash on a level load because the very
    // first time the level shot would not be drawn
    RE_StretchPic(
        frame_data,
        assets,
        view.common,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0,
        1.0,
        0,
    );

    glconfig_out
}

/// Raven `MDXA_IDENT` (`"2LGA"` on an LE host) — cross-verified (never
/// guessed, porting-rules §A2) against the already-ported copy
/// `server_load.rs` carries for the identical oracle `#define` (it lives in
/// `mdx_format.h`, not `tr_model.cpp`, so it is not in this wave's own
/// FILE-SCOPE CONSTANTS section — the never-guess rule permits reuse of an
/// already-verified copy, the same precedent `tr_ghoul2.rs`'s `MDXA_VERSION`
/// duplication already established for this exact class of gap).
///
/// Source: `oracle/codemp/renderer/mdx_format.h:21`
const MDXA_IDENT: i32 =
    (('A' as i32) << 24) + (('G' as i32) << 16) + (('L' as i32) << 8) + ('2' as i32);

/// Raven `MDXM_IDENT` (`"2LGM"` on an LE host) — same cross-verified-reuse
/// disposition as [`MDXA_IDENT`] above.
///
/// Source: `oracle/codemp/renderer/mdx_format.h:20`
const MDXM_IDENT: i32 =
    (('M' as i32) << 24) + (('G' as i32) << 16) + (('L' as i32) << 8) + ('2' as i32);

/// Raven `Q_strncpyz` applied to `model_t::name` (`mod->name = name`,
/// `tr_model.cpp:1268`). `server_load.rs` already carries an identical small
/// helper for its own `mod->name` write site, but that copy is private to
/// its module and this wave's scope is this file only (touch nothing else),
/// so this is a second small definition rather than a cross-module
/// promotion — same rationale as this file's `ll` above.
///
/// Source: `oracle/codemp/qcommon/q_shared.c` (`Q_strncpyz`)
fn write_qpath(dest: &mut [c_char; MAX_QPATH], src: &str) {
    for slot in dest.iter_mut() {
        *slot = 0;
    }
    let bytes = src.as_bytes();
    let n = bytes.len().min(MAX_QPATH - 1);
    for (i, &b) in bytes[..n].iter().enumerate() {
        dest[i] = b as c_char;
    }
}

/// Raven `static qhandle_t RE_RegisterModel_Actual( const char *name )` —
/// the client model-registration workhorse [`RE_RegisterModel`] wraps with
/// the `gbInsideRegisterModel` re-entrancy guard. Raven marks it `static`
/// (file-internal linkage); translated as a module-private fn here.
///
/// Lookup is a case-insensitive full-name map read (`rm.hash`, the same
/// `mhHashTable`-replacement `RenderModels::re_insert_model_into_hash`
/// already establishes, `TRM-D3`/ruling 53) — `generateHashValue` is not
/// reproduced, so this packet's own FILE-SCOPE CONSTANTS `FILE_HASH_SIZE`
/// has no call site in this port (same disposition
/// `re_insert_model_into_hash`'s own doc comment already states for the
/// bucket scheme it subsumes).
///
/// Both `#ifndef DEDICATED` blocks are this port's live leg (DEC-40: the R3
/// renderer track is the CLIENT port; the jampDed disposition is scoped to
/// the dedicated-server link set):
/// - `R_SyncRenderThread()` (`:1270-1273`) — hence the `assets` parameter
///   carrying `tr.registered` (`tr_cmds.rs:296`).
/// - `RE_LoadWorldMap_Actual(va("maps/%s.bsp", name+1), tr.bspModels[
///   tr.numBSPModels-1], tr.numBSPModels)` (`:1232-1234`). Raven hands over a
///   fixed-array element; the port loads into a locally-owned [`WorldAsset`]
///   and moves it into `RenderAssets::bsp_models` at the same
///   `num_bsp_models - 1` index afterwards — `RE_LoadWorldMap_Actual` takes
///   `assets: &mut RenderAssets` *and* `world: &mut WorldAsset`, so the world
///   cannot be borrowed out of `assets` while it runs. Same shape
///   `RE_LoadWorldMap` (`tr_bsp.rs`) already uses for the `index == 0` load.
///   This is `bsp_models`' first writer.
///
/// `R_LoadMDXM` is `tr_ghoul2.cpp`'s sibling loader, called here through
/// [`RenderModels::r_load_mdxm`] with this family's carrier bundle (file-top
/// DEC-42.3 note), mirroring `R_LoadMDXA`'s sibling
/// (`tr_ghoul2.rs::r_load_mdxa`).
///
/// The `#ifdef _DEBUG` `r_noPrecacheGLA` early-return (`:1375-1380`) and the
/// `#ifdef _DEBUG else { Com_Printf(...) }` branch (`:1388-1392`) are
/// dropped — `_DEBUG`-only, compiled out under `-DNDEBUG` (house convention;
/// `tr_init.rs:876` drops that same cvar's registration for the identical
/// reason; this file's own `r_load_md3` doc comment already applies the
/// same disposition to its sibling debug branch).
///
/// Source: `oracle/codemp/renderer/tr_model.cpp:1169-1400`
#[allow(clippy::too_many_arguments)]
fn RE_RegisterModel_Actual(
    qs: &mut QSharedScratch,
    world_load: &mut WorldLoadState,
    assets: &mut RenderAssets,
    view: &mut EngineHostView,
    cvars: &RendererCvars,
    rm: &mut RenderModels,
    img_state: &mut TrImageState,
    sky_view: &mut viewParms_t,
    world_effects: &mut WorldEffectsState,
    name: &str,
) -> qhandle_t {
    let warn = S_COLOR_YELLOW.to_str().expect("S_COLOR_YELLOW is ASCII");

    if name.is_empty() {
        com_printf(view.common, "RE_RegisterModel: NULL name\n");
        return 0;
    }

    if name.len() >= MAX_QPATH {
        let red = S_COLOR_RED.to_str().expect("S_COLOR_RED is ASCII");
        Com_DPrintf(view.common, &format!("{red}Model name exceeds MAX_QPATH\n"));
        return 0;
    }

    //
    // search the currently loaded models
    //
    // see if the model is already loaded
    if let Some(&handle) = rm.hash.get(&name.to_ascii_lowercase()) {
        return handle;
    }

    if name.as_bytes().first() == Some(&b'#') {
        rm.num_bsp_models += 1;

        // Source: oracle/codemp/renderer/tr_model.cpp:1232-1234
        let index = rm.num_bsp_models;
        let mut world = WorldAsset::default();
        RE_LoadWorldMap_Actual(
            qs,
            world_load,
            assets,
            view,
            cvars,
            rm,
            img_state,
            sky_view,
            world_effects,
            &format!("maps/{}.bsp", &name[1..]),
            &mut world,
            index,
        );
        // `tr.bspModels[tr.numBSPModels - 1] = <the loaded world>`: Raven's
        // store is a fixed `MAX_SUB_BSP` array indexed by the just-bumped
        // counter, so the port writes that same index (growing the `Vec` to
        // reach it), never appends blindly — `media_level_load_begin` zeroes
        // `num_bsp_models` on every level load while the `Vec` persists.
        let slot = (index - 1) as usize;
        if assets.bsp_models.len() <= slot {
            assets.bsp_models.resize_with(slot + 1, WorldAsset::default);
        }
        assets.bsp_models[slot] = world;

        let temp = format!("*{}-0", rm.num_bsp_models);
        if let Some(&handle) = rm.hash.get(&temp.to_ascii_lowercase()) {
            return handle;
        }

        return 0;
    }

    if name.as_bytes().first() == Some(&b'*') {
        // don't create a bad model for a bsp model
        if !name.eq_ignore_ascii_case("*default.gla") {
            return 0;
        }
    }

    // allocate a new model_t

    let Some(handle) = rm.r_alloc_model() else {
        com_printf(
            view.common,
            &format!("{warn}RE_RegisterModel: R_AllocModel() failed for '{name}'\n"),
        );
        return 0;
    };
    let idx = handle as usize;

    // only set the name after the model has been successfully loaded
    write_qpath(&mut rm.models.slot_mut(idx).name, name);

    // make sure the render thread is stopped
    R_SyncRenderThread(assets, view.common, cvars);

    let mut lod: i32 = if name.contains(".md3") {
        // this loads the md3s in reverse so they can be biased
        (rm.models.slot(idx).md3.len() - 1) as i32
    } else {
        0
    };
    rm.models.slot_mut(idx).numLods = 0;

    //
    // load the files
    //
    let mut num_loaded: i32 = 0;

    while lod >= 0 {
        let mut filename = name.to_string();

        if lod != 0 {
            if let Some(dot) = filename.rfind('.') {
                filename.truncate(dot);
            }
            filename.push_str(&format!("_{lod}.md3"));
        }

        if let Some((buf, mut already_cached)) =
            rm.re_register_models_get_disk_file(view, &filename)
        {
            // important that from now on we pass 'filename' instead of
            // 'name' to all model load functions, because 'filename'
            // accounts for any LOD mangling etc so guarantees unique
            // lookups for yet more internal caching...
            //
            // `ident = *(unsigned *)buf; if (!bAlreadyCached) ident =
            // LittleLong(ident);` — reading the first 4 bytes as LE gives
            // the same result in both arms on this LE-only port (same
            // `server_load.rs`/`r_load_md3` precedent).
            let ident = i32::from_le_bytes(buf[0..4].try_into().unwrap());

            let loaded = match ident {
                MDXA_IDENT => rm.r_load_mdxa(
                    qs,
                    world_load,
                    assets,
                    view,
                    cvars,
                    img_state,
                    sky_view,
                    handle,
                    &buf,
                    &filename,
                    &mut already_cached,
                ),
                MDXM_IDENT => rm.r_load_mdxm(
                    qs,
                    world_load,
                    assets,
                    view,
                    cvars,
                    img_state,
                    sky_view,
                    world_effects,
                    handle,
                    &buf,
                    &filename,
                    &mut already_cached,
                ),
                MD3_IDENT => r_load_md3(
                    qs,
                    world_load,
                    assets,
                    view,
                    cvars,
                    rm,
                    img_state,
                    sky_view,
                    handle,
                    lod,
                    &buf,
                    &filename,
                    &mut already_cached,
                ),
                _ => {
                    com_printf(
                        view.common,
                        &format!("{warn}RE_RegisterModel: unknown fileid for {filename}\n"),
                    );
                    // `default: goto fail;` skips the `FS_FreeFile` call
                    // below entirely; `buf` is simply dropped here (Raven
                    // leaks it, kept faithful, §A2).
                    rm.models.slot_mut(idx).r#type = modtype_t::MOD_BAD;
                    rm.re_insert_model_into_hash(name, handle);
                    return 0;
                }
            };

            if !already_cached {
                // important to check!!
                view.fs_free_file(buf);
            }

            if !loaded {
                if lod == 0 {
                    rm.models.slot_mut(idx).r#type = modtype_t::MOD_BAD;
                    rm.re_insert_model_into_hash(name, handle);
                    return 0;
                }
                break;
            }

            rm.models.slot_mut(idx).numLods += 1;
            num_loaded += 1;
            // if we have a valid model and are biased so that we won't
            // see any higher detail ones, stop loading them
            if lod <= view.common.cvar(cvars.r_lodbias).integer {
                break;
            }
        }
        // GetDiskFile failure -> `continue` in the C `for` loop; fall
        // through to the decrement step below.

        lod -= 1;
    }

    if num_loaded != 0 {
        // duplicate into higher lod spots that weren't loaded, in case
        // the user changes r_lodbias on the fly
        let mut l = lod - 1;
        while l >= 0 {
            let m = rm.models.slot_mut(idx);
            m.numLods += 1;
            m.md3[l as usize] = m.md3[(l + 1) as usize];
            l -= 1;
        }

        // `#ifdef _DEBUG r_noPrecacheGLA` early-return dropped, see doc
        // comment above. Source: oracle/codemp/renderer/tr_model.cpp:1375-1380

        rm.re_insert_model_into_hash(name, handle);
        // DEC-65 ruling 1: the slot is finished, so record its blocks for publication.
        rm.mark_block(handle);
        return handle;
    }

    // `#ifdef _DEBUG else { Com_Printf(...) }` dropped, see doc comment
    // above. Source: oracle/codemp/renderer/tr_model.cpp:1388-1392

    // fail:
    // we still keep the model_t around, so if the model name is asked for
    // again, we won't bother scanning the filesystem
    rm.models.slot_mut(idx).r#type = modtype_t::MOD_BAD;
    rm.re_insert_model_into_hash(name, handle);
    0
}

/// Raven `qhandle_t RE_RegisterModel( const char *name )` — wraps
/// [`RE_RegisterModel_Actual`] with the `gbInsideRegisterModel` re-entrancy
/// guard (`RenderModels::inside_register_model`, the field that struct
/// already carries for exactly this purpose — its own doc comment cites
/// this fn by name).
///
/// Source: `oracle/codemp/renderer/tr_model.cpp:1407-1417`
#[allow(clippy::too_many_arguments)]
pub fn RE_RegisterModel(
    qs: &mut QSharedScratch,
    world_load: &mut WorldLoadState,
    assets: &mut RenderAssets,
    view: &mut EngineHostView,
    cvars: &RendererCvars,
    rm: &mut RenderModels,
    img_state: &mut TrImageState,
    sky_view: &mut viewParms_t,
    world_effects: &mut WorldEffectsState,
    name: &str,
) -> qhandle_t {
    let was_inside = rm.inside_register_model;
    rm.inside_register_model = true;

    let q = RE_RegisterModel_Actual(
        qs,
        world_load,
        assets,
        view,
        cvars,
        rm,
        img_state,
        sky_view,
        world_effects,
        name,
    );

    rm.inside_register_model = was_inside;

    q
}
