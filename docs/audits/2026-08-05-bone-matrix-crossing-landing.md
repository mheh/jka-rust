# 2026-08-05 - the bone-matrix crossing landing (gh#31 step-005)

This record freezes the lane-review of gh#31 step-005, which executed DEC-65 ruling 2: the Ghoul2 skeleton transform runs sim-side at scene-add, per-entity bone matrices cross in the frame package, and the draw legs run render-side with no host gate. The work ran as an opus code lane under `.claude/packets/31/step-005/packet.md` (commits `c34ec3fc`, `37cfa349`, `d43b6509`), a conformance clerk on opus walked the whole diff and re-ran every gate, and a style commit `a3501348` closed the comment findings. The branch merged to master at `ec7c934c`. Follow-ups land on gh#31, and the finding dispositions live in the packet's Amendments section.

## The conformance clerk's report, verbatim

I read the packet whole, walked every hunk of all 21 changed files, and re-ran every gate. Report below.

---

# 1. Letter violations

I walked every hunk against the surface contract. **No new `pub` item, signature, `#[repr]`, trap or dispatcher arm, cvar, `FrameEvent` variant, engine hook, or dependency appears outside the contract's list.** Mechanical evidence:

- Added `pub` surface, complete: `Ghoul2ModelRender` + its 8 fields, `Ghoul2RenderPayload` + its 2 fields, `pub mod ghoul2_model_render; pub mod ghoul2_render_payload;`, `RefEntity::ghoul2_render`, `CRenderSurface::model_ordinal`, `G2SurfaceRef::model_ordinal`, `build_ghoul2_render_payload`. Every one is named in the contract.
- Added `fn` surface, complete: `build_ghoul2_render_payload` only. Its signature is character-for-character the contract's.
- Removed `pub` surface: `set_ghoul2`, `ghoul2_mut`, `WalkWarnings::entity_models`, `CRenderSurface::bone_cache`, `G2SurfaceRef::bone_cache`. All five are named in the contract.
- No `Cargo.toml` or `Cargo.lock` in the diff. No `#[repr]` line added or removed. No `FrameEvent` variant. No cvar registration (`r_noServerGhoul2` is only read, never registered anew).

**Two write-scope notes, both quoted:**

**(a) `crates/mp/renderer/src/render_state/mod.rs` is not on the enumerated write scope.** The scope reads "`crates/mp/renderer/src/render_state/` - new `ghoul2_model_render.rs` and `ghoul2_render_payload.rs`, plus `placeholders.rs` and `walk_warnings.rs`." `mod.rs` is not in that enumeration. The hunk:

```rust
 pub mod frame_state;
+pub mod ghoul2_model_render;
+pub mod ghoul2_render_payload;
 pub mod handle;
```

**(b) `dev_harness.rs` and `ui_harness.rs` are not on the enumerated write scope** but are covered by the catch-all clause "Any caller `cargo check` shows broken by a parameter deletion or the new `RE_AddRefEntityToScene` arity, edit-only to pass the new shape." Both hunks are pure argument deletion:

```rust
                             &WorldLoadState::default(),
-                            // 2D-only harness: no entity walk to host.
-                            None,
                             self.registries.img_state.pending_uploads.drain().collect(),
```

```rust
                     &self.host.re.world_load,
-                    // Menu frames draw 2D only, so there is no entity walk to host.
-                    None,
                     self.host.re.img_state.pending_uploads.drain().collect(),
```

`crates/mp/engine/client/src/fx/fx_host.rs` was in scope and was not touched. That is a non-change, not a violation.

---

# 2. The named hunks, verbatim

### 2.1 `CG_R_ADDREFENTITYTOSCENE`, `crates/mp/engine/client/src/cl_cgame.rs`

```rust
    } else if op == MpCgameImport::CG_R_ADDREFENTITYTOSCENE as c_int {
        // SAFETY: `VMA(1)` is the module's `refEntity_t` (porting-rules §D11).
        let ent = unsafe { &*(vma(vc, args, 1) as *const refEntity_t) };
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let g2 = unsafe { g2_from_view(view) };
        // The DEC-65 ruling 2 crossing is built before the `re` cast, and that order is what keeps the two casts apart.
        // The build can re-register a model through the `re` hooks, which call `Arc::make_mut` on the published registry.
        let no_server_ghoul2 = {
            // SAFETY: view-constructor slot, single-threaded, no other live cast.
            let re = unsafe { re_from_view(view) };
            view.common.cvar(re.cvars.r_noServerGhoul2).integer
        };
        let payload = build_ghoul2_render_payload(g2, view, ent, no_server_ghoul2);
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let re = unsafe { re_from_view(view) };
        RE_AddRefEntityToScene(
            &mut re.frame_data,
            &re.sim.published,
            &mut re.scene,
            ent,
            payload,
        );
        0
```

### 2.2 `UI_R_ADDREFENTITYTOSCENE`, `crates/mp/engine/client/src/cl_ui.rs`

```rust
    } else if trap == MpUiImport::UI_R_ADDREFENTITYTOSCENE as c_int {
        // SAFETY: `VMA(1)` is the module's `refEntity_t` (porting-rules §D11).
        let ent = unsafe { &*(vma(view.common, args, 1) as *const refEntity_t) };
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let g2 = unsafe { g2_from_view(view) };
        // The DEC-65 ruling 2 crossing is built before the `re` cast, and that order is what keeps the two casts apart.
        // The build can re-register a model through the `re` hooks, which call `Arc::make_mut` on the published registry.
        let no_server_ghoul2 = {
            // SAFETY: view-constructor slot, single-threaded, no other live cast.
            let re = unsafe { re_from_view(view) };
            view.common.cvar(re.cvars.r_noServerGhoul2).integer
        };
        let payload = build_ghoul2_render_payload(g2, view, ent, no_server_ghoul2);
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let re = unsafe { re_from_view(view) };
        RE_AddRefEntityToScene(
            &mut re.frame_data,
            &re.sim.published,
            &mut re.scene,
            ent,
            payload,
        );
        0
```

Both arms run the contract's four steps in order: `g2` cast, cvar read in a closed scope, build with no `re` binding live, then the `re` cast and the add.

### 2.3 `build_ghoul2_render_payload`, `crates/mp/renderer/src/tr_ghoul2.rs`

```rust
/// Builds the DEC-65 ruling 2 crossing for one scene entity, or `None` when the entity carries no live Ghoul2 instance.
/// The caller must finish this call before it takes any `re`-slot borrow, because the setup path can re-register a model through the `re` hooks.
///
/// This is the sim-side front half of [`r_add_ghoul_surfaces`]: the setup, the transform, and the per-bone evaluation the render side used to run.
/// A stale replay token returns `None`, which the walk reads as an absent instance, exactly as it read a dead handle before.
///
/// Source: `oracle/codemp/renderer/tr_ghoul2.cpp:3383-3538`
pub fn build_ghoul2_render_payload(
    g2: &mut Ghoul2System,
    host: &mut EngineHostView,
    ent: &refEntity_t,
    no_server_ghoul2: i32,
) -> Option<Arc<Ghoul2RenderPayload>> {
    let handle = ghoul2_token_decode(ent.ghoul2)?;
    let mut ghoul2 = CGhoul2Info_v { mItem: handle.0 };

    if !ghoul2.is_valid(g2) {
        return None;
    }

    // The `MOD_BAD` arm reads this answer in place of the instance list, so it is taken before the setup can move any `mModelindex`.
    let have_models = g2api_have_we_ghoul2_models(g2, &ghoul2);

    // Raven checks `r_noServerGhoul2` before the transform, and ruling B keeps that order here.
    // A suppressed instance crosses with no models, so both dispatch arms draw what they drew before.
    if no_server_ghoul2 != 0 {
        return Some(Arc::new(Ghoul2RenderPayload {
            have_models,
            models: Vec::new(),
        }));
    }

    if !g2_setup_model_pointers_v(g2, host, &ghoul2) {
        return Some(Arc::new(Ghoul2RenderPayload {
            have_models,
            models: Vec::new(),
        }));
    }

    // `G2API_GetTime` never reads its argument, so the builder needs no refdef clock.
    // Source: oracle/codemp/ghoul2/G2_API.cpp:179-188
    let current_time = g2api_get_time(g2, 0);

    // Raven culls the entity and transforms only what survives (`R_GCullModel`, `:3383-3538`), and the view does not exist at scene-add.
    // Ruling A accepts the consequence: every added instance transforms.
    // The smoothing history therefore advances for entities the render cull would have left stale.
    let model_list =
        g2_construct_render_skeleton(g2, host, &mut ghoul2, current_time, ent.modelScale);

    let mut models: Vec<Ghoul2ModelRender> = Vec::with_capacity(model_list.len());
    for &i in &model_list {
        let item = ghoul2.mItem;
        let inst = &g2.info_array.get(item)[i as usize];

        if !inst.valid || (inst.flags & GHOUL2_NOMODEL) != 0 || (inst.flags & GHOUL2_NORENDER) != 0
        {
            continue;
        }

        let model = inst.model;
        let custom_skin = inst.custom_skin;
        let skin = inst.skin;
        let lod_bias = inst.lod_bias;
        let surface_root = inst.surface_root;
        let slist = inst.slist.clone();
        let bltlist = inst.bltlist.clone();
        let bone_cache = inst.bone_cache;

        // Raven evaluates a bone at backend draw time, and only the bones a drawn surface references.
        // This eager pass marks every bone rendered, which only `CBoneCache::WasRendered` observes, and its one consumer is the deferred gore chain.
        let bones: Vec<mdxaBone_t> = match bone_cache.and_then(|id| g2.bone_caches.get_mut(id)) {
            Some(cache) => (0..cache.final_bones.len())
                .map(|bone| cache.eval_render(bone as i32))
                .collect(),
            None => Vec::new(),
        };

        models.push(Ghoul2ModelRender {
            model,
            custom_skin,
            skin,
            lod_bias,
            surface_root,
            slist,
            bltlist,
            bones,
        });
    }

    Some(Arc::new(Ghoul2RenderPayload {
        have_models,
        models,
    }))
}
```

### 2.4 `crates/mp/renderer/src/render_state/ghoul2_model_render.rs` (whole file, new)

```rust
//! `Ghoul2ModelRender` - one render-visible model of a Ghoul2 entity, snapshotted at scene-add.

use mp_engine_ghoul2::shared::bolt_info_t::boltInfo_t;
use mp_engine_ghoul2::shared::surface_info_t::surfaceInfo_t;
use mp_qshared::shared::{mdxaBone_t, qhandle_t};

/// One render-visible model of a Ghoul2 entity, snapshotted sim-side at scene-add.
/// It has no Raven counterpart: DEC-65 ruling 2 replaces the render side's reach into the live `CGhoul2Info` with these values.
/// Every field is POD or owned, so the type is `Send + Sync` without unsafe.
pub struct Ghoul2ModelRender {
    /// The instance's registered model handle, which the walk resolves against the published registry.
    pub model: qhandle_t,
    /// `mCustomSkin` on the instance.
    pub custom_skin: qhandle_t,
    /// `mSkin` on the instance.
    pub skin: qhandle_t,
    /// `mLodBias` on the instance.
    pub lod_bias: i32,
    /// `mSurfaceRoot` on the instance.
    pub surface_root: i32,
    /// The instance's surface-override list, cloned at scene-add.
    pub slist: Vec<surfaceInfo_t>,
    /// The instance's bolt list, cloned at scene-add.
    pub bltlist: Vec<boltInfo_t>,
    /// The composed render matrix per bone, indexed by the global bone index `surface.bone_ref` yields.
    /// Empty when the instance has no built bone cache, and the walk then drops the model's surfaces.
    pub bones: Vec<mdxaBone_t>,
}
```

### 2.5 `crates/mp/renderer/src/render_state/ghoul2_render_payload.rs` (whole file, new)

```rust
//! `Ghoul2RenderPayload` - the per-entity Ghoul2 crossing of DEC-65 ruling 2.

use crate::render_state::ghoul2_model_render::Ghoul2ModelRender;

/// The per-entity Ghoul2 crossing of DEC-65 ruling 2: everything the render thread reads for one entity's skinned models.
/// `build_ghoul2_render_payload` (`tr_ghoul2.rs`) fills it at scene-add, and the entity walk plus the vertex decoder read it render-side.
/// It has no Raven counterpart, and every field is POD or owned, so the type is `Send + Sync` without unsafe.
pub struct Ghoul2RenderPayload {
    /// The `G2API_HaveWeGhoul2Models` answer at scene-add, which the `MOD_BAD` arm reads in place of the live instance list.
    pub have_models: bool,
    /// The render-visible models in `G2_Sort_Models` order, empty when `r_noServerGhoul2` suppressed the transform.
    pub models: Vec<Ghoul2ModelRender>,
}
```

Field names, order, and types match the contract block exactly. Neither type derives anything, which the finished file confesses.

### 2.6 `MOD_MDXM` and `MOD_BAD`, `crates/mp/renderer/src/tr_main.rs`

```rust
                    // g2r
                    modtype_t::MOD_MDXM => {
                        // `r_add_ghoul_surfaces` lights the `&mut RefEntity` it
                        // takes, matching the MD3 sibling. The lit fields fold
                        // back onto `entities[n]` so the backend reads them by
                        // entity index.
                        let mut re = ref_entity_from_tr(ent);
                        if let Some(payload) = payloads
                            .get(current_entity_num)
                            .and_then(|p| p.as_ref())
                        {
                            r_add_ghoul_surfaces(
                                &mut re,
                                payload,
                                assets,
                                view,
                                &ori,
                                cvars,
                                &mut walk_scratch.warnings,
                                world_load,
                                frame,
                                fogs,
                                refdef_rdflags,
                                shifted_entity_num,
                                dlights,
                                draw_surfs,
                            );
                            write_back_lighting(ent, &re);
                        }
                    }

                    // null model axis
                    modtype_t::MOD_BAD => {
                        let mut re = ref_entity_from_tr(ent);
                        if (re.renderfx & RF_THIRD_PERSON) != 0
                            && view.isPortal == 0
                            && (re.renderfx & RF_SHADOW_ONLY) == 0
                        {
                            continue;
                        }

                        // The oracle's `ent->e.ghoul2 && G2API_HaveWeGhoul2Models(...)` pair.
                        // The builder took that answer at scene-add, so an instance whose models all decline to render still skips the null axis.
                        if let Some(payload) = payloads
                            .get(current_entity_num)
                            .and_then(|p| p.as_ref())
                        {
                            if payload.have_models {
                                r_add_ghoul_surfaces(
                                    &mut re,
                                    payload,
                                    ...
                                );
                                write_back_lighting(ent, &re);
                                continue;
                            }
                        }
```

The gate deletion and the marker deletion are in the same hunk:

```rust
-                //TODO: Port R_AddEntitySurfaces Ghoul2 arms render-side
+                // Both entity arms read the published blocks and run on either thread.
+                // DEC-65 ruling 2 moved the Ghoul2 skeleton transform to scene-add, so the legs below read `payloads[n]` and reach no host.
                 // Source: oracle/codemp/renderer/tr_main.cpp:1444-1470
-                // The MD3 arm now reads the published blocks and runs on either thread.
-                // The Ghoul2 legs still reach `EngineHost::model_mdxm`/`model_mdxa` and the sim-confined bone caches.
-                // They therefore keep the host gate below, and a render-side caller draws no player.
```

### 2.7 `decode_ghoul2_surface`, `crates/mp/renderer-gpu/src/pipeline3d.rs`

```rust
 fn decode_ghoul2_surface(
     models: &ModelBlocks,
-    g2: &mut Ghoul2System,
+    payloads: &[Option<Arc<Ghoul2RenderPayload>>],
+    entity_num: i32,
     g2_ref: G2SurfaceRef,
     rgba: [u8; 4],
 ) -> Option<(Vec<WorldVertex>, Vec<u32>, Vec<[f32; 4]>)> {
...
-    // A stale or absent bone-cache handle means the skeleton never built, so the
+    // A missing payload or an out-of-range ordinal means the skeleton never crossed, so the
     // surface is not renderable (Raven's null `boneCache`).
-    let cache = g2.bone_caches.get_mut(g2_ref.bone_cache)?;
+    let payload = payloads.get(entity_num as usize)?.as_ref()?;
+    let bones = &payload.models.get(g2_ref.model_ordinal as usize)?.bones;
...
-        let bone = cache.eval_render(surface.bone_ref(vert.bone_index(0)));
+        let bone = bones[surface.bone_ref(vert.bone_index(0)) as usize];
...
-                let bone2 = cache.eval_render(surface.bone_ref(vert.bone_index(1)));
+                let bone2 = bones[surface.bone_ref(vert.bone_index(1)) as usize];
...
-                    let bone = cache.eval_render(surface.bone_ref(vert.bone_index(k)));
+                    let bone = bones[surface.bone_ref(vert.bone_index(k)) as usize];
...
-                let bone = cache.eval_render(surface.bone_ref(vert.bone_index(k)));
+                let bone = bones[surface.bone_ref(vert.bone_index(k)) as usize];
```

The weight arithmetic lines around them are untouched by the diff.

### 2.8 `FrameExecutor` state deletion and `render_world` payloads, `crates/mp/renderer-gpu/src/frame_exec.rs`

```rust
-    /// `tr.theGhoul2InfoArray` and its bone caches. W2-F5 homes the instance
-    /// owner here, so the caches the render path builds persist across frames
-    /// without a caller threading them in.
-    ghoul2: Ghoul2System,
...
-            ghoul2: Ghoul2System::default(),
...
-    /// Installs a prepared Ghoul2 instance owner, replacing the empty one the
-    /// executor starts with.
-    ///
-    /// W2-F5 homes the owner here, and a harness that built its instances
-    /// through `G2API_InitGhoul2Model` before the GPU came up hands the result
-    /// over with this. The bone caches then persist for the process lifetime.
-    pub fn set_ghoul2(&mut self, g2: Ghoul2System) {
-        self.ghoul2 = g2;
-    }
-
-    /// The executor's Ghoul2 instance owner, for a caller that drives
-    /// `G2API_*` between frames.
-    pub fn ghoul2_mut(&mut self) -> &mut Ghoul2System {
-        &mut self.ghoul2
-    }
```

```rust
+        // The DEC-65 ruling 2 crossings, taken from the same scene window that builds `entities`, so the two slices index alike.
+        // The `Arc` clone is one refcount bump per entity.
+        let payloads: Vec<Option<Arc<Ghoul2RenderPayload>>> = self.scene_entities
+            [self.first_scene_entity..]
+            .iter()
+            .map(|re| re.ghoul2_render.clone())
+            .collect();
```

I confirmed `entities` is built from the identical window (`self.scene_entities[self.first_scene_entity..]`).

### 2.9 Drain-before-pin blocks and pin comments

`crates/mp/renderer-gpu/tests/ghoul2_vertex_golden.rs` (final text):

```rust
    {
        // `RE_EndFrame` drains the registered model blocks into the published registry, and no test reaches it.
        // The drain therefore runs here, and it must land before the pin below.
        // A drain after the pin publishes into a generation the frame does not read, and the frame then draws nothing.
        // Source: crates/mp/renderer/src/tr_cmds.rs:354-358
        if let Some(blocks) = host.models.publish_blocks() {
            host.re.sim.publish_models(blocks);
        }
        // The frame pins the published registry, because `G2_SetupModelPointers` re-registers at scene-add, ahead of the drain above.
        // The client `RE_RegisterModel` hook then calls `Arc::make_mut(&mut re.sim.published)` through the seated `re` slot.
        // The clone holds a second reference, so that call copies on write instead of mutating the allocation this frame reads.
        let pinned = Arc::clone(&host.re.sim.published);
```

`crates/mp/renderer-gpu/tests/entity_golden.rs` carries the identical block. The `Arc::clone` pin line and the three drain lines are unchanged by the diff. Only the first pin comment line changed, in both files:

```rust
-        // The frame pins the published registry, because `G2_SetupModelPointers` re-registers on every entity walk.
+        // The frame pins the published registry, because `G2_SetupModelPointers` re-registers at scene-add, ahead of the drain above.
```

### 2.10 `boot.rs` fallout, `crates/mp/renderer-gpu/src/ui_host/boot.rs`

```rust
-        // The ui background render has no live Ghoul2 state, so it threads an
-        // empty owned system (design point 2).
-        let mut g2_system = Ghoul2System::default();
+        // The spike adds no entity, so it walks with an empty crossing slice (DEC-65 ruling 2).
+        let payloads: Vec<Option<Arc<Ghoul2RenderPayload>>> = Vec::new();
         // The spike walks the world once, so its marks live and die with this
         // call (W2-F4).
         let mut walk_scratch = WorldWalkScratch::default();
         if let Some(world) = assets.world.as_ref() {
             walk_scratch.set_world(world);
         }
-        // The spike runs on the sim thread, so it hands the entity walk the engine host (W2-F1).
...
-            Some(&mut engine_view),
             assets,
             &bmodel_table,
             cvar_snapshot,
             world_load,
             frame,
             &mut walk_scratch,
-            &mut g2_system,
+            &payloads,
```

---

# 3. Ledger mismatches

Five behaviors are visible in the diff and absent from the finished file. A confessed choice is excluded.

### 3.1 The cull and `R_SetupEntityLighting` now run on the two empty-payload paths

The old body returned before the cull for a suppressed `r_noServerGhoul2` and for a failed setup:

```rust
-    // if we don't want server ghoul2 models and this is one, or we just don't
-    // want ghoul2 models at all, then return
-    if cvars.no_server_ghoul2 != 0 {
-        return;
-    }
-
-    if !g2_setup_model_pointers_v(g2, host, &ghoul2) {
-        return;
-    }
```

Both now cross as `Some(payload)` with `models: Vec::new()`, so the arm calls `r_add_ghoul_surfaces`, whose body now opens on:

```rust
) {
    // cull the entire model if the merged bounding box is outside the frustum
    let r_nocull_integer = cvars.nocull;
    ...
    let cull = r_g_cull_model(...);
    if cull == CULL_OUT {
        return;
    }
    ...
    if !personal_model || cvars.shadows > 1 {
        R_SetupEntityLighting(cvars, assets, world_load, frame, refdef_rdflags, dlights, ent);
    }
```

and the caller then runs `write_back_lighting(ent, &re);`. The finished file's bullet says only "Both preserve the old draw-nothing behavior in the `MOD_MDXM` arm and the old skip-the-null-axis behavior in the `MOD_BAD` arm." It does not mention that the cull and the lighting write-back now execute where they previously did not.

### 3.2 Both model goldens hardcode the `no_server_ghoul2` argument as the literal `0`

```rust
        build_ghoul2_render_payload(&mut g2, &mut view, &g2_ent, 0)
```
```rust
        build_ghoul2_render_payload(&mut g2, &mut view, &ent, 0)
```

The old path read the value off `RenderCvarSnapshot` inside `r_add_ghoul_surfaces`. The finished file does not mention the literal.

### 3.3 `world_harness.rs` reads the `App`-held snapshot, not a per-frame read

```rust
        let no_server_ghoul2 = self.cvars.no_server_ghoul2;
```

Not mentioned in the finished file.

### 3.4 `RenderCvarSnapshot::no_server_ghoul2` now has no reader inside the renderer crate

Grep across `crates/mp`: the field is declared at `render_cvar_snapshot.rs:108`, written at `:148` and `:186`, and read at exactly one site, `crates/mp/renderer-gpu/src/bin/world_harness.rs:833`. The renderer's own read deleted with the hunk in 3.1. Not mentioned.

### 3.5 The `MOD_MDXM` guard changed subject

```rust
-                        if let Some(handle) = re.ghoul2 {
+                        if let Some(payload) = payloads
+                            .get(current_entity_num)
+                            .and_then(|p| p.as_ref())
+                        {
```

An entity whose token decodes but whose instance fails `is_valid` previously entered the branch, returned from `r_add_ghoul_surfaces` at the validity check, and still ran `write_back_lighting(ent, &re)`. It now skips the branch entirely. Not mentioned.

---

# 4. The inventories

### Files changed against the write scopes

21 files. In scope by name: `finished.md`, `cl_cgame.rs`, `cl_ui.rs`, `world_harness.rs`, `frame_exec.rs`, `pipeline3d.rs`, `ui_host/boot.rs`, `entity_golden.rs`, `ghoul2_vertex_golden.rs`, `world_golden.rs`, `scene_golden.rs`, `ghoul2_model_render.rs`, `ghoul2_render_payload.rs`, `placeholders.rs`, `walk_warnings.rs`, `tr_ghoul2.rs`, `tr_main.rs`, `tr_scene.rs`. Under the catch-all clause: `dev_harness.rs`, `ui_harness.rs`. Unenumerated: `render_state/mod.rs` (see item 1a).

No fixture changed. `tests/goldens/**` and `ghoul2_verts_stormtrooper.bin` do not appear in `git diff --name-only`. No `oracle/`, `crates/mp/engine/ghoul2/`, `hook_install.rs`, `model_blocks.rs`, `frame_package.rs`, or `tr_cmds.rs` file appears.

### Commits against the bundle

Three commits, cut from the master tip (`merge-base` = `baa02455` = `master`), no split, no reorder.

- `c34ec3fc` = bundle item 1. Thirteen files. `tr_main.rs` carries only the `ghoul2_render: None` line in `ref_entity_from_tr`; both dispatcher arms pass `None`; no builder call is wired. Matches "the crossing surface, inert."
- `37cfa349` = bundle item 2. Fifteen files, the atomic swap.
- `d43b6509` = bundle item 3, the finished file alone.

All three carry `%G?` = `N`, so `--no-gpg-sign` held.

### Commit messages against the rules

- **Subjects.** All three are heading noun phrases: `feat(gh#31 s005): the Ghoul2 crossing surface, inert`, `feat(gh#31 s005): the bone matrices cross, and Ghoul2 draws render-side`, `docs(gh#31 s005): the lane finished file`.
- **Bodies.** Unwrapped paragraphs. No em dash, no en dash, no semicolon, no contraction. Six sentences exceed the STE 25-word cap, longest 36 words: "It decodes the token, checks the instance, takes the have-models answer, applies the `r_noServerGhoul2` gate before the transform per ruling B, runs the setup and the render skeleton, then evaluates every bone in each model's cache."
- **Trailer.** The rule reads "no trailer of any kind." Both `feat` commits end on a line git parses as a trailer. `git interpret-trailers --parse` returns:

```
Gates: cargo build --workspace green, cargo test --workspace green, the ghoul2 vertex golden byte-identical in its fifth configuration, the entity golden byte-identical, both world goldens byte-identical, the seven scene goldens green.
```
```
Gates: cargo build --workspace green, cargo test --workspace green, both world goldens byte-identical.
```

`d43b6509` parses to no trailer. No `Co-Authored-By` and no "Generated with Claude Code" footer appears anywhere.

---

# 5. Repo mechanics on added lines

I scanned all 450 added lines.

- **Function-body `use` declaration:** none. `grep -E '^\+[[:space:]]+use '` over the added lines returns nothing. Every added `use` sits at column 0 at a file head.
- **`todo!()` or other unmarked placeholder:** none. `grep 'todo!\|unimplemented!\|unreachable!\|TODO\|FIXME'` over the added lines returns nothing. The one `TODO: Port` marker in the diff is a deletion, `//TODO: Port R_AddEntitySurfaces Ghoul2 arms render-side`, which the contract required.
- **Newly ported item with no oracle `Source:` cite:** `build_ghoul2_render_payload` carries `/// Source: \`oracle/codemp/renderer/tr_ghoul2.cpp:3383-3538\``. `Ghoul2ModelRender` and `Ghoul2RenderPayload` carry no `Source:` cite. Both state why on their own doc lines: `/// It has no Raven counterpart: DEC-65 ruling 2 replaces the render side's reach into the live \`CGhoul2Info\` with these values.` and `/// It has no Raven counterpart, and every field is POD or owned, so the type is \`Send + Sync\` without unsafe.` The packet calls them "New constructs with no Raven counterpart."
- **New extern forward-declaration block:** none. `grep 'extern'` over the added lines returns nothing.
- **`format!` building a wire string:** none. `grep 'format!'` over the added lines returns nothing.

---

# 6. House-style violations on added lines

I read `~/.claude/skills/house-style/SKILL.md` and `~/.claude/skills/asd-ste100/SKILL.md`.

**Clean:** no em dash or en dash on any added line. No contraction. No marketing adjective. No comment line over 150 columns. No single-line sentence over 25 words.

**Violations found:**

### 6.1 Pet vocabulary, "rides" (`pipeline3d.rs`, added line)

```
/// (DEC-65 ruling 2). The decoded per-vertex normal rides a parallel `[f32; 4]`
```

### 6.2 Pet vocabulary, "seam" (`placeholders.rs`, added line)

```
    /// carries a raw `CGhoul2Info_v *`, so no raw pointer crosses the seam here.
```

### 6.3 Semicolon in prose (`tr_main.rs`, added line)

```
/// `entities` under the same scene-window rule; `fogs` is `tr.world->fogs`
```

### 6.4 Column-wrapped comment lines, the anti-column-wrap gate

Six added blocks break under 150 columns at a width, not at a sentence end or one clause boundary. Joined, each of the first two sits under 150 characters.

`pipeline3d.rs`:
```
    // A missing payload or an out-of-range ordinal means the skeleton never crossed, so the
    // surface is not renderable (Raven's null `boneCache`).
```

`tr_main.rs`:
```
/// `payloads`/`assets`/`cvars`/`entities`/`scratch`/`view`/`draw_surfs` are
/// `R_AddEntitySurfaces`/`R_AddTerrainSurfaces`/`R_SetupProjection`'s own
/// already-ported parameters, threaded straight through.
```
```
/// arrive on the `cvars` snapshot this fn already carries for the shader and
/// model lookups below.
/// There is therefore no leaf-function reason to split the cvar reads out as
/// separate parameters the way `R_CullLocalBox`'s `r_nocull_integer` does.
```
```
/// and the ordinal of the model inside the entity's `Ghoul2RenderPayload`, so the
/// backend re-locates the surface and reads the composed matrices off that
/// crossing instead of holding raw pointers.
```

`pipeline3d.rs`:
```
/// main arm): the bone matrix per weight comes off the entity's
/// `Ghoul2RenderPayload`, which the sim-side builder composed with `EvalRender`
/// (DEC-65 ruling 2). The decoded per-vertex normal rides a parallel `[f32; 4]`
```
```
/// Returns `None` when the model has no mdxm block, when the entity carries no
/// payload, or when the payload holds no model at this ordinal, so the caller
/// counts a decode failure and skips the surface.
```

`tr_ghoul2.rs`:
```
/// the LOD, the surface index, and the payload ordinal, so the backend re-locates
/// the surface and reads the bone matrices off the entity's crossing.
```

All six sit inside pre-existing 80-column doc blocks. The memory rule reads "never infer wrap width from the surrounding file."

### 6.5 A comment that narrates mechanics (`frame_exec.rs`)

```
        // The `Arc` clone is one refcount bump per entity.
```

### 6.6 Doc-comment length against the `///` content rule

The house rule for a method doc is "1-2 lines. What it does, plus the one behavioral gotcha." `build_ghoul2_render_payload` carries four prose lines plus the `Source:` cite:

```
/// Builds the DEC-65 ruling 2 crossing for one scene entity, or `None` when the entity carries no live Ghoul2 instance.
/// The caller must finish this call before it takes any `re`-slot borrow, because the setup path can re-register a model through the `re` hooks.
///
/// This is the sim-side front half of [`r_add_ghoul_surfaces`]: the setup, the transform, and the per-bone evaluation the render side used to run.
/// A stale replay token returns `None`, which the walk reads as an absent instance, exactly as it read a dead handle before.
```

The first two lines are the contract's own doc text verbatim.

### 6.7 The finished file, prose surface

No semicolon, no em dash, no contraction. Two sentences exceed the 25-word cap, 27 and 37 words. The 37-word one:

```
No fixture moved, no golden moved, the payload carried every field the builder needed, no new `pub` item on `mp_engine_ghoul2` was required, both dispatcher arms satisfied the order contract, and every frame-pinned `Arc::clone` stayed where it was.
```

---

# 7. The gate claims, re-run

I ran every claimed gate myself, from the worktree root, on `gh31-step-005-bone-matrices`, each as one foreground command, serial. `JKA_GOLDEN_BLESS` was unset in every run. `git status --porcelain` was empty before and after every golden run, so no fixture or golden was written.

| Claim in the finished file | My real output |
|---|---|
| `cargo build --workspace`: green | `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 6.71s`. No warning, no error. **Matches.** |
| `cargo test --workspace`: green | 517 passed, 0 failed, 0 `test result: FAILED`. **Matches.** |
| `cargo test -p mp_renderer_gpu --test world_golden -- --ignored --test-threads=1`: 2 passed, byte-identical | `test golden_world_duel1 ... ok` / `test golden_world_ffa2 ... ok` / `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 38.62s`. **Matches.** |
| `--test ghoul2_vertex_golden -- --ignored --test-threads=1`: 1 passed, byte-identical | `test golden_ghoul2_verts_stormtrooper ... ok` / `test result: ok. 1 passed; 0 failed; ... finished in 15.25s`. **Matches.** |
| `--test entity_golden -- --ignored --test-threads=1`: 1 passed, `entity_duel1.png` byte-identical at `CHANNEL_TOLERANCE = 0` | `test golden_entity_duel1 ... ok` / `test result: ok. 1 passed; 0 failed; ... finished in 15.49s`. `CHANNEL_TOLERANCE` is `0` in the file, unchanged by the diff. **Matches.** |
| `--test scene_golden -- --test-threads=1`: 7 passed | 7 tests, all ok, `test result: ok. 7 passed; 0 failed; ... finished in 3.86s`. **Matches.** |
| `cargo check --workspace --all-targets` reports zero warnings | The first run was fully cached, so I touched every `.rs` file in the diff range and re-ran. Full re-check of `mp_renderer`, `mp_engine_server`, `mp_engine_client`, `mp_engine_core`, `mp_renderer_gpu`, `jampgame`, `mp_app`, `mp_client_app`. Grep for `warning` and `error` over the whole output: 0 hits. **Matches.** |
| "the fixture's fifth configuration" | The `.bin` fixture does not appear in `git diff master..HEAD --name-only`. The test's compare path ran, not its bless path. The byte-identical claim holds. **Matches.** |
| "no commit touches `mp_game`, the server, or any `jampded` link-set crate" | No `crates/mp/game/**`, `crates/mp/engine/server/**`, `crates/mp/engine/qcommon/**`, or `crates/mp/engine/botlib/**` file is in the diff. **Matches.** |
| "No caller of the deleted `set_ghoul2`/`ghoul2_mut` remains anywhere in the workspace" | Grep over `crates/` and `tools/` for `set_ghoul2` and `ghoul2_mut` returns only `set_ghoul2_capture`, a different symbol. **Matches.** |

Every gate claim in the finished file reproduces.

---

# 8. The unverified list

Named plainly. I could not check any of these mechanically, and I assume nothing about them.

1. **Ruling A's transform-before-cull divergence.** No fixed-scene golden culls a Ghoul2 entity, so no gate observes the bone caches' `mLastTouch`/`mLastLerp` advancing for entities the render cull would skip. Live play is the only remaining check. The finished file lists this as an open gap.
2. **The eager `was_rendered` marking.** `CBoneCache::was_rendered` has no test observer. The finished file lists this as an open gap.
3. **The `re`-slot aliasing soundness of the order contract.** Both dispatcher arms take a `g2` cast and then pass `view` itself into the builder, which is the contracted shape. I ran no miri, no sanitizer, and no thread-safety tool. The order-contract argument is a reading of the source, not a machine result.
4. **The two live dispatcher arms themselves.** No gate exercises `CG_R_ADDREFENTITYTOSCENE` or `UI_R_ADDREFENTITYTOSCENE`. Both goldens and the harness call `RE_AddRefEntityToScene` directly and build the payload by hand. The arms compile; nothing runs them.
5. **The `boot.rs` spike.** No test drives `load_world_and_render`. The empty `payloads` vector and the undrained registry are unexercised.
6. **The `r_noServerGhoul2` nonzero path.** Every build site in the tree passes `0` or a snapshot whose default is `0`. Item 3.1's changed cull-and-lighting behavior therefore runs on no gate.
7. **The `entity_num`-to-`payloads`-index correspondence in `decode_ghoul2_surface`.** I read that `render_world` builds both slices from the same window and that `R_DecomposeSort` yields the entity number, but I ran no test that would fail if the two indices diverged for a multi-scene frame or a mirror view.
8. **Release-profile behavior.** Every gate ran on the debug profile. No release run.
9. **ILP32 / i686.** No cross-check ran. Per the project memory, no local i686 `cargo check` exists on this machine.
10. **The lockstep referee.** Not run. The exemption claim holds by file list, but the referee itself was not executed.
11. **`clippy`.** Not part of the packet's gates, and not run.

## Rulings

**2026-08-05 - the lane-review disposition.** The findings closed as: fix the style items in one commit on the branch, record the accepted behavior findings as a packet Amendment, then merge per the packet's disposition. The dispositions live in the Amendments section of `.claude/packets/31/step-005/packet.md` (the empty-payload cull consequence of ruling B accepted, the guard subject change accepted as output-identical, the `mod.rs` enumeration gap accepted, the `Gates:` trailer letter edge accepted for these commits with a process note for future packets, the snapshot-field orphan noted without action, and the clerk's "seam" flag discarded as repo vocabulary).

## Follow-ups

**2026-08-05 - the style commit.** `a3501348` closed findings 6.1, 6.3, 6.4, and 6.5: one word, one semicolon, seven re-breaks to one sentence per line, one deleted mechanics line. Comments only, `cargo check --workspace --all-targets` clean. Findings 6.6 and 6.7 were accepted without change: two of the four builder doc lines are the contract's own text, and the finished file is the lane's frozen record.

**2026-08-05 - the merge.** `gh31-step-005-bone-matrices` merged to master at `ec7c934c`. DEC-65 ruling 2 carries its execution note in `docs/decisions.md`.
