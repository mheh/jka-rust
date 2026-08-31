//! `BModelTable` — the brush-submodel facts the render-side walk reads out of
//! the model registry (W2-F8).

use mp_qshared::shared::qhandle_t;

use crate::tr_model::render_models::RenderModels;

/// One model handle's row: the two `model_t` scalars the brush-submodel path reads, plus the world that owns the
/// submodel.
///
/// Source: `oracle/codemp/renderer/tr_local.h:1117-1135` (`model_t`)
#[derive(Clone, Copy)]
pub struct BModelEntry {
    /// `model_t::bmodel` as an index into `WorldAsset::bmodels`, or `-1` for a
    /// handle that names no brush submodel. The pointer itself has no twin:
    /// `RenderModels::register_bmodel` records the index instead.
    pub bmodel_index: i32,
    /// The world that owns this submodel: `0` for the main world, `k` for `RenderAssets::bsp_models[k - 1]`.
    /// Raven's `bmodel_t::firstSurface` pointer carries this, so the oracle's row has no twin field.
    ///
    /// Source: `oracle/codemp/renderer/tr_bsp.cpp:1442-1450`
    pub world_index: i32,
    /// `model_t::bspInstance`, the RMG flag that turns on entity lighting for
    /// a sub-BSP instance. Raven's `qboolean`.
    pub bsp_instance: i32,
}

impl Default for BModelEntry {
    /// The row an unregistered handle resolves to, matching the zeroed
    /// `Hunk_Alloc` block `ModelPool::by_handle` returns for an out-of-range
    /// handle.
    ///
    /// `world_index` is `0` here and inert, because the row already fails the brush test on `bmodel_index = -1`.
    fn default() -> BModelEntry {
        BModelEntry {
            bmodel_index: -1,
            world_index: 0,
            bsp_instance: 0,
        }
    }
}

/// The model registry as the render-side walk sees it: one plain-integer row
/// per registered handle.
///
/// The registry itself cannot cross to the render thread.
/// `ModelPool` entries own the `mdxm`/`mdxa`/`md3` raw block pointers the DEC-35 mdx views hand out.
/// The pool is therefore deliberately not `Clone` and is not `Send`.
/// The brush-submodel path reads only the two scalars in [`BModelEntry`], so W2-F8 crosses those instead of the registry.
/// The table travels with the world generation on the frame package, because both are rebuilt by the same map load.
///
/// DEC-65 ruling 4 splits the entity walk's two model reads.
/// The brush test reads `bmodel_index` here, because a brush handle never enters the published registry.
/// `model_type` resolves from `RenderAssets::models`, which republishes at every `RE_EndFrame` drain.
pub struct BModelTable {
    /// Indexed by the bare `qhandle_t`, which is the pool slot (DEC-42.2).
    entries: Vec<BModelEntry>,
}

impl BModelTable {
    /// Reads every registered handle's row out of `models`.
    ///
    /// Call this whenever the loaded world changes: the inline submodels
    /// register during `RE_LoadWorldMap`, so the table is only valid for the
    /// world generation it was built beside.
    pub fn build(models: &RenderModels) -> BModelTable {
        let count = models.num_models().max(0) as usize;
        let mut entries = Vec::with_capacity(count);
        for slot in 0..count {
            let model = models.get_model(slot as qhandle_t);
            let (world_index, bmodel_index) = match models.bmodel_location(slot as qhandle_t) {
                Some((world, submodel)) => (world as i32, submodel as i32),
                None => (0, -1),
            };
            entries.push(BModelEntry {
                bmodel_index,
                world_index,
                bsp_instance: model.bspInstance,
            });
        }
        BModelTable { entries }
    }

    /// The table a render thread starts with, before any map load.
    pub fn empty() -> BModelTable {
        BModelTable {
            entries: Vec::new(),
        }
    }

    /// The row `handle` resolves to.
    ///
    /// Raven's `R_GetModelByHandle` hands an out-of-range handle the default model.
    /// `ModelPool::by_handle` reproduces that with slot 0's zeroed entry, and this returns the same all-zero row.
    /// A bad handle therefore fails the brush test and resolves `MOD_BAD` through the published registry, exactly as it did.
    ///
    /// Source: `oracle/codemp/renderer/tr_model.cpp:1665-1680`
    pub fn get(&self, handle: qhandle_t) -> BModelEntry {
        if handle < 1 {
            return BModelEntry::default();
        }
        match self.entries.get(handle as usize) {
            Some(entry) => *entry,
            None => BModelEntry::default(),
        }
    }
}

impl Default for BModelTable {
    fn default() -> BModelTable {
        BModelTable::empty()
    }
}
