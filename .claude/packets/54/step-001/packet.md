# Packet gh#54 step-001 - the weather group

## Scope

This step makes rain and snow run and draw. A map with weather builds real particle clouds on this client today, and nothing steps them and nothing draws them.

The step closes all five `TODO: Port` markers in `crates/mp/renderer/src/tr_worldeffects/world_effects.rs`: the four type stubs (`trRefdef_t::frametime`, `trRefdef_t::rdflags`, `orientationr_t` origin and axis, `bmodel_t` bounds) and `CWeatherParticleCloud::Update`'s per-particle loop. It then gives the subsystem the two things the markers do not cover: a per-frame caller, and a draw. It ends with one new world golden on `maps/mp/ctf2.bsp`, the one stock MP map that ships weather.

The step does not port the Xbox point-sprite path, the dead `CWorldEffect`/`CWorldEffectsSystem`/`SParticle` header classes, the five weather symbols with no MP caller, or the `r_we` console-command registration. It adds no cvar, no ABI surface, and no third-party crate. It touches no file under `crates/mp/game/`, `crates/mp/cgame/`, `crates/mp/ui/`, or `crates/sp/`, so the lockstep referee is not a gate here. It does touch `crates/mp/engine/client/`, which is not a `jampded` link-set crate, so the referee stays off the list.

Six oracle behaviors in this area are quirks, not defects, and this step preserves every one. Rows 5, 9, and 11 name them, and the lane must not correct any of them.

## The oracle, cited

### What runs the weather, and when

`RE_RenderScene` queues the weather command as the last act of a scene: `RE_RenderWorldEffects();` (`oracle/codemp/renderer/tr_scene.cpp:868`). That function fills one bufferless `RC_WORLD_EFFECTS` marker (`oracle/codemp/renderer/tr_cmds.cpp:291-300`), the backend dispatches it at `case RC_WORLD_EFFECTS:` (`oracle/codemp/renderer/tr_backend.cpp:1944`), and `RB_WorldEffects` flushes the tess batch, calls `RB_RenderWorldEffects`, and reopens the batch (`oracle/codemp/renderer/tr_backend.cpp:1885-1904`).

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

`rdflags` is read twice, from two different refdefs (`:1515-1517`). `RDF_NOWORLDMODEL` (value 1) comes from `tr.refdef`. `RDF_SKYBOXPORTAL` (value 8) comes from `backEnd.refdef`. Both constants already exist in the port (`crates/mp/renderer/src/tr_public/ref_flags.rs:41,57`). No other `RDF_*` bit is read in this TU.

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
4. Respawn (`:1204-1227`). Only a particle that is both out of range and not rendering respawns. With a spawn plane it lands on the plane. Without one, `mRange.Wrap` wraps it.
5. The fade machine (`:1229-1291`) and the render count (`:1295-1298`). The alpha ceiling is `mColor[3]`, not 1.0. The count re-reads the flag rather than the local, so a particle that faded out this frame is not counted.

**The rand-draw trap, `:1215-1217`.** These three lines look like the `VectorMA` multi-eval trap and are not:

```c
					part->mPosition		-= (mSpawnPlaneNorm* mSpawnPlaneDistance); 
					part->mPosition		+= (mSpawnPlaneRight*WE_flrand(-mSpawnPlaneSize, mSpawnPlaneSize)); 
					part->mPosition		+= (mSpawnPlaneUp*   WE_flrand(-mSpawnPlaneSize, mSpawnPlaneSize)); 
```

`CVec3` has no scalar `operator*`. The only multiply is `CVec3 operator*(const CVec3&)` (`oracle/codemp/Ravl/CVec.h:628`), and the float converts through the non-explicit broadcast constructor `CVec3(const float val)` (`oracle/codemp/Ravl/CVec.h:570`). So each line makes exactly **one** draw and that one value scales x, y, and z alike. A per-component transcription would draw six times instead of two and desynchronize the stream for the rest of the session.

### `CWeatherParticleCloud::Render` - `oracle/codemp/renderer/tr_WorldEffects.cpp:1311-1480`

The draw is a self-contained fixed-function GL block. There is no `shader_t`, no `tess`, no `RB_BeginSurface`, and no `drawSurf_t`. The cloud binds one `image_t` and issues its own geometry.

```c
		GL_State((mBlendMode==0)?(GLS_ALPHA):(GLS_SRCBLEND_ONE | GLS_DSTBLEND_ONE));
		GL_Bind(mImage);
```

`oracle/codemp/renderer/tr_WorldEffects.cpp:1319-1320`. `GLS_ALPHA` is `(GLS_SRCBLEND_SRC_ALPHA | GLS_DSTBLEND_ONE_MINUS_SRC_ALPHA)` (`oracle/codemp/renderer/tr_local.h:1683`). Neither value sets `GLS_DEPTHMASK_TRUE` or `GLS_DEPTHTEST_DISABLE`, so weather depth-tests and does not depth-write.

`mGLModeEnum` is `(mVertexCount==3)?(GL_TRIANGLES):(GL_QUADS)` and nothing else on the PC build (`:944`; the `GL_POINTS` assignment at `:941` is inside `#ifdef _XBOX`). Every point-sprite branch at `:1326`, `:1344`, `:1349`, and `:1463` is therefore dead here. The port never carried `mGLModeEnum`, and it does not need it.

The live state is `GL_Cull(CT_TWO_SIDED)` (`:1362`) and the per-cloud min and mag filter `(mFilterMode==0)?(GL_LINEAR):(GL_NEAREST)` (`:1364-1365`). Both filters are unmipmapped. Rain sets `mFilterMode = 1`, so the nearest case is live, not theoretical.

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

`oracle/codemp/renderer/tr_WorldEffects.cpp:1393-1403`.

The geometry is absolute world coordinates, because the loaded matrix is the plain world model matrix and nothing else is pushed. The triangle arm (`:1414-1430`) emits UVs `(1,0)`, `(0,1)`, `(0,0)` at offsets `0`, `+mCameraLeft`, `+mCameraLeftPlusUp`. The quad arm (`:1434-1459`) emits UVs `(0,0)`, `(1,0)`, `(1,1)`, `(0,1)` at offsets `-mCameraLeftMinusUp`, `-mCameraLeftPlusUp`, `+mCameraLeftMinusUp`, `+mCameraLeftPlusUp`.

### How weather turns on in MP

Two paths reach `R_WorldEffectCommand`, and the port carries both end to end.

`fx_snow` and `fx_rain` map entities call `G_EffectIndex("*snow")` and `G_EffectIndex(va("*rain init %i", ent->count))` (`oracle/codemp/game/g_misc.c:2521-2524,2538`), which registers the string as a `CS_EFFECTS` configstring (`oracle/codemp/game/g_utils.c:148-151`). cgame routes any `*`-prefixed effect string to `CG_ParseWeatherEffect` (`oracle/codemp/cgame/cg_servercmds.c:806-814`, `oracle/codemp/cgame/cg_main.c:1393-1399`), which strips the star and calls `trap_R_WorldEffectCommand`. A `misc_weather_zone` brush is server-side dead (`oracle/codemp/game/g_misc.c:3488-3494`) and cgame-live, reaching `trap_WE_AddWeatherZone` (`oracle/codemp/cgame/cg_main.c:3646-3649`).

The RMG path reads the `RMG_weather` cvar and issues the commands directly (`oracle/codemp/renderer/tr_arioche.cpp:99-112`). It only fires when `com_RMG` is set, so a static BSP never takes it.

No console command named `weather`, `rain`, or `snow` exists. The one console entry is `r_we`, registered in `R_Register` and gated on `sv_cheats` (`oracle/codemp/renderer/tr_init.cpp:1196`, `oracle/codemp/renderer/tr_WorldEffects.cpp:1583-1591`). That registration belongs to this ticket's step-004, not here.

## The port as it stands

### Everything except the frame loop already runs

`crates/mp/renderer/src/tr_worldeffects/world_effects.rs` is 2082 lines and almost entirely live. `R_WorldEffectCommand` runs in full with all nineteen branches (`:1560-2029`). `COutside::Cache` runs a real `CM_PointContents` scan (`:634-712`). `CWeatherParticleCloud::Initialize` calls a real `R_FindImageFile` (`:992-1061`). `CWindZone::Update` runs its full physics (`:360-389`).

The live chain reaches the renderer today. cgame's `CG_ParseWeatherEffect` (`crates/mp/cgame/src/cg_main.rs:1550-1556`) and `CG_CreateWeatherZoneFromSpawnEnt` (`crates/mp/cgame/src/cg_main.rs:2670-2677`) both fire, and the trap arms land at `crates/mp/engine/client/src/cl_cgame.rs:3623-3643`. So a weather map builds real clouds with real loaded textures and real point caches.

Two functions are stubs, and both sit on the frame path.

`CWeatherParticleCloud::Update` (`:1150-1297`) panics on its first statement. The `todo!()` at `:1170-1172` binds the four camera vectors, so the whole body below it is dead and `#[allow(unreachable_code, unused_variables)]` at `:1149` silences the compiler. The per-particle loop at `:1284-1295` has a second `todo!()` as its entire body.

`CWeatherParticleCloud::Render` (`:1317-1319`) is a counter and nothing else. Its deferral note names the fixed-function GL surface and DEC-37 A13.2 as the reason.

`RB_RenderWorldEffects` (`:1401-1504`) panics at the second term of its guard (`:1424-1426`) once a world is loaded.

### Nothing calls the frame path

`RB_RenderWorldEffects` has exactly one caller, `RB_WorldEffects` (`crates/mp/renderer/src/tr_backend.rs:904-921`). `RB_WorldEffects` has zero callers anywhere in the workspace. `RE_RenderWorldEffects` does not exist: `crates/mp/renderer/src/tr_cmds.rs:245-258` carries the `DEFERRED` note that escalated it, and `crates/mp/renderer/src/tr_scene.rs:1346-1350` carries the matching note at the oracle's own call site.

So no marker in this file can fire in play today. The symptom is silence, not a crash.

### The four stubbed reads all exist

Every value the four type-stub markers wait on is already on a landed struct. The marker comments that call them "not yet a field" are stale, and this step corrects them.

- `TrRefdef::frametime` is `pub frametime: i32` (`crates/mp/renderer/src/render_state/placeholders.rs:293`). `RE_RenderScene` computes it from `fd.time - scene.last_time`, clamps it to 0 through 500, and writes it (`crates/mp/renderer/src/tr_scene.rs:1248-1273`).
- `TrRefdef::rdflags` is `pub rdflags: i32` (`crates/mp/renderer/src/render_state/placeholders.rs:297`), written at `crates/mp/renderer/src/tr_scene.rs:1274`. `RDF_NOWORLDMODEL` and `RDF_SKYBOXPORTAL` are `crates/mp/renderer/src/tr_public/ref_flags.rs:41,57`.
- The view orientation is `TrRefdef::view_origin` and `TrRefdef::view_axis` (`crates/mp/renderer/src/render_state/placeholders.rs:281,283`). Row 5 explains why those two, and not `FrameState::view`.
- The world bounds are `assets.world.as_ref().map(|w| w.bmodels[0].bounds)`. `BModel::bounds` is `crates/mp/renderer/src/tr_bsp.rs:220`, filled by `R_LoadSubmodels` at `crates/mp/renderer/src/tr_bsp.rs:1749-1756`.

### The wind-zone carrier has no owner

`WindZoneState` (`crates/mp/renderer/src/tr_worldeffects/world_effects.rs:427-438`) holds Raven's `mGlobalWindDirection`, `mGlobalWindSpeed`, and `mGlobalWindVelocity` trio (`oracle/codemp/renderer/tr_WorldEffects.cpp:73-76`). It appears only as a parameter, at `crates/mp/renderer/src/tr_backend.rs:906` and `crates/mp/renderer/src/tr_surfacesprites.rs:555,918`. No struct holds one anywhere in the workspace, so nothing can call the functions that take it. Row 3 settles this.

### The draw side does not exist

`grep -rn -i "weather" crates/mp/renderer-gpu/src/` returns nothing. There is no weather pipeline, pass, arm, or vertex path in the GPU crate, and nothing there imports `tr_worldeffects`.

The nearest analogue is complete and reusable. `RT_SPRITE` builds a camera-facing quad from `view.ori.axis[1]` and `view.ori.axis[2]` (`crates/mp/renderer-gpu/src/pipeline3d.rs:5293-5325`), and `add_quad_stamp_ext` writes its four corners and six indices (`:4875-4908`). The one 3D vertex row is `WorldVertex`, 44 bytes with a `[u8; 4]` colour (`:133-147`). Blend state comes from `blend_state_from_gls(state_bits)` (`crates/mp/renderer-gpu/src/blend.rs:62`), and `PipelineKey` carries `blend`, `depth_equal`, `depth_write`, and `depth_bias` (`crates/mp/renderer-gpu/src/pipeline3d.rs:811`), so one blend mode costs one cached pipeline. Both existing pipeline builders hard-code `TriangleList` (`crates/mp/renderer-gpu/src/pipeline3d.rs:3435`, `crates/mp/renderer-gpu/src/pipeline2d.rs:441`), which is what the quad and triangle arms need.

So the draw needs no new vertex type, no new blend path, and no new WGSL. It needs an entry point, because a weather cloud is not a `DrawSurf` and has no `shader_t`, and it needs a nearest-filter sampler, because `GpuImages` builds only a repeat and a clamp sampler (`crates/mp/renderer-gpu/src/gpu_images.rs:140-141`) and rain asks for `GL_NEAREST`.

### The golden rig is a frozen clock, one frame per test

Every suite sets `const FROZEN_TIME_MS: i32 = 12345;` and renders exactly one frame (`crates/mp/renderer-gpu/tests/world_golden.rs:65,98,290`). There is no loop and no stepper. `SceneState::last_time` starts at 0, so the first `RE_RenderScene` yields `frametime = 12345` clamped to 500, and `mSecondsElapsed` is 0.5. That is reproducible.

`boot::load_world` takes any map path with no allowlist (`crates/mp/renderer-gpu/src/ui_host/boot.rs:472-510`), and it calls `RE_LoadWorldMap` alone. It never calls `CM_LoadMap`, so the collision world stays empty in the rig. `COutside::Cache` then reads contents 0 at every cell, falls through to `mCacheInit = true; mMarkedOutside = false;` (`crates/mp/renderer/src/tr_worldeffects/world_effects.rs:708-711`), and every point tests as outside. That is a rig property, not a port divergence, and it makes the golden stable.

`R_InitWorldEffects` seeds its generator from the wall clock: `self.rng.srand(Com_Milliseconds(host) as u32)` (`crates/mp/renderer/src/tr_worldeffects/world_effects.rs:1371`). Row 7 handles that. `Rng` already carries the two streams separately, `holdrand` for `Q_irand` and `crt_holdrand` for the C runtime `rand` (`crates/native/math/src/rng.rs:103-131`), so the oracle's two-stream split is already correct.

### Twenty-one fixtures, twenty-one tests

`crates/mp/renderer-gpu/tests/goldens/` holds twenty PNG files and one BIN, one per test, across five suites. `CHANNEL_TOLERANCE` is 0 in all five, so every comparison is byte-exact.

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

### `crates/mp/renderer/src/tr_worldeffects/world_effects.rs`

`WorldEffectsState` gains one field:

```rust
    /// Raven's `mGlobalWindDirection`, `mGlobalWindSpeed`, and `mGlobalWindVelocity` file statics, the same DEC-37 A13.3 promotion as `mOutside`.
    /// The trio had no owner before this step, so nothing could call the functions that take it.
    ///
    /// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:73-76`
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

`Update` drops the unused `_frame: &FrameState`. `RB_RenderWorldEffects` drops `wind: &mut WindZoneState` per row 3, drops `rng: &mut Rng` per row 4, and takes `refdef: &TrRefdef` in place of `frame: &FrameState` per row 6.

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
/// Raven `RE_RenderWorldEffects` - queues the scene's weather pass.
/// Raven's bufferless `RC_WORLD_EFFECTS` marker becomes the frame's `WorldEffects` event, which carries the batch the pass draws.
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

`WorldVertex`, `SurfaceRange`, `WorldGeometry`, `PipelineKey`, `build_world_pipeline`, `ensure_pipeline`, `collect_stage_items`, and `draw` all keep their shapes. `draw_weather` builds its `PipelineKey` from `blend_state_from_gls(batch.state_bits)` with `depth_write` false and no bias, and reuses `ensure_pipeline`.

### `crates/mp/renderer-gpu/src/frame_exec.rs`

`FrameExecutor` gains one arm in the event walk and nothing else:

```rust
                FrameEvent::WorldEffects(weather) => { /* rebuild the view from the last scene refdef, then Pipeline3d::draw_weather */ }
```

`WorldStats` gains one counter, `weather_vertices: u32`. `execute_frame`, `execute_package`, `set_world`, `drop_world`, and `render_world` keep their signatures.

### `crates/mp/engine/client/src/cl_cgame.rs` and `crates/mp/engine/client/src/cl_ui.rs`

Each `RE_RenderScene` trap arm gains one `RE_RenderWorldEffects` call directly after it, the oracle's own placement at `oracle/codemp/renderer/tr_scene.cpp:868`. No signature changes and no other edit in either file.

### `crates/mp/renderer-gpu/tests/world_golden.rs`

One new scene step and one new test, in the shape of `golden_world_marks_duel1`:

```rust
#[test] #[ignore] fn golden_world_weather_ctf2()
```

Row 7 holds its recipe, its bless procedure, and its defect conditions.

### Fixtures

One new PNG under `crates/mp/renderer-gpu/tests/goldens/`: `world_weather_ctf2.png`.

Anything not on this list is out of scope, and the agent must not add it. No new third-party crate, because a dependency of the DEC-49 kind is a user ruling and this packet may never grant one. No point-sprite path, no `mGLModeEnum` field, no port of `CWorldEffect`, `CWorldEffectsSystem`, or the header's `SParticle`. No deletion of `R_IsOutside`, `R_IsShaking`, `R_IsOutsideCausingPain`, `R_GetWindGusting`, `R_GetChanceOfSaberFizz`, or `FrameEvent::WorldEffectCommand`. No `r_we` console registration, which is this ticket's step-004. No new WGSL file and no change to `world.wgsl` or `world_pbr.wgsl`. No cvar, no ABI change, no change to any file under `crates/mp/game/`, `crates/mp/cgame/`, `crates/mp/ui/`, `crates/mp/uishared/`, or `crates/sp/`. Every committed fixture except the one new PNG is read-only.

## Open rows

**Row 1 - the step boundary (user ruling).** **Proposed default: one step for the whole chain, commits 1 through 8.**

The ticket's own words for this group are "Rain and snow maps hit this in live play". Closing the five markers alone does not reach that. It leaves a cloud that computes correct physics and draws nothing, with no image gate to prove any of it, because the only honest gate for a billboard emitter is a picture.

The cost is a wide lane. It closes five markers, finds a home for one orphan carrier, ports a 170-line draw body, adds one `FrameEvent` variant, adds one GPU pass, edits two engine trap arms, and blesses one image.

The alternative is a split: commits 1 through 4 plus the finished file become step-001, and commits 5 through 7 become step-002. The bundle below is ordered so that split is a clean cut with no rework. Commits 1 through 4 leave every fixture byte-identical and add no producer for the new event, so they stand alone and green.

If the user takes the split, step-001's gate is the unit tests of commits 2 and 4 and nothing more, and the finished file records that no image proves the work.

**Row 2 - where the weather step runs (user ruling).** **Proposed default: trap-side, right after `RE_RenderScene`, with the batch crossing as a `FrameEvent`.**

`RB_RenderWorldEffects` needs `&mut EngineHostView` for the `CM_PointContents` scan inside `COutside::Cache` and for `com_printf`, plus `&mut WorldEffectsState` and its generator. `FrameExecutor::execute_frame` (`crates/mp/renderer-gpu/src/frame_exec.rs:417`) takes none of those. It gets `assets: &RenderAssets` and no host, no collision world, and no mutable renderer state.

The trap arm has all of it. `crates/mp/engine/client/src/cl_cgame.rs` already builds an `EngineHostView` and reaches `re.world_effects` directly for the two weather traps (`:3623-3643`), which is the DEC-59 ruling 1 idiom. The placement matches the oracle exactly, because `RE_RenderWorldEffects` is the statement after the scene in `RE_RenderScene` (`oracle/codemp/renderer/tr_scene.cpp:868`).

The values `Update` needs are all in scope there. The refdef the trap arm passes to `RE_RenderScene` carries `frametime`, `rdflags`, `view_origin`, and `view_axis`, and `assets.world` carries the bmodel bounds.

The alternative is to thread the host, the collision world, the generator, and mutable renderer state into `execute_frame`. That inverts DEC-37 ruling 2's split, which exists to keep exactly those things off the render thread. A third option runs `COutside::Cache` once at map load and only the per-frame step render-side, which is this default plus extra machinery for no gain.

The consequence to state plainly: the port computes the particle state on the sim side and draws it on the render side, where the oracle does both in the backend. Nothing observes the difference, because the batch is a pure function of the refdef and the cloud state, and both cross in one event.

**Row 3 - `WindZoneState`'s owner (mechanical).** **Proposed default: a `wind` field on `WorldEffectsState`.**

Raven's `mGlobalWindDirection`, `mGlobalWindSpeed`, and `mGlobalWindVelocity` are file statics in the same TU as `mOutside`, `mParticleClouds`, and `mWindZones` (`oracle/codemp/renderer/tr_WorldEffects.cpp:73-76`). Those three already live on `WorldEffectsState` under DEC-37 A13.3, and the wind trio belongs beside them. `RB_RenderWorldEffects` then drops its `wind` parameter and writes `self.wind`. The two `tr_surfacesprites` call sites keep their `&WindZoneState` parameter and borrow the field.

The alternative is a second field on `RendererFrontend` beside `world_effects`, which splits one TU's statics across two owners for no reason.

**Row 4 - the two generators (mechanical).** **Proposed default: one, `self.rng`, and the `rng` parameter is dropped.**

`WorldEffectsState::rng` is a single `native_math::rng::Rng`, and that type already carries both of Raven's streams separately: `holdrand` for `Q_irand` and `crt_holdrand` for the C runtime `rand` (`crates/native/math/src/rng.rs:103-131`). `R_WorldEffectCommand` and `CWeatherParticleCloud::Initialize` already draw from `self.rng`. `RB_RenderWorldEffects`'s separate `rng: &mut Rng` parameter has no caller and would be a third stream with no oracle twin.

The borrow works because the fields are disjoint. `self.mParticleClouds[i].Update(&mut self.rng, &self.mOutside, ...)` borrows three different fields of `self`.

**Row 5 - the view orientation's source (mechanical).** **Proposed default: `refdef.view_origin` and `refdef.view_axis`.**

The port holds the view orientation in three places, and only one is right here.

`FrameState::ori` (`crates/mp/renderer/src/render_state/frame_state.rs:23`) is `backEnd.ori`, the model-space orientation, not `viewParms.ori`. Nothing writes it. `render_state::placeholders::ViewParms` (`:367-386`) carries only `pvs_origin`, `frustum`, and `vis_bounds`, so `FrameState::view` has no orientation at all. The ABI `viewParms_t::ori` (`crates/mp/renderer/src/tr_local/view_parms_t.rs:16`) is the right one, and the render thread fills it from exactly two lines: `parms.ori.origin = refdef.view_origin; parms.ori.axis = refdef.view_axis;` (`crates/mp/renderer-gpu/src/frame_exec.rs:812-813`), which is the oracle's own `tr_scene.cpp:848-851`.

So reading the refdef gives the identical values with no ABI struct and no new state. The one case where the two diverge is a portal or mirror view, which overwrites `newParms.ori.origin` (`oracle/codemp/renderer/tr_main.cpp:1002`) while the refdef keeps the main view. Weather returns early on `RDF_SKYBOXPORTAL` anyway, and this step draws weather once per scene from the scene's own refdef.

The same row settles the two-refdef read. Raven takes `RDF_NOWORLDMODEL` from `tr.refdef` and `RDF_SKYBOXPORTAL` from `backEnd.refdef` (`oracle/codemp/renderer/tr_WorldEffects.cpp:1515-1517`). The port has one refdef at the call site, so both bits read from it. The two copies differ only when a scene's command is executed after a later scene overwrote the front-end copy, which the port's one-event-per-scene ordering cannot produce. The lane records this in two lines at the site.

**Row 6 - `SetViewportAndScissor` at this site (mechanical).** **Proposed default: retire the call, keep a one-line note with its `Source:` cite.**

`SetViewportAndScissor` is a deferred no-op with an empty body (`crates/mp/renderer/src/tr_backend.rs:210-213`). It is the only reason `RB_RenderWorldEffects` takes `frame: &FrameState`, and the parameter is otherwise unread. The real viewport work belongs to the render pass in `Pipeline3d`, which sets it from the refdef already. Keeping a no-op call alive only to justify a parameter is worse than a note.

The same applies to `qglLoadMatrixf(backEnd.viewParms.world.modelMatrix)` (`oracle/codemp/renderer/tr_WorldEffects.cpp:1525`). Its existing `DEFERRED` note at `crates/mp/renderer/src/tr_worldeffects/world_effects.rs:1435-1439` stays, because `draw_weather` supplies the world clip matrix on the GPU side.

**Row 7 - the live gate (user ruling).** **Proposed default: one new world golden on `maps/mp/ctf2.bsp`, driven by a scripted `snow` command with a fixed seed, rendered on the second weather frame, plus unit tests on the pure math.**

`ctf2` is the only stock MP map that ships weather. Its entity lump carries `fx_snow` with `count 1000` and three `misc_weather_zone` brushes. No other stock MP map has either key, and the SP maps that do (`hoth2`, `t1_rail`, `t2_wedge`, `vjun1`, `yavin2`) are out of this client's scope.

The rig cannot run the game and cgame chain that normally issues the command, so the fixture calls the parser directly, the same way `golden_world_marks_duel1` calls `RE_RegisterShader` directly. The recipe:

1. `boot::load_world(&mut host, "maps/mp/ctf2.bsp")`.
2. `host.re.world_effects.rng.srand(0)`, the fixed seed. `rng` and `srand` are both already public, so this needs no production surface.
3. `host.re.world_effects.R_WorldEffectCommand(..., Some(b"snow"))`, which is the exact string `SP_CreateSnow` registers as `*snow` (`oracle/codemp/game/g_misc.c:2521`).
4. Two weather frames. The first builds the outside cache and returns without updating anything, which is the oracle's own `if (!mOutside.Initialized())` branch (`oracle/codemp/renderer/tr_WorldEffects.cpp:1544-1547`). The second spawns and integrates.
5. Render one frame at the frozen clock and compare.

The determinism holds. `SceneState::last_time` starts at 0, so `frametime` is 12345 clamped to 500 and `mSecondsElapsed` is 0.5. Snow sets `mFade = 10`, so `particleFade` is 5.0 and every in-view particle reaches its `mColor[3]` ceiling of 0.75 on the first update. The collision world is empty in the rig, so every point tests outside and the whole cloud is eligible.

This exercises DEC-66 ruling 4's own clause. That ruling said a test-only reseed graduates as its own ruling if a golden proves fragile, and a wall-clock seed is exactly that case. The reseed is test-only and touches no production code.

Named defect conditions. The correct image is the `ctf2` spawn view with snowflakes over it. An image with no snowflake means the cloud computed and did not draw, which is the whole bug. A world that renders differently from a no-weather `ctf2` render is a defect, because weather does not depth-write and cannot change what is behind it. A panic in `mParticles` or `mPointCache` means an index escaped. Zero drawn vertices with a non-zero `mParticleCountRender` means the batch crossed empty.

The unit tests are cheap and need no assets or GPU: the two-draw count on the spawn-plane respawn per row 9, `SVecRange::Wrap` across each axis, the fade machine's four transitions, and the render count's re-read of the flag.

The alternative is unit tests alone. They prove the physics and prove nothing about a pixel, which is the whole ticket.

**Row 8 - the nearest sampler (mechanical).** **Proposed default: a third sampler on `GpuImages` plus a `weather_bind_group` builder.**

Raven sets `GL_TEXTURE_MIN_FILTER` and `GL_TEXTURE_MAG_FILTER` per cloud, both to `GL_LINEAR` or both to `GL_NEAREST` (`oracle/codemp/renderer/tr_WorldEffects.cpp:1364-1365`). Every rain preset sets `mFilterMode = 1`, so the nearest case is live. `GpuImages` builds a repeat and a clamp sampler and nothing else (`crates/mp/renderer-gpu/src/gpu_images.rs:140-141`), and the weather image always loads with `GL_CLAMP` (`crates/mp/renderer/src/tr_worldeffects/world_effects.rs:1005-1016`), so one nearest-clamp sampler covers it. Neither weather filter uses mips, because Raven sets the plain `GL_LINEAR`, not `GL_LINEAR_MIPMAP_LINEAR`.

The alternative is to ignore `mFilterMode` and always sample linear. That makes rain visibly softer than retail on the one preset family that matters most.

**Row 9 - the broadcast rand draws (mechanical, this is the transcription trap).** **Proposed default: one `WE_flrand` draw per line at `:1216-1217`, broadcast to all three components.**

`CVec3` has no scalar `operator*`. The float converts through the broadcast constructor `CVec3(const float val)` (`oracle/codemp/Ravl/CVec.h:570`) and the multiply is componentwise `CVec3 * CVec3` (`oracle/codemp/Ravl/CVec.h:628`). So each of the two lines draws once and scales x, y, and z with the same value. A per-component write draws six times instead of two and shifts the stream for every later draw in the session.

The neighbouring `(mSpawnPlaneNorm * mSpawnPlaneDistance)` at `:1215` is the same broadcast and draws nothing.

By contrast `SVecRange::Pick` (`oracle/codemp/renderer/tr_WorldEffects.cpp:156-161`) genuinely is three draws in x, y, z order, and the port already has it right (`crates/mp/renderer/src/tr_worldeffects/world_effects.rs:182-186`). The lane must not unify the two.

**Row 10 - the colour quantization (mechanical, a named divergence).** **Proposed default: `WeatherVertex` carries the `f32` colour and the executor rounds to `WorldVertex`'s `[u8; 4]` at one site.**

`qglColor4f` takes floats and the fixed-function pipeline converts them to fixed point before interpolation. `WorldVertex` already carries `[u8; 4]` (`crates/mp/renderer-gpu/src/pipeline3d.rs:133-147`) and every other billboard in the port goes through it, so a weather-only float colour path would be a second vertex row for one subsystem. Keeping the float value in the seam payload and rounding once at the vertex build keeps the divergence at one named site with a two-line note.

The alternative is a dedicated weather vertex row with a float colour, which costs a second pipeline layout and a second WGSL entry point for a difference no image gate can see.

**Row 11 - the uninitialized `mRotationChangeTimer.mMax` read (mechanical, rule 19).** **Proposed default: no code change, one note at the site.**

`SIntRange::Clear` writes `mMin` twice and never `mMax`, and `Reset` writes `mRotationChangeTimer.mMin` twice and never `mMax` (`oracle/codemp/renderer/tr_WorldEffects.cpp:229-233,999-1000`). `mMax` is then read at `:1071` through `Q_irand(2000, mMax)`. `mParticleClouds` is a global, so its storage is zero-initialized and the effective call is `Q_irand(2000, 0)`, which the clamp at `:1072-1075` floors at 1.

The port already reproduces this exactly. `CWeatherParticleCloud::new` sets `mRotationChangeTimer: SIntRange { mMin: 0, mMax: 0 }` (`crates/mp/renderer/src/tr_worldeffects/world_effects.rs:955`), and `Reset` carries the double-`mMin` write with its own note (`:1109-1112`). There is nothing to build, only a line at the `Pick` call recording why the zero is the defined value.

## Pause triggers, named for this step

- Any committed fixture other than the new PNG moves. STOP. No commit before commit 6 has a producer for the weather event, so a session with no weather command draws exactly what it drew before.
- `GpuImages` turns out not to expose what `weather_bind_group` needs. STOP and report, per row 8. Do not copy the texture or build a second image store.
- The weather draw looks like it should be a `DrawSurf` with a synthesized shader. STOP. A cloud binds a raw `image_t` with no `shader_t` in the oracle, and inventing a shader is speculative behavior.
- A second vertex row for weather looks necessary. STOP, per row 10. `WorldVertex` covers it.
- The per-frame force or friction looks frame-rate dependent and wrong. STOP. It is frame-rate dependent in the oracle too (`oracle/codemp/renderer/tr_WorldEffects.cpp:1193-1194`), and only the position step is time-scaled.
- The respawn multiply looks like it needs one draw per component. STOP, per row 9. It is one draw broadcast to three components.
- The first weather frame draws nothing and that looks like a bug. STOP. The cache-building frame renders nothing in the oracle too (`oracle/codemp/renderer/tr_WorldEffects.cpp:1544-1547`), and row 7's fixture runs two frames for that reason.
- `mRotationChangeTimer.mMax` looks uninitialized and fixable. STOP, per row 11.
- `R_InitWorldEffects`'s wall-clock seed looks like it should change in production. STOP, per row 7. The reseed is test-only.
- The `r_we` console command looks like it belongs here. STOP. It is this ticket's step-004.
- The five caller-less weather functions look like dead code to delete. STOP. Deletion is not this step's work.
- `RE_RenderScene`'s signature looks like it should take the host and the world-effects state. STOP, per row 2. The call sits in the trap arm.
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

Twenty-one committed fixtures, one per test. The lockstep referee is not required, because no commit touches `mp_game`, the server, or any `jampded` link-set crate.

1. **The four stubbed reads land.** All four type-stub markers close. `Update` reads the view origin and axis from its new parameters, `RB_RenderWorldEffects` reads `refdef.rdflags`, `refdef.frametime`, and `assets.world.bmodels[0].bounds`, and the stale doc comments that call these fields absent are corrected. The per-particle `todo!()` stays. Files: `crates/mp/renderer/src/tr_worldeffects/world_effects.rs`. Subject: `fix(gh#54 s001): the four world-effects reads land`. Gates: the full battery, all twenty-one fixtures byte-identical. Nothing calls this code, so the battery proves the build and nothing more.

2. **The particle loop.** The fifth marker closes. `Update` gains `outside: &COutside` and the loop is transcribed: the first-time spawn, the integration, the classification, the respawn, the fade machine, and the render count. Unit tests cover the row-9 draw count, `SVecRange::Wrap`, the fade transitions, and the render count. Files: `crates/mp/renderer/src/tr_worldeffects/world_effects.rs`. Subject: `feat(gh#54 s001): the weather particle loop`. Gates: the full battery, all twenty-one fixtures byte-identical.

3. **The wind trio finds its owner.** `WorldEffectsState::wind`, per row 3. `RB_RenderWorldEffects` and `RB_WorldEffects` drop their `wind` and `rng` parameters per rows 3 and 4, and take `refdef: &TrRefdef` in place of `frame: &FrameState` per row 6. The `tr_surfacesprites` call sites borrow the field. Files: `crates/mp/renderer/src/tr_worldeffects/world_effects.rs`, `crates/mp/renderer/src/tr_backend.rs`, `crates/mp/renderer/src/tr_surfacesprites.rs`. Subject: `feat(gh#54 s001): the wind zone state finds its owner`. Gates: the full battery, all twenty-one fixtures byte-identical.

4. **The cloud renders into a batch.** `WeatherVertex`, `WeatherCloudBatch`, and `WeatherFrame`. `Render` emits the triangle and quad arms into a batch instead of only counting, and `RB_RenderWorldEffects` returns the frame. Unit tests cover both vertex arms' offsets and UVs and both blend modes' colour. Still no caller. Files: `crates/mp/renderer/src/render_state/weather_frame.rs`, `crates/mp/renderer/src/render_state/mod.rs`, `crates/mp/renderer/src/tr_worldeffects/world_effects.rs`, `crates/mp/renderer/src/tr_backend.rs`. Subject: `feat(gh#54 s001): the cloud renders into a frame batch`. Gates: the full battery, all twenty-one fixtures byte-identical.

5. **The GPU pass.** `FrameEvent::WorldEffects`, the executor arm, `Pipeline3d::draw_weather`, `GpuImages::sampler_nearest` and `weather_bind_group`, and the `WorldStats::weather_vertices` counter. No producer yet, so the pass never runs. Files: `crates/mp/renderer/src/render_state/frame_event.rs`, `crates/mp/renderer-gpu/src/frame_exec.rs`, `crates/mp/renderer-gpu/src/pipeline3d.rs`, `crates/mp/renderer-gpu/src/gpu_images.rs`. Subject: `feat(gh#54 s001): the weather pass draws a frame batch`. Gates: the full battery, all twenty-one fixtures byte-identical, which is the proof the new pass is inert with no producer.

6. **The frame drives world effects.** `RE_RenderWorldEffects` in `tr_cmds.rs` and the two trap-arm calls. The `DEFERRED` notes in `tr_cmds.rs` and `tr_scene.rs` retire. Weather now runs and draws in live play. Files: `crates/mp/renderer/src/tr_cmds.rs`, `crates/mp/renderer/src/tr_scene.rs`, `crates/mp/engine/client/src/cl_cgame.rs`, `crates/mp/engine/client/src/cl_ui.rs`. Subject: `feat(gh#54 s001): the frame drives world effects`. Gates: the full battery, all twenty-one fixtures byte-identical. No existing fixture issues a weather command, so the drive is inert in every one of them.

7. **The weather golden.** `golden_world_weather_ctf2` and its PNG, after the row-7 STOP. Files: `crates/mp/renderer-gpu/tests/world_golden.rs` and the new PNG. Subject: `test(gh#54 s001): the weather world golden`. Gates: the full battery, with the new golden green at tolerance zero and the other twenty-one fixtures byte-identical.

8. **The finished file**, per the packet skill: assumptions and choices keyed to their commits, deviations or the word "none", the commit list with gate results, and open gaps. File: `.claude/packets/54/step-001/finished.md`. Subject: `process(gh#54 s001): finished file`.

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

None.
