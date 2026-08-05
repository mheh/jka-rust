//! `ModelBlocks` - the published model registry the frame package carries.
//!
//! DEC-65 ruling 1 (`docs/decisions.md:1526-1534`) publishes the registered blocks to the render thread.
//! `RenderModels` keeps the registry itself, and this is the read-only copy that rides inside `RenderAssets` beside
//! the world asset.
//! Every entry names its blocks by `Arc` and byte offset, so nothing here is a raw pointer.

use core::ffi::c_void;
use std::sync::Arc;

use mp_engine_qcommon::qfiles::md3_header_t::md3Header_t;
use mp_host_interface::mdx::mdxm::MdxmView;
use mp_qshared::shared::qhandle_t;

use crate::render_state::model_block::ModelBlock;
use crate::tr_local::modtype_t::modtype_t;

/// One registered model slot, published by block and offset rather than by pointer.
/// A multi-LOD MD3 spreads over up to three cache entries, because `RE_RegisterModel_Actual` builds a per-LOD file
/// name, so each family slot names the block that owns it.
///
/// Type definition source: idiomatic infra, no oracle cite (DEC-65 ruling 1 over
/// `oracle/codemp/renderer/tr_local.h:1117-1135`).
#[derive(Clone)]
pub struct PublishedModel {
    /// `model_t::type`.
    pub model_type: modtype_t,
    /// `model_t::numLods`.
    pub num_lods: i32,
    /// `model_t::md3[MD3_MAX_LODS]`, each loaded LOD paired with its owning block and its offset in it.
    pub md3: [Option<(Arc<ModelBlock>, usize)>; 3],
    /// `model_t::mdxm`.
    pub mdxm: Option<(Arc<ModelBlock>, usize)>,
    /// `model_t::mdxa`.
    pub mdxa: Option<(Arc<ModelBlock>, usize)>,
    /// `model_t::name`, the registered file name the bad-frame warning prints.
    /// The on-disk `md3Header_t::name` is a different string, so the warning reads this instead.
    pub name: String,
}

impl PublishedModel {
    /// The `md3Header_t` one loaded LOD publishes, `None` for an absent slot.
    pub fn md3_ptr(&self, lod: usize) -> Option<*const md3Header_t> {
        let (block, offset) = self.md3.get(lod)?.as_ref()?;
        // SAFETY: `mark_block` computed this offset by subtracting the block base from the finished `model_t` pointer.
        // The offset therefore lands inside the block, which is immutable while shared (`render_state/model_block.rs:127-141`).
        // The borrow on `self` keeps the entry's `Arc` alive for as long as the caller holds the pointer.
        Some(unsafe { block.base_ptr().add(*offset) } as *const md3Header_t)
    }

    /// The DEC-35 view over the published `.glm` block, `None` for a model with no mdxm block.
    pub fn mdxm_view(&self) -> Option<MdxmView<'_>> {
        let (block, offset) = self.mdxm.as_ref()?;
        // SAFETY: see [`Self::md3_ptr`].
        // The block is the endian-swap-completed `.glm` image, self-sized by its `ofsEnd` field, which is what `MdxmView::from_block` reads.
        Some(unsafe { MdxmView::from_block(block.base_ptr().add(*offset) as *const c_void) })
    }

    /// `true` when any family slot names this exact block.
    fn holds(&self, block: &Arc<ModelBlock>) -> bool {
        let names = |slot: &Option<(Arc<ModelBlock>, usize)>| {
            slot.as_ref()
                .is_some_and(|(held, _)| Arc::ptr_eq(held, block))
        };
        self.md3.iter().any(names) || names(&self.mdxm) || names(&self.mdxa)
    }
}

/// One entry per registered model slot, keyed by `qhandle_t`.
/// The sim thread rebuilds this at each registration completion and `RE_EndFrame` hands the whole registry over.
///
/// Type definition source: idiomatic infra, no oracle cite (DEC-65 ruling 1).
#[derive(Clone, Default)]
pub struct ModelBlocks {
    /// Indexed by slot number, `None` for a slot that names no block.
    entries: Vec<Option<PublishedModel>>,
}

impl ModelBlocks {
    /// The published entry for a model handle, `None` for an unregistered or evicted slot.
    pub fn get(&self, handle: qhandle_t) -> Option<&PublishedModel> {
        if handle < 0 {
            return None;
        }
        self.entries.get(handle as usize).and_then(|e| e.as_ref())
    }

    /// The slot count, which is the highest published handle plus one.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Publish one finished model slot, growing the slot vector to reach it.
    pub(crate) fn set(&mut self, handle: qhandle_t, entry: PublishedModel) {
        if handle < 0 {
            return;
        }
        let slot = handle as usize;
        if self.entries.len() <= slot {
            self.entries.resize_with(slot + 1, || None);
        }
        self.entries[slot] = Some(entry);
    }

    /// Drop every entry that names this block.
    /// Eviction calls this, or the registry clone would keep the evicted bytes resident and `r_modelpoolmegs`
    /// reclamation would stop freeing memory.
    pub(crate) fn remove_block(&mut self, block: &Arc<ModelBlock>) {
        for entry in self.entries.iter_mut() {
            if entry.as_ref().is_some_and(|e| e.holds(block)) {
                *entry = None;
            }
        }
    }

    /// Drop every entry, for the `model_init` and `hunk_clear` pool resets.
    pub(crate) fn clear(&mut self) {
        self.entries.clear();
    }
}

// The published registry crosses to the render thread inside `RenderAssets`, so both types must be `Send` and
// `Sync`. `ModelBlocks` holds no raw pointer of its own and inherits the claim from `ModelBlock`'s impls.
const fn assert_send_sync<T: Send + Sync>() {}
const _: () = assert_send_sync::<ModelBlock>();
const _: () = assert_send_sync::<ModelBlocks>();
