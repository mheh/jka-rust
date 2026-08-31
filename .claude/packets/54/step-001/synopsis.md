# Synopsis gh#54 step-001 - the weather group

## Intent

This step makes rain and snow run and draw, the first of gh#54's five groups. It closes all five `TODO: Port` markers in `crates/mp/renderer/src/tr_worldeffects/world_effects.rs`, then adds the two things the markers do not cover: a per-frame caller and a GPU pass.

The survey found the honest shape. A weather map already builds real clouds with real textures on this client, because `R_WorldEffectCommand` and the whole cgame chain are live. Nothing steps them and nothing draws them: `RB_WorldEffects` has zero callers, `RE_RenderWorldEffects` does not exist, `Render` is a counter, and `crates/mp/renderer-gpu/` has no weather code at all. All four type stubs are stale marker comments, because every value they wait on is already on a landed struct.

## Surface contract

- `WeatherVertex`, `WeatherCloudBatch`, `WeatherFrame` (new file `render_state/weather_frame.rs`).
- `WorldEffectsState::wind` (new field, the orphan `WindZoneState`'s home).
- `FrameEvent::WorldEffects` (new variant).
- `RE_RenderWorldEffects` (new fn in `tr_cmds.rs`).
- `Pipeline3d::draw_weather` (new method).
- `GpuImages::sampler_nearest` and `GpuImages::weather_bind_group` (new).
- `WorldStats::weather_vertices` (new counter).
- `CWeatherParticleCloud::Update`, `CWeatherParticleCloud::Render`, `RB_RenderWorldEffects`, `RB_WorldEffects` change signature.
- `golden_world_weather_ctf2` with `world_weather_ctf2.png`.

Anything not on this list is out of scope. No point-sprite path, no `mGLModeEnum`, no `CWorldEffect` family, no `r_we` registration, no dead-symbol deletion, no new WGSL, no cvar, no ABI change, no new crate.

## Commits

1. `fix(gh#54 s001): the four world-effects reads land` - the four type stubs, from the refdef and the world bmodel.
2. `feat(gh#54 s001): the weather particle loop` - the fifth marker, plus unit tests.
3. `feat(gh#54 s001): the wind zone state finds its owner` - the `wind` field and the three signature changes.
4. `feat(gh#54 s001): the cloud renders into a frame batch` - the seam types and the `Render` body.
5. `feat(gh#54 s001): the weather pass draws a frame batch` - the event, the executor arm, the pass, the sampler.
6. `feat(gh#54 s001): the frame drives world effects` - `RE_RenderWorldEffects` and the two trap arms. Live from here.
7. `test(gh#54 s001): the weather world golden` - one new PNG after its bless STOP.
8. `process(gh#54 s001): finished file`.

Every commit gates on `cargo build --workspace`, `cargo test --workspace -- --test-threads=1`, and the five golden suites run serially. All twenty-one committed fixtures stay byte-identical through every commit. The lockstep referee is not a gate.

## Open rows

1. **The step boundary** - user ruling. Default: one step for the whole chain. Closing the five markers alone leaves a cloud that computes and draws nothing, with no image gate. The bundle is ordered so commits 1-4 are a clean split point if the user prefers two steps.
2. **Where the weather step runs** - user ruling. Default: trap-side, right after `RE_RenderScene`, with the batch crossing as a `FrameEvent`. `execute_frame` has no host, no collision world, and no mutable renderer state, and `COutside::Cache` needs all three.
3. **`WindZoneState`'s owner** - mechanical. Default: a `wind` field on `WorldEffectsState`, beside the other statics from the same Raven TU.
4. **The two generators** - mechanical. Default: one, `self.rng`. `Rng` already carries Raven's two streams separately, and the `rng` parameter has no caller.
5. **The view orientation's source** - mechanical. Default: `refdef.view_origin` and `refdef.view_axis`, which is exactly what fills `viewParms.ori`. The same row collapses Raven's two-refdef `rdflags` read into one, with a note.
6. **`SetViewportAndScissor` at this site** - mechanical. Default: retire the no-op call and keep a note. It is the only reason the fn takes a `FrameState`.
7. **The live gate** - user ruling. Default: a world golden on `maps/mp/ctf2.bsp`, the one stock MP map with weather, driven by a scripted `snow` command with a test-only fixed seed and rendered on the second weather frame. This exercises DEC-66 ruling 4's own reseed clause.
8. **The nearest sampler** - mechanical. Default: a third sampler on `GpuImages`. Every rain preset sets `mFilterMode = 1`, so `GL_NEAREST` is live.
9. **The broadcast rand draws** - mechanical. Default: one draw per line at the spawn-plane respawn, broadcast to all three components. `CVec3` has no scalar multiply, so a per-component write would triple the draws and shift the stream.
10. **The colour quantization** - mechanical. Default: the seam payload keeps the `f32` colour and the executor rounds to `WorldVertex`'s `[u8; 4]` at one named site.
11. **The uninitialized `mRotationChangeTimer.mMax`** - mechanical. Default: no code change. The port already reproduces the oracle's zero, so only a note is owed.

## Dispatch flags

- Oracle ambiguity: **true**. Raven reads `rdflags` from two different refdefs, reads an uninitialized range bound, and hides a one-draw broadcast behind what looks like a per-component multiply.
- New state home: **true**. `WorldEffectsState` gains the orphan wind trio, and a new seam payload family crosses to the render thread.
- ABI or parity-gate surface: **true**. One new committed golden joins the battery, one new `FrameEvent` variant crosses the seam, and four functions change signature. No ABI change.
- Divergence proposal: **true**. Three named divergences: the compute runs sim-side and the draw runs render-side, the two refdef reads collapse into one, and the vertex colour quantizes to bytes.
