# Packet gh#54 step-001 - the weather group

Ratified 2026-08-31. Every open row is closed. The audit is at `.claude/packets/54/step-001/audit.md` (`53ec6a62`), and the fold below carries its cite corrections and its four verdicts.

## Scope

This step makes rain and snow run and draw. A map with weather builds real particle clouds on this client today, and nothing steps them and nothing draws them.

The step closes all five `TODO: Port` markers in `crates/mp/renderer/src/tr_worldeffects/world_effects.rs`: the four type stubs (`trRefdef_t::frametime`, `trRefdef_t::rdflags`, `orientationr_t` origin and axis, `bmodel_t` bounds) and `CWeatherParticleCloud::Update`'s per-particle loop. It then gives the subsystem the two things the markers do not cover: a per-frame caller, and a draw. It ends with one new world golden on `maps/mp/ctf2.bsp`, the one stock MP map that ships weather.

The step does not port the Xbox point-sprite path, the dead `CWorldEffect`/`CWorldEffectsSystem`/`SParticle` header classes, the five weather symbols with no MP caller, or the `r_we` console-command registration. It adds no cvar, no ABI surface, and no third-party crate. It touches no file under `crates/mp/game/`, `crates/mp/cgame/`, `crates/mp/ui/`, or `crates/sp/`, so the lockstep referee is not a gate here. It does touch `crates/mp/engine/client/`, which is not a `jampded` link-set crate, so the referee stays off the list.

Six oracle behaviors in this area are quirks, not defects, and this step preserves every one. Rows 5, 9, and 11 name them, and the lane must not correct any of them.

## The oracle, cited

### What runs the weather, and when

`RE_RenderScene` queues the weather command as the last act of a scene: `RE_RenderWorldEffects();` (`oracle/codemp/renderer/tr_scene.cpp:868`). That function fills one bufferless `RC_WORLD_EFFECTS` marker (`oracle/codemp/renderer/tr_cmds.cpp:291-300`), the backend dispatches it at `case RC_WORLD_EFFECTS:` (`oracle/codemp/renderer/tr_backend.cpp:1944`), and `RB_WorldEffects` flushes the tess batch, calls `RB_RenderWorldEffects`, and reopens the batch (`oracle/codemp/renderer/tr_backend.cpp:1886-1905`).

The queue is once per scene, not once per frame. A frame with N scenes queues N markers and the backend steps `Update` N times.

`RB_RenderWorldEffects` (`oracle/codemp/renderer/tr_WorldEffects.cpp:1513-1580`) does six things in order.

1. The four-term early return (`:1515-1521`): no world, `tr.refdef.rdflags & RDF_NOWORLDMODEL`, `backEnd.refdef.rdflags & RDF_SKYBOXPORTAL`, or no clouds.
2. `SetViewportAndScissor()` and `qglLoadMatrixf(backEnd.viewParms.world.modelMatrix)` (`:1523-1525`).
3. `mMillisecondsElapsed = backEnd.refdef.frametime`, clamped to 1.0 and 1000.0, then `mSecondsElapsed = mMillisecondsElapsed / 1000.0f` (`:1530-1539`).
4. On the first frame only, `mOutside.Cache()` (`:1544-1547`). That frame updates nothing and draws nothing, because everything else sits in the `else` at `:1548`.
5. The wind-zone sum, skipped while `mFrozen` (`:1552-1564`).
6. Per cloud, `Update()` then `Render()`, interleaved in one loop (`:1569-1574`).

`Update` and `Render` are Raven's own names. There is no `Draw`, and weather never enters the draw-surface list.

### The four values the stubs wait on

`frametime` is `backEnd.refdef.frametime` (`oracle/codemp/renderer/tr_WorldEffects.cpp:1530`), not `tr.refdef.frametime`. `RE_RenderScene` computes it as `fd->time - lastTime` and clamps it to 0 through 500 (`oracle/codemp/renderer/tr_scene.cpp:741,758-765`), and `RB_DrawSurfs` copies the whole refdef into `backEnd.refdef` (`oracle/codemp/renderer/tr_backend.cpp:1626`). The `< 1` clamp means a zero value never divides by zero. It makes the weather crawl at one thousandth speed instead.

`rdflags` is read twice, from two different refdefs (`:1515-1517`). `RDF_NOWORLDMODEL` (value 1) comes from `tr.refdef`, the front-end copy. `RDF_SKYBOXPORTAL` (value 8) comes from `backEnd.refdef`. Both constants already exist in the port (`crates/mp/renderer/src/tr_public/ref_flags.rs:41,57`). No other `RDF_*` bit is read in this TU. Row 2 holds what this split means for the port, because it is the fourth named divergence and not a detail.

The view orientation is `backEnd.viewParms.ori`, read at exactly one live site:

```c
			mCameraPosition	= backEnd.viewParms.ori.origin;
			mCameraForward	= backEnd.viewParms.ori.axis[0];
			mCameraLeft		= backEnd.viewParms.ori.axis[1];
			mCameraDown		= backEnd.viewParms.ori.axis[2];
```

`oracle/codemp/renderer/tr_WorldEffects.cpp:1061-1064`. That orientation is filled straight from the scene refdef: `VectorCopy( fd->vieworg, parms.ori.origin ); VectorCopy( fd->viewaxis[0], parms.ori.axis[0] );` and the two siblings (`oracle/codemp/renderer/tr_scene.cpp:848-851`). The port already reproduces that pairing on the render side (`crates/mp/renderer-gpu/src/frame_exec.rs:812-813`), so the refdef's view origin and axis are the faithful source.

A zero axis is the silent failure mode. `mCameraLeft` and `mCameraDown` collapse, every billboard degenerates to a point, `partToCamera.Dot(mCameraForward)` becomes 0 and fails the `> 0.0f` test at `:1202`, so no particle ever sets `FLAG_RENDER` and the system runs its physics and draws nothing.

The world bounds are the fallback zone when no `misc_weather_zone` brush supplied one:

```c
		if (!mWeatherZones.size())
		{
			Com_Printf("WARNING: No Weather Zones Encountered");
			AddWeatherZone(tr.world->bmodels[0].bounds[0], tr.world->bmodels[0].bounds[1]);
		}
```

`oracle/codemp/renderer/tr_WorldEffects.cpp:559-563`. `bmodels[0]` is always the worldspawn brush model, so the bounds are the whole map.

### `CWeatherParticleCloud::Update` - `oracle/codemp/renderer/tr_WorldEffects.cpp:1039-1306`

`particleFade = (mFade * mSecondsElapsed)` is computed once per cloud per frame, before the freeze gate (`:1050`).

The camera block (`:1058-1101`) copies the four view vectors, then spins the billboard basis when `mRotationChangeNext != -1`. The spin draws two random numbers on two different streams: `mRotation.Pick(mRotationDeltaTarget)` on the C runtime stream (`:1070`) and `mRotationChangeTimer.Pick(mRotationChangeNext)` on the `holdrand` stream (`:1071`). The rotation mix is asymmetric and load-bearing:

```c
				mCameraLeft *= (c * mWidth);
				mCameraLeft.ScaleAdd(mCameraDown, (s * mWidth * -1.0f));

				mCameraDown *= (c * mHeight);
				mCameraDown.ScaleAdd(TempCamLeft, (s * mHeight));
```

`oracle/codemp/renderer/tr_WorldEffects.cpp:1090-1094`. `mCameraLeft` mixes the new `mCameraDown`, and `mCameraDown` mixes the saved `TempCamLeft`. The port already carries this block correctly (`crates/mp/renderer/src/tr_worldeffects/world_effects.rs:1198-1218`).

The freeze gate sits after the camera and range work (`:1166-1169`), so a frozen cloud still tracks the camera and still draws.

The per-particle loop (`:1175-1304`) is the fifth marker's body. Its five phases:

1. First-time spawn. `if (!mPopulated) mRange.Pick(part->mPosition);` (`:1180-1183`). `mPopulated` is set true after the loop (`:1305`), so the first update spawns every particle. `SVecRange::Pick` (`:156-161`) is three separate draws, x then y then z.
2. Integration (`:1187-1196`). `partForce = force; partForce /= part->mMass; part->mVelocity += partForce; part->mVelocity *= mFrictionInverse; part->mPosition.ScaleAdd(part->mVelocity, mSecondsElapsed);`. Force and friction apply per frame with no time factor. Only the position step is time-scaled. This is frame-rate dependent and stays that way.
3. Classification (`:1198-1202`). `partOutside` calls the three-argument overload `mOutside.PointOutside(part->mPosition, mWidth, mHeight)` (`:665-703`), which never checks `mCacheInit`. `partInView` is a half-space test against the camera forward axis, not a frustum test.
4. Respawn (`:1204-1227`). Only a particle that is both out of range and not rendering respawns. With a spawn plane it lands on the plane. Without one, `mRange.Wrap` wraps it. The trailing `partInRange = true;` at `:1226` is a dead store, and the pause triggers below say so.
5. The fade machine (`:1229-1291`) and the render count (`:1295-1298`). The alpha ceiling is `mColor[3]`, not 1.0. The count re-reads the flag rather than the local, so a particle that faded out this frame is not counted.

**The rand-draw trap, `:1216-1217`.** These two lines look like the `VectorMA` multi-eval trap and are not:

```c
					part->mPosition		+= (mSpawnPlaneRight*WE_flrand(-mSpawnPlaneSize, mSpawnPlaneSize)); 
					part->mPosition		+= (mSpawnPlaneUp*   WE_flrand(-mSpawnPlaneSize, mSpawnPlaneSize)); 
```

`CVec3` has no binary scalar `operator*`. The only multiply is `CVec3 operator*(const CVec3&)` (`oracle/codemp/Ravl/CVec.h:628`), and the float converts through the non-explicit broadcast constructor `CVec3(const float val)` (`oracle/codemp/Ravl/CVec.h:570`). So each line makes exactly **one** draw and that one value scales x, y, and z alike. A per-component transcription would draw six times instead of two and desynchronize the stream for the rest of the session. The neighbouring `(mSpawnPlaneNorm * mSpawnPlaneDistance)` at `:1215` is the same broadcast and draws nothing.

### `CWeatherParticleCloud::Render` - `oracle/codemp/renderer/tr_WorldEffects.cpp:1311-1480`

The draw is a self-contained fixed-function GL block. There is no `shader_t`, no `tess`, no `RB_BeginSurface`, and no `drawSurf_t`. The cloud binds one `image_t` and issues its own geometry.

```c
		GL_State((mBlendMode==0)?(GLS_ALPHA):(GLS_SRCBLEND_ONE | GLS_DSTBLEND_ONE));
		GL_Bind(mImage);
```

`oracle/codemp/renderer/tr_WorldEffects.cpp:1319-1320`. `GLS_ALPHA` is `(GLS_SRCBLEND_SRC_ALPHA | GLS_DSTBLEND_ONE_MINUS_SRC_ALPHA)` (`oracle/codemp/renderer/tr_local.h:1683`). Neither value sets `GLS_DEPTHMASK_TRUE` or `GLS_DEPTHTEST_DISABLE`, so weather depth-tests and does not depth-write.

`mGLModeEnum` is `(mVertexCount==3)?(GL_TRIANGLES):(GL_QUADS)` and nothing else on the PC build (`:944`; the `GL_POINTS` assignment at `:941` is inside `#ifdef _XBOX`). Every point-sprite branch at `:1326`, `:1344`, `:1349`, and `:1463` is therefore dead here. The port never carried `mGLModeEnum`, and it does not need it.

The live state is `GL_Cull(CT_TWO_SIDED)` (`:1362`) and the per-cloud min and mag filter `(mFilterMode==0)?(GL_LINEAR):(GL_NEAREST)` (`:1364-1365`). Both filters are unmipmapped. Every rain preset sets `mFilterMode = 1` (`:1718`, `:1739`, `:1760`, `:1788`), and the `snow` preset leaves it at the `Reset` default of 0 (`:988`).

The colour is per particle, and the blend mode picks which channels take the alpha:

```c
			if (mBlendMode==0)
			{
				qglColor4f(mColor[0], mColor[1], mColor[2], part->mAlpha);
			}
			else
			{
				qglColor4f(mColor[0]*part->mAlpha, mColor[1]*part->mAlpha, mColor[2]*part->mAlpha, mColor[3]*part->mAlpha);
			}
```

`oracle/codemp/renderer/tr_WorldEffects.cpp:1393-1403`. The `snow` preset uses blend mode 1, so the golden's every pixel runs through the premultiplied branch at `:1402`.

The geometry is absolute world coordinates, because the loaded matrix is the plain world model matrix and nothing else is pushed. The triangle arm (`:1414-1430`) emits UVs `(1,0)`, `(0,1)`, `(0,0)` at offsets `0`, `+mCameraLeft`, `+mCameraLeftPlusUp`. The quad arm (`:1434-1459`) emits UVs `(0,0)`, `(1,0)`, `(1,1)`, `(0,1)` at offsets `-mCameraLeftMinusUp`, `-mCameraLeftPlusUp`, `+mCameraLeftMinusUp`, `+mCameraLeftPlusUp`.

### How weather turns on in MP

Two paths reach `R_WorldEffectCommand`, and the port carries both end to end.

`fx_snow` registers three effect strings, not one:

```c
void SP_CreateSnow( gentity_t *ent )
{ 
	G_EffectIndex("*snow");
	G_EffectIndex("*fog");
	G_EffectIndex("*constantwind (100 100 -100)");
}
```

`oracle/codemp/game/g_misc.c:2522-2527`, ported at `crates/mp/game/src/g_misc.rs:2574-2578`. `SP_CreateRain` calls `G_EffectIndex(va("*rain init %i", ent->count))` at `oracle/codemp/game/g_misc.c:2537`, and it is the one of the two that reads `count`. `G_EffectIndex` registers the string as a `CS_EFFECTS` configstring (`oracle/codemp/game/g_utils.c:148-151`), and `G_FindConfigstringIndex` returns the existing index for a string already registered (`oracle/codemp/game/g_utils.c:74-82`), so a duplicate entity costs nothing.

cgame routes any `*`-prefixed effect string to `CG_ParseWeatherEffect` (`oracle/codemp/cgame/cg_servercmds.c:807-816`, `oracle/codemp/cgame/cg_main.c:1395-1400`), which strips the star and calls `trap_R_WorldEffectCommand`. A `misc_weather_zone` brush is server-side dead (`oracle/codemp/game/g_misc.c:3488-3494`) and cgame-live, reaching `trap_WE_AddWeatherZone` (`oracle/codemp/cgame/cg_main.c:3646-3649`).

The RMG path reads the `RMG_weather` cvar and issues the commands directly (`oracle/codemp/renderer/tr_arioche.cpp:99-112`). It only fires when `com_RMG` is set, so a static BSP never takes it.

No console command named `weather`, `rain`, or `snow` exists. The one console entry is `r_we`, registered in `R_Register` and gated on `sv_cheats` (`oracle/codemp/renderer/tr_init.cpp:1196`, `oracle/codemp/renderer/tr_WorldEffects.cpp:1583-1591`). That registration belongs to this ticket's step-004, not here.

## The port as it stands

### Everything except the frame loop already runs

`crates/mp/renderer/src/tr_worldeffects/world_effects.rs` is 2082 lines and almost entirely live. `R_WorldEffectCommand` runs in full with all nineteen branches (`:1560-2029`). `COutside::Cache` runs a real `CM_PointContents` scan (`:634-712`). `CWeatherParticleCloud::Initialize` calls a real `R_FindImageFile` (`:992-1061`). `CWindZone::Update` runs its full physics (`:360-389`).

The live chain reaches the renderer today. cgame's `CG_ParseWeatherEffect` (`crates/mp/cgame/src/cg_main.rs:1554-1557`) and `CG_CreateWeatherZoneFromSpawnEnt` (`crates/mp/cgame/src/cg_main.rs:2674-2677`) both fire, and the trap arms land at `crates/mp/engine/client/src/cl_cgame.rs:3623-3643`. So a weather map builds real clouds with real loaded textures and real point caches.

Two functions are stubs, and both sit on the frame path.

`CWeatherParticleCloud::Update` (`:1150-1297`) panics on its first statement. The `todo!()` at `:1170-1172` binds the four camera vectors, so the whole body below it is dead and `#[allow(unreachable_code, unused_variables)]` at `:1149` silences the compiler. The per-particle loop at `:1284-1295` has a second `todo!()` as its entire body, at `:1292-1294`.

`CWeatherParticleCloud::Render` (`:1317-1319`) is a counter and nothing else. Its deferral note names the fixed-function GL surface and DEC-37 A13.2 as the reason.

`RB_RenderWorldEffects` (`:1401-1504`) panics at the second term of its guard (`:1424-1426`) once a world is loaded.

### Nothing calls the frame path

`RB_RenderWorldEffects` has exactly one caller, `RB_WorldEffects` (`crates/mp/renderer/src/tr_backend.rs:904-921`). `RB_WorldEffects` has zero callers anywhere in the workspace. `RE_RenderWorldEffects` does not exist: `crates/mp/renderer/src/tr_cmds.rs:245-258` carries the `DEFERRED` note that escalated it, and `crates/mp/renderer/src/tr_scene.rs:1346-1350` carries the matching note at the oracle's own call site.

So no marker in this file can fire in play today. The symptom is silence, not a crash.

### Three stubbed reads exist, and the fourth is mooted

Three of the four type-stub markers state that a field does not exist, and the landed struct refutes each one. The fourth is a different case, and the audit corrected the packet's first reading of it.

- `TrRefdef::frametime` is `pub frametime: i32` (`crates/mp/renderer/src/render_state/placeholders.rs:293`). `RE_RenderScene` computes it from `fd.time - scene.last_time`, clamps it to 0 through 500, and writes it (`crates/mp/renderer/src/tr_scene.rs:1248-1273`). The marker at `:1443` is stale.
- `TrRefdef::rdflags` is `pub rdflags: i32` (`crates/mp/renderer/src/render_state/placeholders.rs:297`), written at `crates/mp/renderer/src/tr_scene.rs:1274`. `RDF_NOWORLDMODEL` and `RDF_SKYBOXPORTAL` are `crates/mp/renderer/src/tr_public/ref_flags.rs:41,57`. The marker at `:1416` is stale.
- The world bounds are `assets.world.as_ref().map(|w| w.bmodels[0].bounds)`. `BModel::bounds` is `crates/mp/renderer/src/tr_bsp.rs:220`, filled by `R_LoadSubmodels` at `crates/mp/renderer/src/tr_bsp.rs:1749-1756`. The marker at `:1461` is stale.
- The `orientationr_t` marker at `:1160` is **not stale**. `OrientationR` is not an empty placeholder (`crates/mp/renderer/src/render_state/placeholders.rs:414-431` carries `origin`, `axis`, `view_origin`, `model_matrix`), but the placeholder `ViewParms` at `:367-386` really has no `ori` field, and `Update` reaches the view only through `_frame: &FrameState`, whose `view` is that placeholder. On the path the marker sits on, the read is genuinely blocked. Row 5's ruling makes the marker **moot** by taking the value from the refdef instead, and commit 1's note must say that and not "the field already exists".

### The wind-zone carrier has no owner

`WindZoneState` (`crates/mp/renderer/src/tr_worldeffects/world_effects.rs:427-438`) holds Raven's `mGlobalWindVelocity`, `mGlobalWindDirection`, and `mGlobalWindSpeed` trio (`oracle/codemp/renderer/tr_WorldEffects.cpp:73-75`). It appears at three parameter sites and nowhere else: `crates/mp/renderer/src/tr_backend.rs:906`, `crates/mp/renderer/src/tr_surfacesprites.rs:555,918`, and `crates/mp/renderer/src/tr_worldeffects/world_effects.rs:1403`. No struct in the workspace holds one, so nothing can call the functions that take it. Row 3 settles this.

### The draw side does not exist

`grep -rni "weather" crates/mp/renderer-gpu/src/` returns nothing. There is no weather pipeline, pass, arm, or vertex path in the GPU crate, and nothing there imports `tr_worldeffects`.

The nearest analogue is complete and reusable. `RT_SPRITE` builds a camera-facing quad from `view.ori.axis[1]` and `view.ori.axis[2]` (`crates/mp/renderer-gpu/src/pipeline3d.rs:5293-5325`), and `add_quad_stamp_ext` writes its four corners and six indices (`:4875-4909`). The one 3D vertex row is `WorldVertex`, 44 bytes with a `[u8; 4]` colour (`:133-141`), and its fields are private, so a caller builds one through the existing constructor. Blend state comes from `blend_state_from_gls(state_bits)` (`crates/mp/renderer-gpu/src/blend.rs:61`), which already handles both weather blend modes and already unit-tests the additive one at `:166`. `PipelineKey` carries `blend`, `depth_equal`, `depth_write`, and `depth_bias` (`crates/mp/renderer-gpu/src/pipeline3d.rs:811`), so one blend mode costs one cached pipeline. Both existing pipeline builders hard-code `TriangleList` (`crates/mp/renderer-gpu/src/pipeline3d.rs:3401` onward, `crates/mp/renderer-gpu/src/pipeline2d.rs:441`), which is what the quad and triangle arms need.

So the draw needs no new vertex type, no new blend path, and no new WGSL. It needs an entry point, because a weather cloud is not a `DrawSurf` and has no `shader_t`, and it needs a nearest-filter sampler, because `GpuImages` builds only a repeat and a clamp sampler (`crates/mp/renderer-gpu/src/gpu_images.rs:140-141`) and rain asks for `GL_NEAREST`.

### The golden rig is a frozen clock, one frame per test

Every suite sets `const FROZEN_TIME_MS: i32 = 12345;` and renders exactly one frame (`crates/mp/renderer-gpu/tests/world_golden.rs:65,98,290`). There is no loop and no stepper.

`RE_RenderScene` advances its own carrier on every call:

```rust
    let mut frametime = fd.time - scene.last_time;
    scene.last_time = fd.time;
```

`crates/mp/renderer/src/tr_scene.rs:1248-1249`, faithful to `oracle/codemp/renderer/tr_scene.cpp:741-742`. So a fixture that submits the same clock twice gets `frametime = 500` on the first call and `frametime = 0` on the second, which `RB_RenderWorldEffects` floors at 1.0 ms. Row 7's recipe advances the clock per call for exactly this reason.

`boot::load_world` takes any map path with no allowlist (`crates/mp/renderer-gpu/src/ui_host/boot.rs:472-519`), and it calls `RE_LoadWorldMap` alone. `CM_LoadMap` appears nowhere in the file, so the collision world stays empty in the rig. `COutside::Cache` then reads contents 0 at every cell and falls through on `!self.mCacheInit` to `mCacheInit = true; mMarkedOutside = false;` (`crates/mp/renderer/src/tr_worldeffects/world_effects.rs:707-710`), and every point tests as outside. The rig also runs no cgame, so no `misc_weather_zone` reaches `R_AddWeatherZone` and `COutside::Cache` takes its map-sized fallback zone (`oracle/codemp/renderer/tr_WorldEffects.cpp:559-563`) where live ctf2 has three brush zones. Both are rig properties, not port divergences. The golden proves the draw path and byte-stability, and it proves nothing about zone or cache behavior. The finished file records that.

`R_InitWorldEffects` seeds its generator from the wall clock: `self.rng.srand(Com_Milliseconds(host) as u32)` (`crates/mp/renderer/src/tr_worldeffects/world_effects.rs:1371`), faithful to `srand(Com_Milliseconds());` at `oracle/codemp/renderer/tr_WorldEffects.cpp:1491`. Row 7 and the Amendments section handle that.

`Rng` carries the two streams separately, `holdrand` for `Q_irand` (`crates/native/math/src/rng.rs:113`) and `crt_holdrand` for the C runtime `rand` (`:121`), so the oracle's two-stream split is already correct. Weather draws from both. `WE_flrand` and `SVecRange::Pick` go through the CRT stream, and `SIntRange::Pick` goes through `Q_irand` on `holdrand`. `Rng::srand` seeds the CRT state alone (`crates/native/math/src/rng.rs:187-189`) and `Rng::Rand_Init` seeds `holdrand` (`:145`), so a fixture that pins one stream has not pinned the subsystem.

### Twenty-one fixtures, twenty-one tests

`crates/mp/renderer-gpu/tests/goldens/` holds twenty PNG files and one BIN, one per test: five world, eleven scene, two entity, two hud, and one ghoul2 vertex. `CHANNEL_TOLERANCE` is 0 in the four image suites (`entity_golden.rs:76`, `hud_golden.rs:68`, `scene_golden.rs:73`, `world_golden.rs:73`). `ghoul2_vertex_golden.rs` has no such constant, because it compares a vertex `.bin`.

## Surface contract

### `crates/mp/renderer/src/render_state/weather_frame.rs`

One new file holding the seam payload. These are port-invented carrier types, not ported Raven types, so the one-type-per-file rule for Raven types does not split them.

```rust
/// One vertex of a weather billboard: `qglVertex3f`'s position, `qglTexCoord2f`'s texcoord, and the colour `qglColor4f` chose for this particle.
///
/// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:1393-1459`
#[derive(Clone, Copy)]
pub struct WeatherVertex {
    pub position: vec3_t,
    pub st: [f32; 2],
    pub color: [f32; 4],
}

/// One cloud's billboards for this frame, with the image and the GL blend bits `Render` binds before it draws them.
/// `nearest_filter` is Raven's `mFilterMode != 0`, the per-cloud `GL_NEAREST` min and mag filter.
///
/// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:1319-1320,1364-1365`
pub struct WeatherCloudBatch {
    pub image: Option<ImageHandle>,
    pub state_bits: u32,
    pub nearest_filter: bool,
    pub vertices: Vec<WeatherVertex>,
    pub indices: Vec<u32>,
}

/// Every cloud's batch for one frame, in `mParticleClouds` order.
/// The order is the oracle's own draw order, and later clouds blend over earlier ones.
///
/// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:1569-1574`
pub struct WeatherFrame {
    pub clouds: Vec<WeatherCloudBatch>,
}
```

`WeatherFrame::is_empty` is the only method, so the executor arm can skip a frame with no cloud without opening a pass.

`WeatherFrame` deliberately carries no view. The positional invariant stands in its place, and the lane states it in the type's doc: one weather batch per frame, built from the world scene's refdef, with its `FrameEvent` emitted inside that scene's event span so the executor draws it under the view that built it. A future multi-view consumer revisits this as its own ruling.

### `crates/mp/renderer/src/tr_worldeffects/world_effects.rs`

`WorldEffectsState` gains one field:

```rust
    /// Raven's `mGlobalWindVelocity`, `mGlobalWindDirection`, and `mGlobalWindSpeed` file statics, the same DEC-37 A13.3 promotion as `mOutside`.
    /// The trio had no owner before this step, so nothing could call the functions that take it.
    ///
    /// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:73-75`
    pub wind: WindZoneState,
```

Three signatures change. All three are in-crate, and none crosses the ABI.

```rust
/// Raven `CWeatherParticleCloud::Update`.
///
/// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:1039-1306`
pub fn Update(
    &mut self,
    rng: &mut Rng,
    outside: &COutside,
    view_origin: vec3_t,
    view_axis: [vec3_t; 3],
    frozen: bool,
    wind_velocity: vec3_t,
    seconds_elapsed: f32,
)

/// Raven `CWeatherParticleCloud::Render`.
///
/// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:1311-1480`
pub fn Render(&self, particles_rendered: &mut i32) -> WeatherCloudBatch

/// Raven `RB_RenderWorldEffects`.
///
/// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:1513-1580`
pub fn RB_RenderWorldEffects(
    &mut self,
    assets: &RenderAssets,
    refdef: &TrRefdef,
    host: &mut EngineHostView,
) -> WeatherFrame
```

`Update` drops the unused `_frame: &FrameState`. `RB_RenderWorldEffects` drops `wind: &mut WindZoneState` per row 3, drops `rng: &mut Rng` per row 4, and takes `refdef: &TrRefdef` in place of `frame: &FrameState` per row 6. Its guard reads both `RDF_NOWORLDMODEL` and `RDF_SKYBOXPORTAL` from the one refdef it is handed, which is the submitted scene's own, per row 2.

No other function in this file changes shape. `COutside`, `CWindZone`, `SVecRange`, `SFloatRange`, `SIntRange`, `R_WorldEffectCommand`, `R_InitWorldEffects`, `R_ShutdownWorldEffects`, and `R_WorldEffect_f` all keep their signatures and their bodies.

### `crates/mp/renderer/src/tr_backend.rs`

`RB_WorldEffects` keeps its role and takes the new shape:

```rust
pub fn RB_WorldEffects(
    world_effects: &mut WorldEffectsState,
    assets: &RenderAssets,
    refdef: &TrRefdef,
    host: &mut EngineHostView,
) -> WeatherFrame
```

Its two `DEFERRED` tess notes stay. `SetViewportAndScissor` is a no-op at this site and is retired per row 6.

### `crates/mp/renderer/src/tr_cmds.rs`

```rust
/// Raven `RE_RenderWorldEffects` - steps the scene's weather and queues its batch.
/// Raven's bufferless `RC_WORLD_EFFECTS` marker becomes the frame's `WorldEffects` event, which carries the batch the pass draws.
/// The caller passes the submitted scene's own refdef, so the guard gates on that scene's flags and not on a later one's.
///
/// Source: `oracle/codemp/renderer/tr_cmds.cpp:291-300`
pub fn RE_RenderWorldEffects(
    frame: &mut FrameData,
    world_effects: &mut WorldEffectsState,
    assets: &RenderAssets,
    refdef: &TrRefdef,
    host: &mut EngineHostView,
)
```

The `DEFERRED: RE_RenderWorldEffects` note at `:245-258` retires, and so does its twin at `crates/mp/renderer/src/tr_scene.rs:1346-1350`. The `RE_RenderAutoMap` note beside each one stays untouched.

### `crates/mp/renderer/src/render_state/frame_event.rs`

One new variant:

```rust
    /// Raven's `RC_WORLD_EFFECTS` backend command, carrying the batch `RB_RenderWorldEffects` already built.
    /// The oracle queues it after the scene's `RC_DRAW_SURFS`, so this event follows `RenderScene` and the pass draws over the world.
    ///
    /// Source: `oracle/codemp/renderer/tr_scene.cpp:868`, `oracle/codemp/renderer/tr_cmds.cpp:291-300`
    WorldEffects(WeatherFrame),
```

`WorldEffectCommand(String)` stays as it is. It has no producer, and retiring it is not this step's work.

### `crates/mp/renderer-gpu/src/gpu_images.rs`

One new sampler beside the two that exist, and one bind-group builder:

```rust
    /// The unmipmapped nearest sampler Raven's `mFilterMode != 0` clouds bind.
    /// Weather sets both the min and the mag filter, and neither weather filter uses mips.
    ///
    /// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:1364-1365`
    sampler_nearest: Sampler,

    /// The weather bind group for one cloud: its image with the clamp wrap `R_FindImageFile` gave it, and the filter its `mFilterMode` chose.
    pub fn weather_bind_group(&self, gpu: &Gpu, handle: Option<ImageHandle>, nearest: bool) -> BindGroup
```

`layout`, `bind_group`, `world_bind_group`, `view_bind_group`, `upload_pending`, and `upload_staged` keep their signatures and bodies.

### `crates/mp/renderer-gpu/src/pipeline3d.rs`

One new method on `Pipeline3d`, so the weather pass reuses the depth texture the world pass wrote, the cached pipelines, and the globals buffer:

```rust
    /// The GPU half of Raven `CWeatherParticleCloud::Render`: one draw per cloud, after the world pass, depth-tested and depth-write off.
    /// The pass sets its own viewport and scissor from `view`, the same values the world pass used, because `SetViewportAndScissor` is retired at the CPU site.
    /// The pass draws two-sided. That is faithful, not incidental: Raven sets `GL_Cull(CT_TWO_SIDED)` for weather at `oracle/codemp/renderer/tr_WorldEffects.cpp:1362`.
    /// Returns the vertex count drawn, which the frame stats report.
    ///
    /// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:1311-1480`
    pub fn draw_weather(
        &mut self,
        gpu: &Gpu,
        target: &TextureView,
        weather: &WeatherFrame,
        view: &viewParms_t,
        gpu_images: &mut GpuImages,
    ) -> u32
```

`WorldVertex`, `SurfaceRange`, `WorldGeometry`, `PipelineKey`, `build_world_pipeline`, `ensure_pipeline`, `collect_stage_items`, and `draw` all keep their shapes. `draw_weather` builds its `PipelineKey` from `blend_state_from_gls(batch.state_bits)` with `depth_write` false and no bias, and reuses `ensure_pipeline`. `PipelineKey` gains no cull field in this step, per row 12.

### `crates/mp/renderer-gpu/src/frame_exec.rs`

`FrameExecutor` gains one arm in the event walk and nothing else:

```rust
                FrameEvent::WorldEffects(weather) => { /* rebuild the view from the scene refdef this arm follows, then Pipeline3d::draw_weather */ }
```

`WorldStats` gains one counter, `weather_vertices: u32`. `execute_frame`, `execute_package`, `set_world`, `drop_world`, and `render_world` keep their signatures.

### `crates/mp/engine/client/src/cl_cgame.rs` and `crates/mp/engine/client/src/cl_ui.rs`

Each `RE_RenderScene` trap arm gains one `RE_RenderWorldEffects` call directly after it, the oracle's own placement at `oracle/codemp/renderer/tr_scene.cpp:868`. Neither arm builds an `EngineHostView` today. `view` is a dispatcher parameter and the arms reach the renderer bundle with `re_from_view(view)` (DEC-59 ruling 1), and the new call takes the same route. No signature changes and no other edit in either file.

### `crates/mp/renderer-gpu/tests/world_golden.rs`

One new scene step and one new test, in the shape of `golden_world_marks_duel1`:

```rust
#[test] #[ignore] fn golden_world_weather_ctf2()
```

Row 7 holds its recipe, its bless procedure, and its defect conditions.

### Fixtures

One new PNG under `crates/mp/renderer-gpu/tests/goldens/`: `world_weather_ctf2.png`.

Anything not on this list is out of scope, and the agent must not add it. No new third-party crate, because a dependency of the DEC-49 kind is a user ruling and this packet may never grant one. No point-sprite path, no `mGLModeEnum` field, no port of `CWorldEffect`, `CWorldEffectsSystem`, or the header's `SParticle`. No deletion of `R_IsOutside`, `R_IsShaking`, `R_IsOutsideCausingPain`, `R_GetWindGusting`, `R_GetChanceOfSaberFizz`, or `FrameEvent::WorldEffectCommand`. No `r_we` console registration, which is this ticket's step-004. No cull field on `PipelineKey`. No new WGSL file and no change to `world.wgsl` or `world_pbr.wgsl`. No cvar, no ABI change, no change to any file under `crates/mp/game/`, `crates/mp/cgame/`, `crates/mp/ui/`, `crates/mp/uishared/`, or `crates/sp/`. Every committed fixture except the one new PNG is read-only.

## The settled rows

**Row 1 - the step boundary (user ruling, ratified 2026-08-31).** **Ratified: one step for the whole chain, commits 1 through 8.**

The ticket's own words for this group are "Rain and snow maps hit this in live play". Closing the five markers alone does not reach that. It leaves a cloud that computes correct physics and draws nothing, with no image gate to prove any of it, because the only honest gate for a billboard emitter is a picture. Under a split, step-001's marker corrections would be proved by nothing, which commit 1's own gate paragraph already concedes.

The split-point fact stays on record as history. Commits 1 through 4 add no producer for `FrameEvent::WorldEffects` and leave every fixture byte-identical, so they would have stood alone and green. The ruling takes the whole chain anyway.

**Row 2 - where the weather step runs, with row 5 folded in (user ruling, ratified as amended 2026-08-31).** **Ratified: trap-side in the `RE_RenderScene` arm, gated on the submitted scene's own `rdflags` carrying neither `RDF_NOWORLDMODEL` nor `RDF_SKYBOXPORTAL`, with the batch crossing as a `FrameEvent`.**

The premise holds. `FrameExecutor::execute_frame` (`crates/mp/renderer-gpu/src/frame_exec.rs:417`) takes `gpu`, `target`, `target_texture`, `frame_data`, `assets`, `world_load`, `uploads`, `gpu_images`, `noise`, `float_time`, and `cvars`. It holds `&mut self`, `&mut Gpu`, and `&mut GpuImages`, and it holds no `EngineHostView`, no collision world, and no mutable `WorldEffectsState`. `COutside::Cache` needs the first two (`crates/mp/renderer/src/tr_worldeffects/world_effects.rs:634`, `CM_PointContents` at `oracle/codemp/renderer/tr_WorldEffects.cpp:598,645`), and `RB_RenderWorldEffects` needs the third.

The trap arm reaches all of it through `re_from_view(view)`, the DEC-59 ruling 1 route the two existing weather traps already take (`crates/mp/engine/client/src/cl_cgame.rs:3623-3643`). The placement matches the oracle, because `RE_RenderWorldEffects` is the statement after the scene in `RE_RenderScene` (`oracle/codemp/renderer/tr_scene.cpp:868`).

**The gate preserves once-per-frame semantics.** The oracle queues one marker per scene and the port makes one call per `RE_RenderScene` trap arm, so the counts match. With the ruled gate, an icon scene or a sky-portal scene never steps, and a normal frame steps exactly once with its world scene.

**Divergence 4, named and ruled cosmetic.** Raven reads `RDF_NOWORLDMODEL` from `tr.refdef`, the front-end copy:

```c
	if (!tr.world ||
		(tr.refdef.rdflags & RDF_NOWORLDMODEL) ||
		(backEnd.refdef.rdflags & RDF_SKYBOXPORTAL) ||
```

`oracle/codemp/renderer/tr_WorldEffects.cpp:1515-1517`. Every command runs at backend time, after the front end finished every scene, so `tr.refdef` holds the **last** scene's flags. `CG_Draw3DModel` sets `refdef.rdflags = RDF_NOWORLDMODEL;` (`oracle/codemp/cgame/cg_draw.c:485`) and runs from `CG_Draw2D`, after `CG_DrawActive`'s world scene (`oracle/codemp/cgame/cg_draw.c:8573`). So on any retail frame that draws a 3D icon, every weather command that frame returns early, and weather neither steps nor draws for that whole frame. `cg_draw3dIcons` and `cg_drawIcons` both default on, so this is the ordinary case and not an edge.

The port reads each submitted scene's own refdef, so it steps and draws once on exactly those frames. The user ruled the difference cosmetic. Weather runs at a steady rate in the port where retail stutters it against the icon-drawing frames, and no gate observes it.

**Row 5 folds in here.** The orientation and both `rdflags` bits read from the same submitted scene's refdef. The orientation half is exact: Raven fills `parms.ori` from the scene refdef (`oracle/codemp/renderer/tr_scene.cpp:848-851`) and the port reproduces it at `crates/mp/renderer-gpu/src/frame_exec.rs:812-813`, so the refdef gives the identical values with no ABI struct and no new state. The `rdflags` half is the collapse of Raven's two refdefs into the port's one, and that collapse is the mechanism of divergence 4 above. It is recorded there, not dismissed. The draft's sentence saying the port's ordering "cannot produce" the differing case is deleted, because that case is precisely what retail hits on every 3D-icon frame.

**The precedent is DEC-65 ruling 2, not DEC-37 ruling 2.** DEC-65 ruling 2 already ruled this exact shape: the transform runs sim-side at scene-add time, plain per-entity data crosses in the frame package, Raven transforms at backend draw time, and the image-golden gate verifies the parity. Weather differs from bone matrices in one way the lane must hold in mind. Bone matrices are stateless per frame, so a timing shift is invisible. `CWeatherParticleCloud::Update` integrates velocity and advances two random streams, so a step-count or step-gate difference compounds across frames. That is why divergence 4 needed a ruling rather than a note.

**Row 3 - `WindZoneState`'s owner (mechanical, ratified 2026-08-31).** **Ratified as drafted: a `wind` field on `WorldEffectsState`.**

Raven's `mGlobalWindVelocity`, `mGlobalWindDirection`, and `mGlobalWindSpeed` are file statics in the same TU as `mOutside`, `mParticleClouds`, and `mWindZones` (`oracle/codemp/renderer/tr_WorldEffects.cpp:73-75`). Those three already live on `WorldEffectsState` under DEC-37 A13.3, and the wind trio belongs beside them. `RB_RenderWorldEffects` then drops its `wind` parameter and writes `self.wind`. The two `tr_surfacesprites` call sites keep their `&WindZoneState` parameter and borrow the field.

The alternative is a second field on `RendererFrontend` beside `world_effects`, which splits one TU's statics across two owners for no reason.

**Row 4 - the two generators (mechanical, ratified 2026-08-31).** **Ratified as drafted: one, `self.rng`, and the `rng` parameter is dropped.**

DEC-66 ruling 1 already settles it: owner-embedded instances are the `rng.rs` doctrine, and world effects already runs the pattern renderer-side. `WorldEffectsState::rng` is a single `native_math::rng::Rng`, and that type already carries both of Raven's streams separately (`crates/native/math/src/rng.rs:113,121`). The port's `WE_flrand` draws from the CRT stream (`crates/mp/renderer/src/tr_worldeffects/world_effects.rs:66-68`), matching `oracle/codemp/renderer/tr_WorldEffects.cpp:13-15`. `RB_RenderWorldEffects`'s separate `rng: &mut Rng` parameter has no caller and would be a third stream with no oracle twin.

The borrow works because the fields are disjoint. `self.mParticleClouds[i].Update(&mut self.rng, &self.mOutside, ...)` borrows three different fields of `self`.

**Row 5 - the view orientation's source.** Merged into row 2 and ruled there.

**Row 6 - `SetViewportAndScissor` at this site (mechanical, ratified 2026-08-31).** **Ratified as drafted: retire the call, keep a one-line note with its `Source:` cite.**

`SetViewportAndScissor` is a deferred no-op with an empty body (`crates/mp/renderer/src/tr_backend.rs:210-213`). It is the only reason `RB_RenderWorldEffects` takes `frame: &FrameState`, and the parameter is otherwise unread. The real viewport work belongs to the render pass, and the surface contract now says so: `Pipeline3d::draw_weather` sets its viewport and scissor from the `view` it takes.

The same applies to `qglLoadMatrixf(backEnd.viewParms.world.modelMatrix)` (`oracle/codemp/renderer/tr_WorldEffects.cpp:1525`). Its existing `DEFERRED` note at `crates/mp/renderer/src/tr_worldeffects/world_effects.rs:1435-1439` stays, because `draw_weather` supplies the world clip matrix on the GPU side.

**Row 7 - the live gate (user ruling, ratified as amended 2026-08-31).** **Ratified: one new world golden on `maps/mp/ctf2.bsp`, with the recipe replaced whole. The unit tests stand as drafted.**

`ctf2` is the only stock MP map that ships weather. All 23 stock MP maps were scanned. Its entity lump carries **two** `fx_snow` entities, at origins `1552 3408 336` and `-944 656 336`, each with `count 1000`, plus three `misc_weather_zone` brushes with models `*1`, `*2`, and `*3`. The doubling is inert, because `G_FindConfigstringIndex` returns the existing index for a string already registered (`oracle/codemp/game/g_utils.c:74-82`). The `count` key is dead for snow: `SP_CreateSnow` never reads `ent->count`, and the 1000 particles come from the preset's own `nCloud.Initialize(1000, "gfx/effects/snowflake1.bmp");` (`oracle/codemp/renderer/tr_WorldEffects.cpp:1808`). `SP_CreateRain` is the one that reads `count`.

The rig cannot run the game and cgame chain that normally issues the commands, so the fixture calls the parser directly, the same way `golden_world_marks_duel1` calls `RE_RegisterShader` directly. `R_WorldEffectCommand` is a method taking seven arguments besides the receiver (`crates/mp/renderer/src/tr_worldeffects/world_effects.rs:1560-1569`) whose `command` is `Option<&[u8]>`, so the test builds the full host bundle and coerces its byte literals to slices. Everything it touches is public, so this needs no production surface.

The recipe:

1. Boot `maps/mp/ctf2.bsp` and issue the three commands `SP_CreateSnow` implies, in its order: `snow`, `fog`, and `constantwind (100 100 -100)` (`oracle/codemp/game/g_misc.c:2522-2527`). The single-command draft was wrong. Without `constantwind` the global wind velocity is zero and the snow falls straight down under gravity alone, which is not the retail picture. The triple builds two particle clouds, `snow` at `oracle/codemp/renderer/tr_WorldEffects.cpp:1798` and `fog` at `:1879`, plus one global wind zone at `:1662`.
2. Reseed **both** `Rng` streams to fixed constants, after the weather commands and before stepping. `Rng::srand` seeds the CRT state alone (`crates/native/math/src/rng.rs:187-189`) and `Rng::Rand_Init` seeds `holdrand` (`:145`), and weather draws from both. The `holdrand` stream is live on this exact path, because the `snow` preset sets `nCloud.mRotationChangeNext = 0;` (`oracle/codemp/renderer/tr_WorldEffects.cpp:1811`), which fires `mRotationChangeTimer.Pick` at `:1071` on the first update. The draft's single `srand(0)` pinned one stream of two.
3. Submit the scene through roughly sixty fixed-dt steps, advancing `fd.time` by the step on each call, so `frametime` is a real number rather than the zero a repeated frozen clock yields (`crates/mp/renderer/src/tr_scene.rs:1248-1249`). The fade then reaches its 0.75 ceiling before capture. The draft's arithmetic was wrong on this point and its expected image followed the error: at one frozen clock the spawning update runs at `mSecondsElapsed = 0.001`, `particleFade` is 0.01, and a near-blank image is the correct result, which would have sent the lane hunting a bug that is not there.
4. Render and capture the final step, then compare.

Named defect conditions, rewritten. The correct image is the `ctf2` scene with visible developed particles and the fog overlay over it. An image with no discernible weather is the defect. The twenty-one existing fixtures must stay byte-identical. A panic in `mParticles` or `mPointCache` means an index escaped, and zero drawn vertices with a non-zero `mParticleCountRender` means the batch crossed empty.

The bless is eyes-on and stops, as every image fixture does: run once with `JKA_GOLDEN_BLESS=1`, run again without it to confirm the byte-identical pass, then STOP before the commit that carries the PNG so the user looks at the image.

The determinism ruling is minted rather than borrowed. The Amendments section carries its text, and it lands as a dated DEC-66 amendment in `docs/decisions.md` when the step merges. The draft's claim that DEC-66 ruling 4's clause already covered this was not honest: that clause names scene-order fragility as the trigger, the trigger here is a faithful wall-clock seed, and "graduates as its own ruling" means a DEC row the draft never proposed.

The unit tests stand and are cheap, needing no assets and no GPU: the two-draw count on the spawn-plane respawn per row 9, `SVecRange::Wrap` across each axis, the fade machine's four transitions, and the render count's re-read of the flag.

**Row 8 - the nearest sampler (mechanical, ratified as amended 2026-08-31).** **Ratified: `GpuImages::sampler_nearest` and the filter-mode plumb land as contracted. The gate claim is corrected.**

Raven sets `GL_TEXTURE_MIN_FILTER` and `GL_TEXTURE_MAG_FILTER` per cloud, both to `GL_LINEAR` or both to `GL_NEAREST` (`oracle/codemp/renderer/tr_WorldEffects.cpp:1364-1365`). Every rain preset sets `mFilterMode = 1` (`:1718`, `:1739`, `:1760`, `:1788`), so the nearest case is live in play from commit 6 onward. `GpuImages` builds a repeat and a clamp sampler and nothing else (`crates/mp/renderer-gpu/src/gpu_images.rs:140-141`), and the weather image always loads with `GL_CLAMP` (`crates/mp/renderer/src/tr_worldeffects/world_effects.rs:1005-1016`), so one nearest-clamp sampler covers it. Neither weather filter uses mips, because Raven sets the plain `GL_LINEAR`, not `GL_LINEAR_MIPMAP_LINEAR`.

The correction: the `snow` branch (`:1798-1817`) never touches `mFilterMode`, so it keeps the `Reset` default of 0 (`:988`). The ctf2 golden therefore exercises the linear path only and never binds `sampler_nearest`. That surface lands with no automated gate. The finished file records it in the open gaps, and rain in live play verifies it.

**Row 9 - the broadcast rand draws (mechanical, ratified 2026-08-31).** **Ratified as drafted: one `WE_flrand` draw per line at `:1216-1217`, broadcast to all three components.**

`CVec3` declares `inline CVec3 operator* (const CVec3 &t) const` (`oracle/codemp/Ravl/CVec.h:628`) and no binary scalar multiply. The only scalar form is the compound `operator*=` at `:620`, which cannot appear in `a * b`. The non-explicit `CVec3(const float val)` at `:570` broadcasts. So each of the two lines draws once and scales x, y, and z with the same value. A per-component write draws six times instead of two and shifts the stream for every later draw in the session.

`:1215` is the `mSpawnPlaneDistance` broadcast and draws nothing. By contrast `SVecRange::Pick` (`oracle/codemp/renderer/tr_WorldEffects.cpp:156-161`) genuinely is three draws in x, y, z order, and the port already has it right (`crates/mp/renderer/src/tr_worldeffects/world_effects.rs:182-186`). The lane must not unify the two.

**Row 10 - the colour quantization (mechanical, ratified 2026-08-31).** **Ratified as drafted: `WeatherVertex` carries the `f32` colour and the executor rounds to `WorldVertex`'s `[u8; 4]` at one site.**

`qglColor4f` takes floats and the fixed-function pipeline converts them to fixed point before interpolation. `WorldVertex` already carries `[u8; 4]` (`crates/mp/renderer-gpu/src/pipeline3d.rs:133-141`) and every other billboard in the port goes through it, so a weather-only float colour path would be a second vertex row for one subsystem. Its fields are private, so the executor builds through the existing constructor. Keeping the float value in the seam payload and rounding once at the vertex build keeps the divergence at one named site with a two-line note.

**The rule and the clamp are pinned here, not left to the transcriber.** Each channel is clamped to `[0.0, 1.0]` first, then multiplied by 255.0, then rounded to nearest with `.round()`, then cast to `u8`. This matters because `snow` uses blend mode 1, so its channels are `mColor[i] * part->mAlpha` (`oracle/codemp/renderer/tr_WorldEffects.cpp:1402`) and every pixel of the golden depends on whether the lane rounds or truncates.

**Row 11 - the uninitialized `mRotationChangeTimer.mMax` read (mechanical, ratified as amended 2026-08-31).** **Ratified: no code change, and the note text is rewritten to the proven fact.**

`SIntRange::Clear` writes `mMin = 0; mMin = 0;` (`oracle/codemp/renderer/tr_WorldEffects.cpp:229-233`), and `Reset` writes `mRotationChangeTimer.mMin = 500;` then `mRotationChangeTimer.mMin = 2000;` (`:999-1000`). `mMax` is never written and stays 0. The port reproduces it at `crates/mp/renderer/src/tr_worldeffects/world_effects.rs:955` and `:1109-1112`, and the zeroed `mMax` stands as the rule-19 defined behavior.

The note the lane writes must say the real consequence. `irand` increments `max` first and then shifts a signed product:

```c
	max++;
	holdrand = (holdrand * 214013L) + 2531011L;
	result = holdrand >> 17;
	result = ((result * (max - min)) >> 15) + min;
```

`oracle/codemp/game/q_math.c:1464-1467`. With `min = 2000` and `max = 0` this is `((result * -1999) >> 15) + 2000` over `result` in `[0, 32767]`, which lands uniformly in `[1, 2000]`. One `holdrand` draw per pick, and the `if (mRotationChangeNext<=0)` clamp at `:1072-1075` never fires. The port matches the arithmetic exactly, including the arithmetic right shift (`crates/native/math/src/rng.rs:62-65`). So the reversed range yields a pseudo-random rotation interval and the timer varies.

The draft's sentence claiming the port "reproduces the oracle's zero" and that the clamp floors the result at 1 is deleted. It was false, and left in place the lane would have written a false fact into the source.

This row is live on the golden path, because the `snow` preset sets `mRotationChangeNext = 0` (`oracle/codemp/renderer/tr_WorldEffects.cpp:1811`) and the pick fires on the first update.

**Row 12 - the contract gaps and the correction batch (mechanical, ratified as amended 2026-08-31).**

1. **The cull mode is faithful, and it enters the contract.** Weather draws with `GL_Cull(CT_TWO_SIDED)` (`oracle/codemp/renderer/tr_WorldEffects.cpp:1362`), so `cull_mode: None` for this pass is the oracle's own instruction and not an accident of the current world pipeline. `draw_weather`'s doc cites that line, and the temporary-accident framing is dropped. `PipelineKey` gains no cull field in this step.
2. **`WeatherFrame` carries no view, and the positional invariant stands in its place.** One weather batch per frame, built from the world scene's refdef, with its `FrameEvent` emitted inside that scene's event span so the executor draws it under the view that built it. The lane states this in the type's doc and in `draw_weather`'s. A future multi-view consumer revisits it as its own ruling.
3. **The correction batch lands.** The `orientationr_t` marker is **mooted** by row 5's source, not stale, and commit 1's note says so. `CHANNEL_TOLERANCE` is zero in four suites, not five. `ctf2` carries two `fx_snow` entities and `snow` never reads `count`. Every cite the audit corrected is applied above: `RB_WorldEffects` at `oracle/codemp/renderer/tr_backend.cpp:1886-1905`, the wind statics at `oracle/codemp/renderer/tr_WorldEffects.cpp:73-75`, `SP_CreateSnow` at `oracle/codemp/game/g_misc.c:2522-2527` with `SP_CreateRain` at `:2537`, `CG_ParseWeatherEffect` at `oracle/codemp/cgame/cg_main.c:1395-1400` and `crates/mp/cgame/src/cg_main.rs:1554-1557`, `CG_CreateWeatherZoneFromSpawnEnt` at `crates/mp/cgame/src/cg_main.rs:2674-2677`, the cgame configstring branch at `oracle/codemp/cgame/cg_servercmds.c:807-816`, `WorldVertex` at `crates/mp/renderer-gpu/src/pipeline3d.rs:133-141` with `add_quad_stamp_ext` at `:4875-4909` and `build_world_pipeline` at `:3401`, `blend_state_from_gls` at `crates/mp/renderer-gpu/src/blend.rs:61`, `boot::load_world` at `crates/mp/renderer-gpu/src/ui_host/boot.rs:472-519`, the `COutside::Cache` fall-through at `crates/mp/renderer/src/tr_worldeffects/world_effects.rs:707-710` on `!self.mCacheInit`, the second `todo!()` at `:1292-1294`, and the `WindZoneState` parameter sites at three places rather than two.
4. **The DEC-66 amendment text** is in the Amendments section below, ready to land in `docs/decisions.md` when the step merges.

## Pause triggers, named for this step

- Any committed fixture other than the new PNG moves. STOP. No commit before commit 6 has a producer for the weather event, so a session with no weather command draws exactly what it drew before.
- `partInRange = true;` at `oracle/codemp/renderer/tr_WorldEffects.cpp:1226` looks like it should be followed by a recomputed `partInView`. STOP. The assignment is a dead store in the oracle and stays dead. `partInView` was computed at `:1202`, and the fade machine at `:1231-1291` and the render count at `:1295-1298` read only `partRendering`, `partInView`, and `part->mFlags`.
- `GpuImages` turns out not to expose what `weather_bind_group` needs. STOP and report, per row 8. Do not copy the texture or build a second image store.
- The weather draw looks like it should be a `DrawSurf` with a synthesized shader. STOP. A cloud binds a raw `image_t` with no `shader_t` in the oracle, and inventing a shader is speculative behavior.
- A second vertex row for weather looks necessary. STOP, per row 10. `WorldVertex` covers it.
- `cull_mode: None` looks like an accident of the world pipeline that should be pinned with a new `PipelineKey` field. STOP, per row 12. Two-sided is what Raven asks for at `oracle/codemp/renderer/tr_WorldEffects.cpp:1362`, and the cull field is not this step.
- The per-frame force or friction looks frame-rate dependent and wrong. STOP. It is frame-rate dependent in the oracle too (`oracle/codemp/renderer/tr_WorldEffects.cpp:1193-1194`), and only the position step is time-scaled.
- The respawn multiply looks like it needs one draw per component. STOP, per row 9. It is one draw broadcast to three components.
- The first weather frame draws nothing and that looks like a bug. STOP. The cache-building frame renders nothing in the oracle too (`oracle/codemp/renderer/tr_WorldEffects.cpp:1544-1547`), and row 7's fixture steps many frames for that reason among others.
- `mRotationChangeTimer.mMax` looks uninitialized and fixable. STOP, per row 11. Write the `[1, 2000]` note, not the false one.
- `R_InitWorldEffects`'s wall-clock seed looks like it should change in production. STOP, per row 7 and the Amendments. The reseed is fixture-only.
- The `r_we` console command looks like it belongs here. STOP. It is this ticket's step-004.
- The five caller-less weather functions look like dead code to delete. STOP. Deletion is not this step's work.
- `RE_RenderScene`'s signature looks like it should take the host and the world-effects state. STOP, per row 2. The call sits in the trap arm.
- `WeatherFrame` looks like it should carry its view. STOP, per row 12. The ordering invariant is the contract, and changing it is a new ruling.
- Verification is `cargo build` or `cargo check` plus the golden suites, never rust-analyzer, which is stale in this workspace.

## Commit bundle

The full gate battery, named once and referenced per commit. Every golden run is serial with `--test-threads=1`, each as one foreground command with a long timeout. Two engine boots in parallel threads crash in the GPU init path, and the world-golden pk3 inflate aborts without it.

- `cargo build --workspace`. An intermediate commit may carry warnings, and the bundle's final state must build with zero warnings.
- `cargo test --workspace -- --test-threads=1`.
- `cargo test -p mp_renderer_gpu --test world_golden -- --ignored --test-threads=1`, all five world goldens byte-identical.
- `cargo test -p mp_renderer_gpu --test scene_golden -- --test-threads=1`, all eleven scene goldens byte-identical.
- `cargo test -p mp_renderer_gpu --test entity_golden -- --ignored --test-threads=1`, byte-identical.
- `cargo test -p mp_renderer_gpu --test ghoul2_vertex_golden -- --ignored --test-threads=1`, byte-identical.
- `cargo test -p mp_renderer_gpu --test hud_golden -- --test-threads=1` and the same with `--ignored`, both byte-identical.

Twenty-one committed fixtures, one per test. `CHANNEL_TOLERANCE` is zero in the four image suites, and the ghoul2 suite compares a vertex `.bin`. The lockstep referee is not required, because no commit touches `mp_game`, the server, or any `jampded` link-set crate.

1. **The four stubbed reads land.** All four type-stub markers close. `Update` reads the view origin and axis from its new parameters, `RB_RenderWorldEffects` reads `refdef.rdflags`, `refdef.frametime`, and `assets.world.bmodels[0].bounds`, and the three stale doc comments are corrected. The `orientationr_t` note says the marker is mooted by the refdef source, not that the field already exists, per row 12. The per-particle `todo!()` stays. Files: `crates/mp/renderer/src/tr_worldeffects/world_effects.rs`. Subject: `fix(gh#54 s001): the four world-effects reads land`. Gates: the full battery, all twenty-one fixtures byte-identical. Nothing calls this code, so the battery proves the build and nothing more.

2. **The particle loop.** The fifth marker closes. `Update` gains `outside: &COutside` and the loop is transcribed: the first-time spawn, the integration, the classification, the respawn with its dead store intact, the fade machine, and the render count. The row-11 note lands at the `Pick` call. Unit tests cover the row-9 draw count, `SVecRange::Wrap`, the fade transitions, and the render count. Files: `crates/mp/renderer/src/tr_worldeffects/world_effects.rs`. Subject: `feat(gh#54 s001): the weather particle loop`. Gates: the full battery, all twenty-one fixtures byte-identical.

3. **The wind trio finds its owner.** `WorldEffectsState::wind`, per row 3. `RB_RenderWorldEffects` and `RB_WorldEffects` drop their `wind` and `rng` parameters per rows 3 and 4, and take `refdef: &TrRefdef` in place of `frame: &FrameState` per row 6. The `tr_surfacesprites` call sites borrow the field. Files: `crates/mp/renderer/src/tr_worldeffects/world_effects.rs`, `crates/mp/renderer/src/tr_backend.rs`, `crates/mp/renderer/src/tr_surfacesprites.rs`. Subject: `feat(gh#54 s001): the wind zone state finds its owner`. Gates: the full battery, all twenty-one fixtures byte-identical.

4. **The cloud renders into a batch.** `WeatherVertex`, `WeatherCloudBatch`, and `WeatherFrame`, with the row-12 positional invariant in the payload doc. `Render` emits the triangle and quad arms into a batch instead of only counting, and `RB_RenderWorldEffects` returns the frame. Unit tests cover both vertex arms' offsets and UVs and both blend modes' colour. Still no caller. Files: `crates/mp/renderer/src/render_state/weather_frame.rs`, `crates/mp/renderer/src/render_state/mod.rs`, `crates/mp/renderer/src/tr_worldeffects/world_effects.rs`, `crates/mp/renderer/src/tr_backend.rs`. Subject: `feat(gh#54 s001): the cloud renders into a frame batch`. Gates: the full battery, all twenty-one fixtures byte-identical.

5. **The GPU pass.** `FrameEvent::WorldEffects`, the executor arm, `Pipeline3d::draw_weather` with its viewport, cull, and quantization notes, `GpuImages::sampler_nearest` and `weather_bind_group`, and the `WorldStats::weather_vertices` counter. No producer yet, so the pass never runs. Files: `crates/mp/renderer/src/render_state/frame_event.rs`, `crates/mp/renderer-gpu/src/frame_exec.rs`, `crates/mp/renderer-gpu/src/pipeline3d.rs`, `crates/mp/renderer-gpu/src/gpu_images.rs`. Subject: `feat(gh#54 s001): the weather pass draws a frame batch`. Gates: the full battery, all twenty-one fixtures byte-identical, which is the proof the new pass is inert with no producer.

6. **The frame drives world effects.** `RE_RenderWorldEffects` in `tr_cmds.rs` and the two trap-arm calls, with the row-2 gate on the submitted scene's own `rdflags`. The `DEFERRED` notes in `tr_cmds.rs` and `tr_scene.rs` retire. Weather now runs and draws in live play, and divergence 4 becomes observable from here. Files: `crates/mp/renderer/src/tr_cmds.rs`, `crates/mp/renderer/src/tr_scene.rs`, `crates/mp/engine/client/src/cl_cgame.rs`, `crates/mp/engine/client/src/cl_ui.rs`. Subject: `feat(gh#54 s001): the frame drives world effects`. Gates: the full battery, all twenty-one fixtures byte-identical. No existing fixture issues a weather command, so the drive is inert in every one of them.

7. **The weather golden.** `golden_world_weather_ctf2` and its PNG, on the row-7 recipe and after its bless STOP. Files: `crates/mp/renderer-gpu/tests/world_golden.rs` and the new PNG. Subject: `test(gh#54 s001): the weather world golden`. Gates: the full battery, with the new golden green at tolerance zero and the other twenty-one fixtures byte-identical.

8. **The finished file**, per the packet skill: assumptions and choices keyed to their commits, deviations or the word "none", the commit list with gate results, and open gaps. The open gaps must name three: `sampler_nearest` has no automated gate per row 8, the golden proves neither zone nor cache behavior because the rig has no collision world and no cgame zones, and divergence 4 is verified by live play alone. File: `.claude/packets/54/step-001/finished.md`. Subject: `process(gh#54 s001): finished file`.

Every commit uses `--no-gpg-sign`, a heading subject, an STE body, and no trailer of any kind: no `Co-Authored-By`, no generated-with footer. Gate results are written as plain sentences inside the body, and a gate paragraph opens with prose, so no line parses as a git trailer.

## Write scopes

Branch `gh54-step-001-weather-lane`, cut from `wf/54-renderer-complement`. A worktree builder runs `git merge wf/54-renderer-complement --no-gpg-sign` as its first act.

- `crates/mp/renderer/src/tr_worldeffects/world_effects.rs` - the five markers, the three signatures, the `wind` field, and the unit tests.
- `crates/mp/renderer/src/render_state/weather_frame.rs` - new, the three seam types.
- `crates/mp/renderer/src/render_state/mod.rs` - the one `pub mod` line.
- `crates/mp/renderer/src/render_state/frame_event.rs` - the one new variant only.
- `crates/mp/renderer/src/tr_backend.rs` - `RB_WorldEffects` only.
- `crates/mp/renderer/src/tr_surfacesprites.rs` - the two `WindZoneState` call sites only.
- `crates/mp/renderer/src/tr_cmds.rs` - `RE_RenderWorldEffects` and the retiring note only.
- `crates/mp/renderer/src/tr_scene.rs` - the retiring note only.
- `crates/mp/renderer-gpu/src/pipeline3d.rs` - `draw_weather` and its imports.
- `crates/mp/renderer-gpu/src/gpu_images.rs` - `sampler_nearest` and `weather_bind_group` only.
- `crates/mp/renderer-gpu/src/frame_exec.rs` - the new event arm and the stats counter only.
- `crates/mp/engine/client/src/cl_cgame.rs`, `crates/mp/engine/client/src/cl_ui.rs` - one added call each, after `RE_RenderScene`.
- `crates/mp/renderer-gpu/tests/world_golden.rs` - the new scene step and test.
- `crates/mp/renderer-gpu/tests/goldens/world_weather_ctf2.png` - new, blessed under the row-7 STOP.
- `.claude/packets/54/step-001/` for `finished.md`, for session-directed `packet.md` tail appends, and for the vet's `vet.md`.

Any other caller `cargo check` shows broken by the three changed signatures is in scope on edit-only terms.

Everything else is read-only, including `oracle/`, every file under `crates/mp/game/`, `crates/mp/cgame/`, `crates/mp/ui/`, `crates/mp/uishared/`, `crates/sp/`, every WGSL shader, every other committed fixture, and `~/Developer/jka/` beyond read-only asset reads. Source files change through the Edit tool only.

## Disposition

After a clean lane-review: open the pull request from `gh54-step-001-weather-lane` into `wf/54-renderer-complement` and merge it on GitHub with a merge commit, per DEC-67. The umbrella branch merges to master once at the end of the gh#54 campaign, not per step. Never squash, and never commit on master. The session never pushes or opens the pull request unprompted. It prepares the branch, asks, and the user rules on the push and on the merge.

## Amendments

**2026-08-31 - the ratification walk closed all twelve rows.** The audit is at `.claude/packets/54/step-001/audit.md` (`53ec6a62`). Rows 3, 4, 6, 9, and 10 are ratified as drafted and audit-cleared. Rows 1, 2, 5, 7, 8, 11, and 12 are ratified as amended, and each row above carries its folded text.

- Row 1, the step boundary: one step for the whole chain. The commit 1 through 4 split-point sentence stays as history.
- Row 2, with row 5 merged: trap-side in the `RE_RenderScene` arm, gated on the submitted scene's own `rdflags`. Divergence 4 is named and ruled cosmetic. The two-refdef collapse is recorded inside that divergence and is no longer dismissed. The precedent cite is corrected to DEC-65 ruling 2.
- Row 7, the live gate: the recipe is replaced whole. The retail command triple, the both-stream reseed, roughly sixty advancing fixed-dt steps, and rewritten defect conditions.
- Row 8, the nearest sampler: the surface lands as contracted, and the gate claim corrects. Snow leaves `mFilterMode` at 0, so no automated gate binds `sampler_nearest`.
- Row 11, the uninitialized `mMax`: no code change, and the note rewrites to the proven `[1, 2000]` behavior. The false "reproduces the oracle's zero" sentence is deleted.
- Row 12, the contract gaps and corrections: the cull mode is faithful and cited, `WeatherFrame` carries the positional invariant instead of a view, and the eleven cite corrections plus the three fact corrections land.

**2026-08-31 - the DEC-66 determinism amendment, minted.** Golden fixtures may reseed both `Rng` streams to fixed constants after the weather commands and before stepping. The live path keeps the faithful wall-clock seed, `srand(Com_Milliseconds())` (`oracle/codemp/renderer/tr_WorldEffects.cpp:1491`, ported at `crates/mp/renderer/src/tr_worldeffects/world_effects.rs:1371`). `Rng::srand` seeds the CRT state alone and `Rng::Rand_Init` seeds `holdrand`, and weather draws from both, so pinning one stream does not pin the subsystem. This text lands as a dated DEC-66 amendment in `docs/decisions.md` when the step merges.

**2026-08-31 - the bind-group layout amendment, ruled as proposed.** `GpuImages::weather_bind_group` gains `layout: &BindGroupLayout` as its second parameter. The full signature is `pub fn weather_bind_group(&self, gpu: &Gpu, layout: &BindGroupLayout, handle: Option<ImageHandle>, nearest: bool) -> BindGroup`, and `draw_weather` passes `&self.texture_layout`. The group this builds is group 1 of the world pipeline, because `draw_weather` reuses `ensure_pipeline`, and that group-1 layout is the four-entry world texture layout held privately at `crates/mp/renderer-gpu/src/pipeline3d.rs:936`. `GpuImages` owns only the two-entry 2D layout, so it cannot reach the world layout on its own. Both siblings that build a world-pipeline group already take the layout explicitly: `world_bind_group` at `crates/mp/renderer-gpu/src/gpu_images.rs:348` and `view_bind_group` at `:391`. Nothing else in the contract changes.
