# Synopsis gh#31 step-010 - the renderfx closure

Ratified 2026-08-29. All eight open rows closed, and the packet body carries the folded shape. Audit: `.claude/packets/31/step-010/audit.md`.

## Intent

This step closes the last dark row of the DEC-54 renderfx census, `RF_DISTORTION`, by porting its whole chain: the post-render deferral list, the per-entity screen capture, and the stage arm that binds the captured square. It also gives the already-live disintegrate pair and volumetric arm their first image goldens, and records the `RF_NOSHADOW` disposition, which needs no port because both oracle reads gate on a stencil-shadow backend this workspace does not have.

## Surface contract

- `MAX_POST_RENDERS` (new const in `mp_renderer::tr_backend`).
- `Gpu::headless_texture` (new accessor).
- `GpuImages::view_bind_group` (new method).
- `execute_package`, `execute_frame`, `render_world`, `Pipeline3d::draw` each gain `target_texture`.
- `Pipeline3d::screen_image` (new private field) and `ScreenImage` (new private type).
- `StageDrawItem::post_render_ent` and `StageDrawItem::screen_image` (new fields).
- `world_coord_to_screen_coord_float`, `world_coord_to_screen_coord`, `screen_capture_rect`, `capture_screen_image` (new private free fns).
- The per-surface post-render partition, one render pass per deferred surface, the screen bind, the extended bind-group cache key, and the `build_dynamic_block` `v` flip.
- The `RF_DISTORTION` marker and the whole `Warned::Distortion` arm deleted.
- Four `RF_NOSHADOW` deferral notes converted to `//TODO: Port` markers.
- `scene_distortion` with its test, `golden_entity_renderfx_duel1`, and their two PNGs.

Anything not on this list is out of scope. No shadow backend, no `RB_DistortionFill`, no dynamic glow, no WGSL shader, no ABI change, no cvar, no `FrameEvent` variant, no new crate.

## Commits

1. `feat(gh#31 s010): the frame target texture reaches the backend` - the new parameter across twelve sites, no draw changes.
2. `feat(gh#31 s010): the post-render deferral` - the per-surface partition, the LIFO reversal, one pass per deferred surface.
3. `feat(gh#31 s010): the per-entity screen capture` - the screen image, the keep-alive vector, each capture encoded before its own surface's pass.
4. `feat(gh#31 s010): the RF_DISTORTION stage arm` - the bind, the `v` flip, the marker deletion, the warning removal.
5. `test(gh#31 s010): the distortion golden` - one new PNG after its bless STOP.
6. `test(gh#31 s010): the disintegrate and volumetric golden` - one new PNG after its bless STOP.
7. `docs(gh#31 s010): the RF_NOSHADOW disposition` - comment text only, four sites.
8. `process(gh#31 s010): finished file`.

Every commit gates on `cargo build --workspace`, `cargo test --workspace -- --test-threads=1`, and the five golden suites run serially. Commit 1 may carry one unused-parameter warning, and the final state builds at zero.

## The settled rows

1. **The `RF_DISTORTION` scope, as proposed.** The census counts 2,125 submissions, so DEC-54's complement rule does not cover it. The whole chain lands now.
2. **The target texture reaches the backend, as proposed.** A `target_texture: &wgpu::Texture` parameter on four functions plus a `Gpu::headless_texture` accessor. Twelve sites change, five in test files, three of those files otherwise unedited.
3. **The screen-image texture shape, amended.** One square texture on `Pipeline3d`, rebuilt on a side-length change, filling per deferred surface: capture, draw that surface, capture the next. One slot serves any number of distortion entities. `capture_screen_image` moves a replaced texture into a keep-alive vector held until the submit. `build_dynamic_block` flips `v` on a screen-image stage, which cancels the opposite row order the two worlds store a copied rect in and restores parity.
4. **The two goldens, amended.** Both as drafted, through the step-007 bless procedure with a STOP and a named defect condition each. The synthetic distortion sprite sits well off the horizontal centre, because a centred sprite captures the same square with or without the mirror term. Its defect condition names the mirrored screen position as the expected content.
5. **`scene_renderfx_tint.png` under the deferral, as proposed.** The deferral lands as written, and a moved golden is a STOP plus an eyes-on bless.
6. **The distortion stage's GL state, amended.** The oracle's state chain is separate from its bind chain, and a plain `RF_DISTORTION` stage reaches the default `GL_State( stateBits )` arm, so drawing under the stage's own state is parity. Divergence note 4 is deleted and no site note is written.
7. **The post-render replay, amended.** The per-surface partition, per-surface stage order, LIFO reversal, and surface cap stand. The replay adds the oracle's capture-and-draw interleave: one render pass per deferred surface, so capture N sees deferred draws 1 through N-1. The drafted all-captures-then-one-pass shape would have left every distortion stage sampling the last capture of a frame holding no deferred draw.
8. **The `RF_NOSHADOW` disposition, amended.** No port. Four marker sites: the write-site note at `tr_ghoul2.rs:2459-2462`, and the reads at `tr_mesh.rs:512-514`, `tr_ghoul2.rs:840-842`, and `:909-911`. The asymmetry note scopes to the ghoul2 reads, because SP's md3 projection arm does not test the flag either.

Rows 9 and 10 were cleared by the audit and not walked: transcribe `lastPostEnt` as the run compare the oracle writes, not a per-frame set, and home `MAX_POST_RENDERS` in `mp_renderer::tr_backend`.

## Dispatch flags

- Oracle ambiguity: **true**. A zero or oversized capture radius is undefined, the horizontal mirror in the capture rect has no justification in the code, and the `tr.distortionShader` branches and the full-screen capture are dead or unreachable.
- New state home: **true**. `Pipeline3d` gains a persistent screen-image texture, and `MAX_POST_RENDERS` lands in `mp_renderer::tr_backend`.
- ABI or parity-gate surface: **true**. Two new committed goldens join the gate battery, one existing golden may need a re-bless, and four functions change signature across twelve sites. No ABI change.
- Divergence proposal: **true**. Three site notes survive the walk: the zero-radius skip, the oversized-radius clamp, and the preserved mirror. The `v` flip is parity, not a divergence, and row 6 deleted the fourth note.
