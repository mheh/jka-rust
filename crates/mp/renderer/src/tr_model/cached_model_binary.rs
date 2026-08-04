//! `CachedEndianedModelBinary` — the `CachedModels` cache-entry type, plus the
//! cache lifecycle/eviction free-fns that operate over `RenderModels.cached`
//! (`TRM-D3`/rulings 52,40,58; `_s`-suffix dropped per ruling 40).
//!
//! Design: `docs/subsystems/tr-model.md` (FROZEN), `## Files roster` (this
//! file's entry), `## Method transcription table`. Per `## Seam definition`'s
//! "Internal (private to `RenderModels`, per §D12)" list, every free-fn below
//! is an `impl RenderModels` method (not a bare free function) — Raven kept
//! them as file-scope statics operating on the file-scope `CachedModels`
//! global; §B3 forbids the statics, so they become methods on the struct that
//! now owns that map (`render_models.rs`). Exact signatures for these
//! internal helpers are **not** pinned by the doc (§D12 porter latitude) —
//! this skeleton picks a concrete shape so every call site has something to
//! transcribe against; a later porter may still adjust it.
//!
//! Visibility: methods called from sibling `tr_model` submodules
//! (`render_models.rs`'s `model_free`, `server_load.rs`'s
//! `register_server_model`/`server_load_mdxa`/`server_load_mdxm`) are
//! `pub(crate)` — mirroring `render_models.rs`'s `pub(crate)` fields, the
//! minimal visibility that keeps the split `impl RenderModels` blocks
//! compiling while staying "internal to `mp_renderer`" (§F17). Methods used
//! only within this file stay module-private.
//!
//! **§20-dropped, no stub (`## Files roster`/`divergences`):**
//! - `RE_RegisterMedia_LevelLoadEnd` (`tr_model.cpp:577`) — sole caller is the
//!   client `cl_cgame.cpp:1942`, zero dedicated callers (`TRM-D5`/ruling
//!   59b); the live eviction path is [`RenderModels::models_level_load_end`].
//!
//! `RE_RegisterModels_Malloc` (client, `tr_model.cpp:179`) + its
//! `#ifndef DEDICATED` shader-poke replay (`:221-242`) were §20-dropped under
//! the dedicated-only-scope FROZEN design (`TRM-D3`/ruling 54); the R3 client
//! track superseded that (DEC-40) and both are live in
//! [`super::frontend::re_register_models_malloc`], the replay reading and
//! writing this file's cache through [`RenderModels::shader_register_requests`]
//! / [`RenderModels::poke_shader_index`].
//!
//! Source: `oracle/codemp/renderer/tr_model.cpp:48-68,70-568`

use core::ffi::{c_char, c_void};
use std::sync::Arc;

use mp_host_interface::mdx::mdxa::{MdxaParsed, MdxaView};
use mp_host_interface::mdx::mdxm::{MdxmParsed, MdxmView};
use mp_host_interface::EngineHost;
use mp_qshared::common::mp::qcommon::tags::memtag_t;
use mp_qshared::shared::{qhandle_t, ForceReload_e};

use crate::render_state::model_block::ModelBlock;

use super::aligned_bytes::AlignedBytes;
use super::render_models::RenderModels;
use super::server_load::read_qpath;

/// Raven `sDEFAULT_GLA_NAME ".gla"` — the program-internal default skeleton
/// name, never disk-loaded (the `FakeGLAFile` intercept) and never dumped by
/// `RE_RegisterModels_DumpNonPure`.
///
/// Source: `oracle/codemp/renderer/mdx_format.h:69`; used at
/// `oracle/codemp/renderer/tr_model.cpp:143,438`
const DEFAULT_GLA_NAME: &str = "*default.gla";

/// Raven `FakeGLAFile[]` — the 294-byte program-internal default-skeleton
/// `.gla` blob `RE_RegisterModels_GetDiskFile` hands back (as if read from
/// disk) for [`DEFAULT_GLA_NAME`], never disk-loaded.
///
/// Source: `oracle/codemp/renderer/tr_model.cpp:95-116`
#[rustfmt::skip]
const FAKE_GLA_FILE: [u8; 294] = [
    0x32, 0x4C, 0x47, 0x41, 0x06, 0x00, 0x00, 0x00, 0x2A, 0x64, 0x65, 0x66, 0x61, 0x75, 0x6C, 0x74,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x3F, 0x01, 0x00, 0x00, 0x00,
    0x14, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x18, 0x01, 0x00, 0x00, 0x68, 0x00, 0x00, 0x00,
    0x26, 0x01, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x4D, 0x6F, 0x64, 0x56, 0x69, 0x65, 0x77, 0x20,
    0x69, 0x6E, 0x74, 0x65, 0x72, 0x6E, 0x61, 0x6C, 0x20, 0x64, 0x65, 0x66, 0x61, 0x75, 0x6C, 0x74,
    0x00, 0xCD, 0xCD, 0xCD, 0xCD, 0xCD, 0xCD, 0xCD, 0xCD, 0xCD, 0xCD, 0xCD, 0xCD, 0xCD, 0xCD, 0xCD,
    0xCD, 0xCD, 0xCD, 0xCD, 0xCD, 0xCD, 0xCD, 0xCD, 0xCD, 0xCD, 0xCD, 0xCD, 0xCD, 0xCD, 0xCD, 0xCD,
    0xCD, 0xCD, 0xCD, 0xCD, 0xCD, 0xCD, 0xCD, 0xCD, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF,
    0x00, 0x00, 0x80, 0x3F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x3F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x3F, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x80, 0x3F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x3F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x3F, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFD, 0xBF, 0xFE, 0x7F, 0xFE, 0x7F, 0xFE, 0x7F,
    0x00, 0x80, 0x00, 0x80, 0x00, 0x80,
];

/// The `RE_RegisterModels_GetDiskFile` result — Raven's `qboolean` return +
/// two out-params (`void **ppvBuffer`, `qboolean *pqbAlreadyCached`) collapse
/// per §C7 to `Option<(Vec<u8>, bool)>`: `None` = read failure, `Some((bytes,
/// already_cached))` on success. `bytes` is always an owned copy — freshly
/// read from disk, the [`DEFAULT_GLA_NAME`] `FakeGLAFile` intercept, or (on a
/// cache hit) a clone of the entry's own [`AlignedBytes`] — so
/// `server_load.rs`'s `server_load_mdxa`/`server_load_mdxm` can read
/// `buffer: &[u8]` without aliasing `RenderModels.cached` (a borrow-safety
/// divergence from Raven's raw-pointer-into-the-cache return, not a behavior
/// change: the same bytes, same `already_cached` semantics).
pub(crate) type GetDiskFileResult = (Vec<u8>, bool);

/// Raven `CachedEndianedModelBinary_s` (`_s`-suffix dropped, ruling 40 —
/// internal types get idiomatic names). One `CachedModels` map entry: the
/// parsed/endian-swapped model disk block plus its cache bookkeeping.
///
/// Source: `oracle/codemp/renderer/tr_model.cpp:48-65`
pub(crate) struct CachedEndianedModelBinary {
    /// `pModelDiskImage` — the parsed model block. `None` mirrors
    /// `pModelDiskImage == NULL` (a freshly-`Default`-constructed entry, per
    /// the `(*CachedModels)[sModelName]` `operator[]` insert-if-missing
    /// idiom). 16-byte-aligned, heap-pinned; in-place mutable for the `LL()`
    /// swap (`TRM-D4`/ruling 58; ruling 52 ownership).
    ///
    /// DEC-65 ruling 1 puts the block behind an `Arc`, so the published registry can name it.
    /// The bytes and the two DEC-35 sidecars moved into [`ModelBlock`] with it.
    ///
    /// Source: `oracle/codemp/renderer/tr_model.cpp:50`
    disk_image: Option<Arc<ModelBlock>>,

    /// `iAllocSize` — "may be useful for mem-query, but I don't actually need
    /// it" (Raven). Backs [`RenderModels`]'s local `GetModelDataAllocSize`
    /// sum (`TRM-D3`/ruling 54 consequence — no Zone-allocator seam).
    ///
    /// Source: `oracle/codemp/renderer/tr_model.cpp:51`
    alloc_size: i32,

    /// `ShaderRegisterData` — `vector<pair<int,int>>` of
    /// (name-offset, poke-offset) pairs recorded by
    /// `RE_RegisterModels_StoreShaderRequest`. Recorded server-side even
    /// though the server never replays it; the client replay
    /// (`RE_RegisterModels_Malloc`'s `#ifndef DEDICATED` block) reads it back
    /// through [`RenderModels::shader_register_requests`].
    ///
    /// Source: `oracle/codemp/renderer/tr_model.cpp:46-47,52`
    shader_register_data: Vec<(i32, i32)>,

    /// `iLastLevelUsedOn` — `-1` init; the eviction key
    /// (`RE_RegisterModels_LevelLoadEnd`/`_DumpNonPure` read it against
    /// [`RenderModels::media_get_level`]).
    ///
    /// Source: `oracle/codemp/renderer/tr_model.cpp:53,62`
    last_level_used_on: i32,

    /// `iPAKFileCheckSum` — `-1` if not from a PAK, else the pure-pak
    /// checksum `fs_file_is_in_pak` stamped at registration
    /// (`TRM-D5`/ruling 59a).
    ///
    /// Source: `oracle/codemp/renderer/tr_model.cpp:54,63`
    pak_file_checksum: i32,
}

impl Default for CachedEndianedModelBinary {
    /// Mirrors `CachedEndianedModelBinary_s`'s default constructor.
    ///
    /// Source: `oracle/codemp/renderer/tr_model.cpp:57-64`
    fn default() -> Self {
        Self {
            disk_image: None,
            alloc_size: 0,
            shader_register_data: Vec::new(),
            last_level_used_on: -1,
            pak_file_checksum: -1,
        }
    }
}

impl RenderModels {
    /// Raven `RE_RegisterModels_GetDiskFile` — returns the cached block
    /// (`already_cached = true`) or, on a cache miss, `FS_ReadFile`s from
    /// disk (`already_cached = false`). Special-cases [`DEFAULT_GLA_NAME`]:
    /// returns a copy of the program-internal [`FAKE_GLA_FILE`] blob as
    /// though it were read from disk, never touching the host FS. See
    /// [`GetDiskFileResult`] for the return-shape rationale.
    ///
    /// Source: `oracle/codemp/renderer/tr_model.cpp:125-171`
    pub(crate) fn re_register_models_get_disk_file(
        &mut self,
        host: &mut impl EngineHost,
        model_file_name: &str,
    ) -> Option<GetDiskFileResult> {
        // Q_strncpyz+Q_strlwr(sModelName) — the lowercased map key/FS path.
        let model_name = model_file_name.to_lowercase();
        // (*CachedModels)[sModelName] — `operator[]` inserts a default entry
        // on a miss, a real side effect this call reproduces (`.or_default()`).
        let entry = self.cached.entry(model_name.clone()).or_default();

        if let Some(disk_image) = &entry.disk_image {
            // Cache hit: return a copy of the cached block, `already_cached = true`.
            // This is a read-only copy out, not an aliasing borrow into `self.cached`
            // (see `GetDiskFileResult`'s doc).
            return Some((disk_image.bytes().to_vec(), true));
        }

        // strcmp against the ORIGINAL (not lowercased) name — case-sensitive,
        // matching Raven's `!strcmp(sDEFAULT_GLA_NAME ".gla", psModelFileName)`.
        if model_file_name == DEFAULT_GLA_NAME {
            // Fake it like it was found on disk; never touches the host FS.
            return Some((FAKE_GLA_FILE.to_vec(), false));
        }

        match host.fs_read_file(&model_name) {
            Some(bytes) => {
                host.print(&format!(
                    "RE_RegisterModels_GetDiskFile(): Disk-loading \"{model_file_name}\"\n"
                ));
                Some((bytes, false))
            }
            None => None,
        }
    }

    /// Raven `RE_RegisterServerModels_Malloc` — the server-side cache
    /// ingest. On a fresh entry (`self.cached[key].disk_image.is_none()`):
    /// morphs `disk_buffer` into a fresh 16-byte-aligned [`AlignedBytes`]
    /// (`Some` arm — the `TRM-D4`(a) one-time ingest copy replacing Raven's
    /// zero-copy `Z_MorphMallocTag`) or zero-fills one of `size` bytes with
    /// no source (`None` arm — the "limb hierarchy creation" case), records
    /// `alloc_size`, stamps `pak_file_checksum` via `host.fs_file_is_in_pak`
    /// on `Some` (`TRM-D5`/ruling 59a), and returns `already_found = false`.
    /// On a repeat entry: the client shader-poke replay is commented out
    /// ("No. Bad.", `:293-316`, client-dead per `TRM-D3`/ruling 54), so this
    /// arm only sets `already_found = true`. Always stamps
    /// `last_level_used_on = self.media_get_level()`. Returns the entry's
    /// [`AlignedBytes`] base pointer — `server_load.rs` casts it to
    /// `*mut mdxaHeader_t`/`*mut mdxmHeader_t` and stores it into
    /// `model.mdxa`/`model.mdxm` directly, matching Raven's
    /// `mod->mdxa = (mdxaHeader_t*) RE_RegisterServerModels_Malloc(...)`
    /// (`unsafe`-confined at that cast site, §D11; `tag` distinguishes
    /// `TAG_MODEL_GLA`/`TAG_MODEL_GLM` per call site even though no Zone
    /// seam is reproduced, `TRM-D3`/ruling 54 consequence).
    ///
    /// Source: `oracle/codemp/renderer/tr_model.cpp:253-322`
    pub(crate) fn re_register_server_models_malloc(
        &mut self,
        host: &mut impl EngineHost,
        size: i32,
        disk_buffer: Option<&[u8]>,
        model_file_name: &str,
        tag: memtag_t,
    ) -> (*mut u8, bool) {
        // No Zone-allocator seam is reproduced (TRM-D3/ruling 54 consequence);
        // `tag` only ever distinguished the `Z_MorphMallocTag`/`Z_Malloc` pool.
        let _ = tag;

        let model_name = model_file_name.to_lowercase();
        let level = self.media_get_level();
        // (*CachedModels)[sModelName] — insert-if-missing, same as `GetDiskFile`.
        let entry = self.cached.entry(model_name.clone()).or_default();

        let already_found = if entry.disk_image.is_none() {
            // Fresh entry: morph the just-loaded buffer (TRM-D4(a) ingest copy)
            // or `Z_Malloc` a zero-filled block for the "limb hierarchy
            // creation" NULL case.
            let aligned = match disk_buffer {
                Some(bytes) => AlignedBytes::copy_from(bytes),
                None => AlignedBytes::zeroed(size as usize),
            };
            entry.disk_image = Some(Arc::new(ModelBlock::new(aligned)));
            entry.alloc_size = size;

            if let Some(checksum) = host.fs_file_is_in_pak(&model_name) {
                entry.pak_file_checksum = checksum;
            }

            false
        } else {
            // Repeat entry: the client shader-poke replay is commented out
            // server-side ("No. Bad.", tr_model.cpp:293-316; TRM-D3/ruling 54).
            true
        };

        entry.last_level_used_on = level;

        let block = entry
            .disk_image
            .as_mut()
            .expect("disk_image is always Some by this point");
        let ptr = if already_found {
            // A repeat entry may already be published, so the Arc is not unique here.
            // The three loaders return before any write on this arm, so a read-only base is enough.
            block.base_ptr() as *mut u8
        } else {
            // The fresh arm just built this Arc, so the endian swaps behind the returned pointer land in a block
            // nobody else holds.
            Arc::get_mut(block)
                .expect("a just-created block Arc is unique")
                .base_ptr_mut()
        };

        (ptr, already_found)
    }

    /// Build the DEC-35 parse-once `MdxmParsed` sidecar over the named entry's
    /// swap-completed `.glm` block and store it beside the block. Called once by
    /// `server_load_mdxm` on the fresh-load path (never on a cache hit, where
    /// the sidecar already exists).
    pub(crate) fn store_parsed_mdxm(&mut self, model_file_name: &str) {
        let key = model_file_name.to_lowercase();
        if let Some(entry) = self.cached.get_mut(&key) {
            if let Some(block) = entry.disk_image.as_mut() {
                // SAFETY: DEC-35 — the block is the live, endian-swap-completed
                // `.glm` block (self-sized by `ofsEnd`); parsing is a pure read.
                let view = unsafe { MdxmView::from_block(block.base_ptr() as *const c_void) };
                let parsed = MdxmParsed::parse(view);
                Arc::get_mut(block)
                    .expect("the fresh-load path holds the only Arc, and the registration-completion mark is what first shares a block")
                    .set_parsed_mdxm(parsed);
            }
        }
    }

    /// Build the DEC-35 parse-once `MdxaParsed` sidecar over the named entry's
    /// swap-completed `.gla` block. See [`Self::store_parsed_mdxm`].
    pub(crate) fn store_parsed_mdxa(&mut self, model_file_name: &str) {
        let key = model_file_name.to_lowercase();
        if let Some(entry) = self.cached.get_mut(&key) {
            if let Some(block) = entry.disk_image.as_mut() {
                // SAFETY: DEC-35 — the block is the live, endian-swap-completed
                // `.gla` block (self-sized by `ofsEnd`); parsing is a pure read.
                let view = unsafe { MdxaView::from_block(block.base_ptr() as *const c_void) };
                let parsed = MdxaParsed::parse(view);
                Arc::get_mut(block)
                    .expect("the fresh-load path holds the only Arc, and the registration-completion mark is what first shares a block")
                    .set_parsed_mdxa(parsed);
            }
        }
    }

    /// The DEC-35 `(block, parsed)` pair backing `EngineHost::model_mdxm` —
    /// Raven `R_GetModelByHandle(h)->mdxm`. Resolves the model's `.glm` block
    /// pointer and its parse-once sidecar (looked up by the model's cache-key
    /// name); both null when the loader pointer is NULL.
    pub(crate) fn model_mdxm_ptrs(&self, handle: qhandle_t) -> (*mut c_void, *const c_void) {
        let m = self.get_model(handle);
        let block = m.mdxm as *mut c_void;
        if block.is_null() {
            return (core::ptr::null_mut(), core::ptr::null());
        }
        let key = read_qpath(&m.name).to_lowercase();
        let parsed = self
            .cached
            .get(&key)
            .and_then(|e| e.disk_image.as_ref())
            .and_then(|b| b.parsed_mdxm())
            .map(|p| p as *const MdxmParsed as *const c_void)
            .unwrap_or(core::ptr::null());
        (block, parsed)
    }

    /// The DEC-35 `(block, parsed)` pair backing `EngineHost::model_mdxa` —
    /// Raven `R_GetModelByHandle(h)->mdxa`, with the same `animIndex`
    /// resolution the raw pointer path used (a GLM handle resolves its `.gla`
    /// through `mdxm->animIndex`). Both null when the resolved loader pointer is
    /// NULL.
    pub(crate) fn model_mdxa_ptrs(&self, handle: qhandle_t) -> (*mut c_void, *const c_void) {
        let m = self.get_model(handle);
        let (block, owner_name) = if !m.mdxa.is_null() {
            (m.mdxa as *mut c_void, read_qpath(&m.name))
        } else if m.mdxm.is_null() {
            return (core::ptr::null_mut(), core::ptr::null());
        } else {
            // SAFETY: `mdxm` is the loader's live parsed block for this handle.
            let anim_index = unsafe { (*m.mdxm).animIndex };
            let am = self.get_model(anim_index);
            (am.mdxa as *mut c_void, read_qpath(&am.name))
        };
        if block.is_null() {
            return (core::ptr::null_mut(), core::ptr::null());
        }
        let key = owner_name.to_lowercase();
        let parsed = self
            .cached
            .get(&key)
            .and_then(|e| e.disk_image.as_ref())
            .and_then(|b| b.parsed_mdxa())
            .map(|p| p as *const MdxaParsed as *const c_void)
            .unwrap_or(core::ptr::null());
        (block, parsed)
    }

    /// Raven `RE_RegisterModels_StoreShaderRequest` — pushes a
    /// (name-offset, poke-offset) pair onto the named entry's
    /// `shader_register_data`. Raven computes the two offsets via pointer
    /// subtraction against `pModelDiskImage`; the port takes them
    /// pre-computed as byte offsets (`server_load.rs` derives them from its
    /// own `AlignedBytes`-relative field positions — no raw pointer diffing
    /// once the buffer is owned data, not aliased `char*`/`int*`). Raven
    /// `assert(0)`s (fatal-bug class, fork 1) if the entry's
    /// `pModelDiskImage` is `NULL` — "should never happen, means that we're
    /// being called on a model that wasn't loaded".
    ///
    /// Source: `oracle/codemp/renderer/tr_model.cpp:70-92`
    pub(crate) fn re_register_models_store_shader_request(
        &mut self,
        model_file_name: &str,
        name_offset: i32,
        poke_offset: i32,
    ) {
        let model_name = model_file_name.to_lowercase();
        let entry = self.cached.entry(model_name).or_default();

        assert!(
            entry.disk_image.is_some(),
            "RE_RegisterModels_StoreShaderRequest: \"{model_file_name}\" has no disk image — \
             called on a model that wasn't loaded (fork-1 fatal-bug class, tr_model.cpp:83)"
        );

        entry.shader_register_data.push((name_offset, poke_offset));
    }

    /// The named entry's recorded shader-registration requests, each resolved
    /// to `(shader name, poke offset)` — the read side of
    /// [`Self::re_register_models_store_shader_request`], consumed by
    /// `frontend.rs`'s [`re_register_models_malloc`] repeat-registration
    /// replay, where Raven indexes `pModelDiskImage` by the stored name
    /// offset directly (`char *psShaderName = &((char*)pModelDiskImage)
    /// [iShaderNameOffset]`). Kept here rather than handing the disk image
    /// out, so the block stays private to this file. An unknown entry, or one
    /// with no disk image, has no requests.
    ///
    /// [`re_register_models_malloc`]: super::frontend::re_register_models_malloc
    ///
    /// Source: `oracle/codemp/renderer/tr_model.cpp:224-231`
    pub(crate) fn shader_register_requests(&self, model_file_name: &str) -> Vec<(String, i32)> {
        let key = model_file_name.to_lowercase();
        let Some(entry) = self.cached.get(&key) else {
            return Vec::new();
        };
        let Some(disk_image) = &entry.disk_image else {
            return Vec::new();
        };

        entry
            .shader_register_data
            .iter()
            .map(|&(name_offset, poke_offset)| {
                // SAFETY: `disk_image` owns `disk_image.len()` initialized
                // bytes for its whole life (`TRM-D4`), and `name_offset` is a
                // byte offset into that same block, recorded off it by
                // `re_register_models_store_shader_request`; `read_qpath`
                // stops at the first NUL inside the block-tail slice.
                let name = unsafe {
                    let base = disk_image.base_ptr().add(name_offset as usize) as *const c_char;
                    read_qpath(core::slice::from_raw_parts(
                        base,
                        disk_image.len() - name_offset as usize,
                    ))
                };
                (name, poke_offset)
            })
            .collect()
    }

    /// Poke a resolved shader index into the named entry's disk image at
    /// `poke_offset` — Raven's `*piShaderPokePtr = ...`, the write side of the
    /// replay above. No-op for an unknown entry or one with no disk image.
    ///
    /// DEC-65 ruling B makes this write copy-on-write. `Arc::make_mut` clones the block when a published frame
    /// still holds it, so the write lands in a block only this thread can see, and the clone gets a higher
    /// generation. The caller must re-read the entry's base pointer afterwards, because the entry can now name a
    /// different allocation ([`Self::block_base_ptr`]).
    ///
    /// Source: `oracle/codemp/renderer/tr_model.cpp:231,235-240`
    pub(crate) fn poke_shader_index(
        &mut self,
        model_file_name: &str,
        poke_offset: i32,
        value: i32,
    ) {
        let key = model_file_name.to_lowercase();
        let Some(entry) = self.cached.get_mut(&key) else {
            return;
        };
        let Some(disk_image) = &mut entry.disk_image else {
            return;
        };
        let block = Arc::make_mut(disk_image);

        // SAFETY: as [`Self::shader_register_requests`] — `poke_offset` is a
        // byte offset into this same live block, recorded off it by the store
        // call; written unaligned because the offset is a file-layout field
        // position, not a Rust-typed one. `Arc::make_mut` above made the block
        // unique, so no other holder sees this write.
        unsafe {
            let slot = block.base_ptr_mut().add(poke_offset as usize) as *mut i32;
            slot.write_unaligned(value);
        }

        block.bump_generation();
    }

    /// The named entry's current block base, `None` for an unknown entry or one with no disk image.
    /// [`super::frontend::re_register_models_malloc`] re-reads this after the poke replay, because a
    /// copy-on-write poke can leave the entry naming a different allocation than the one the caller started with.
    pub(crate) fn block_base_ptr(&self, model_file_name: &str) -> Option<*mut u8> {
        let key = model_file_name.to_lowercase();
        self.cached
            .get(&key)
            .and_then(|entry| entry.disk_image.as_ref())
            .map(|block| block.base_ptr() as *mut u8)
    }

    /// Raven `RE_RegisterModels_LevelLoadEnd` — the live eviction path
    /// (`z_memman_pc.cpp:226`'s `Z_Malloc`-fail recovery calls it with
    /// `delete_all_unused = qtrue`). Guarded by `self.inside_register_model`
    /// (Raven's `gbInsideRegisterModel`, blocking re-entrant eviction during
    /// a load, `:345-348`). Walks `self.cached` in sorted-key order,
    /// evicting entries whose `last_level_used_on` is stale — either "not
    /// used this exact level" (`delete_all_unused`) or "used on an older
    /// level" (`!delete_all_unused`, gated by `self.get_model_data_alloc_size()
    /// > r_modelpoolmegs * 1024*1024` via `host.cvar_integer`, `TRM-D2`) —
    /// until the gate clears. Returns `true` iff at least one model was
    /// freed (the `z_malloc`-fail recovery "try again" signal).
    ///
    /// Source: `oracle/codemp/renderer/tr_model.cpp:337-409`
    pub fn models_level_load_end(
        &mut self,
        host: &mut impl EngineHost,
        delete_all_unused: bool,
    ) -> bool {
        host.print("RE_RegisterModels_LevelLoadEnd():\n");

        let mut at_least_one_freed = false;

        if self.inside_register_model {
            host.print("(Inside RE_RegisterModel (z_malloc recovery?), exiting...\n");
        } else {
            let current_level = self.media_get_level();
            // §19: Raven computes `r_modelpoolmegs->integer * 1024 * 1024` in 32-bit
            // `int`; a large cvar (e.g. 9999) overflows — signed-overflow UB that
            // x86 wraps. `wrapping_mul` reproduces that defined-in-practice wrap
            // (Rust debug would otherwise panic).
            let max_model_bytes = host
                .cvar_integer("r_modelpoolmegs")
                .wrapping_mul(1024)
                .wrapping_mul(1024);
            let mut loaded_model_bytes = self.get_model_data_alloc_size();

            // Sorted-key snapshot (BTreeMap order == std::map order); walked
            // left to right exactly once, mirroring the C++ for-loop's
            // erase-returns-next-iterator/no-separate-increment idiom (`:355`).
            let mut keys: Vec<String> = self.cached.keys().cloned().collect();
            let mut idx = 0usize;

            while idx < keys.len() && (delete_all_unused || loaded_model_bytes > max_model_bytes) {
                let key = keys[idx].clone();
                let delete_this = self
                    .cached
                    .get(&key)
                    .map(|model| {
                        if delete_all_unused {
                            model.last_level_used_on != current_level
                        } else {
                            model.last_level_used_on < current_level
                        }
                    })
                    .unwrap_or(false);

                if delete_this {
                    host.print(&format!("Dumping \"{key}\""));
                    if let Some(model) = self.cached.remove(&key) {
                        if model.disk_image.is_some() {
                            at_least_one_freed = true;
                        }
                    }
                    loaded_model_bytes = self.get_model_data_alloc_size();
                    // Erase advanced the iterator to the next entry; `idx`
                    // already indexes it once the consumed key is dropped.
                    keys.remove(idx);
                } else {
                    idx += 1;
                }
            }
        }

        host.print("RE_RegisterModels_LevelLoadEnd(): Ok\n");

        at_least_one_freed
    }

    /// Raven `RE_RegisterModels_DumpNonPure` — scans `self.cached` in
    /// sorted-key order and evicts any entry whose current PAK membership
    /// (`host.fs_file_is_in_pak`, `TRM-D5`/ruling 59a) no longer matches its
    /// stamped `pak_file_checksum` — `None` (not-in-a-pure-pak) or a
    /// checksum mismatch both dump; [`DEFAULT_GLA_NAME`] is never dumped
    /// ("that's program internal anyway"). Called only from
    /// [`RenderModels::media_level_load_begin`]'s `sv_pure` arm.
    ///
    /// Source: `oracle/codemp/renderer/tr_model.cpp:418-465`
    fn re_register_models_dump_non_pure(&mut self, host: &mut impl EngineHost) {
        host.print("RE_RegisterModels_DumpNonPure():\n");

        let keys: Vec<String> = self.cached.keys().cloned().collect();
        for key in keys {
            // `Some(checksum)` = the `==1` in-pure-pak path; `None` = every
            // `-1` path (disk-only, not-found, illegal path, non-pure pak).
            let dump = match host.fs_file_is_in_pak(&key) {
                None => true,
                Some(checksum) => self
                    .cached
                    .get(&key)
                    .map(|entry| checksum != entry.pak_file_checksum)
                    .unwrap_or(false),
            };

            // stricmp(...) != 0, i.e. NOT (case-insensitively) the default
            // skeleton name — "that's program internal anyway".
            if dump && !key.eq_ignore_ascii_case(DEFAULT_GLA_NAME) {
                host.print(&format!("Dumping none pure model \"{key}\""));
                self.cached.remove(&key);
            }
        }

        host.print("RE_RegisterModels_DumpNonPure(): Ok\n");
    }

    /// Raven `RE_RegisterModels_Info_f` — prints each cached entry's index,
    /// name, and `alloc_size` (`host.print`) in `self.cached`'s sorted-key
    /// order, then the running total in bytes and MB.
    ///
    /// This method's pinned pub signature lives in the doc's `## Seam
    /// definition`, but no `## Files roster` entry names its home file —
    /// grouped here because it walks `self.cached` in the same sorted order
    /// as `re_register_models_dump_non_pure`/`re_register_models_delete_all`
    /// (a doc gap flagged, not a signature mismatch).
    ///
    /// Source: `oracle/codemp/renderer/tr_model.cpp:467-491`
    pub fn models_info_f(&self, host: &mut impl EngineHost) {
        let total_models = self.cached.len();
        let mut total_bytes = 0i32;

        for (index, (name, entry)) in self.cached.iter().enumerate() {
            host.print(&format!(
                "{index}/{total_models}: \"{name}\" ({} bytes)",
                entry.alloc_size
            ));
            total_bytes += entry.alloc_size;
        }

        host.print(&format!(
            "{total_bytes} bytes total ({:.2}MB)\n",
            total_bytes as f32 / 1024.0 / 1024.0
        ));
    }

    /// Raven `RE_RegisterModels_DeleteAll` — unconditionally frees and
    /// erases every `self.cached` entry (`Z_Free` -> `AlignedBytes`'s
    /// `Drop`). Host-free (Raven: "don't use ri.xxx functions since the
    /// renderer may not be running here"). Called by
    /// [`RenderModels::media_level_load_begin`]'s `bDeleteModels` arm and by
    /// [`RenderModels::model_free`](super::render_models::RenderModels::model_free)
    /// (`render_models.rs`).
    ///
    /// Source: `oracle/codemp/renderer/tr_model.cpp:496-516`
    pub(crate) fn re_register_models_delete_all(&mut self) {
        // Dropping each entry drops its `Option<AlignedBytes>`, which frees
        // the block (`AlignedBytes::drop`, mirroring `Z_Free`).
        self.cached.clear();
    }

    /// Raven `GetModelDataAllocSize` — `Z_MemSize(TAG_MODEL_MD3) +
    /// Z_MemSize(TAG_MODEL_GLM) + Z_MemSize(TAG_MODEL_GLA)`. The port
    /// derives this as a **local sum** `Σ cached[*].alloc_size` over
    /// `self.cached` instead of a Zone-allocator query — byte-exact on the
    /// dedicated build because every non-server `TAG_MODEL_*` producer is
    /// §20-dropped/frozen-dead (`TRM-D3`/ruling 54 consequence; `##
    /// Raven ground truth`). No `EngineHost` seam needed.
    ///
    /// Source: `oracle/codemp/renderer/tr_model.cpp:326-331`
    fn get_model_data_alloc_size(&self) -> i32 {
        self.cached.values().map(|entry| entry.alloc_size).sum()
    }

    /// Raven `RE_RegisterMedia_LevelLoadBegin` — on `force ==
    /// eForceReload_MODELS`/`eForceReload_ALL`, calls
    /// `re_register_models_delete_all`; else, iff `host.cvar_integer("sv_pure")`
    /// is nonzero, calls `re_register_models_dump_non_pure` (`TRM-D2`). Zeros
    /// `self.num_bsp_models` (the `#ifndef DEDICATED`
    /// `R_Images_DeleteLightMaps` tail is §C10-folded — dedicated-dead, no
    /// model state). Bumps `self.current_level` only when `map_name` differs
    /// from `self.prev_map_name` (`Q_stricmp`), then stores `map_name` as the
    /// new `prev_map_name`.
    ///
    /// Source: `oracle/codemp/renderer/tr_model.cpp:522-566`
    pub fn media_level_load_begin(
        &mut self,
        host: &mut impl EngineHost,
        map_name: &str,
        force: ForceReload_e,
    ) {
        let delete_models = matches!(
            force,
            ForceReload_e::eForceReload_MODELS | ForceReload_e::eForceReload_ALL
        );

        if delete_models {
            self.re_register_models_delete_all();
        } else if host.cvar_integer("sv_pure") != 0 {
            self.re_register_models_dump_non_pure(host);
        }

        // The `#ifndef DEDICATED` `R_Images_DeleteLightMaps()` tail (`:543-551`)
        // is §C10-folded — dedicated-dead, no model state.
        self.num_bsp_models = 0;

        // Q_stricmp != 0, i.e. the map name actually changed.
        if !self.prev_map_name.eq_ignore_ascii_case(map_name) {
            self.prev_map_name = map_name.to_string();
            self.current_level += 1;
        }
    }

    /// Raven `RE_RegisterMedia_GetLevel` — returns `self.current_level`
    /// (Raven's `giRegisterMedia_CurrentLevel`).
    ///
    /// Source: `oracle/codemp/renderer/tr_model.cpp:568-571`
    pub fn media_get_level(&self) -> i32 {
        self.current_level
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mp_host_interface::mock::MockHost;

    /// Build a `CachedEndianedModelBinary` cache entry directly (bypassing the
    /// registration path) for eviction/dump tests that only care about the
    /// bookkeeping fields.
    fn model(
        bytes: &[u8],
        last_level_used_on: i32,
        pak_file_checksum: i32,
    ) -> CachedEndianedModelBinary {
        CachedEndianedModelBinary {
            disk_image: Some(Arc::new(ModelBlock::new(AlignedBytes::copy_from(bytes)))),
            alloc_size: bytes.len() as i32,
            shader_register_data: Vec::new(),
            last_level_used_on,
            pak_file_checksum,
        }
    }

    #[test]
    fn get_disk_file_intercepts_default_gla_without_touching_fs() {
        let mut rm = RenderModels::default();
        let mut host = MockHost::new();

        let (bytes, already_cached) = rm
            .re_register_models_get_disk_file(&mut host, DEFAULT_GLA_NAME)
            .expect("the fake blob is always available");

        assert_eq!(bytes, FAKE_GLA_FILE.to_vec());
        assert!(!already_cached);
        assert_eq!(host.fs_reads, 0, "the fake blob must never hit the FS");
    }

    #[test]
    fn get_disk_file_reads_through_host_on_a_miss_then_serves_the_cache() {
        let mut rm = RenderModels::default();
        let mut host = MockHost::new().with_file("models/foo.glm", b"hello".to_vec());

        let (bytes, already_cached) = rm
            .re_register_models_get_disk_file(&mut host, "models/foo.glm")
            .expect("fixture present");
        assert_eq!(bytes, b"hello");
        assert!(!already_cached);
        assert_eq!(host.fs_reads, 1);

        // Morph the disk buffer into the cache, as `RE_RegisterServerModel` would.
        let (_ptr, found) = rm.re_register_server_models_malloc(
            &mut host,
            bytes.len() as i32,
            Some(&bytes),
            "models/foo.glm",
            memtag_t::TAG_MODEL_GLM,
        );
        assert!(!found);

        // A different-case name resolves to the same lowercased cache key.
        let (cached_bytes, already_cached) = rm
            .re_register_models_get_disk_file(&mut host, "MODELS/FOO.GLM")
            .expect("now cached");
        assert_eq!(cached_bytes, b"hello");
        assert!(already_cached);
        assert_eq!(host.fs_reads, 1, "a cache hit must not touch the FS again");
    }

    #[test]
    fn malloc_stamps_pak_checksum_only_on_a_fresh_entry() {
        let mut rm = RenderModels::default();
        let mut host = MockHost::new();
        host.pak_files.insert("models/foo.glm".to_string(), 1234);

        let (_ptr, found) = rm.re_register_server_models_malloc(
            &mut host,
            4,
            Some(&[1, 2, 3, 4]),
            "models/foo.glm",
            memtag_t::TAG_MODEL_GLM,
        );
        assert!(!found);
        assert_eq!(rm.cached["models/foo.glm"].pak_file_checksum, 1234);

        // A repeat registration reports `already_found = true` and must NOT
        // re-touch the checksum (the fresh-entry-only stamp, TRM-D5/ruling 59a).
        host.pak_files.insert("models/foo.glm".to_string(), 9999);
        let (_ptr, found) = rm.re_register_server_models_malloc(
            &mut host,
            4,
            Some(&[9, 9, 9, 9]),
            "models/foo.glm",
            memtag_t::TAG_MODEL_GLM,
        );
        assert!(found);
        assert_eq!(rm.cached["models/foo.glm"].pak_file_checksum, 1234);
    }

    #[test]
    fn malloc_zero_fills_the_limb_hierarchy_null_path() {
        let mut rm = RenderModels::default();
        let mut host = MockHost::new();

        let (ptr, found) = rm.re_register_server_models_malloc(
            &mut host,
            8,
            None,
            "internal/limb",
            memtag_t::TAG_MODEL_GLA,
        );
        assert!(!found);
        assert!(!ptr.is_null());
        assert_eq!(
            rm.cached["internal/limb"]
                .disk_image
                .as_ref()
                .unwrap()
                .len(),
            8
        );
    }

    #[test]
    #[should_panic(expected = "has no disk image")]
    fn store_shader_request_panics_on_a_model_that_was_never_loaded() {
        let mut rm = RenderModels::default();
        rm.re_register_models_store_shader_request("never/loaded.glm", 0, 4);
    }

    #[test]
    fn store_shader_request_records_the_offset_pair() {
        let mut rm = RenderModels::default();
        let mut host = MockHost::new();
        rm.re_register_server_models_malloc(
            &mut host,
            4,
            Some(&[0u8; 4]),
            "models/foo.glm",
            memtag_t::TAG_MODEL_GLM,
        );

        rm.re_register_models_store_shader_request("MODELS/FOO.GLM", 10, 20);
        assert_eq!(
            rm.cached["models/foo.glm"].shader_register_data,
            vec![(10, 20)]
        );
    }

    #[test]
    fn poke_shader_index_copies_a_shared_block_instead_of_writing_it() {
        let mut rm = RenderModels::default();
        let mut host = MockHost::new();
        rm.re_register_server_models_malloc(
            &mut host,
            8,
            Some(&[0u8; 8]),
            "models/foo.md3",
            memtag_t::TAG_MODEL_MD3,
        );
        // A published frame would hold exactly this second Arc.
        let held = Arc::clone(rm.cached["models/foo.md3"].disk_image.as_ref().unwrap());

        rm.poke_shader_index("models/foo.md3", 4, 7);

        // The held block keeps the bytes and the generation it was published with.
        assert_eq!(&held.bytes()[4..8], &[0u8, 0, 0, 0]);
        assert_eq!(held.generation(), 0);

        // The entry now names a different block, carrying the poked value and a higher generation.
        let poked = rm.cached["models/foo.md3"].disk_image.as_ref().unwrap();
        assert_eq!(&poked.bytes()[4..8], &7i32.to_le_bytes());
        assert_eq!(poked.generation(), 1);
        assert!(!Arc::ptr_eq(&held, poked));
    }

    #[test]
    fn poke_shader_index_writes_an_unshared_block_in_place() {
        let mut rm = RenderModels::default();
        let mut host = MockHost::new();
        rm.re_register_server_models_malloc(
            &mut host,
            8,
            Some(&[0u8; 8]),
            "models/foo.md3",
            memtag_t::TAG_MODEL_MD3,
        );
        let base = rm.block_base_ptr("models/foo.md3").unwrap();

        rm.poke_shader_index("models/foo.md3", 4, 7);

        // Nobody else held the block, so `Arc::make_mut` wrote it where it was.
        assert_eq!(rm.block_base_ptr("models/foo.md3"), Some(base));
        let entry = rm.cached["models/foo.md3"].disk_image.as_ref().unwrap();
        assert_eq!(&entry.bytes()[4..8], &7i32.to_le_bytes());
        assert_eq!(entry.generation(), 1);
    }

    #[test]
    fn level_load_end_delete_all_unused_evicts_only_off_current_level_entries() {
        let mut rm = RenderModels::default();
        let mut host = MockHost::new();
        host.set_cvar("r_modelpoolmegs", "9999"); // never trips the byte-threshold arm
        rm.current_level = 5;

        rm.cached.insert("alpha".into(), model(b"a", 5, -1));
        rm.cached.insert("beta".into(), model(b"b", 3, -1));
        rm.cached.insert("gamma".into(), model(b"c", 5, -1));

        let freed = rm.models_level_load_end(&mut host, true);

        assert!(freed);
        assert_eq!(
            rm.cached.keys().cloned().collect::<Vec<_>>(),
            vec!["alpha".to_string(), "gamma".to_string()]
        );
    }

    #[test]
    fn level_load_end_pool_megs_gate_stops_once_under_budget() {
        let mut rm = RenderModels::default();
        let mut host = MockHost::new();
        // 1 model-pool megabyte: budget is exactly 1024*1024 bytes.
        host.set_cvar("r_modelpoolmegs", "1");
        rm.current_level = 5;

        // Two stale (older-level) entries big enough to exceed the budget
        // until at least one is evicted; "gamma" is current and kept.
        rm.cached
            .insert("alpha".into(), model(&vec![0u8; 900_000], 1, -1));
        rm.cached
            .insert("beta".into(), model(&vec![0u8; 900_000], 2, -1));
        rm.cached.insert("gamma".into(), model(b"kept", 5, -1));

        let freed = rm.models_level_load_end(&mut host, false);

        assert!(freed);
        // Sorted-key sweep frees only "alpha" before dropping under budget.
        assert_eq!(
            rm.cached.keys().cloned().collect::<Vec<_>>(),
            vec!["beta".to_string(), "gamma".to_string()]
        );
    }

    #[test]
    fn level_load_end_is_a_no_op_while_inside_register_model() {
        let mut rm = RenderModels::default();
        let mut host = MockHost::new();
        host.set_cvar("r_modelpoolmegs", "0");
        rm.inside_register_model = true;
        rm.cached.insert("alpha".into(), model(b"a", -1, -1));

        let freed = rm.models_level_load_end(&mut host, true);

        assert!(!freed);
        assert_eq!(rm.cached.len(), 1);
    }

    #[test]
    fn dump_non_pure_never_dumps_the_default_gla_even_on_mismatch() {
        let mut rm = RenderModels::default();
        let mut host = MockHost::new();
        rm.cached
            .insert(DEFAULT_GLA_NAME.to_string(), model(&[0u8; 4], 0, 42));
        // No `pak_files` fixture -> `fs_file_is_in_pak` returns `None`, which
        // would trip the dump gate for any other name.

        rm.re_register_models_dump_non_pure(&mut host);

        assert!(rm.cached.contains_key(DEFAULT_GLA_NAME));
    }

    #[test]
    fn dump_non_pure_dumps_on_checksum_mismatch_and_keeps_a_match() {
        let mut rm = RenderModels::default();
        let mut host = MockHost::new();
        rm.cached
            .insert("stale.glm".to_string(), model(&[0u8; 4], 0, 111));
        rm.cached
            .insert("fresh.glm".to_string(), model(&[0u8; 4], 0, 222));
        host.pak_files.insert("stale.glm".to_string(), 999); // mismatch -> dump
        host.pak_files.insert("fresh.glm".to_string(), 222); // match -> keep

        rm.re_register_models_dump_non_pure(&mut host);

        assert!(!rm.cached.contains_key("stale.glm"));
        assert!(rm.cached.contains_key("fresh.glm"));
    }

    #[test]
    fn media_level_load_begin_bumps_level_only_on_a_new_map_name() {
        let mut rm = RenderModels::default();
        let mut host = MockHost::new();

        rm.media_level_load_begin(&mut host, "hoth1", ForceReload_e::eForceReload_NOTHING);
        assert_eq!(rm.media_get_level(), 1);

        rm.media_level_load_begin(&mut host, "HOTH1", ForceReload_e::eForceReload_NOTHING);
        assert_eq!(
            rm.media_get_level(),
            1,
            "case-insensitive same-map must not bump"
        );

        rm.media_level_load_begin(&mut host, "yavin1", ForceReload_e::eForceReload_NOTHING);
        assert_eq!(rm.media_get_level(), 2);
    }

    #[test]
    fn media_level_load_begin_force_models_deletes_everything() {
        let mut rm = RenderModels::default();
        let mut host = MockHost::new();
        rm.cached.insert("alpha".into(), model(b"a", 0, -1));

        rm.media_level_load_begin(&mut host, "hoth1", ForceReload_e::eForceReload_MODELS);

        assert!(rm.cached.is_empty());
    }

    #[test]
    fn models_info_f_totals_alloc_size_over_the_cache() {
        let rm = {
            let mut rm = RenderModels::default();
            rm.cached.insert("a".into(), model(&[0u8; 10], 0, -1));
            rm.cached.insert("b".into(), model(&[0u8; 20], 0, -1));
            rm
        };
        let mut host = MockHost::new();

        rm.models_info_f(&mut host);

        assert!(host.prints.last().unwrap().starts_with("30 bytes total"));
    }
}
