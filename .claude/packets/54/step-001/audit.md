# Audit gh#54 step-001 - the weather packet

Audited draft: `.claude/packets/54/step-001/packet.md` at commit `b16e8e3f`, on branch `gh54-step-001-weather`.

The walk read every `oracle/` cite the packet names before it read the draft. It then checked each Rust surface-contract claim against the real files, checked the three named divergences against `docs/decisions.md`, and scanned the retail asset tree for the ctf2 claim.

The packet is strong. Its oracle transcription is accurate, its trap findings are real, and rows 3, 4, 6, 9 and 10 hold as written. Two rows carry defects that would send the lane after a phantom bug, and one divergence is missing from the packet entirely.

## Open rows

### Row 1 - the step boundary (user ruling). CLEARED.

The split cut is real. Commits 1 through 4 add no producer for `FrameEvent::WorldEffects` and leave every fixture byte-identical, so they stand alone and green.

One fact the walk should carry into the ruling. Under the split, step-001's four "stale marker" corrections are proved by nothing, because commit 1's own gate paragraph already concedes it: "Nothing calls this code, so the battery proves the build and nothing more."

### Row 2 - where the weather step runs (user ruling). CHALLENGED, on three grounds.

**The premise holds.** `FrameExecutor::execute_frame` (`crates/mp/renderer-gpu/src/frame_exec.rs:417`) takes `gpu`, `target`, `target_texture`, `frame_data`, `assets`, `world_load`, `uploads`, `gpu_images`, `noise`, `float_time` and `cvars`. There is no `EngineHostView` and no collision world, and `COutside::Cache` needs both: `pub fn Cache(&mut self, host: &mut EngineHostView, world_bmodel_bounds: Option<[vec3_t; 2]>)` (`crates/mp/renderer/src/tr_worldeffects/world_effects.rs:634`), whose scan calls `CM_PointContents` (`oracle/codemp/renderer/tr_WorldEffects.cpp:598,645`). The packet's phrase "no mutable renderer state" is loose, because `execute_frame` does hold `&mut self`, `&mut Gpu` and `&mut GpuImages`. What it lacks is the mutable `WorldEffectsState`. The looseness does not change the conclusion.

**Ground 1: the oracle is once per scene, not once per frame, so the placement does not double-step.** `RE_RenderWorldEffects();` is the last statement of `RE_RenderScene` (`oracle/codemp/renderer/tr_scene.cpp:868`), so a frame with N scenes queues N `RC_WORLD_EFFECTS` markers and the backend steps `Update` N times. The trap-side placement queues one call per `RE_RenderScene` trap arm, which is the same count. The animation rate does not double. The packet never claimed once-per-frame semantics, and it is right not to.

**Ground 2: the real divergence is the `RDF_NOWORLDMODEL` gate, and it is live-observable. The packet must name it as a fourth divergence.** Raven reads that bit from the front-end refdef, not the backend copy:

```c
	if (!tr.world ||
		(tr.refdef.rdflags & RDF_NOWORLDMODEL) ||
		(backEnd.refdef.rdflags & RDF_SKYBOXPORTAL) ||
```

`oracle/codemp/renderer/tr_WorldEffects.cpp:1515-1517`. Every command runs at backend time, after the front end finished every scene, so `tr.refdef` holds the **last** scene's flags. `CG_Draw3DModel` sets `refdef.rdflags = RDF_NOWORLDMODEL;` (`oracle/codemp/cgame/cg_draw.c:485`) and runs from `CG_Draw2D`, after `CG_DrawActive`'s world scene at `oracle/codemp/cgame/cg_draw.c:8573`. So on any retail frame that draws a 3D icon, every weather command that frame returns early and weather neither steps nor draws. The port reads each scene's own refdef, so it steps and draws on exactly those frames. This is not an edge case. `cg_draw3dIcons` and `cg_drawIcons` both default on.

The packet's row 2 says "Nothing observes the difference, because the batch is a pure function of the refdef and the cloud state". That sentence is false for live play, and it must go.

**Ground 3: row 2 cites the wrong precedent.** It argues from DEC-37 ruling 2 and DEC-59 ruling 1. The governing precedent is DEC-65 ruling 2, which already ruled this exact shape: "The transform runs sim-side at scene-add time, and plain per-entity bone matrices cross in the frame package ... Raven transforms at backend draw time, we transform at scene-add ... and the gh#31 image-golden gate verifies the parity." Row 2 should cite it, and should also state where weather differs from it. Bone matrices are stateless per frame, so a timing shift is invisible. `CWeatherParticleCloud::Update` integrates velocity and advances two RNG streams, so a step-count or step-gate difference compounds across frames.

### Row 3 - `WindZoneState`'s owner (mechanical). CLEARED.

The statics are where the packet says:

```c
CVec3		mGlobalWindVelocity;
CVec3		mGlobalWindDirection;
float		mGlobalWindSpeed;
```

`oracle/codemp/renderer/tr_WorldEffects.cpp:73-75`. The packet's range `:73-76` also swallows `int mParticlesRendered;` at `:76`, which is a different static and is threaded as a parameter instead. Correct the range to `:73-75`.

The orphan claim is right and slightly understated. `WindZoneState` (`crates/mp/renderer/src/tr_worldeffects/world_effects.rs:427-438`) appears at three parameter sites, not two: `crates/mp/renderer/src/tr_backend.rs:906`, `crates/mp/renderer/src/tr_surfacesprites.rs:555,918`, and `crates/mp/renderer/src/tr_worldeffects/world_effects.rs:1403`. No struct in the workspace holds one.

### Row 4 - the two generators (mechanical). CLEARED, and it is already ruled.

DEC-66 ruling 1 settles this row outright and the packet does not cite it: "Owner-embedded instances are the `rng.rs` doctrine, and world-effects already runs the pattern renderer-side." Add the cite.

The stream split is right. `Rng` carries `holdrand: HoldrandLcg` (`crates/native/math/src/rng.rs:113`) and `crt_holdrand: u32` (`:121`), and the port's own `WE_flrand` draws from the CRT stream, which matches the oracle:

```rust
pub fn WE_flrand(rng: &mut Rng, min: f32, max: f32) -> f32 {
    ((rng.rand() as f32 * (max - min)) / (RAND_MAX + 1) as f32) + min
}
```

`crates/mp/renderer/src/tr_worldeffects/world_effects.rs:66-68`, against `inline float WE_flrand(float min, float max) { return ((rand() * (max - min)) / (RAND_MAX+1)) + min; }` at `oracle/codemp/renderer/tr_WorldEffects.cpp:13-15`.

### Row 5 - the view orientation's source (mechanical). CLEARED on the orientation, CHALLENGED on the `rdflags` half.

The orientation half is exact. Raven fills `parms.ori` from the scene refdef:

```c
	VectorCopy( fd->vieworg, parms.ori.origin );
	VectorCopy( fd->viewaxis[0], parms.ori.axis[0] );
```

`oracle/codemp/renderer/tr_scene.cpp:848-849`, and the port reproduces it at `crates/mp/renderer-gpu/src/frame_exec.rs:812-813` (`parms.ori.origin = refdef.view_origin; parms.ori.axis = refdef.view_axis;`). Reading the refdef gives the identical values.

The `rdflags` half is misfiled. Row 5 disposes of the two-refdef read in two lines at the site: "The two copies differ only when a scene's command is executed after a later scene overwrote the front-end copy, which the port's one-event-per-scene ordering cannot produce." That sentence describes the exact case retail hits on every 3D-icon frame, and then treats the port's inability to reproduce it as an absence of divergence. It is the divergence. Move it to row 2 as the fourth named divergence, with the `CG_Draw3DModel` evidence above.

### Row 6 - `SetViewportAndScissor` at this site (mechanical). CLEARED, with one contract gap.

The no-op is real:

```rust
pub fn SetViewportAndScissor(_frame: &FrameState) {
    // DEFERRED: R4 — SetViewportAndScissor (see doc comment above) (DEC-37 A13.2)
    // Source: oracle/codemp/renderer/tr_backend.cpp:457-467
}
```

`crates/mp/renderer/src/tr_backend.rs:210-213`. Retiring the call and the `frame: &FrameState` parameter is right.

The gap: the surface contract never says who applies the viewport and scissor for the weather pass. `Pipeline3d::draw_weather` takes `view: &viewParms_t`, and the packet should state that the pass sets its viewport from that view, or that it inherits the world pass's.

### Row 7 - the live gate (user ruling). CHALLENGED. This is the packet's one load-bearing defect.

**Defect 1: the frametime arithmetic is wrong, and the named defect conditions follow it into error.** Row 7 says "`SceneState::last_time` starts at 0, so `frametime` is 12345 clamped to 500 and `mSecondsElapsed` is 0.5", then "Snow sets `mFade = 10`, so `particleFade` is 5.0 and every in-view particle reaches its `mColor[3]` ceiling of 0.75 on the first update."

`RE_RenderScene` advances the carrier on every call:

```rust
    let mut frametime = fd.time - scene.last_time;
    scene.last_time = fd.time;
```

`crates/mp/renderer/src/tr_scene.rs:1248-1249`, faithful to `oracle/codemp/renderer/tr_scene.cpp:741-742`. The rig renders at one frozen clock, `const FROZEN_TIME_MS: i32 = 12345;` (`crates/mp/renderer-gpu/tests/world_golden.rs:65`). So the first weather frame gets `frametime = 500` and does nothing but `mOutside.Cache()`, and the second gets `frametime = 0`, which `RB_RenderWorldEffects` floors at 1.0 ms (`oracle/codemp/renderer/tr_WorldEffects.cpp:1531-1534`). The update that actually spawns runs at `mSecondsElapsed = 0.001`, so `particleFade` is 0.01 and no particle reaches 0.75. It also integrates position at one thousandth of the intended step.

The image is still deterministic, so the gate still gates. The harm is the packet's own defect table: "An image with no snowflake means the cloud computed and did not draw, which is the whole bug." At alpha 0.01 over an additive blend, a near-blank image is the correct result, and the lane would hunt a bug that is not there. Row 7 must either run the two weather frames at two distinct clock values, or restate the arithmetic and the expected image for `mSecondsElapsed = 0.001`.

**Defect 2: the scripted command is not the retail configuration.** `fx_snow` registers three effect strings, not one:

```c
void SP_CreateSnow( gentity_t *ent )
{ 
	G_EffectIndex("*snow");
	G_EffectIndex("*fog");
	G_EffectIndex("*constantwind (100 100 -100)");
}
```

`oracle/codemp/game/g_misc.c:2522-2527`, and the port matches at `crates/mp/game/src/g_misc.rs:2574-2578`. So live ctf2 builds two particle clouds (`snow` at `oracle/codemp/renderer/tr_WorldEffects.cpp:1798` and `fog` at `:1879`) plus one global wind zone (`constantwind` at `:1662`). Without `constantwind` the global wind velocity is zero and the snow falls straight down under gravity alone, which changes the picture. Row 7 issues `snow` only. It must issue the retail triple, or say why it does not.

**Defect 3: `srand(0)` pins one stream of the two the subsystem draws from.** `Rng::srand` seeds the CRT state only:

```rust
    pub fn srand(&mut self, seed: c_uint) {
        self.crt_holdrand = seed as u32;
    }
```

`crates/native/math/src/rng.rs:187-189`. The Raven `holdrand` stream is seeded by `Rng::Rand_Init` (`:145`). Weather draws from both. `SVecRange::Pick` and the spawn-plane respawn go through `WE_flrand` on the CRT stream, and `SIntRange::Pick` goes through `Q_irand` on `holdrand` (`crates/mp/renderer/src/tr_worldeffects/world_effects.rs:248-250`). That second stream is live on the golden path, because the `snow` preset sets `nCloud.mRotationChangeNext = 0;` (`oracle/codemp/renderer/tr_WorldEffects.cpp:1811`), which fires `mRotationChangeTimer.Pick(mRotationChangeNext)` at `:1071` on the first update. Row 7 must pin `holdrand` as well, or state why the rig leaves it deterministic.

**Defect 4: the DEC-66 ruling 4 claim is not honest as written.** The clause reads: "If a golden proves fragile against scene-order changes, a test-only `srand` reseed before capture graduates then, as its own ruling." Neither half matches. The trigger here is a wall-clock seed, not scene-order fragility, and the wall-clock seed is faithful (`srand(Com_Milliseconds());` at `oracle/codemp/renderer/tr_WorldEffects.cpp:1491`, ported at `crates/mp/renderer/src/tr_worldeffects/world_effects.rs:1371`). And "graduates as its own ruling" means a DEC row, which the packet does not propose. Row 7 must carry a DEC-66 amendment line for the user to rule on, not borrow the clause.

**Correction 5: the ctf2 fact is wrong in two details, both harmless.** The retail scan of all 23 stock MP maps confirms ctf2 is the only one with weather. Its lump carries **two** `fx_snow` entities, at origins `1552 3408 336` and `-944 656 336`, each with `count 1000`, plus the three `misc_weather_zone` brushes (`model` `*1`, `*2`, `*3`). The doubling is behaviorally inert, because `G_FindConfigstringIndex` returns the existing index for a string already registered (`oracle/codemp/game/g_utils.c:74-82`). Separately, the `count` key is dead for snow: `SP_CreateSnow` never reads `ent->count`, and the 1000 in the image comes from the preset's own `nCloud.Initialize(1000, "gfx/effects/snowflake1.bmp");` at `oracle/codemp/renderer/tr_WorldEffects.cpp:1808`. `SP_CreateRain` does read `count`.

**Gap 6: the rig's weather zones differ from live play, and the packet only names the collision half.** The rig runs no cgame, so no `misc_weather_zone` reaches `R_AddWeatherZone` and `mWeatherZones` is empty. `COutside::Cache` then takes its fallback:

```c
		if (!mWeatherZones.size())
		{
			Com_Printf("WARNING: No Weather Zones Encountered");
			AddWeatherZone(tr.world->bmodels[0].bounds[0], tr.world->bmodels[0].bounds[1]);
		}
```

`oracle/codemp/renderer/tr_WorldEffects.cpp:559-563`. So the golden runs one map-sized zone where live ctf2 has three brush zones. With the empty collision world the packet already names, the golden proves the draw path and byte-stability. It does not prove zone or cache behavior. The finished file should record that.

**Practical note 7: the call shorthand does not compile as written.** `R_WorldEffectCommand` is a method taking seven arguments besides the receiver (`crates/mp/renderer/src/tr_worldeffects/world_effects.rs:1560-1569`), and `command` is `Option<&[u8]>`, so `Some(b"snow")` is `Option<&[u8; 4]>` and needs a slice coercion. Everything is public, so the packet's "no production surface" claim holds. The test needs the full host bundle.

### Row 8 - the nearest sampler (mechanical). CLEARED on the oracle fact, CHALLENGED on the gate.

The oracle fact is exact: `qglTexParameterf(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, (mFilterMode==0)?(GL_LINEAR):(GL_NEAREST));` at `oracle/codemp/renderer/tr_WorldEffects.cpp:1364`, and every rain preset sets `mFilterMode = 1` (`:1718`, `:1739`, `:1760`, `:1788`). `GpuImages` really does build two samplers and no others (`crates/mp/renderer-gpu/src/gpu_images.rs:140-141`).

The gate misses it. The `snow` branch (`:1798-1817`) never touches `mFilterMode`, so it keeps the `Reset` default `mFilterMode = 0;` (`:988`). The ctf2 snow golden therefore never binds `sampler_nearest`, and the new sampler and the new bind-group builder land with no gate at all. Row 8 needs a rain case, or the packet must record the untested surface.

### Row 9 - the broadcast rand draws (mechanical). CLEARED. Verified verbatim.

`CVec3` declares `inline CVec3 operator* (const CVec3 &t) const` (`oracle/codemp/Ravl/CVec.h:628`) and no binary scalar multiply. The only scalar form is the compound `const CVec3 &operator*= (const float d)` at `:620`, which cannot appear in `a * b`. The non-explicit `CVec3(const float val) {v[0]=val; v[1]=val; v[2]=val;}` at `:570` broadcasts. So each of

```c
					part->mPosition		+= (mSpawnPlaneRight*WE_flrand(-mSpawnPlaneSize, mSpawnPlaneSize)); 
					part->mPosition		+= (mSpawnPlaneUp*   WE_flrand(-mSpawnPlaneSize, mSpawnPlaneSize)); 
```

`oracle/codemp/renderer/tr_WorldEffects.cpp:1216-1217`, makes exactly one draw. The packet's reading is correct and its trap warning is warranted. One cite tidy: the heading says `:1215-1217`, and the two draws are `:1216-1217`. Line `:1215` is the `mSpawnPlaneDistance` broadcast and draws nothing, which the row body already says.

The contrast case is right too. `SVecRange::Pick` is three draws in x, y, z order (`oracle/codemp/renderer/tr_WorldEffects.cpp:156-161`) and the port has it (`crates/mp/renderer/src/tr_worldeffects/world_effects.rs:182-186`).

### Row 10 - the colour quantization (mechanical). CLEARED, with one thing to pin.

`WorldVertex` carries `color: [u8; 4]` and asserts 44 bytes (`crates/mp/renderer-gpu/src/pipeline3d.rs:137`, `:190`). Keeping the float in the payload and rounding once is right. The packet's range `:133-147` should be `:133-141`, and the fields are private, so the executor builds the vertex through the existing constructor.

Pin the rule. Snow uses blend mode 1, so its channels are `mColor[i]*part->mAlpha` (`oracle/codemp/renderer/tr_WorldEffects.cpp:1402`) and the golden's every pixel depends on whether the lane rounds or truncates. Row 10 must name one rule and one clamp, not leave it to the transcriber.

### Row 11 - the uninitialized `mRotationChangeTimer.mMax` read (mechanical). Disposition CLEARED, rationale DISPUTED.

The bug is exactly where the packet says. `SIntRange::Clear` writes `mMin = 0; mMin = 0;` (`oracle/codemp/renderer/tr_WorldEffects.cpp:229-233`), and `Reset` writes `mRotationChangeTimer.mMin = 500;` then `mRotationChangeTimer.mMin = 2000;` (`:999-1000`). `mMax` is never written and stays 0. The port reproduces it at `crates/mp/renderer/src/tr_worldeffects/world_effects.rs:955` and `:1109-1112`. "No code change" is the right disposition.

The stated reason is wrong, and as written the lane would record a false fact in the source. The packet says the call "is `Q_irand(2000, 0)`, which the clamp at `:1072-1075` floors at 1." It does not. `irand` increments `max` first and then shifts a signed product:

```c
	max++;
	holdrand = (holdrand * 214013L) + 2531011L;
	result = holdrand >> 17;
	result = ((result * (max - min)) >> 15) + min;
```

`oracle/codemp/game/q_math.c:1464-1467`. With `min = 2000` and `max = 0` this is `((result * -1999) >> 15) + 2000` over `result` in [0, 32767], which lands uniformly in [1, 2000]. The `if (mRotationChangeNext<=0)` clamp at `:1072-1075` never fires. The port matches the arithmetic exactly, including the arithmetic right shift:

```rust
        let max = max + 1;
        self.0 = self.0.wrapping_mul(214013).wrapping_add(2531011);
        let result = (self.0 >> 17) as c_int;
        (result.wrapping_mul(max - min) >> 15).wrapping_add(min)
```

`crates/native/math/src/rng.rs:62-65`. So the disposition survives and the note must say the real thing: the reversed range yields a pseudo-random rotation interval in [1, 2000], and the clamp is dead.

This row is live on the golden path. The `snow` preset sets `mRotationChangeNext = 0` (`oracle/codemp/renderer/tr_WorldEffects.cpp:1811`), so the pick fires on the first update.

## Load-bearing claims

| Claim | Verdict | Evidence |
| --- | --- | --- |
| `RB_WorldEffects` has zero callers | CONFIRMED | Workspace grep returns the definition at `crates/mp/renderer/src/tr_backend.rs:904` and its doc line at `:888`, and nothing else. |
| `RE_RenderWorldEffects` does not exist | CONFIRMED | Only `DEFERRED` notes, at `crates/mp/renderer/src/tr_cmds.rs:245` and `crates/mp/renderer/src/tr_scene.rs:1346,1355`. No function. |
| `CWeatherParticleCloud::Render` is a counter | CONFIRMED | `pub fn Render(&self, particles_rendered: &mut i32) { *particles_rendered += self.mParticleCountRender; }` at `crates/mp/renderer/src/tr_worldeffects/world_effects.rs:1317-1319`. |
| renderer-gpu has no weather code | CONFIRMED | `grep -rni weather crates/mp/renderer-gpu/src/` returns nothing. |
| All four type-stub markers are stale | **DISPUTED** | Three are stale. The fourth is not. See below. |
| `ctf2` is the only stock MP map with weather | CONFIRMED, with two detail corrections | All 23 stock MP maps scanned from `~/Developer/jka/jamp-client/base/`. Only ctf2. It carries two `fx_snow`, not one, and the `count` key is dead for snow. See row 7 correction 5. |
| Twenty-one committed fixtures, one per test | CONFIRMED | 20 PNG plus 1 BIN. Five world, eleven scene, two entity, two hud, one ghoul2 vertex. |
| `CHANNEL_TOLERANCE` is 0 in all five suites | **DISPUTED** | It is 0 in four: `entity_golden.rs:76`, `hud_golden.rs:68`, `scene_golden.rs:73`, `world_golden.rs:73`. `ghoul2_vertex_golden.rs` has no such constant, because it compares a vertex `.bin`. |
| The gate battery matches the gh#50 step-001 precedent | CONFIRMED | Line for line, with the counts updated from four world goldens and twenty fixtures to five and twenty-one. `--test-threads=1` appears on every golden line and on the workspace test line. |
| Row 9's one-draw broadcast | CONFIRMED | `oracle/codemp/Ravl/CVec.h:570,628`. See row 9. |
| Row 11's uninitialized read | CONFIRMED as a fact, its stated consequence DISPUTED | See row 11. |

### The fourth type-stub marker is not stale

Three markers state plainly that a field does not exist and are refuted by the landed struct. `crates/mp/renderer/src/tr_worldeffects/world_effects.rs:1416` says `rdflags` "is not yet a field on the landed `TrRefdef`", and it is, at `crates/mp/renderer/src/render_state/placeholders.rs:297`. `:1443` says the same of `frametime`, which is at `placeholders.rs:293`. `:1461` says `world_t::bmodels` "is not yet a field on the landed `WorldAsset`", and `R_LoadSubmodels` fills it at `crates/mp/renderer/src/tr_bsp.rs:1749-1756`.

The `orientationr_t` marker at `:1160` is different:

```
        //TODO: Port orientationr_t (viewParms_t::ori origin/axis)
        // Source: oracle/codemp/renderer/tr_local.h:109-114,629-644
        // (`OrientationR`/`ViewParms` are still empty placeholders — landed
        // by the not-yet-run `tr_main` R3 wave); used at
        // oracle/codemp/renderer/tr_WorldEffects.cpp:1061-1064
```

`OrientationR` is not empty (`placeholders.rs:414-431` carries `origin`, `axis`, `view_origin`, `model_matrix`), but the placeholder `ViewParms` at `placeholders.rs:367-386` really has no `ori` field, and `Update` reaches the view only through `_frame: &FrameState`, whose `view` is that placeholder. On the path the marker sits on, the read is genuinely blocked.

The packet contradicts itself here. Its survey section says "The marker comments that call them 'not yet a field' are stale", and its own row 5 says "`render_state::placeholders::ViewParms` (`:367-386`) carries only `pvs_origin`, `frustum`, and `vis_bounds`, so `FrameState::view` has no orientation at all." Row 5 is right. The marker is not stale, it is made moot by row 5's ruling that the value comes from the refdef. Commit 1's note must say that, not "the field already exists."

## Contract gaps

**The cull mode is unspecified and it matters.** Weather draws with `GL_Cull(CT_TWO_SIDED)` (`oracle/codemp/renderer/tr_WorldEffects.cpp:1362`), which the packet states in prose and then drops. `PipelineKey` carries `blend`, `depth_equal`, `depth_write` and `depth_bias` and no cull field (`crates/mp/renderer-gpu/src/pipeline3d.rs:811`), and `build_world_pipeline` hard-codes `cull_mode: None` with this comment at `:3431-3434`: "Culling is off for this wave: the frontend has already culled surfaces, and per-shader cull sidedness lands with a later wave." So the weather draw is two-sided by accident, not by contract, and the wave that lands per-shader cull will silently break it. The packet must record the dependency at `draw_weather`.

**The `WeatherFrame` payload does not carry the view it was built from.** The executor arm is specified as "rebuild the view from the last scene refdef, then `Pipeline3d::draw_weather`". The batch is computed on the sim side from one refdef, and the view is reconstructed render-side from another. They agree only by event ordering. Under DEC-65's payload doctrine the crossing should carry what the render side needs, or the packet must state the ordering invariant explicitly and say what enforces it.

## Cite corrections

- `RB_WorldEffects` is `oracle/codemp/renderer/tr_backend.cpp:1886-1905`, not `:1885-1904`. The `case RC_WORLD_EFFECTS:` cite at `:1944` is exact.
- The wind statics are `oracle/codemp/renderer/tr_WorldEffects.cpp:73-75`, not `:73-76`.
- `SP_CreateSnow` is `oracle/codemp/game/g_misc.c:2522-2527` and the packet quotes only its first line. `SP_CreateRain`'s `G_EffectIndex` call is `:2537`, not `:2538`.
- `CG_ParseWeatherEffect` is `oracle/codemp/cgame/cg_main.c:1395-1400`, not `:1393-1399`. The port's is `crates/mp/cgame/src/cg_main.rs:1554-1557`, not `:1550-1556`.
- `CG_CreateWeatherZoneFromSpawnEnt` in the port is `crates/mp/cgame/src/cg_main.rs:2674-2677`, not `:2670-2677`.
- The cgame configstring branch is `oracle/codemp/cgame/cg_servercmds.c:807-816`, not `:806-814`.
- `WorldVertex` is `crates/mp/renderer-gpu/src/pipeline3d.rs:133-141`, not `:133-147`. `add_quad_stamp_ext` is `:4875-4909`. `build_world_pipeline` starts at `:3401`.
- `blend_state_from_gls` is `crates/mp/renderer-gpu/src/blend.rs:61`, not `:62`. It already handles both weather blend modes, and `:166` already unit-tests the additive one.
- `boot::load_world` is `crates/mp/renderer-gpu/src/ui_host/boot.rs:472-519`, not `:472-510`. The substance holds: no allowlist, and `CM_LoadMap` appears nowhere in the file.
- The `COutside::Cache` fall-through is `crates/mp/renderer/src/tr_worldeffects/world_effects.rs:707-710`, not `:708-711`, and its condition is `!self.mCacheInit`, not "contents are 0".
- The second `todo!()` in `Update` is at `:1292-1294`. The packet's `:1284-1295` names the loop.
- Row 2 says the trap arm "already builds an `EngineHostView`". It does not build one. `view` is a dispatcher parameter, and the arms cast it with `re_from_view(view)`.

## One missing pause trigger

`partInRange = true;` at `oracle/codemp/renderer/tr_WorldEffects.cpp:1226` is a dead store. `partInView` was already computed at `:1202`, and the fade machine at `:1231-1291` and the render count at `:1295-1298` read only `partRendering`, `partInView` and `part->mFlags`. A transcriber who notices will want to recompute `partInView` after the respawn. Add a STOP: the assignment is dead in the oracle and stays dead.

## Verdict - what the walk must carry beyond rows 1, 2 and 7

1. **The fourth divergence.** The `RDF_NOWORLDMODEL` gate change is live-observable and belongs in the synopsis divergence list, in row 2, and in the finished file. Row 5's dismissal sentence is deleted.
2. **Row 7 is rewritten before the lane starts.** Four defects: the frametime arithmetic and every expectation built on it, the single-command recipe against the retail triple, the one-stream seed, and the DEC-66 graduation form. A DEC-66 amendment line is drafted for the user to rule on.
3. **Row 11's note text is corrected.** The reversed range yields [1, 2000] and the clamp is dead. The disposition does not change.
4. **The fourth type-stub marker is reclassified.** It is not stale. Row 5's ruling makes it moot, and commit 1's note must say so.
5. **Row 8 gains a gate or records the gap.** Snow leaves `mFilterMode` at 0, so the ctf2 golden never binds `sampler_nearest`.
6. **Row 10 pins the rounding rule and the clamp.** Snow uses blend mode 1, so the golden depends on it.
7. **The cull mode enters the surface contract.** `PipelineKey` has no cull field and the world pipeline's `cull_mode: None` is documented as temporary.
8. **The `WeatherFrame` view coupling is settled.** Either the payload carries its view, or the packet states and defends the ordering invariant.
9. **The `partInRange` dead-store STOP is added** to the pause triggers.
10. **Row 4 cites DEC-66 ruling 1, and row 2 cites DEC-65 ruling 2.** Both rows argue a point that is already ruled.
11. **The cite corrections above are applied**, including the `CHANNEL_TOLERANCE` count (four suites, not five) and the ctf2 entity facts (two `fx_snow`, dead `count` key).

Rows 3, 6 and 9 are cleared with no action beyond the cite tidies.
