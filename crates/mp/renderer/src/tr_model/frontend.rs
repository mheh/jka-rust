//! Raven `tr_model.cpp` client-rendering remainder (R3) — the server-subset
//! loaders live in this dir's other files and are NOT re-ported here.
//!
//! Source: `oracle/codemp/renderer/tr_model.cpp`

use core::ffi::c_void;

use mp_engine_qcommon::qfiles::md3_header_t::md3Header_t;
use mp_engine_qcommon::qfiles::md3_tag_s::md3Tag_t;
use mp_host_interface::mdx::mdxm::MdxmView;

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
