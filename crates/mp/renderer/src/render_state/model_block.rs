//! `ModelBlock` - the model byte block the sim thread loads and the render thread reads.
//!
//! DEC-65 ruling 1 (`docs/decisions.md:1526-1534`) puts the parsed md3/mdxm/mdxa bytes behind an `Arc`, so a
//! published frame can hold them while the sim thread keeps registering models.
//! The block owns the `AlignedBytes` allocation and the two DEC-35 parse-once sidecars that used to sit beside it
//! on the cache entry.
//!
//! This file is the second justified exception to this module's interior-safety law, after `handle.rs`.
//! The law bans raw pointers here, and `AlignedBytes` holds one, because the frozen ghoul2 seam derefs `model_t`
//! pointers into the block across frames (`tr_model/aligned_bytes.rs`).
//! DEC-65 ruling 1 homes the published block in this module anyway, and the `Send`/`Sync` argument below is what
//! pays for that.

use std::sync::Arc;

use mp_host_interface::mdx::mdxa::MdxaParsed;
use mp_host_interface::mdx::mdxm::MdxmParsed;

use crate::tr_model::aligned_bytes::AlignedBytes;

/// One published model's bytes plus its parse-once sidecars.
/// The block owns its allocation, and every byte write lands while the `Arc` is unique.
/// The shader poke goes through `Arc::make_mut`, so a frame the render thread already holds never sees a byte change.
///
/// Type definition source: idiomatic infra, no oracle cite (DEC-65 ruling 1 over
/// `oracle/codemp/renderer/tr_model.cpp:50`).
pub struct ModelBlock {
    /// `pModelDiskImage` - the 16-byte-aligned, address-stable disk image.
    ///
    /// Source: `oracle/codemp/renderer/tr_model.cpp:50`
    bytes: AlignedBytes,

    /// The DEC-35 `MdxmParsed` index for a `.glm` block, `None` for every other format.
    /// The `Arc` shares the index instead of re-parsing it when a copy-on-write poke clones the block.
    parsed_mdxm: Option<Arc<MdxmParsed>>,

    /// The DEC-35 `MdxaParsed` index for a `.gla` block. See [`Self::parsed_mdxm`].
    parsed_mdxa: Option<Arc<MdxaParsed>>,

    /// Bumped by each copy-on-write shader poke, so a reader can tell two versions of one block apart.
    generation: u32,
}

impl ModelBlock {
    /// Take ownership of a freshly ingested disk image.
    /// The sidecars arrive later through the setters, while the load path still holds the only `Arc`.
    pub(crate) fn new(bytes: AlignedBytes) -> Self {
        Self {
            bytes,
            parsed_mdxm: None,
            parsed_mdxa: None,
            generation: 0,
        }
    }

    /// The block bytes as a slice.
    pub fn bytes(&self) -> &[u8] {
        // SAFETY: `AlignedBytes` owns `len` initialized bytes at `as_ptr` for its whole life (`TRM-D4`).
        unsafe { core::slice::from_raw_parts(self.bytes.as_ptr(), self.bytes.len()) }
    }

    /// The 16-byte-aligned base pointer the `*const mdx*Header_t`/`*const md3Header_t` casts read through.
    pub fn base_ptr(&self) -> *const u8 {
        self.bytes.as_ptr()
    }

    /// The same base pointer, writable, for the load-time endian swaps.
    /// Reach it through `Arc::get_mut` or `Arc::make_mut`, or the write lands in a block another thread holds.
    pub(crate) fn base_ptr_mut(&mut self) -> *mut u8 {
        self.bytes.as_mut_ptr()
    }

    /// Store the DEC-35 `.glm` index built over these bytes.
    pub(crate) fn set_parsed_mdxm(&mut self, parsed: MdxmParsed) {
        self.parsed_mdxm = Some(Arc::new(parsed));
    }

    /// Store the DEC-35 `.gla` index built over these bytes.
    pub(crate) fn set_parsed_mdxa(&mut self, parsed: MdxaParsed) {
        self.parsed_mdxa = Some(Arc::new(parsed));
    }

    /// The allocation length - Raven's `CachedEndianedModelBinary_s::iAllocSize`.
    ///
    /// Source: `oracle/codemp/renderer/tr_model.cpp:51`
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    /// The copy-on-write version of this block.
    pub fn generation(&self) -> u32 {
        self.generation
    }

    /// The DEC-35 `.glm` index, `None` until the loader parses it and for every other format.
    pub fn parsed_mdxm(&self) -> Option<&MdxmParsed> {
        self.parsed_mdxm.as_deref()
    }

    /// The DEC-35 `.gla` index, `None` until the loader parses it and for every other format.
    pub fn parsed_mdxa(&self) -> Option<&MdxaParsed> {
        self.parsed_mdxa.as_deref()
    }

    /// Record one more copy-on-write version of this block.
    /// `Arc::make_mut` hands the shader poke a unique block, and this marks it as a later version than the one a
    /// held frame still reads.
    pub(crate) fn bump_generation(&mut self) {
        self.generation = self.generation.wrapping_add(1);
    }
}

impl Clone for ModelBlock {
    /// `Arc::make_mut` calls this when the shader poke finds the block shared.
    /// The bytes are deep-copied into a fresh allocation, and the sidecars are shared by refcount, because a parse
    /// is immutable and stores byte offsets rather than addresses.
    fn clone(&self) -> Self {
        Self {
            bytes: AlignedBytes::copy_from(self.bytes()),
            parsed_mdxm: self.parsed_mdxm.clone(),
            parsed_mdxa: self.parsed_mdxa.clone(),
            generation: self.generation,
        }
    }
}

// SAFETY: `AlignedBytes` is `!Send` and `!Sync` because it holds a `NonNull<u8>`, and these two impls claim the
// block is safe to share anyway. Four invariants carry the claim, and the pause trigger in the step-002 packet
// names the one event that would break the second one.
//
// 1. The block owns its allocation, and the type has no interior mutability.
// 2. Every byte write happens while the Arc is unique: the load-time endian swaps and the sidecar parse run
//    before the slot's registration-completion mark (Arc::get_mut), and the shader poke runs through
//    Arc::make_mut. The survey `docs/audits/2026-08-04-model-block-publication-survey.md` section E found the
//    poke to be the one post-load byte write in the tree.
// 3. The raw pointers model_t derives from base_ptr are sim-thread-only and read-only after the mark. The render
//    thread reads bytes and offsets only.
// 4. The allocation is freed exactly once, by the last Arc drop.
unsafe impl Send for ModelBlock {}
// SAFETY: see the four invariants above.
unsafe impl Sync for ModelBlock {}
