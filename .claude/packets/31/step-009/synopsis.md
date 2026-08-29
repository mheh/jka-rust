# Synopsis gh#31 step-009 - the FX mini-refent backend arms

## Intent

This step lights the three generated entity kinds the FX module submits engine-side, `RT_ORIENTED_QUAD`, `RT_CYLINDER` and `RT_ELECTRICITY`, which the trap census could not see and which draw nothing on the live client today. It also closes the one render-side random draw DEC-66 unblocked but left at a stand-in, `RB_SurfaceSaberGlow`'s hilt radius, and corrects the module doc that wrongly calls the three kinds census-complement fog.

## Surface contract

- `TrSurfaceShapeState::f_count` (new field) and `#[derive(Default)]` on the type.
- `LIGHTNING_RECURSION_LEVEL` (new const in `mp_renderer::tr_surface`).
- `Pipeline3d::rng` and `Pipeline3d::shape` (new private fields).
- `build_entity_geometry` gains `refdef_time`, `rng` and `shape`.
- `do_line2`, `do_cylinder_part`, `create_shape`, `apply_shape`, `do_bolt_seg` (new private free fns).
- The three new match arms, plus the saber-glow hilt expression.
- `scene_fx_oriented_quad`, `scene_fx_cylinder`, `scene_fx_electricity` with their three tests.
- `tests/goldens/scene_fx_oriented_quad.png`, `scene_fx_cylinder.png`, `scene_fx_electricity.png`, and a re-blessed `scene_saber_glow.png`.

Anything not on this list is out of scope. No FX module change, no ABI change, no cvar, no `FrameEvent` variant, no new crate.

## Commits

1. `feat(gh#31 s009): the render-side RNG owner and the shape state` - the state home and the threading, no draw changes.
2. `feat(gh#31 s009): the oriented-quad and cylinder arms` - the two random-free arms and the corrected module doc.
3. `test(gh#31 s009): the oriented-quad and cylinder goldens` - two new PNGs after their bless STOPs.
4. `feat(gh#31 s009): the electricity arm` - the four helpers, the arm, and the DEC-66 note deletions.
5. `feat(gh#31 s009): the saber-glow hilt radius` - the one expression plus the re-blessed `scene_saber_glow.png`.
6. `test(gh#31 s009): the electricity golden` - the third new PNG after its bless STOP.
7. `process(gh#31 s009): finished file`.

Every commit gates on `cargo build --workspace` with zero warnings, `cargo test --workspace -- --test-threads=1`, and every committed fixture byte-identical across the world, scene, entity, ghoul2 and hud suites, each run serially.

## Open rows

1. **user ruling - where the three live arms land.** DEC-66 ruling 2 cites `mp_renderer/tr_surface.rs`, which is the dead `tess` transcription with `todo!()` leaf emitters. Default: land them in `pipeline3d.rs::build_entity_geometry`, the backend every census step feeds and the only path a golden gates.
2. **user ruling - the saber-glow hilt radius.** Default: close it now, since DEC-66 names it, and re-bless `scene_saber_glow.png`. The alternative is to defer it to step-010.
3. **user ruling - the four goldens and the bless flow.** Default: three new scene goldens plus the one re-bless, through the step-007 procedure with a STOP per PNG and a named defect condition for each image.
4. **mechanical - the state home.** Default: `Pipeline3d` owns `rng` and `shape`, `TrSurfaceShapeState` keeps its canonical home and gains `f_count`, and `RendererFrontend::rng` is not reused.
5. **mechanical - the dead fork branch.** Default: transcribe `RF_FORKED` as MP writes it. MP never assigns `f_count`, so the branch never runs, while SP sets it to 3.
6. **mechanical - the `oldorigin` write.** Default: compute a local `end` and never write the entity. This is parity, not a divergence, and takes the saber-glow arm's note shape.
7. **mechanical - the vertex cap on a long bolt.** Default: no cap and no flush, as every other CPU-built arm does. Record the fact in the finished file, and treat an over-cap bolt as a pause trigger.
8. **mechanical - the imports and the census comment.** Default: `Q_random` and `RotatePointAroundVector` from `mp_qshared::shared::q_math`, `Q_crandom` and `MakeNormalVectors` from `native_math::qmath`, and the fog list keeps only `RT_BEAM` and `RT_ORIENTEDLINE`.

## Dispatch flags

- Oracle ambiguity: **true**. `f_count` is never assigned in MP so the fork branch is dead code, `axis[0]` carries three non-axis values, `RB_SurfaceElectricity` writes its own input entity, and a zero `RF_GROW` duration divides by zero.
- New state home: **true**. `Pipeline3d` gains a persistent `Rng` and a `TrSurfaceShapeState`, and `TrSurfaceShapeState` gains `f_count`.
- ABI or parity-gate surface: **true**. Three new committed goldens join the gate battery and one existing golden is re-blessed. No ABI change.
- Divergence proposal: **true**. DEC-66 ruling 3's stream split reaches its first drawing arm here, and the oracle's `oldorigin` write becomes a local.
