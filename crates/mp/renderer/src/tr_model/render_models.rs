//! `RenderModels` — the aggregate owner struct for the MP dedicated-server
//! model loader + cache (§F idiomatic reimplementation).
//!
//! Design: `docs/subsystems/tr-model.md` (FROZEN). This file is the shared
//! roster owner: it pins the struct's fields and its `Default`/construction
//! story so the sibling per-class files (`cached_model_binary.rs`,
//! `server_load.rs`) can transcribe their `impl RenderModels` method blocks
//! against these fields. The fields are `pub(crate)` (not the doc's illustrative
//! private spelling) because those methods live in sibling submodules of
//! `tr_model` — `pub(crate)` is the minimal visibility that keeps the split
//! `impl` blocks compiling while staying "internal to `mp_renderer`" (§F17;
//! `TRM-D3`/ruling 53).

use std::collections::{BTreeMap, HashMap};

use mp_engine_qcommon::qfiles::md3_limits::MD3_MAX_LODS;
use mp_host_interface::EngineHost;
use mp_qshared::shared::{qhandle_t, qtrue};

use crate::render_state::model_blocks::{ModelBlocks, PublishedModel};
use crate::tr_local::modtype_t::modtype_t;

use super::cached_model_binary::CachedEndianedModelBinary;
use super::model_pool::{ModelData, ModelPool};
use super::server_load::{read_qpath, write_qpath};
use super::server_skin::ServerSkin;

/// Raven's renderer model registry — the `CachedModels` map, the `tr.models[]`
/// pool + `mhHashTable`, and the loader's file-static bookkeeping, gathered onto
/// one owner struct (`TRM-D3`/ruling 53). A direct `Engine.render_models` field,
/// reached via the ruling-43 `render_models_call` split-borrow accessor — never
/// a bare `&mut` (an `impl EngineHost for Engine` would alias the very field
/// being mutated; `## State ownership`).
///
/// Raven kept these as scattered file-scope globals (`CachedModels`, `tr.models`,
/// `tr.numModels`, `mhHashTable`, `giRegisterMedia_CurrentLevel`, `sPrevMapName`,
/// `gbInsideRegisterModel`, `tr.numBSPModels`); §B3 forbids the statics, so they
/// become fields here.
///
/// `RenderModels` is not `ZeroValid` (its `Vec`/`String`/`BTreeMap`/`HashMap`
/// fields are not all-zero-valid), so `Engine::new` writes it in place through
/// `MaybeUninit` — `addr_of_mut!((*p).render_models).write(RenderModels::default())`
/// (`engine.rs`, LIFE-Q9).
///
/// Source: `oracle/codemp/renderer/tr_model.cpp:35-36,67-68,521,560,1406`;
/// `oracle/codemp/renderer/tr_local.h:1396-1397`
pub struct RenderModels {
    /// `tr.models[MAX_MOD_KNOWN=1024]` + `tr.numModels` — the Hunk-allocated
    /// `model_t*` pool, carrying the R2 arena mechanics in place (slot-0
    /// reservation, generation counting, `handle_at_slot`) per the
    /// `docs/subsystems/tr-model.md` amendment 2026-07-27 (#51). Entries stay
    /// `Box`-pinned so a registered `model_t*` is address-stable
    /// (`G2_API.cpp:2716` caches `currentModel`; the DEC-35 mdx views read the
    /// blocks out of this pool). Cap at `MAX_MOD_KNOWN` → `R_AllocModel`
    /// returns `None`.
    ///
    /// Source: `oracle/codemp/renderer/tr_model.cpp:611-624`;
    /// `oracle/codemp/renderer/tr_local.h:1396-1397`
    pub(crate) models: ModelPool,

    /// `mhHashTable[FILE_HASH_SIZE]` intrusive chains, replaced by a
    /// case-insensitive name → handle map (`TRM-D3`/ruling 53). Lookup-only, no
    /// ordered iteration, so container choice is §D12 latitude — `HashMap`
    /// suffices; `generateHashValue` is not reproduced.
    ///
    /// Source: `oracle/codemp/renderer/tr_model.cpp:35-36,653-667`
    pub(crate) hash: HashMap<String, qhandle_t>,

    /// `CachedModels` — the endian-swapped model-block cache. `BTreeMap` keeps
    /// the `std::map` sorted-key iteration order the eviction loops and
    /// `Info_f` walk depend on (`TRM-D3`; key = lowercased model name).
    ///
    /// Source: `oracle/codemp/renderer/tr_model.cpp:67-68`
    pub(crate) cached: BTreeMap<String, CachedEndianedModelBinary>,

    /// `giRegisterMedia_CurrentLevel` — the level counter eviction keys off.
    ///
    /// Source: `oracle/codemp/renderer/tr_model.cpp:521`
    pub(crate) current_level: i32,

    /// `RE_RegisterMedia_LevelLoadBegin`'s `sPrevMapName` file-static (bumps the
    /// level only when the map name changed).
    ///
    /// Source: `oracle/codemp/renderer/tr_model.cpp:560`
    pub(crate) prev_map_name: String,

    /// `gbInsideRegisterModel` — the re-entrancy guard around
    /// `RE_RegisterModel_Actual` that blocks `LevelLoadEnd` eviction mid-load.
    ///
    /// Source: `oracle/codemp/renderer/tr_model.cpp:1406`
    pub(crate) inside_register_model: bool,

    /// `tr.numBSPModels` — incremented by the server; the `#ifndef DEDICATED`
    /// `RE_LoadWorldMap_Actual` body is dead here.
    ///
    /// Source: `oracle/codemp/renderer/tr_model.cpp:541,1231`
    pub(crate) num_bsp_models: i32,

    /// `tr.skins[MAX_SKINS]` + `tr.numSkins` — the skin registry, homed here
    /// per user ruling 2026-07-12 (server skins name-pool), joining the model
    /// registry this struct already owns. `qhandle_t` = index; slot 0 is the
    /// default skin `init_skins` seeds (`server_skins.rs`).
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:1409-1410`
    pub(crate) skins: Vec<ServerSkin>,

    /// The server-shader name pool — `R_FindServerShader`'s hash-table rows
    /// flattened to bare names (user ruling 2026-07-12): server shader objects
    /// carry only the name, the sole field the dedicated path ever reads
    /// (`G2_surfaces.cpp:212`). Slot 0 stands in for `tr.defaultShader`
    /// (`server_skins.rs`).
    ///
    /// Source: `oracle/codemp/renderer/tr_shader.cpp:3560-3596`
    pub(crate) server_shaders: Vec<String>,

    /// Brush `model_t` handle -> its `WorldAsset::bmodels` index. The owned
    /// replacement for `R_LoadSubmodels`' `model->bmodel = out` pointer write:
    /// the render walk resolves a handle to the owned submodel row through this
    /// map instead of the retired `model_t::bmodel` raw pointer, which the
    /// struct no longer carries. Filled by [`Self::register_bmodel`].
    ///
    /// Source: `oracle/codemp/renderer/tr_bsp.cpp:1441`
    pub(crate) bmodel_indices: HashMap<qhandle_t, usize>,

    /// The DEC-65 ruling 1 published view of this registry: one entry per registered slot, naming its blocks by
    /// `Arc` and byte offset. `RE_EndFrame` hands a clone to `RenderAssetsSim`, and the render thread reads it out
    /// of the frame package.
    ///
    /// No oracle counterpart: Raven's renderer has one thread and no publish step.
    pub(crate) blocks: ModelBlocks,

    /// Set by every registration, eviction and pool reset, cleared by [`Self::publish_blocks`].
    /// The dedicated server marks slots with no render thread to publish to, and this flag is what lets that stay
    /// inert instead of failing.
    pub(crate) blocks_dirty: bool,
}

impl Default for RenderModels {
    /// The construction story (`## State ownership`): the hash/cache start
    /// empty (filled lazily by `R_ModelInit`/`R_AllocModel`/registration), the
    /// pool starts at its A12 slot-0 reservation with `tr.numModels == 0`, the
    /// level counter and BSP count start at `0`, the previous map name empty,
    /// and the re-entrancy guard `false`. Matches Raven's zero-init file
    /// statics plus the lazily-`new`d `CachedModels` map.
    fn default() -> Self {
        Self {
            models: ModelPool::new(),
            hash: HashMap::new(),
            cached: BTreeMap::new(),
            current_level: 0,
            prev_map_name: String::new(),
            inside_register_model: false,
            num_bsp_models: 0,
            skins: Vec::new(),
            server_shaders: Vec::new(),
            bmodel_indices: HashMap::new(),
            blocks: ModelBlocks::default(),
            blocks_dirty: false,
        }
    }
}

impl RenderModels {
    /// Raven `R_ModelInit` / `R_SVModelInit` — `R_SVModelInit` is a bare
    /// wrapper that calls `R_ModelInit` (the `#endif // !DEDICATED` sits
    /// directly above it, so it is the always-compiled, dedicated-live entry;
    /// folded into one method per §C10). Lazily `new`s `CachedModels`
    /// (mirrored here by `cached` already existing via `Default`), resets
    /// `tr.numModels`/`mhHashTable` to empty, and reserves `models[0]` as the
    /// `MOD_BAD` NULL model via `R_AllocModel`.
    ///
    /// Source: `oracle/codemp/renderer/tr_model.cpp:1655-1657,1662-1679`
    pub fn model_init(&mut self) {
        // `if(!CachedModels) CachedModels = new CachedModels_t;` — the map is
        // already live here (`cached` exists via `Default`), so there is nothing
        // to lazily construct.

        // `tr.numModels = 0; memset(mhHashTable, 0, ...)`. `ModelPool::reset`
        // is the DEC-42.1 registry teardown: the entries stay in place (Raven
        // leaves the `tr.models[]` array untouched and just resets the
        // high-water mark) while every pre-reset handle above slot 0 goes
        // stale; `r_alloc_model` re-creates slot 0 below.
        self.models.reset();
        self.hash.clear();
        // The reset renumbers every handle above slot 0, so the published entries no longer name what they claim.
        self.blocks.clear();
        self.blocks_dirty = true;

        // leave a space for NULL model
        let null_model = self
            .r_alloc_model()
            .expect("R_AllocModel for the reserved NULL model must succeed at init");
        self.models.slot_mut(null_model as usize).r#type = modtype_t::MOD_BAD;
    }

    /// Raven `R_ModelFree` — on a live `CachedModels` map, runs
    /// `RE_RegisterModels_DeleteAll` (the `cached_model_binary.rs` eviction
    /// free-fn, host-free) then drops the map.
    ///
    /// Source: `oracle/codemp/renderer/tr_model.cpp:1692-1699`
    pub fn model_free(&mut self) {
        // Raven: `if(CachedModels){ RE_RegisterModels_DeleteAll(); delete
        // CachedModels; CachedModels = NULL; }`. Here `cached` is an owned
        // `BTreeMap`, never a NULL-able pointer, so free every entry via
        // `re_register_models_delete_all` and leave the (now empty) map in
        // place — the Rust equivalent of `delete`+`= NULL`.
        self.re_register_models_delete_all();
    }

    /// Raven `R_HunkClearCrap` — a Hunk-reset teardown: calls the `tr_shader.cpp`
    /// `KillTheShaderHashTable` cross-ref, then zeros `tr.numModels`/
    /// `mhHashTable`/`tr.numShaders`/`tr.numSkins`. The skin registry and the
    /// server-shader name pool (the slice's flattened shader hash table, user
    /// ruling 2026-07-12 (server skins name-pool)) are this struct's fields
    /// now; only `tr.numShaders` stays §20 (client shader-compile counter).
    ///
    /// Source: `oracle/codemp/renderer/tr_model.cpp:1682-1690`
    pub fn hunk_clear(&mut self) {
        // `KillTheShaderHashTable()` — the name pool is this slice's flattened
        // server-shader hash table.
        self.server_shaders.clear();
        // `tr.numModels = 0; memset(mhHashTable, 0, ...)` — the same DEC-42.1
        // pool teardown `model_init` runs, minus the slot-0 re-creation
        // (Raven's `R_HunkClearCrap` drops the mark and stops there).
        self.models.reset();
        self.hash.clear();
        // Same reason as `model_init`: the reset invalidates every published handle.
        self.blocks.clear();
        self.blocks_dirty = true;
        // `tr.numSkins = 0` (the skin memory itself lived on the just-reset
        // hunk); `tr.numShaders = 0` stays §20, not a field of this struct.
        self.skins.clear();
    }

    /// Raven `R_GetModelByHandle` — out-of-range `index` (`< 1` or
    /// `>= tr.numModels`) falls back to `models[0]`, the default/NULL model;
    /// this is the read path `EngineHost::model_mdxm`/`model_mdxa` resolve
    /// through (`G2SV-D5`, `TRM-D1`(b)).
    ///
    /// Source: `oracle/codemp/renderer/tr_model.cpp:591-604`
    pub fn get_model(&self, handle: qhandle_t) -> &ModelData {
        self.models.by_handle(handle)
    }

    /// `tr.numModels` — the registered high-water mark. `BModelTable::build`
    /// walks `0..num_models` to read every live row.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:1397`
    pub fn num_models(&self) -> i32 {
        self.models.num_models()
    }

    /// Raven `R_Modellist_f` — prints each registered model's `dataSize`/LOD
    /// count/name (`host.print`), then the running total.
    ///
    /// Source: `oracle/codemp/renderer/tr_model.cpp:1705-1730`
    pub fn modellist_f(&self, host: &mut impl EngineHost) {
        let mut total: i32 = 0;
        for m in self.models.registered() {
            let mut lods = 1;
            for j in 1..MD3_MAX_LODS {
                if !m.md3[j].is_null() && m.md3[j] != m.md3[j - 1] {
                    lods += 1;
                }
            }
            // Com_Printf("%8i : (%i) %s\n", mod->dataSize, lods, mod->name)
            host.print(&format!(
                "{:8} : ({}) {}\n",
                m.dataSize,
                lods,
                read_qpath(&m.name)
            ));
            total += m.dataSize;
        }
        // Com_Printf("%8i : Total models\n", total)
        host.print(&format!("{total:8} : Total models\n"));
    }

    /// Raven `R_AllocModel` — `Hunk_Alloc`s the next `model_t`, sets
    /// `->index = tr.numModels`, appends to the pool, and returns `NULL` at
    /// the `MAX_MOD_KNOWN` cap. Internal to the `tr_model` subsystem (`§D12`);
    /// `pub(super)` so the sibling `server_load.rs` registration path can call
    /// it while it stays off the crate's public seam. `model_init` and that
    /// path are its only callers.
    ///
    /// Source: `oracle/codemp/renderer/tr_model.cpp:611-624`
    pub(super) fn r_alloc_model(&mut self) -> Option<qhandle_t> {
        // The mechanics themselves live on `ModelPool` (`model_pool.rs`) —
        // sequential high-water allocation, the `MAX_MOD_KNOWN` cap, the
        // zeroed `Hunk_Alloc` entry with `->index` set, and the generation the
        // vacating reset assigned. The returned `qhandle_t` is the bare slot
        // number DEC-42.2 pins.
        self.models.alloc()
    }

    /// Raven `RE_InsertModelIntoHash` — replaced by a direct
    /// case-insensitive-name → handle map insert (`TRM-D3`/ruling 53);
    /// `generateHashValue` (`:635`) is not reproduced — the map subsumes the
    /// bucket. Internal to the `tr_model` subsystem (`§D12`); `pub(super)` so
    /// the sibling `server_load.rs` registration path — its only caller — can
    /// reach it while it stays off the crate's public seam.
    ///
    /// Source: `oracle/codemp/renderer/tr_model.cpp:653-667`
    pub(super) fn re_insert_model_into_hash(&mut self, name: &str, handle: qhandle_t) {
        // Raven chains `modelHash_t` nodes keyed by `generateHashValue` and, on
        // lookup, compares the full name case-insensitively (`Q_stricmp`). The
        // name→handle map subsumes both bucket and node (`TRM-D3`/ruling 53);
        // storing the ASCII-lowercased name makes lookups case-insensitive
        // without reproducing `generateHashValue`.
        self.hash.insert(name.to_ascii_lowercase(), handle);
    }

    /// Raven `R_LoadSubmodels`' per-submodel `model_t` registration — allocate a
    /// pool slot, mark it `MOD_BRUSH`, name it `*i` for the main world or
    /// `*index-i` with `bspInstance` set for an RMG instance, then hash-insert
    /// the name. Records the handle against its `WorldAsset::bmodels` index so
    /// the render walk reaches the owned submodel row.
    ///
    /// `world_index` is `R_LoadSubmodels`' own `index` argument (0 for the main
    /// world, nonzero for an RMG instance); `submodel` is the submodel's index
    /// into `WorldAsset::bmodels`.
    ///
    /// Source: `oracle/codemp/renderer/tr_bsp.cpp:1433-1463`
    pub(crate) fn register_bmodel(&mut self, submodel: usize, world_index: i32) -> qhandle_t {
        // `assert(model != NULL)` — R_AllocModel only fails at the MAX_MOD_KNOWN
        // cap, which this reserved-pool registration never reaches.
        let handle = self
            .r_alloc_model()
            .expect("R_AllocModel for a brush submodel must succeed");
        let idx = handle as usize;

        let name = if world_index != 0 {
            format!("*{world_index}-{submodel}")
        } else {
            format!("*{submodel}")
        };

        {
            let slot = self.models.slot_mut(idx);
            slot.r#type = modtype_t::MOD_BRUSH;
            // Raven's `model->bmodel = out` write has no twin. The
            // `bmodel_indices` side map replaces that pointer.
            if world_index != 0 {
                slot.bspInstance = qtrue;
            }
            write_qpath(&mut slot.name, &name);
        }

        self.re_insert_model_into_hash(&name, handle);
        self.bmodel_indices.insert(handle, submodel);
        handle
    }

    /// The `WorldAsset::bmodels` index a brush `model_t` handle was registered
    /// against by [`Self::register_bmodel`] — the owned replacement for the
    /// `model_t::bmodel` deref. `None` for a handle that names no brush submodel.
    pub(crate) fn bmodel_index(&self, handle: qhandle_t) -> Option<usize> {
        self.bmodel_indices.get(&handle).copied()
    }

    /// The registered model handle for a case-insensitive name, or `None` when
    /// no model of that name is registered. Inline brush submodels register
    /// under `*<n>` ([`Self::register_bmodel`]), so `"*1"` names the first
    /// inline submodel.
    ///
    /// This stands in for `RE_RegisterModel`'s opening hash lookup for a debug
    /// harness that already registered the model. The full `RE_RegisterModel`
    /// (`frontend.rs`) takes 13 parameters, so a caller that only needs a handle
    /// for an already-registered name uses this, not the loader. It is not a
    /// general model lookup: a name never registered returns `None`.
    ///
    /// Source: `oracle/codemp/renderer/tr_model.cpp:1407-1417`
    /// (`RE_RegisterModel_Actual` hash lookup)
    pub fn handle_for_name(&self, name: &str) -> Option<qhandle_t> {
        self.hash.get(&name.to_ascii_lowercase()).copied()
    }

    /// The published block registry, but only when a registration, an eviction or a reset changed it since the
    /// last call. `RE_EndFrame` drains this once per frame.
    pub fn publish_blocks(&mut self) -> Option<ModelBlocks> {
        if !self.blocks_dirty {
            return None;
        }
        self.blocks_dirty = false;
        Some(self.blocks.clone())
    }

    /// Record one finished model slot into the published registry.
    /// This runs at registration completion, after the LOD-duplicate loop, so the recorded offsets match the final
    /// `model_t` pointers and an already-found load carries its replayed shader pokes.
    pub(super) fn mark_block(&mut self, handle: qhandle_t) {
        let model = self.get_model(handle);
        let model_type = model.r#type;
        let num_lods = model.numLods;
        let md3 = model.md3;
        let mdxm = model.mdxm;
        let mdxa = model.mdxa;
        let name = read_qpath(&model.name);

        let entry = PublishedModel {
            model_type,
            num_lods,
            md3: [
                self.block_containing(md3[0] as *const u8),
                self.block_containing(md3[1] as *const u8),
                self.block_containing(md3[2] as *const u8),
            ],
            mdxm: self.block_containing(mdxm as *const u8),
            mdxa: self.block_containing(mdxa as *const u8),
            name,
        };

        self.blocks.set(handle, entry);
        self.blocks_dirty = true;
    }
}

#[cfg(test)]
mod tests {
    use super::super::model_pool::MAX_MOD_KNOWN;
    use super::*;

    #[test]
    fn alloc_model_assigns_sequential_indices() {
        let mut rm = RenderModels::default();
        assert_eq!(rm.r_alloc_model(), Some(0));
        assert_eq!(rm.r_alloc_model(), Some(1));
        assert_eq!(rm.models.num_models(), 2);
        assert_eq!(rm.models.slot(0).index, 0);
        assert_eq!(rm.models.slot(1).index, 1);
    }

    #[test]
    fn alloc_model_caps_at_max_mod_known() {
        let mut rm = RenderModels::default();
        while rm.models.num_models() < MAX_MOD_KNOWN as i32 {
            assert!(rm.r_alloc_model().is_some());
        }
        assert_eq!(rm.r_alloc_model(), None);
    }

    #[test]
    fn model_init_reserves_null_model_as_mod_bad() {
        let mut rm = RenderModels::default();
        rm.model_init();
        assert_eq!(rm.models.num_models(), 1);
        assert!(matches!(rm.models.slot(0).r#type, modtype_t::MOD_BAD));
    }

    #[test]
    fn get_model_out_of_range_returns_default_slot() {
        let mut rm = RenderModels::default();
        rm.model_init(); // reserves models[0]
        rm.r_alloc_model(); // index 1
                            // index 0, negative, and >= num_models all fall back to models[0]
        assert_eq!(rm.get_model(0).index, 0);
        assert_eq!(rm.get_model(-5).index, 0);
        assert_eq!(rm.get_model(99).index, 0);
        assert_eq!(rm.get_model(1).index, 1);
    }

    #[test]
    fn register_bmodel_names_and_maps_main_world_and_rmg_instances() {
        let mut rm = RenderModels::default();
        rm.model_init(); // reserves models[0]

        // Main world (index 0): name "*i", bspInstance stays false.
        let h0 = rm.register_bmodel(0, 0);
        let h1 = rm.register_bmodel(1, 0);
        assert_eq!(read_qpath(&rm.models.slot(h0 as usize).name), "*0");
        assert_eq!(read_qpath(&rm.models.slot(h1 as usize).name), "*1");
        assert!(matches!(rm.models.slot(h0 as usize).r#type, modtype_t::MOD_BRUSH));
        assert_eq!(rm.models.slot(h0 as usize).bspInstance, 0);
        assert_eq!(rm.bmodel_index(h0), Some(0));
        assert_eq!(rm.bmodel_index(h1), Some(1));
        assert_eq!(rm.hash.get("*1"), Some(&h1));

        // RMG instance (index 3): name "*3-2", bspInstance set.
        let h2 = rm.register_bmodel(2, 3);
        assert_eq!(read_qpath(&rm.models.slot(h2 as usize).name), "*3-2");
        assert_eq!(rm.models.slot(h2 as usize).bspInstance, 1);
        assert_eq!(rm.bmodel_index(h2), Some(2));

        // A handle that names no submodel resolves to None.
        assert_eq!(rm.bmodel_index(999), None);
    }

    #[test]
    fn insert_into_hash_is_case_insensitive() {
        let mut rm = RenderModels::default();
        rm.re_insert_model_into_hash("Models/Player.glm", 7);
        assert_eq!(rm.hash.get("models/player.glm"), Some(&7));
    }

    #[test]
    fn hunk_clear_resets_registry_but_keeps_default_slot_reachable() {
        let mut rm = RenderModels::default();
        rm.model_init();
        rm.r_alloc_model();
        rm.re_insert_model_into_hash("foo", 1);
        rm.hunk_clear();
        assert_eq!(rm.models.num_models(), 0);
        assert!(rm.hash.is_empty());
        // Raven leaves tr.models[0] in place after a hunk reset; get_model still
        // resolves the stale default slot rather than panicking.
        assert_eq!(rm.get_model(0).index, 0);
    }
}
