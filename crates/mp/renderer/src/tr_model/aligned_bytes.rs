//! `AlignedBytes` — the model disk-image buffer type (`TRM-D4`/ruling 58,
//! superseding ruling 52's `Box<[u8]>` spelling).
//!
//! Not a Raven class — idiomatic infra forced by ruling 58 over the 1-byte
//! allocator alignment of an ordinary `Box<[u8]>`/`Vec<u8>` (and the
//! `EngineHost::fs_read_file` `Vec<u8>` those buffers originate from). The
//! model header leading fields are `i32` (4-byte-aligned), so reinterpreting
//! the cached bytes as `#[repr(C)]` `mdxaHeader_t`/`mdxmHeader_t` and swapping
//! multi-byte fields in place (the `LL()` swaps, `tr_model.cpp:734-739,857-863`)
//! would be UB over a 1-byte-aligned block. `AlignedBytes` owns a 16-byte-
//! aligned heap block (`alloc::alloc`/`alloc::alloc_zeroed` +
//! `Layout::from_size_align(len, 16)`, mirroring `Z_Malloc`'s alignment
//! guarantee), is heap-pinned/address-stable (the frozen ghoul2 seam derefs
//! raw pointers into it across frames — `CBoneCache` parent-seeding, skeleton
//! build, per-call ragdoll basepose, `tr_ghoul2.cpp:416-421,614-615`), and
//! in-place mutable for the endian swap. `Drop` deallocates the same `Layout`
//! it was allocated with (mirrors `Z_Free`).
//!
//! Consumed by `CachedEndianedModelBinary::disk_image` (`cached_model_binary.rs`,
//! as `Option<AlignedBytes>`; `None` mirrors `pModelDiskImage == NULL`),
//! `server_load.rs`'s `*mut mdxaHeader_t`/`*mut mdxmHeader_t` cast sites, and
//! the `model_mdxm`/`model_mdxa` `EngineHost` seam (`G2SV-D5`) — all of which
//! stay `unsafe`-confined at the point of cast (§D11) with a debug alignment
//! assert at each cast site; the mdx header types are never named here.
//!
//! Design: `docs/subsystems/tr-model.md` (FROZEN), `## Files roster` (this
//! file's entry), `TRM-D4`/ruling 58, `TRM-D3`(a) (the ruling-52 ownership
//! contract this buffer inherits: no re-parse, NULL-parity, address-stable,
//! opaque at the seam).

use core::alloc::Layout;
use core::ptr::{self, NonNull};
use std::alloc::{alloc_zeroed, dealloc, handle_alloc_error};

/// The model disk-image buffer — a 16-byte-aligned, heap-pinned, in-place-
/// mutable byte block. Owns `len` bytes at `ptr`; `Drop` frees them with the
/// same `Layout` they were allocated with.
///
/// Type definition source: idiomatic infra, no oracle cite (`TRM-D4`/ruling 58).
pub struct AlignedBytes {
    ptr: NonNull<u8>,
    len: usize,
}

impl AlignedBytes {
    /// The alignment `Z_Malloc` guarantees and the `*mut mdx*Header_t` casts
    /// require (`TRM-D4`/ruling 58).
    const ALIGN: usize = 16;

    /// The `Layout` this buffer was (or would be) allocated with — shared by
    /// the allocating ctors and `Drop` so they always agree.
    fn layout(len: usize) -> Layout {
        Layout::from_size_align(len, Self::ALIGN)
            .expect("AlignedBytes: length overflows a valid 16-byte-aligned layout")
    }

    /// Allocate `len` zero-filled bytes with no source buffer — backs the
    /// NULL "limb hierarchy creation" `Z_Malloc(iSize, eTag, qfalse)` path
    /// (`RE_RegisterServerModels_Malloc`'s `pvDiskBufferIfJustLoaded == NULL`
    /// arm, `tr_model.cpp:275-277`).
    ///
    /// Source: `oracle/codemp/renderer/tr_model.cpp:277`
    pub fn zeroed(len: usize) -> Self {
        if len == 0 {
            // A zero-size allocation request is not a valid `alloc` layout
            // (size must be non-zero); `NonNull::dangling` is the standard
            // stand-in, never dereferenced since `len == 0`.
            return Self {
                ptr: NonNull::dangling(),
                len: 0,
            };
        }

        let layout = Self::layout(len);
        // SAFETY: `layout` has a non-zero size (checked above) and a valid
        // (power-of-two) alignment (`Self::ALIGN`).
        let raw = unsafe { alloc_zeroed(layout) };
        let ptr = NonNull::new(raw).unwrap_or_else(|| handle_alloc_error(layout));
        Self { ptr, len }
    }

    /// Allocate `bytes.len()` bytes and copy `bytes` in — the `TRM-D4`(a)
    /// one-time ingest copy that replaces Raven's zero-copy
    /// `Z_MorphMallocTag` re-tag-in-place morph (which reuses the
    /// `FS_ReadFile` `Z_Malloc` block directly and cannot be reproduced over
    /// the 1-byte-aligned `EngineHost::fs_read_file` `Vec<u8>`).
    ///
    /// Source: `oracle/codemp/renderer/tr_model.cpp:271-273` (the Rust-only
    /// ingest-copy divergence site; `TRM-D4`(a)/§F19)
    pub fn copy_from(bytes: &[u8]) -> Self {
        let buf = Self::zeroed(bytes.len());
        if !bytes.is_empty() {
            // SAFETY: `buf` was just allocated by `Self::zeroed(bytes.len())`,
            // so `buf.ptr` is valid for `bytes.len()` writes and does not
            // overlap the distinct `bytes` source slice.
            unsafe {
                ptr::copy_nonoverlapping(bytes.as_ptr(), buf.ptr.as_ptr(), bytes.len());
            }
        }
        buf
    }

    /// The 16-byte-aligned base pointer, mutable — the `*mut mdx*Header_t`
    /// cast sites (`server_load.rs`) and the in-place `LL()` field swaps read
    /// and write through this. `unsafe`-confined at the seam (§D11); callers
    /// debug-assert alignment at the cast site.
    ///
    /// Source: `oracle/codemp/renderer/tr_model.cpp:717-718,840-841` (the
    /// `RE_RegisterServerModels_Malloc` return value those casts operate on)
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        debug_assert_eq!(
            self.ptr.as_ptr() as usize % Self::ALIGN,
            0,
            "AlignedBytes base pointer is not 16-byte aligned"
        );
        self.ptr.as_ptr()
    }

    /// Read-only view of the base pointer (see [`Self::as_mut_ptr`]).
    ///
    /// Source: `oracle/codemp/renderer/tr_model.cpp:717-718,840-841`
    pub fn as_ptr(&self) -> *const u8 {
        debug_assert_eq!(
            self.ptr.as_ptr() as usize % Self::ALIGN,
            0,
            "AlignedBytes base pointer is not 16-byte aligned"
        );
        self.ptr.as_ptr() as *const u8
    }

    /// The allocation length — Raven's `CachedEndianedModelBinary_s::iAllocSize`.
    ///
    /// Source: `oracle/codemp/renderer/tr_model.cpp:51,209,281`
    pub fn len(&self) -> usize {
        self.len
    }

    /// `true` iff [`Self::len`] is zero (clippy `len_without_is_empty`
    /// pairing; no distinct Raven state — `iAllocSize` is never queried for
    /// zero-ness in the oracle).
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl Drop for AlignedBytes {
    /// Deallocates the same `Layout` this buffer was allocated with — mirrors
    /// `Z_Free(pModelDiskImage)` (`RE_RegisterModels_LevelLoadEnd`/
    /// `_DumpNonPure`/`_DeleteAll` eviction sites).
    ///
    /// Source: `oracle/codemp/renderer/tr_model.cpp:384,445,508`
    fn drop(&mut self) {
        if self.len != 0 {
            let layout = Self::layout(self.len);
            // SAFETY: `self.ptr` was allocated by `alloc_zeroed` with this
            // exact `layout` (`Self::layout(self.len)`) in `Self::zeroed`, and
            // is never freed anywhere else.
            unsafe {
                dealloc(self.ptr.as_ptr(), layout);
            }
        }
    }
}

const _: () = assert!(AlignedBytes::ALIGN == 16);
