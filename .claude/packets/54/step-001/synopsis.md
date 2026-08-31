# Synopsis gh#54 step-001 - the weather group

Ratified 2026-08-31. All twelve rows closed. The packet body carries the folded shape. Audit: `.claude/packets/54/step-001/audit.md` (`53ec6a62`).

## Intent

This step makes rain and snow run and draw, the first of gh#54's five groups. It closes all five `TODO: Port` markers in `crates/mp/renderer/src/tr_worldeffects/world_effects.rs`, then adds the two things the markers do not cover: a per-frame caller and a GPU pass.

The survey found the honest shape. A weather map already builds real clouds with real textures on this client, because `R_WorldEffectCommand` and the whole cgame chain are live. Nothing steps them and nothing draws them: `RB_WorldEffects` has zero callers, `RE_RenderWorldEffects` does not exist, `Render` is a counter, and `crates/mp/renderer-gpu/` has no weather code at all. Three of the four type-stub markers are stale, and the fourth is mooted by row 2's refdef source rather than stale.

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

Anything not on this list is out of scope. No point-sprite path, no `mGLModeEnum`, no `CWorldEffect` family, no `r_we` registration, no dead-symbol deletion, no `PipelineKey` cull field, no new WGSL, no cvar, no ABI change, no new crate.

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

## The settled rows

1. **The step boundary, ruled.** One step for the whole chain. Closing the five markers alone leaves a cloud that computes and draws nothing, proved by nothing. The commit 1 through 4 split point stays on record as history.
2. **Where the step runs, with row 5 merged, amended.** Trap-side in the `RE_RenderScene` arm, gated on the submitted scene's own `rdflags` carrying neither `RDF_NOWORLDMODEL` nor `RDF_SKYBOXPORTAL`, with the batch crossing as a `FrameEvent`. The gate keeps once-per-frame semantics: icon and sky-portal scenes never step. **Divergence 4 named and ruled cosmetic**: retail's backend command reads the last scene's refdef, so a 3D icon freezes and hides all weather for that frame, and the port steps and draws once. The two-refdef collapse is recorded inside that divergence, not dismissed. The precedent is DEC-65 ruling 2.
3. **`WindZoneState`'s owner, as drafted.** A `wind` field on `WorldEffectsState`, beside the other statics from the same Raven TU.
4. **The two generators, as drafted.** One, `self.rng`, per DEC-66 ruling 1. `Rng` already carries Raven's two streams separately.
5. **The view orientation's source.** Merged into row 2 and ruled there.
6. **`SetViewportAndScissor`, as drafted.** Retire the no-op call and keep a note. `draw_weather` sets the viewport from the view it takes.
7. **The live gate, amended, recipe replaced whole.** The golden boots `maps/mp/ctf2.bsp` and issues the retail triple `snow`, `fog`, `constantwind (100 100 -100)`. It reseeds **both** `Rng` streams to fixed constants after the commands and before stepping. It submits the scene through roughly sixty fixed-dt steps with `fd.time` advancing per call, so `frametime` is real and the fade reaches its 0.75 ceiling before capture. Defect conditions: visible developed particles and the fog overlay, an image with no discernible weather is the defect, the twenty-one existing fixtures byte-identical, eyes-on bless STOP. The determinism rule is minted, not borrowed from DEC-66 ruling 4.
8. **The nearest sampler, amended.** The sampler and the filter plumb land as contracted, and rain is live from commit 6. The gate claim corrects: snow leaves `mFilterMode` at 0, so the ctf2 golden exercises the linear path only and `sampler_nearest` has no automated gate. The finished file records it and live play verifies it.
9. **The broadcast rand draws, as drafted.** One draw per line at `:1216-1217`, broadcast to all three components.
10. **The colour quantization, as drafted.** The payload keeps the `f32` colour and the executor rounds once. The rule is pinned: clamp to `[0, 1]`, times 255, round to nearest, cast.
11. **The uninitialized `mMax`, amended.** No code change. The note rewrites to the proven fact: the reversed range gives a uniform `[1, 2000]` through `irand`'s `max++` and signed shift, and the clamp is dead. The false "reproduces the oracle's zero" sentence is deleted.
12. **The contract gaps and corrections, amended.** `cull_mode: None` is faithful, cited to `GL_Cull(CT_TWO_SIDED)`, with no `PipelineKey` cull field this step. `WeatherFrame` carries no view and states the positional invariant instead. The eleven cite corrections land, plus three fact corrections: the `orientationr_t` marker is mooted not stale, `CHANNEL_TOLERANCE` is zero in four suites not five, and ctf2 carries two `fx_snow` entities whose `count` key snow never reads.

The DEC-66 determinism amendment is minted in the packet's Amendments section and lands in `docs/decisions.md` when the step merges.

## Dispatch flags

- Oracle ambiguity: **true**. Raven reads `rdflags` from two different refdefs, reads an uninitialized range bound, and hides a one-draw broadcast behind what looks like a per-component multiply.
- New state home: **true**. `WorldEffectsState` gains the orphan wind trio, and a new seam payload family crosses to the render thread.
- ABI or parity-gate surface: **true**. One new committed golden joins the battery, one new `FrameEvent` variant crosses the seam, and four functions change signature. No ABI change.
- Divergence proposal: **true**. Three named divergences: the compute runs sim-side and the draw runs render-side, the vertex colour quantizes to bytes, and divergence 4, the per-scene `rdflags` gate that drops retail's last-scene weather freeze.
