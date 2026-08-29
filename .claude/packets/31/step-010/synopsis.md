# Synopsis gh#31 step-010 - the renderfx closure

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
- The post-render partition, the second render pass, the screen bind, and the extended bind-group cache key.
- The `RF_DISTORTION` marker and the whole `Warned::Distortion` arm deleted.
- The two `RF_NOSHADOW` deferral notes converted to `//TODO: Port` markers.
- `scene_distortion` with its test, `golden_entity_renderfx_duel1`, and their two PNGs.

Anything not on this list is out of scope. No shadow backend, no `RB_DistortionFill`, no dynamic glow, no ABI change, no cvar, no `FrameEvent` variant, no new crate.

## Commits

1. `feat(gh#31 s010): the frame target texture reaches the backend` - the new parameter and every call site, no draw changes.
2. `feat(gh#31 s010): the post-render deferral` - the partition, the LIFO reversal, the second pass.
3. `feat(gh#31 s010): the per-entity screen capture` - the screen image and the copy between the two passes.
4. `feat(gh#31 s010): the RF_DISTORTION stage arm` - the bind, the marker deletion, the warning removal.
5. `test(gh#31 s010): the distortion golden` - one new PNG after its bless STOP.
6. `test(gh#31 s010): the disintegrate and volumetric golden` - one new PNG after its bless STOP.
7. `docs(gh#31 s010): the RF_NOSHADOW disposition` - comment text only.
8. `process(gh#31 s010): finished file`.

Every commit gates on `cargo build --workspace`, `cargo test --workspace -- --test-threads=1`, and the five golden suites run serially. Commit 1 may carry one unused-parameter warning, and the final state builds at zero.

## Open rows

1. **user ruling - the `RF_DISTORTION` scope.** The census counts 2,125 submissions, so DEC-54's complement rule does not cover it. Default: land the whole chain now, since the capture is meaningless without the deferral and the bind is meaningless without the capture.
2. **user ruling - the target texture reaches the backend.** `Pipeline3d::draw` cannot copy out of a `TextureView`. Default: add a `target_texture: &wgpu::Texture` parameter to four functions and a `Gpu::headless_texture` accessor, touching ten call sites including four test files.
3. **user ruling, new design - the screen-image texture shape.** No oracle equivalent, since `qglCopyTexImage2D` re-specifies its texture per capture. Default: one square texture on `Pipeline3d`, rebuilt on a side-length change, filled by `copy_texture_to_texture` between two passes.
4. **user ruling - the two goldens.** Default: `scene_distortion.png` synthetic and `entity_renderfx_duel1.png` retail-`#[ignore]`, each through the step-007 bless procedure with a STOP and a named defect condition. The retail one is what gates the vert deform and the volumetric fade, which need normals a sprite does not have.
5. **user ruling - `scene_renderfx_tint.png` under the deferral.** The list also defers `RF_FORCE_ENT_ALPHA`, which that golden already gates. Default: the deferral lands as written, and a moved golden is a STOP plus a row-4 bless.
6. **mechanical - the distortion stage's inherited GL state.** The oracle's arm skips `GL_State`, so it inherits the previous stage's. Default: draw under the stage's own state and record the gap in a two-line site note.
7. **mechanical - the post-render partition and the LIFO order.** Default: partition at the surface, keep each surface's stage order, reverse the order of the deferred surfaces, and cap by surface count.
8. **user ruling - the `RF_NOSHADOW` disposition.** Default: no port. Both reads need `r_shadows` 2 against a retail default of 1, and no shadow backend exists. Convert the two notes to markers and record that MP's projection-shadow add does not test the flag while SP's does.
9. **mechanical - the capture dedup and its retry.** Default: transcribe `lastPostEnt` as written, set only after a successful copy, reset per frame.
10. **mechanical - `MAX_POST_RENDERS`'s home.** Default: `mp_renderer::tr_backend`, the step-009 precedent for `LIGHTNING_RECURSION_LEVEL`.

## Dispatch flags

- Oracle ambiguity: **true**. A zero or oversized capture radius is undefined, the horizontal mirror in the capture rect has no justification in the code, the `tr.distortionShader` branches and the full-screen capture are dead or unreachable, and the distortion stage inherits an unset GL state.
- New state home: **true**. `Pipeline3d` gains a persistent screen-image texture, and `MAX_POST_RENDERS` lands in `mp_renderer::tr_backend`.
- ABI or parity-gate surface: **true**. Two new committed goldens join the gate battery, one existing golden may need a re-bless, and four functions change signature across ten call sites. No ABI change.
- Divergence proposal: **true**. Four site notes: the zero-radius skip, the oversized-radius clamp, the preserved mirror, and the stage's own GL state in place of the oracle's inherited one.
