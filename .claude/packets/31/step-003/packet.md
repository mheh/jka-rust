# Packet gh#31 step-003 - the RendererFrontend absorption

## Scope

**The split decision.** The user left open whether the absorption is its own step or the opening commits of one draw-arm packet. This packet scopes the absorption alone as step-003, and the draw-arm migration goes to step-004. The reason is re-bless attribution: the absorption alone moves the ghoul2 fixture's draw-surf order, because real shader indices untie the 22 tied sort keys, and the gh#35 audit's section B correction exists precisely because an experiment once changed two things at a time and the reorder could not be attributed. A combined packet would re-bless under the absorption and the arm migration together, so an unexpected move could not be pinned on either. The counts support the same split: the absorption alone touches 8 files, 15 construction and destructure sites, 56 dot accesses, 16 `host_view` call sites, one soundness mechanism, and a fixture re-bless, a full lane on the step-002 scale, while the draw arms carry two design points the user has not ruled yet. The user overrules at audit if they disagree.

This step executes the prerequisite ruled at the gh#35 close (2026-08-04): `UiHost` owns a real `RendererFrontend`, so the view's `re` slot is seated and Raven's client register path works for the whole frame render. It delivers three things: the absorption itself (the 12 twin fields collapse into one `re: RendererFrontend` field), the seated `re` slot through a new `host_view` parameter with the gh#35 A1 override removed, and the ghoul2 vertex fixture re-blessed under the client path with its content multiset checked against the recorded digests.

The step does not migrate any draw arm or decoder: `r_add_md3_surfaces`, `r_add_ghoul_surfaces`, the `tr_main.rs:2017-2119` dispatch, and the two `pipeline3d.rs` decoders keep their `models: &RenderModels` reads exactly as they are, and that migration is step-004. The step changes no `RE_*` signature, no `RendererFrontend` field, no live-client code, and nothing in `crates/mp/renderer` beyond doc lines. `UiHost.models` stays a sibling field outside `re`, per `renderer_frontend.rs:62-63`.

Ground truth is the survey record `docs/audits/2026-08-04-step-003-draw-arm-survey.md`. Three of its facts shape the contract:

1. The `UiHost` literal's 12 twin seeds equal `RendererFrontend::new()`'s seeds field for field (survey section A), so the absorption changes no seed value.
2. `G2_SetupModelPointers` re-registers on every frame's entity walk, and the client hook calls `Arc::make_mut(&mut re.sim.published)` unconditionally (`hook_install.rs:87`). A seated slot plus the current `&sim.published` argument to `execute_frame` is mutable aliasing, so the frame-pinned clone below is a soundness requirement, not a style choice (survey section B).
3. The only paths that reach the `re` slot are the two ghoul2 registration hooks, so `with_dc`'s 2D paint view keeps a null slot (survey section A).

## Open rulings for the user

**Ruling 1, `frame_data` (binds this packet).** The frontend owns a persistent `FrameData`, the harness builds one per paint, `RE_ClearScene` appends rather than clears, and nothing in `renderer-gpu` calls `RE_EndFrame`, the only drain. The fork: keep per-call construction and leave `re.frame_data` inert, or adopt `re.frame_data` with an explicit clear at frame start. **Recommendation: keep per-call construction.** The harness's working model is an owned per-paint stream that `ui_harness` extracts by `mem::replace`, adopting the field would add a hand-rolled drain that imitates `RE_EndFrame` inside a rig, and per-call construction changes zero behavior. `re.frame_data` then sits inert beside `frame_sink`, `pending_capture`, and `pending_world`, which are already inert in the harness.

**Ruling 2, the frame-pinned registry clone (binds this packet).** The mechanism in the surface contract: each entity-walk caller clones the published `Arc` before the split borrow and passes the pinned deref, so the hook's `Arc::make_mut` clones instead of mutating the allocation the executor is reading. **Recommendation: confirm the pin at the four `execute_frame` sites.** The alternatives are the gh#35 swap-window shape, which the close refuted, or keeping the A1 override, which the close superseded. The cost is one registry deep-clone per frame that re-registers a ghoul2 model, the same copy-on-write cadence the live client's `FramePackage` already pays.

**Ruling 3, the consumption shape (binds step-004, recorded now for an early ruling).** How the migrated arms read the published copy: direct `(Arc<ModelBlock>, usize)` reads at every site, or borrowed view helpers on `PublishedModel` (`md3_ptr(lod)`, `mdxm_view()`, `mdxa_view()`). **Recommendation: the helpers.** The mdxm path already reads through a view built from one pointer (`mdxm_view_of`, `frontend.rs:191-195`), the MD3 path repeats unsafe base-plus-offset casts at three or more sites that one helper would give a single SAFETY home, and the arms already take `assets: &RenderAssets`, so the migration deletes the `models` parameters rather than replacing them (survey section F).

**Ruling 4, the dispatch `model_type` source (binds step-004, recorded now).** The `tr_main.rs:1980` dispatch resolves `model_type` from `BModelTable`, which is built at world load, so a model registered after the world generation resolves `MOD_BAD` and a post-load MD3 silently misses the `MOD_MESH` arm. **Recommendation: step-004 resolves entity `model_type` from the published entry**, which republishes at every `RE_EndFrame` drain, keeping `BModelTable` for the brush-submodel fields it was built for (survey section D).

## Surface contract

**`UiHost` (`crates/mp/renderer-gpu/src/ui_host/state.rs`)** removes the 12 twin pub fields and gains exactly one field in their place:

```rust
/// The renderer's DEC-42.3 carrier bundle, the same struct the live client seats at `Engine.re`.
pub re: RendererFrontend,
```

`models: RenderModels` stays a sibling field outside `re`. `engine`, `ui`, `input`, `stubs`, and `start` are unchanged. The struct doc's flat-owner rationale is updated to state the two-level split borrow.

**`boot::host_view` (`crates/mp/renderer-gpu/src/ui_host/boot.rs:421-442`)** gains one parameter:

```rust
pub fn host_view<'a>(
    common: &'a mut Common,
    cm: &'a mut CollisionWorld,
    sv: *mut (),
    rm: *mut RenderModels,
    re: *mut RendererFrontend,
) -> EngineHostView<'a>
```

Every entity-walk and boot call site seats it from `&mut host.re` (raw pointer taken before the field destructure, the `models_ptr` precedent). `with_dc` passes null with a comment stating why: the 2D paint path reaches no `re` hook, and the dc holds `&mut` borrows into `re`'s fields, so a seated slot there would alias.

**The A1 override is removed**: the `hooks.RE_RegisterModel = hooks.R_RegisterServerModel` line and its comment block (`boot.rs:133-138`). Ghoul2 registration then runs Raven's client path through the seated slot.

**The frame-pinned registry clone** lands at the four `execute_frame` entity-walk call sites (`tests/ghoul2_vertex_golden.rs:469`, `tests/world_golden.rs`, `tests/scene_golden.rs:414`, `src/bin/world_harness.rs:921`): `let pinned = Arc::clone(&host.re.sim.published);` before the split borrow, `&pinned` as the `assets` argument, with a comment naming the aliasing it prevents. `load_world_and_render` (`boot.rs:722`) is exempt and keeps `Arc::make_mut`: its entity list is empty by construction, so the walk never fires the hook, and `R_TerrainInit` needs the `&mut`.

**The ghoul2 fixture re-blesses**: `crates/mp/renderer-gpu/tests/goldens/ghoul2_verts_stormtrooper.bin` is regenerated under the client path. The re-bless is valid only when the fixture still parses to 22 surfaces whose per-surface digest multiset equals the recorded list in `.claude/packets/35/step-001/finished.md`, multiset digest `f3551f6abb73953d71b6d293306a8889914e34c6a467cb11d44fc632fbef7264`. Order is expected to move, content is not. The test's module-doc bless provenance is updated, and the `a2-attempt.patch` draft text may inform the wording.

**Doc lines**: the `host_view` doc paragraph on the `re` slot, the `state.rs` module and type docs, and the `renderer_frontend.rs:66-67` seating note, which currently names only the platform shell and now also names the harness. `.claude/packets/35/step-001/a2-attempt.patch` is deleted in the commit that lands the absorption, because the attempt it preserves is superseded by delivered code.

Anything not on this list is out of scope, and the agent must not add it. No new third-party crate. No new `pub` item beyond the `host_view` parameter. No `RE_*` signature change, no `FrameEvent` variant, no cvar, no trap arm, no `#[repr]` change.

## Pause triggers, named for this step

- Commit 1's ghoul2 golden is not byte-identical with the override still standing. STOP: the mechanical absorption moved behavior it must not move.
- The re-blessed fixture's content multiset does not equal the recorded digests, or the surface count is not 22. STOP: geometry moved under the client path, which contradicts gh#35 audit claim 8.
- Any path outside the two ghoul2 registration hooks turns out to read the seated `re` slot during a harness frame. STOP: the pin-scope argument no longer covers it.
- `with_dc` or any 2D path turns out to need a seated `re` slot. STOP: the null seat is a contract line and its change is a ruling.

## Commit bundle

1. **The mechanical absorption, override standing.** `UiHost.re` replaces the 12 twins, the 15 construction and destructure sites re-spell, the 56 dot accesses become `host.re.*`, `host_view` gains the `re` parameter with all 16 call sites updated, `with_dc` passes null, and the frame-pinned clone lands at the four sites. The A1 override stays, so every fixture is unchanged and this commit proves the mechanics moved nothing. Gates: `cargo build --workspace`, `cargo test --workspace`, both world goldens byte-identical, the scene golden green, and the ghoul2 golden byte-identical, all golden runs with `--test-threads=1`.
2. **The client path and the re-bless.** Remove the A1 override and its comment, delete `a2-attempt.patch`, re-bless the ghoul2 fixture (`JKA_GOLDEN_BLESS=1`), verify the 22-surface content multiset against the recorded digests and record the comparison in the finished file, and update the module-doc provenance plus the contract's doc lines. Gates: `cargo build --workspace`, `cargo test --workspace`, both world goldens byte-identical, the scene golden green, the ghoul2 golden green against the re-blessed fixture, and the multiset comparison recorded.
3. **The finished file**, per the packet skill: assumptions keyed to commits, deviations or the word "none", the commit list with gate results, and open gaps.

Every commit uses `--no-gpg-sign`, a heading subject, an STE body, and no trailer of any kind.

## Write scopes

Branch `gh31-step-003-absorption`, cut from master.

- `crates/mp/renderer-gpu/src/ui_host/` - `state.rs`, `boot.rs`.
- `crates/mp/renderer-gpu/src/bin/` - `ui_harness.rs`, `world_harness.rs`.
- `crates/mp/renderer-gpu/tests/` - `world_golden.rs`, `scene_golden.rs`, `ghoul2_vertex_golden.rs`, and `goldens/ghoul2_verts_stormtrooper.bin` (the re-bless only).
- `crates/mp/renderer/src/renderer_frontend.rs` - doc lines only.
- `.claude/packets/35/step-001/a2-attempt.patch` - deletion only.
- `.claude/packets/31/step-003/` for `finished.md`.
- Any caller `cargo check` shows broken by the `host_view` parameter, edit-only to pass the new argument.

Everything else is read-only, including `oracle/`.

## Disposition

The draft awaits the user audit, and no lane spawns before the approval. After approval and a clean lane-review: merge to master locally. No push, and no pull request.

## Amendments

**2026-08-04 - the draft awaits the user audit.**

**2026-08-04 - the four open rulings are taken.** The user ruled all four at the packet audit, each on the recommendation: (1) `frame_data` stays per-call construction and `re.frame_data` sits inert; (2) the frame-pinned registry clone is confirmed at the four `execute_frame` sites; (3) step-004 reads the published copy through view helpers on `PublishedModel`; (4) step-004 resolves entity `model_type` from the published entry, and `BModelTable` keeps only its brush-submodel fields. Rulings 1 and 2 bind this packet as written in the surface contract. Rulings 3 and 4 bind the step-004 draft and are recorded here for it. The lane go is pending.
