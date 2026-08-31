# Finished - gh#54 step-001, the weather group

The lane ran the whole eight-commit bundle on branch `gh54-step-001-weather`. `git merge master --no-gpg-sign` was the first act and reported already up to date. Nothing is pushed, and no pull request is open.

## Assumptions and choices, keyed to their commits

### Commit 1 - the four stubbed reads

`RB_RenderWorldEffects` reads `frame.refdef` rather than taking a new parameter. `FrameState` carries a real `TrRefdef` (`crates/mp/renderer/src/render_state/frame_state.rs:21`), so every value the three stale markers waited on was already reachable through the parameter the function held. This kept commit 1 inside `world_effects.rs` alone, as its file list says, and left the `frame` to `refdef` parameter swap to commit 3 with `tr_backend.rs`.

The `orientationr_t` marker is recorded as moot, not stale, per row 12. The placeholder `ViewParms` still carries no `ori` field. `Update` takes the view origin and axis from the refdef instead, which is exact: `RE_RenderScene` fills `parms.ori` from the scene refdef at `oracle/codemp/renderer/tr_scene.cpp:848-851`.

The world bounds read is `assets.world.as_ref().and_then(|w| w.bmodels.first()).map(|b| b.bounds)`. Raven indexes `bmodels[0]` with no length test, which reads out of bounds for a world with no submodel. Porting rule 19 picks `None` for that case, which leaves the cache unbuilt rather than panicking. The note sits at the site.

### Commit 2 - the particle loop

Raven's `ratl::bits_vs` `get_bit`, `set_bit` and `clear_bit` become mask tests and mask updates on the existing `u32` field. Three local mask constants sit above the loop rather than new methods on `CWeatherParticle`, so the transcription adds no surface the contract does not list.

Both dead stores are transcribed and both need a lint allow, each carrying a note that says why the store is dead. `partInRange` at `oracle/codemp/renderer/tr_WorldEffects.cpp:1226` is the one the packet named. `partRendering = false` at `:1275` is its twin, and the compiler found it: the fade-out arm writes the local, and the render count below re-reads the flag instead.

The row-9 broadcast is transcribed by hoisting each `WE_flrand` draw into a named local and scaling all three components by it. This is the opposite of the `VectorMA` multi-eval rule, and the note at the site says so with the `CVec.h:570,628` cite.

`Update` carries `#[allow(clippy::too_many_arguments)]`, matching `Initialize` and `R_WorldEffectCommand` in the same file.

### Commit 3 - the wind trio

`tr_surfacesprites.rs` needed no edit. See the deviations below.

The `SetViewportAndScissor` retirement leaves a note with its `Source:` cite at the call site, per row 6. The `qglLoadMatrixf` `DEFERRED` note beside it stays, because `draw_weather` supplies the world clip matrix on the GPU side.

### Commit 4 - the batch

`GLS_ALPHA` is not a ported constant. `Render` composes it inline from `GLS_SRCBLEND_SRC_ALPHA | GLS_DSTBLEND_ONE_MINUS_SRC_ALPHA`, with a cite to `oracle/codemp/renderer/tr_local.h:1683`. This adds no public surface.

A `GL_QUADS` quad becomes two triangles, `0, 1, 2` and `0, 2, 3`, because both world pipeline builders hard-code `TriangleList`. The four corners keep Raven's own order and UVs.

Every early return in `RB_RenderWorldEffects` returns an empty `WeatherFrame`, including the cache-building frame, which updates nothing and draws nothing in the oracle too.

### Commit 5 - the GPU pass

The executor arm rebuilds the view rather than carrying one, and it needs no new `FrameExecutor` field. The walk keeps the scene refdef in a local as it passes `FrameEvent::RenderScene`, then derives both matrices: `R_RotateForViewer` off the refdef's origin and axis, and `R_SetupProjection` off `self.view_state.view.vis_bounds`, which `R_RenderView` publishes at `crates/mp/renderer/src/tr_main.rs:2744` and copies back into the ABI view at `:2251`. Both matrices come out identical to the world pass's.

`draw_weather` writes the globals and flags buffers after the world pass submitted its own encoder, so reusing both buffers is safe. The pass gates on `r_skipBackEnd`, matching `render_world`'s own backend-submit gate. A `WorldEffects` event with no scene ahead of it warns once through the executor's existing `Warned::Other` slot rather than passing silently.

Three private helpers land inside `pipeline3d.rs` that the surface contract does not name, because they are implementation detail rather than surface: `WeatherRun`, the resolved per-cloud GPU form; `world_vertex_from_weather`, the one quantization site; and `quantize_color_channel`. The quantization follows row 10 exactly: clamp to zero through one, scale by 255, round to nearest, cast.

### Commit 6 - the frame drive

`RE_RenderWorldEffects` pushes the `WorldEffects` event unconditionally, matching Raven's unconditional `RC_WORLD_EFFECTS` marker queue. `WeatherFrame::is_empty` earns its keep on the executor arm, which is what the contract describes.

Each trap arm takes the refdef by cloning it off the frame's last event. `RE_RenderScene` pushes `FrameEvent::RenderScene` unconditionally near its end (`crates/mp/renderer/src/tr_scene.rs:1327`) and `TrRefdef` is `Clone`. The clone lands in its own statement so the shared borrow on `frame_data` ends before the mutable borrow the call needs. `RE_RenderScene` keeps its signature.

### Commit 7 - the golden

The fixture is a dedicated runner rather than a new `SceneStep`, because `run_golden_scene` records one scene and the weather recipe needs sixty stepped scenes with an advancing clock. The five existing world goldens run through the untouched shared runner, so the new fixture cannot perturb them.

The step is sixty frames at a fixed thirty-three milliseconds. The `snow` preset fades at `mFade` 10.0 to a `mColor[3]` ceiling of 0.75, and the `fog` preset at 5.0 to 0.2, so both reach their ceiling within a few steps and the remainder develops the particle spread.

## Deviations

Four, all ruled or approved.

1. **`GpuImages::weather_bind_group` gained `layout: &BindGroupLayout` as its second parameter.** The contract omitted it, and the function cannot be written without it: the group binds as group 1 of the world pipeline, whose four-entry layout is private to `Pipeline3d` (`crates/mp/renderer-gpu/src/pipeline3d.rs:936`), and `GpuImages` owns only the two-entry 2D layout. Both siblings that build a world-pipeline group already take the layout the same way (`gpu_images.rs:348,391`). The lane stopped without writing and the user ruled it as proposed on 2026-08-31. It is a packet Amendment and commit `0dfed259`.

2. **The fixture reseed lands twice, not once.** Row 7 step 2 named one reseed point, after the weather commands and before stepping, and that point cannot pin the fixture. `CWeatherParticleCloud::Initialize` picks every particle's `mMass` off the C runtime stream (`oracle/codemp/renderer/tr_WorldEffects.cpp:928-935`, ported at `crates/mp/renderer/src/tr_worldeffects/world_effects.rs:1050`), and `Initialize` runs inside `R_WorldEffectCommand`, so the two commands take 1060 draws ahead of the ratified reseed point. Three isolated runs under the single reseed gave 2076, 2096 and 2104 weather vertices, and two comparisons against the third differed by 71,550 and 133,748 pixels. The lane stopped, removed the non-reproducible blessed PNG, and the user ruled the double reseed on 2026-08-31. It is a packet Amendment and commit `b7d9f27d`. The DEC-66 amendment text in the packet was corrected to the same shape in that commit.

3. **Commit 3 touched no `tr_surfacesprites.rs`, a narrowing of its file list.** Row 3 says the two call sites keep their `&WindZoneState` parameter, and they do. Neither `R_SurfaceSpriteFrameUpdate` nor `RB_DrawSurfaceSprites` has a caller anywhere in the workspace, so nothing there broke and nothing needed editing. The session approved the narrowing as a record rather than a stop.

4. **One process slip, self-caught and fully undone.** While adding the `RE_RenderWorldEffects` import to `crates/mp/engine/client/src/cl_cgame.rs`, the lane applied one change with a python script, which the standing rule forbids. The lane caught it immediately, reverted the whole file with `git checkout --`, and redid both edits with the Edit tool. No committed content is affected, and no other file was ever touched by a script.

## The commit list, with gate results

The full battery is: `cargo build --workspace` with zero warnings; `cargo test --workspace -- --test-threads=1`; the world goldens with `--ignored --test-threads=1`; the scene goldens with `--test-threads=1`; the entity goldens with `--ignored --test-threads=1`; the ghoul2 vertex golden with `--ignored --test-threads=1`; and the hud goldens both plain and `--ignored`, all `--test-threads=1`. Every run was serial and foreground. The lockstep referee is not a gate here, because no commit touches `mp_game`, the server, or any `jampded` link-set crate.

| Commit | Subject | Gate result |
|---|---|---|
| `0dfed259` | `process(gh#54 s001): the bind-group layout amendment` | Packet text only, no gate |
| `7f8eebbb` | `fix(gh#54 s001): the four world-effects reads land` | Full battery green, 21 of 21 fixtures byte-identical |
| `67e87764` | `feat(gh#54 s001): the weather particle loop` | Full battery green, 21 of 21 fixtures byte-identical |
| `f7af2115` | `feat(gh#54 s001): the wind zone state finds its owner` | Full battery green, 21 of 21 fixtures byte-identical |
| `580ddf97` | `feat(gh#54 s001): the cloud renders into a frame batch` | Full battery green, 21 of 21 fixtures byte-identical |
| `20195f41` | `feat(gh#54 s001): the weather pass draws a frame batch` | Full battery green, 21 of 21 fixtures byte-identical |
| `6de7631c` | `feat(gh#54 s001): the frame drives world effects` | Full battery green, 21 of 21 fixtures byte-identical |
| `b7d9f27d` | `process(gh#54 s001): the reseed amendment` | Packet text only, no gate |
| `51183d1b` | `process(gh#54 s001): the row-7 bless amendment` | Packet text only, no gate |
| `3830ec2f` | `test(gh#54 s001): the weather world golden` | Full battery green, the new golden green at tolerance zero, the 21 pre-existing fixtures byte-identical |
| this file | `process(gh#54 s001): finished file` | Process only, no gate |

The three unit-test groups the bundle asked for all pass: commit 2 covers the row-9 draw count and broadcast, `SVecRange::Wrap` across each axis, and the fade machine's four transitions with the render count's flag re-read; commit 4 covers both vertex arms' offsets and UVs and both blend modes' colour and state bits.

The golden's determinism evidence, required by the reseed Amendment: three isolated runs before the bless each reported 2048 weather vertices, and three isolated comparison runs after the bless each passed at `CHANNEL_TOLERANCE` zero with no `.actual.png` written. The blessed image is `crates/mp/renderer-gpu/tests/goldens/world_weather_ctf2.png`, SHA-256 `4044611f92816317b3c5f94a03bd9c60f04bb61a5906b69771aff9851696aa05`. The user reviewed it in chat and blessed it on 2026-08-31.

The fixture count is now twenty-two: twenty-one PNGs and one BIN.

## Open gaps

**`GpuImages::sampler_nearest` lands with no automated gate.** Row 8 named this. Every rain preset sets `mFilterMode = 1` (`oracle/codemp/renderer/tr_WorldEffects.cpp:1718,1739,1760,1788`), so the nearest path is live in play from commit 6 onward. The `snow` branch never touches `mFilterMode` and keeps `Reset`'s default of 0 (`:988,1798-1817`), so the ctf2 golden exercises the linear path only and never binds `sampler_nearest`. Rain in live play verifies it.

**The golden proves neither zone nor cache behavior.** `boot::load_world` calls `RE_LoadWorldMap` alone and never `CM_LoadMap` (`crates/mp/renderer-gpu/src/ui_host/boot.rs:472-519`), so the collision world stays empty, `CM_PointContents` reads zero at every cell, and `COutside::Cache` falls through to `mCacheInit = true; mMarkedOutside = false`. Every point then tests as outside. The rig also runs no cgame, so no `misc_weather_zone` reaches `R_AddWeatherZone` and `Cache` takes its map-sized fallback zone, where live ctf2 has three brush zones. Both are rig properties, not port divergences. The golden proves the draw path and byte stability.

**Divergence 4 is verified by live play alone.** Raven reads `RDF_NOWORLDMODEL` off `tr.refdef`, the front-end copy, which at backend time holds the last scene's flags. On any retail frame that draws a 3D icon, every weather command that frame returns early, so weather neither steps nor draws for the whole frame. Both icon cvars default on, so that is the ordinary case. The port reads each submitted scene's own refdef and steps once on exactly those frames. The user ruled the difference cosmetic. No gate observes it.

**The DEC-66 amendment still has to reach the ledger.** Its text is in the packet's Amendments section, corrected to the double-reseed shape, and it lands as a dated DEC-66 amendment in `docs/decisions.md` when the step merges.

## Out of scope, untouched, as the contract required

No point-sprite path and no `mGLModeEnum` field. No port of `CWorldEffect`, `CWorldEffectsSystem`, or the header's `SParticle`. No deletion of `R_IsOutside`, `R_IsShaking`, `R_IsOutsideCausingPain`, `R_GetWindGusting`, `R_GetChanceOfSaberFizz`, or `FrameEvent::WorldEffectCommand`. No `r_we` console registration, which is step-004. No cull field on `PipelineKey`. No new WGSL and no shader edits. No cvar, no ABI change, no new third-party crate, and no file touched under `crates/mp/game/`, `crates/mp/cgame/`, `crates/mp/ui/`, `crates/mp/uishared/`, or `crates/sp/`. `oracle/` was read-only throughout, and nothing under `~/Developer/jka/` was written.
