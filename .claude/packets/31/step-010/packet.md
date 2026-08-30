# Packet gh#31 step-010 - the renderfx closure

## Scope

This step closes the last dark row of the DEC-54 renderfx census, `RF_DISTORTION`, and puts the already-live disintegrate and volumetric arms under image goldens. It also rules and records the `RF_NOSHADOW` disposition, which needs no port.

The census counts twelve renderfx flags across the four traces (`docs/plans/2026-07-24-client-port/scene-trap-census.md:161-172`). After step-009 every one of the twelve draws except `RF_DISTORTION`.

- Live in the GPU backend: `RF_DEPTHHACK` and `RF_NODEPTH` through `DepthRange::resolve` (`crates/mp/renderer-gpu/src/pipeline3d.rs:868,870`), `RF_RGB_TINT` (`:4141`), `RF_FORCE_ENT_ALPHA` (`:4203`), `RF_DISINTEGRATE1` and `RF_DISINTEGRATE2` (`:4121-4128`), `RF_VOLUMETRIC` (`:4130`).
- Live in the frontend, which `frame_exec.rs:851` calls through `R_RenderView`: `RF_MINLIGHT` (`crates/mp/renderer/src/tr_light.rs:494`), `RF_LIGHTING_ORIGIN`, `RF_FIRST_PERSON` at its live cull gate (`crates/mp/renderer/src/tr_main.rs:1887`, with the light-logging gate at `tr_light.rs:271`), `RF_THIRD_PERSON` (`crates/mp/renderer/src/tr_mesh.rs:369`).
- Resolved but inert: `RF_DISTORTION`. `EntityFx::resolve` reads the bit (`pipeline3d.rs:4403`), the stage logs once through `Warned::Distortion` (`:2766`, `:2960`), and the draw is unchanged. The open marker is `//TODO: Port RB_IterateStagesGeneric's RF_DISTORTION arm` (`:4378`).
- No gate to answer: `RF_NOSHADOW`. Both oracle reads sit behind `r_shadows->integer == 2`, the stencil-shadow surface add (`oracle/codemp/renderer/tr_mesh.cpp:391-395`, `oracle/codemp/renderer/tr_ghoul2.cpp:2586-2591`). The retail default is `cg_shadows` 1 (`oracle/codemp/renderer/tr_init.cpp:1139`), and this workspace has no shadow backend at all. Every body in `crates/mp/renderer/src/tr_shadows.rs` is a deferred no-op.

So this step delivers three things. It ports the `RF_DISTORTION` chain end to end, it gives the disintegrate pair and the volumetric arm their first goldens, and it converts the four `RF_NOSHADOW` deferral notes to the greppable marker convention with the ghoul2 MP-versus-SP asymmetry recorded.

The step does not touch the FX module, the sim side, `cgame`, `ui`, the 2D path, the sky path, the mark path, or the dlight passes. It adds no cvar, no `FrameEvent` variant, and no ABI surface. It ports no shadow backend and no dynamic-glow pass.

**Carried from step-009, and relevant here.** The per-surface vertex cap against the oracle's `tess` batching is a standing backend architecture fact and this step does not change it. The `oldorigin` portal-view divergence note (step-009 finished file, open gaps) stands untouched. The two stale `tr_surface.rs` doc comments step-009 flagged as a later one-line fix (`crates/mp/renderer/src/tr_surface.rs:108-116` and `:1217-1243`) are still out of contract and stay untouched here.

## The oracle, cited

### The renderfx block

`oracle/codemp/cgame/tr_types.h:17-54` holds twenty-two flag defines. `RF_DISTORTION` is `0x02000` (`:41`, Raven: "area distortion effect -rww"), `RF_FORCEPOST` is `0x200000` (`:54`), `RF_FORCE_ENT_ALPHA` is `0x00400` (`:36`), `RF_NOSHADOW` is `0x00040` (`:26`), `RF_VOLUMETRIC` is `0x00020` (`:24`), and the disintegrate pair is `0x20000` and `0x40000` (`:47-48`).

SP uses a different bit layout entirely: SP `RF_DISTORTION` is `0x400000` (`oracle/code/renderer/tr_types.h:51`) and SP defines `RF_ALPHA_FADE` (`:34`), which MP has no definition for anywhere. Port the MP constants and never an SP one.

### The post-render list (`oracle/codemp/renderer/tr_backend.cpp`)

`RB_RenderDrawSurfList` defers three flags out of the sorted draw and replays them after the whole list.

**The enqueue** (`:778-837`). For every draw surf whose entity is not `TR_WORLDENT`, and while `g_numPostRenders < MAX_POST_RENDERS`, the test is a three-way or: `RF_DISTORTION`, `RF_FORCEPOST`, or `RF_FORCE_ENT_ALPHA` (`:781-783`). A match stores `depthRange`, `entNum`, `drawSurf`, `dlighted`, `fogNum`, and `shader` into `g_postRenders[g_numPostRenders++]` (`:786-823`), restores the four old batch values (`:827-830`), sets `oldSort = -20` so the next surface of the same sort still enters (`:832`), and `continue`s without opening a draw surf (`:835`). Past the 128 cap the surface falls through to the normal path.

`MAX_POST_RENDERS` is 128 (`:655`), `postRender_t` is at `:657-666`, and the two file statics are at `:668-669`. Raven's own comment says the post-render path "lacks much of the optimization that the standard sort-render crap does, so it's slower" (`:653-654`), which is the statement that each entry draws as its own batch with no merging.

**The drain** (`:1003-1214`). The loop is LIFO: `while (g_numPostRenders > 0) { g_numPostRenders--; pRender = &g_postRenders[g_numPostRenders]; ... }` (`:1008-1011`). So the deferred surfaces draw in reverse submission order. Each iteration opens its own `RB_BeginSurface` (`:1013`), sets `backEnd.currentEntity` and the entity transform (`:1032-1049`), applies the stored depth range (`:1055-1071`), captures the screen when the entity is a distortion entity (`:1144-1209`), draws the one surface (`:1211`), and closes with `RB_EndSurface` (`:1212`).

The `eValid` false arm at `:1141-1143` is an empty block, and the world-surface branch it belonged to is commented out (`:1015-1030`, `:838-875`), so `eValid` is always true in the retail build.

**The dead branches.** `tr.distortionShader` is created once as the marker shader `internal_distortion` (`oracle/codemp/renderer/tr_shader.cpp:4166`) and is never assigned to a real surface. The two `shader == tr.distortionShader` tests in `RB_RenderDrawSurfList` (`:816`, `:839`) are inside `/* */` comment blocks. The 2048-square full-screen capture (`:1073-1140`) is inside a comment block too. Port none of them.

### The per-entity screen capture (`oracle/codemp/renderer/tr_backend.cpp:1144-1209`)

The gate is `(entities[pRender->entNum].e.renderfx & RF_DISTORTION) && lastPostEnt != pRender->entNum` (`:1144-1145`). That is a compare against the last captured entity, not a set of every entity captured this frame, so a deferred order of A, B, A captures A twice. `lastPostEnt` starts at `-1` (`:1006`) and takes the entity number after a successful capture (`:1207`). In sort order one entity's surfaces usually sit contiguous, so the run compare and a per-frame set usually agree, and the run compare is the one to transcribe.

The rect comes from the entity itself: `r = R_WorldCoordToScreenCoord(backEnd.currentEntity->e.origin, &x, &y)` and `rad = backEnd.currentEntity->e.radius` (`:1172-1173`). On a false return the capture is skipped and `lastPostEnt` stays put, so a later surface of the same entity retries.

`R_WorldCoordToScreenCoordFloat` (`:597-635`) projects a world point onto the screen. It reads `glConfig.vidWidth/2` and `vidHeight/2` as the center (`:608-609`), copies the three `tr.refdef.viewaxis` rows as forward, right, and up (`:612-614`), subtracts `tr.refdef.vieworg` (`:616`), builds `transformed` as the three dot products in the order right, up, forward (`:618-620`), returns false when `transformed[2] < 0.01` (`:623-626`), and otherwise computes `xzi = xcenter / transformed[2] * (90.0 / tr.refdef.fov_x)` and `yzi = ycenter / transformed[2] * (90.0 / tr.refdef.fov_y)` (`:628-629`) before `*x = xcenter + xzi * transformed[0]` and `*y = ycenter - yzi * transformed[1]` (`:631-632`). The `y` it returns therefore counts down from the top of the screen. `R_WorldCoordToScreenCoord` (`:637-645`) is the integer wrapper and truncates both through `(int)`.

The clamp (`:1178-1198`) is `cX = glConfig.vidWidth - x - (rad/2)` and `cY = glConfig.vidHeight - y - (rad/2)`, each then pushed back to `vidWidth - rad` or `vidHeight - rad` when the box would run off the far edge, and to `0` when it goes negative. The `vidHeight - y` term converts Raven's top-down `y` to the GL framebuffer's bottom-up origin. The `vidWidth - x` term has no such justification and mirrors the box horizontally. It is Raven's own behavior and it ports as written.

The copy is `qglCopyTexImage2D(GL_TEXTURE_2D, 0, GL_RGBA16, cX, cY, rad, rad, 0)` (`:1204`). `qglCopyTexImage2D` re-specifies the texture, so `tr.screenImage` becomes a `rad` by `rad` texture on every capture. Its declaration is `image_t *screenImage;` (`oracle/codemp/renderer/tr_local.h:1337`, Raven: "reserve us a gl texnum to use with RF_DISTORTION") and `R_CreateBuiltinImages` allocates it as an 8 by 8 white placeholder (`oracle/codemp/renderer/tr_image.cpp:2776`).

### The distortion stage arm (`oracle/codemp/renderer/tr_shade.cpp:2163-2169`)

```c
if ( (tess.shader == tr.distortionShader) ||
     (backEnd.currentEntity && (backEnd.currentEntity->e.renderfx & RF_DISTORTION)) )
{ //special distortion effect -rww
    //tr.screenImage should have been set for this specific entity before we got in here.
    GL_Bind( tr.screenImage );
    GL_Cull(CT_TWO_SIDED);
}
```

The whole arm is a texture bind plus a cull change. It computes nothing per vertex. `ComputeColors` and `ComputeTexCoords` already ran at `:2081` and `:2083`, so the stage keeps its own colours and texture coordinates and only swaps which image they sample.

The arm is the head of an `else if` chain, so a distortion stage never reaches `R_BindAnimatedImage` (`:2175`). It sits inside the single-texture branch: a multitextured stage diverts to `DrawMultitextured` at `:2147-2150`, which has no distortion handling.

The bind chain and the state chain are separate. The bind chain is `:2163-2175`, and the state chain is a second `if` chain at `:2177-2206`. Its first arm is the stencil cutout, keyed on `tess.shader == tr.distortionShader` only, so `RF_DISTORTION` never reaches it. `tr_stencilled` is written there and nowhere else, which makes `RB_DistortionFill` and the full-screen `RB_CaptureScreenImage` unreachable through the entity path. Neither is in scope.

A plain `RF_DISTORTION` stage matches neither the stencil arm nor the `RF_FORCE_ENT_ALPHA` arm at `:2189-2202`, so it falls to the default `else { GL_State( stateBits ); }` at `:2203-2206` and draws under its own stage state. Drawing a distortion stage under its own shader state is therefore plain parity and needs no note. A distortion entity that also carries `RF_FORCE_ENT_ALPHA` diverts into the force-alpha arm, which this backend already ports (`crates/mp/renderer-gpu/src/pipeline3d.rs:4418`).

### The disintegrate pair

`ComputeColors` short-circuits both flags together (`oracle/codemp/renderer/tr_shade.cpp:1536-1543`) and sets `killGen`, so the normal rgbGen switch is skipped. `RB_CalcDisintegrateColors` (`oracle/codemp/renderer/tr_shade_calc.cpp:1545-1637`) computes `threshold = (backEnd.refdef.time - ent->endTime) * 0.045f` and, per vertex, `dis = VectorLengthSquared(e.oldorigin - xyz)`. `RF_DISINTEGRATE1` maps `dis` to five bands, `RF_DISINTEGRATE2` to two. `RB_CalcDisintegrateVertDeform` (`:1640-1671`) fires for `RF_DISINTEGRATE2` alone and pushes each vertex along its normal.

Both are already ported and live: `crates/mp/renderer/src/tr_shade_calc.rs:588-640` and `:648-674`, called from `crates/mp/renderer-gpu/src/pipeline3d.rs:4122` and `:4127`. `RF_DISINTEGRATE1` also forces the stage state bits (`tr_shade.cpp:2043-2047`), ported at `pipeline3d.rs:4413-4417`. This step adds no code for them, only a gate.

The vert deform reads the surface normals, so it fires only on a surface that decodes them. In this backend that is an MD3 or a Ghoul2 surface (`pipeline3d.rs:4126`, and the two normal producers at `:2295` and `:2472`). A CPU-built sprite has no normals and keeps its vertices.

### The volumetric arm

`ComputeColors` short-circuits it at `oracle/codemp/renderer/tr_shade.cpp:1554-1581`, with Raven's own note that "this should also be a CGEN type, but that would entail adding new shader commands....which is too much work for one thing". Per vertex it takes `dot = DotProduct(normal, backEnd.refdef.viewaxis[0])`, raises it to the fourth power, clamps anything under `0.2` to zero, and writes `myftol(e.shaderRGBA[0] * (1 - dot))` into all four channels.

Ported and live at `crates/mp/renderer-gpu/src/pipeline3d.rs:4249-4273`. The single cgame submitter is the DEMP2 alt-fire detonation shell (`oracle/codemp/cgame/fx_demp2.c:248`), which is a model, so the census's 50 submissions all carry normals. A surface with no normals takes `dot = 0` and the entity's red channel flat, which proves only that the short-circuit fired.

### `RF_NOSHADOW`

Two renderer reads, both under `r_shadows->integer == 2`: `oracle/codemp/renderer/tr_mesh.cpp:391-396` and `oracle/codemp/renderer/tr_ghoul2.cpp:2586-2611`. One renderer write, the out-of-range Ghoul2 suppression at `oracle/codemp/renderer/tr_ghoul2.cpp:3521-3527`. The cgame side only ever sets the bit, and `CG_PlayerShadow` (`oracle/codemp/cgame/cg_players.c:4612-4707`) never reads it.

The MP projection-shadow add (`r_shadows->integer == 3`) does **not** test `RF_NOSHADOW`. It tests `RF_SHADOW_PLANE` alone (`tr_mesh.cpp:400-405`, `tr_ghoul2.cpp:2614-2622`). SP does test it there (`oracle/code/renderer/tr_ghoul2.cpp:2801`). That asymmetry is load bearing and a port must keep MP's shape, per porting-rules §20.

`r_shadows` is the same cvar as `cg_shadows`: the renderer registers its handle against the literal name (`oracle/codemp/renderer/tr_init.cpp:1139`, default `"1"`). At the retail default neither read runs.

The two Rust sites carry `DEFERRED:` notes rather than the marker convention: `crates/mp/renderer/src/tr_mesh.rs:512-514` and `crates/mp/renderer/src/tr_ghoul2.rs:2459-2462`.

### Random draws and the macro multi-eval trap

None of the bodies this step transcribes draws a random number. `R_WorldCoordToScreenCoordFloat`, the capture clamp, the distortion stage arm, `RB_CalcDisintegrateColors`, `RB_CalcDisintegrateVertDeform`, and the volumetric block contain no `crandom`, `random`, `Q_crandom`, or `Q_random` call. The `VectorMA` and `VectorScale` macros expand their scale argument once per component (`oracle/codemp/game/q_shared.h:1361,1365`), which is why a random-drawing scale argument may never be hoisted, but no site in this step's scope has one. The lane must still not hoist a draw anywhere, and the `Pipeline3d::rng` stream step-009 landed is untouched here.

## The backend gaps this step must bridge

The oracle captures the screen with an immediate-mode GL call in the middle of a draw. This backend is wgpu, and three facts change the shape.

**A render pass owns its attachment.** `Pipeline3d::draw` opens one render pass over the whole item list (`crates/mp/renderer-gpu/src/pipeline3d.rs:1338-1345`), and wgpu forbids a texture copy out of a live attachment. So every capture needs a pass boundary. The oracle interleaves capture and draw per deferred surface (`oracle/codemp/renderer/tr_backend.cpp:1144-1212`), which means capture N reads a frame that already holds deferred draws 1 through N-1. Reproducing that needs one pass per deferred surface: the main pass draws the sorted list, then each deferred surface encodes its capture, if it is due, and opens its own pass with `wgpu::LoadOp::Load` on both attachments. The post-render list is capped at 128 surfaces, so the pass count is bounded.

An all-captures-then-one-pass shape would break two frames out of every one that draws two distortion entities: every capture would read a frame with no deferred surface in it, and the single screen-image slot would leave both stages sampling the last capture. Two cloaked players on screen is ordinary live surface (`oracle/codemp/cgame/cg_players.c:4935,6583,7968`), so the interleave is not optional.

**The target texture is not reachable.** `execute_frame` and `Pipeline3d::draw` take `target: &TextureView` only. A copy needs the `wgpu::Texture`. Both surface arms already own one: the windowed caller holds `frame.texture` from `Gpu::begin_frame` (`crates/mp/client-app/src/render_thread.rs:129`), and the headless arm owns its offscreen texture inside `Gpu`. The windowed surface is configured `COPY_SRC` for `screenshot_tga` (`crates/mp/renderer-gpu/src/gpu.rs:116-118`) and the headless texture is `RENDER_ATTACHMENT | COPY_SRC` (`:380`), so both are already copy sources and no usage flag changes.

**A wgpu texture cannot be re-specified.** `qglCopyTexImage2D` replaces `tr.screenImage`'s storage with a `rad` by `rad` image on every capture, and the stage then samples that image over the full `0..1` texture-coordinate range. A wgpu texture has a fixed size, so the port keeps one screen-image texture and rebuilds it whenever the requested `rad` differs from the one it holds. One slot is enough because the interleave consumes each capture in the pass that immediately follows it, before the next capture overwrites it. A rebuild replaces the texture mid-encode, so the replaced one has to stay alive until the submit, which is what the keep-alive vector in the contract holds.

**The two worlds store a copied rect in opposite row order.** `glCopyTexImage2D` stores the framebuffer rect bottom-up, so `t = 0` samples the rect's bottom row, and `copy_texture_to_texture` stores it top-down, so `v = 0` samples the top row. Identical texture coordinates therefore sample the square vertically flipped against the oracle. Only this framebuffer copy diverges. A file-loaded image agrees between the two worlds, because the loader already resolves the row order. So the port flips `v` for a screen-image stage's texture coordinates, which restores parity rather than declaring a divergence. The flip lands on the CPU: a distortion item is always an entity surface, because the post-render enqueue requires `entityNum != TR_WORLDENT` (`oracle/codemp/renderer/tr_backend.cpp:778`), and every entity stage builds its vertices through `build_dynamic_block` (`crates/mp/renderer-gpu/src/pipeline3d.rs:2995`). No shader changes.

**Culling is already off.** The pipeline sets `cull_mode: None` (`pipeline3d.rs:3191`), with the comment that the frontend has already culled and per-shader sidedness lands later. So `GL_Cull(CT_TWO_SIDED)` is already this backend's state and the arm's cull half needs no code. Record the fact at the site rather than writing a no-op.

## Surface contract

### `crates/mp/renderer/src/tr_backend.rs`

One new const, in the twin that owns Raven's `tr_backend.cpp` statics, the same placement rule step-009 used for `LIGHTNING_RECURSION_LEVEL`:

```rust
/// Raven `MAX_POST_RENDERS` - the post-render queue depth `RB_RenderDrawSurfList` fills.
/// A surface past the cap falls through to the normal sorted draw instead of deferring.
///
/// Source: `oracle/codemp/renderer/tr_backend.cpp:655`
pub const MAX_POST_RENDERS: usize = 128;
```

The deferred `RB_RenderDrawSurfList` note in this file gains one line naming `Pipeline3d::draw` as the live post-render home. No body in this file is ported, and no `todo!()` in it is removed.

### `crates/mp/renderer-gpu/src/gpu.rs`

One new accessor, the headless twin of the windowed caller's `frame.texture`:

```rust
/// The offscreen texture the headless target draws into, the copy source a mid-frame capture reads.
/// Panics on the windowed path, which has no offscreen texture.
pub fn headless_texture(&self) -> &wgpu::Texture
```

Nothing else in this file changes. No usage flag, no format, no resize behavior.

### `crates/mp/renderer-gpu/src/gpu_images.rs`

One new method beside `world_bind_group` (`:348`), which cannot serve here because it resolves its diffuse through an `ImageHandle` and the screen image has none:

```rust
/// Builds the world texture bind group with an explicit diffuse view, the shape the distortion stage needs.
/// The screen image is render-thread scratch with no `ImageHandle`, so it binds by view and takes the clamping sampler.
///
/// Source: `oracle/codemp/renderer/tr_shade.cpp:2167` (`GL_Bind( tr.screenImage )`)
pub fn view_bind_group(
    &self,
    gpu: &Gpu,
    layout: &BindGroupLayout,
    diffuse: &TextureView,
    lightmap: Option<ImageHandle>,
) -> BindGroup
```

### `crates/mp/renderer-gpu/src/frame_exec.rs`

`execute_frame` (`:403`) and `execute_package` (`:347`) each gain one parameter beside the existing `target`, and `render_world` (`:715`) forwards it:

```rust
        target_texture: &wgpu::Texture,
```

No other change. The event loop, the phase order, and the 2D pass are untouched.

### `crates/mp/renderer-gpu/src/pipeline3d.rs`

`Pipeline3d` (`:911-965`) gains one private field, built as `None` in `Pipeline3d::new`:

```rust
    /// Raven's `tr.screenImage`, the square of the frame a distortion stage samples instead of its own texture.
    /// `qglCopyTexImage2D` re-specifies the oracle's texture per capture, so this rebuilds whenever the requested side length changes.
    /// One slot serves every distortion entity, because each capture is drawn in the pass that immediately follows it.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:1337`
    screen_image: Option<ScreenImage>,
```

One new private type beside it:

```rust
/// One capture of the frame into a square texture, held across frames and rebuilt on a side-length change.
///
/// Source: `oracle/codemp/renderer/tr_backend.cpp:1200-1205`
struct ScreenImage {
    side: u32,
    texture: wgpu::Texture,
    view: wgpu::TextureView,
}
```

`StageDrawItem` (`:810-843`) gains two fields:

```rust
    /// The entity number this item defers behind, Raven's `postRender_t::entNum`.
    /// `Some` moves the item into the post-render pass, and the capture reads the entity back by this index.
    ///
    /// Source: `oracle/codemp/renderer/tr_backend.cpp:781-783,1011`
    post_render_ent: Option<i32>,
    /// Whether this stage binds the captured screen image instead of its own diffuse.
    ///
    /// Source: `oracle/codemp/renderer/tr_shade.cpp:2163-2169`
    screen_image: bool,
```

`Pipeline3d::draw` (`:1167`) gains one parameter:

```rust
        target_texture: &wgpu::Texture,
```

Four new private free functions:

```rust
/// Raven `R_WorldCoordToScreenCoordFloat` - projects a world point onto the screen in top-down pixel coordinates.
/// It returns `None` where the point sits at or behind the eye plane, Raven's `transformed[2] < 0.01` early return.
///
/// Source: `oracle/codemp/renderer/tr_backend.cpp:597-635`
fn world_coord_to_screen_coord_float(
    world_coord: vec3_t,
    refdef: &TrRefdef,
    vid_width: i32,
    vid_height: i32,
) -> Option<(f32, f32)>

/// Raven `R_WorldCoordToScreenCoord` - the integer wrapper, truncating both components.
///
/// Source: `oracle/codemp/renderer/tr_backend.cpp:637-645`
fn world_coord_to_screen_coord(
    world_coord: vec3_t,
    refdef: &TrRefdef,
    vid_width: i32,
    vid_height: i32,
) -> Option<(i32, i32)>

/// The source rectangle one distortion capture copies, as `(x, y, side)` in the target texture's top-down coordinates.
/// Raven's `cY` counts from the framebuffer's bottom edge, and the flip back to a top-down origin happens here.
///
/// Source: `oracle/codemp/renderer/tr_backend.cpp:1172-1198`
fn screen_capture_rect(
    e: &refEntity_t,
    refdef: &TrRefdef,
    vid_width: i32,
    vid_height: i32,
) -> Option<(u32, u32, u32)>

/// Encodes one distortion entity's capture into `screen_image`, rebuilding the texture when the side length changed.
/// The oracle re-specifies its texture on every copy, so a rebuild here is the faithful shape rather than a cache miss.
/// A rebuild moves the replaced texture into `keep_alive`, which the caller holds until the submit so an already-encoded pass keeps its binding.
///
/// Source: `oracle/codemp/renderer/tr_backend.cpp:1144-1209`
fn capture_screen_image(
    gpu: &Gpu,
    encoder: &mut wgpu::CommandEncoder,
    target_texture: &wgpu::Texture,
    screen_image: &mut Option<ScreenImage>,
    keep_alive: &mut Vec<ScreenImage>,
    rect: (u32, u32, u32),
)
```

Behavior changes inside existing bodies, each written in the file's established shape and cited at the site:

- `collect_stage_items` fills the two new `StageDrawItem` fields. `post_render_ent` is `Some(entity_num)` when the surface's entity is not the world entity and the entity carries `RF_DISTORTION`, `RF_FORCEPOST`, or `RF_FORCE_ENT_ALPHA`, and while fewer than `MAX_POST_RENDERS` distinct surfaces already deferred. `screen_image` is `EntityFx::resolve`'s `distortion` bit.
- `Pipeline3d::draw` partitions `items` into the main list and the post-render list, keeping every stage of one surface together and reversing the order of the deferred surfaces, which is the oracle's LIFO drain. It draws the main list in the existing pass. It then walks the deferred surfaces in that order and, for each one, encodes its capture when the run compare says it is due and opens one render pass carrying that surface's stage items, with `wgpu::LoadOp::Load` on both the colour and the depth attachment. Capture N therefore reads a frame holding deferred draws 1 through N-1, which is the oracle's interleave.
- The bind-group cache key in `Pipeline3d::draw` (`:1270`) gains the `screen_image` flag, and a `screen_image` item builds its group through `GpuImages::view_bind_group` against the captured view. Its group is built inside its own surface's step, not in the shared pre-pass cache walk, because the view it binds changes per capture. A distortion item with no capture this frame binds its own diffuse, which is the oracle's behavior when `R_WorldCoordToScreenCoord` returns false.
- `build_dynamic_block` flips `v` on the evaluated texture coordinates of a `screen_image` stage, `t -> 1.0 - t`, which cancels the opposite row order the two worlds store a copied rect in. A `≤2`-line note at the site names the reason.
- The `//TODO: Port RB_IterateStagesGeneric's RF_DISTORTION arm` marker and its stand-in note (`:4376-4383`) are deleted, and `EntityFx::distortion`'s doc records that the arm's cull half is already this backend's state.
- `Warned::Distortion` (`:686`), its slot (`:701`), its description (`:716`), and the two `warn_once` calls (`:2766`, `:2960`) are deleted. `Warned::COUNT` (`:690`) drops from 8 to 7.

New imports, merged into the file's existing use groups: `MAX_POST_RENDERS` from `mp_renderer::tr_backend`, `RF_FORCEPOST` from the `mp_qshared` `tr_types` group at `:44-47`, and `TrRefdef` from wherever the file already reaches `FrameState::refdef`. No `use` inside a function body, and no inline fully qualified path in an expression.

### `crates/mp/renderer/src/tr_mesh.rs` and `crates/mp/renderer/src/tr_ghoul2.rs`

Comment text only, at four sites. Each `DEFERRED:` note becomes a `//TODO: Port` marker with a `// Source:` cite naming the missing stencil-shadow backend as the blocker.

- `tr_mesh.rs:512-514`, the md3 stencil- and projection-shadow pushes (`oracle/codemp/renderer/tr_mesh.cpp:391-405`). This is the md3 read.
- `tr_ghoul2.rs:840-842` and `:909-911`, the ghoul2 stencil- and projection-shadow pushes (`oracle/codemp/renderer/tr_ghoul2.cpp:2586-2622`). These are the ghoul2 reads, and the MP-versus-SP asymmetry lives here.
- `tr_ghoul2.rs:2459-2462`, the `bInShadowRange` suppression (`oracle/codemp/renderer/tr_ghoul2.cpp:3525-3528`). This is the one `RF_NOSHADOW` write, not a read, so its marker carries no asymmetry note.

The asymmetry note belongs on the ghoul2 read sites alone. MP's ghoul2 projection arm does not test `RF_NOSHADOW` (`oracle/codemp/renderer/tr_ghoul2.cpp:2614-2622`) and SP's does (`oracle/code/renderer/tr_ghoul2.cpp:2801`). Both trees agree at the md3 site: neither projection arm tests it (`oracle/codemp/renderer/tr_mesh.cpp:400-404`, `oracle/code/renderer/tr_mesh.cpp:416-420`). No code changes in either file.

### `crates/mp/renderer-gpu/tests/scene_golden.rs`

One new scene builder and one new test, in the shape of `scene_depthhack` and `golden_scene_depthhack` (`:726-752`, `:922-930`):

```rust
fn scene_distortion(host: &mut UiHost, frame_data: &mut FrameData, shader: qhandle_t)

#[test] fn golden_scene_distortion()
```

No `#[ignore]`, matching every sibling in this file. `CHANNEL_TOLERANCE` stays 0.

`scene_distortion` draws a backdrop of four opaque sprites in four different colours through `gfx/golden/opaque`, then one `RF_DISTORTION` sprite through `gfx/golden/vertex` over them, with a radius large enough for the capture square to span more than one backdrop colour. The distortion sprite sits well off the horizontal centre of the viewport, because `cX = vidWidth - x - rad/2` reflects the capture square about the vertical centreline and a centred sprite would capture the same square with or without the mirror term. Placing it off centre is what makes the mirror observable. The distortion sprite must land in the post-render pass and sample the reflected backdrop square rather than white.

### `crates/mp/renderer-gpu/tests/entity_golden.rs`

One new scene and one new test, sharing the file's existing boot recipe:

```rust
#[test] #[ignore] fn golden_entity_renderfx_duel1()
```

It draws the `twinpodcc.md3` map object three times in one duel1 frame at three fixed origins, one with `RF_DISINTEGRATE1`, one with `RF_DISINTEGRATE2`, and one with `RF_VOLUMETRIC`. The two disintegrate entities carry an `endTime` and an `oldorigin` chosen so the frozen clock puts the burn threshold inside the model's bounds, which is what makes the colour bands and the vert deform visible. `#[ignore]` matches the file's existing test, because the scene needs the retail assets.

### Fixtures

Two new PNGs under `crates/mp/renderer-gpu/tests/goldens/`: `scene_distortion.png` and `entity_renderfx_duel1.png`.

One conditional re-bless: `scene_renderfx_tint.png`, only if the post-render deferral moves it. See open row 5.

Anything not on this list is out of scope, and the agent must not add it. No new third-party crate, because a dependency of the DEC-49 kind is a user ruling and this packet may never grant one. No shadow backend, no `RB_DistortionFill`, no `RB_CaptureScreenImage` full-screen path, no dynamic-glow pass, no `tr.distortionShader` marker-shader path. No change to any file under `crates/mp/engine/`, `crates/mp/cgame/`, `crates/mp/ui/`, or `crates/mp/uishared/`. No change to `stage2d.rs`, `pipeline2d.rs`, `tr_shade.rs`, `tr_surface.rs`, `tr_shadows.rs`, or any WGSL shader. No new `FrameEvent` variant, no new cvar, no new stat field, no ABI change, and no `WorldVertex` change. `world_golden.rs`, `ghoul2_vertex_golden.rs`, and `hud_golden.rs` take the row-2 argument change and nothing else. Every committed fixture except `scene_renderfx_tint.png` is read-only, and the two new PNGs are the only fixtures this step may create.

## Divergence notes, each ≤2 lines at its site

- **A zero or negative capture radius draws plain.** `rad` comes from `(int)e.radius` and nothing guards it. GL accepts a zero-sized `qglCopyTexImage2D` and leaves an unsampleable texture, and wgpu rejects a zero-sized texture outright. This is rule-19 undefined behavior, so the port picks the defined answer: skip the capture and let the stage bind its own diffuse.
- **A capture wider than the target is clamped.** Raven's clamp produces a negative `cX` when `rad > vidWidth`, and `qglCopyTexImage2D` then reads outside the framebuffer. The port clamps the side length to the smaller target dimension first, which is the same rule-19 choice.
- **The horizontal mirror is preserved.** `cX = vidWidth - x - rad/2` mirrors the captured square, where `cY = vidHeight - y - rad/2` only converts a top-down `y` to GL's bottom-up origin. The mirror is Raven's behavior and it ports as written, with a site note so a reader does not read it as a port bug.

The `v` flip on a screen-image stage is not on this list. It cancels the opposite row order the two worlds store a copied rect in, so it restores parity rather than diverging from it, and its site note says so.

## Open rows

The 2026-08-29 ratification walk closed every row. Rows 1, 2 and 5 are ratified as proposed, rows 3, 4, 6, 7 and 8 are ratified as amended, and the two rows the audit cleared stand. Each row below keeps its text as the lane's instruction.

**Row 1 - the `RF_DISTORTION` scope (user ruling, ratified as proposed).** The census counts 2,125 `RF_DISTORTION` submissions across the four traces (`scene-trap-census.md:164`), and DEC-54's complement rule covers unexercised features, which this is not (`docs/decisions.md:1415`). So the flag is real live surface and a documented complement is not available to it. The question is how much lands now. **Proposed default: land the whole chain in this step, the post-render list, the per-entity capture, and the stage bind.** The three parts are not separable in behavior: the capture is meaningless without the deferral that orders it, and the bind is meaningless without the capture. The alternative is to land the post-render deferral alone for draw-order parity and open a ticket for the capture, which leaves the census's second-largest dark row open past the step the plan named as its closer.

**Row 2 - the target texture reaches the backend (user ruling, ratified as proposed).** `Pipeline3d::draw` cannot copy out of a `TextureView`, and a wgpu texture copy takes a `wgpu::TexelCopyTextureInfo` holding the texture itself, the shape `crates/mp/renderer-gpu/src/gpu.rs:417-437` already uses. **Ratified: `execute_frame`, `execute_package`, `render_world`, and `Pipeline3d::draw` each gain a `target_texture: &wgpu::Texture` parameter beside the existing `target`, and `Gpu` gains a `headless_texture` accessor so the headless callers can pass one.** Twelve sites change: nine `execute_frame` call sites, `execute_package`'s one, and the two internal forwarding sites at `frame_exec.rs:611` and `:899`. Five of the twelve sit in test files, and three of those files are otherwise unedited by this step, `world_golden.rs`, `ghoul2_vertex_golden.rs`, and `hud_golden.rs`, since commits 5 and 6 already edit the other two. The alternative shapes are worse: a `FrameTarget` struct that bundles the two is a wider contract change, and letting `Gpu` remember the last acquired surface texture hides a lifetime the caller already owns.

**Row 3 - the screen-image texture shape (user ruling, new design, ratified as amended).** A wgpu render-to-texture capture has no oracle equivalent shape, so this is a design choice and not a transcription. **Ratified: `Pipeline3d` owns one `Option<ScreenImage>` holding a square texture at the target's own format, `TEXTURE_BINDING | COPY_DST`, rebuilt whenever the requested side length differs from the one it holds.** The amendment is how it fills. It fills per deferred surface, not once for the whole post-render list: capture, draw that surface, capture the next. One slot is therefore enough for any number of distortion entities, because each capture is consumed before the next overwrites it. A rebuild moves the replaced texture into a keep-alive vector the caller holds until the submit, so an already-encoded pass keeps a live binding.

The amendment also drops the claim that this reproduces the oracle's sampling exactly. It does not, on its own. `glCopyTexImage2D` stores the rect bottom-up and `copy_texture_to_texture` stores it top-down, so identical texture coordinates sample the square vertically flipped. The stage's own texture coordinates carry the `v` flip that cancels it, which is what makes the sampling match.

The alternatives each need a divergence: a fixed full-target texture would need a texture-coordinate scale uniform the oracle has no equivalent for, and a fixed maximum size would sample the wrong sub-rect.

**Row 4 - the two goldens (user ruling, ratified as amended).** **Ratified: two new PNGs, each through the step-007 procedure that step-008 and step-009 ratified.** Run once with `JKA_GOLDEN_BLESS=1`, re-run without it to confirm the byte-identical pass, then STOP before the commit that carries the PNG so the user looks at the image. Named defect conditions, one per image. The amendment fixes the synthetic scene's placement, because a centred distortion sprite makes its own defect condition vacuous.

`scene_distortion.png` must show the distortion sprite carrying a recognizable square of the four-colour backdrop, not a flat white or flat coloured quad. A flat quad means the capture never ran or the bind fell back to the stage's own diffuse. The mirror is a positional reflection about the vertical centreline, not a flip of the square's own content, so the sprite must show the backdrop colours that sit at the mirrored screen position rather than the ones directly behind it. The sprite sits well off centre for exactly this reason: at the centre both readings give the same square and the check discriminates nothing. A sprite showing the colours directly behind it means the `vidWidth - x` term was corrected away, which is a divergence this packet does not permit. A sprite whose captured square reads upside down against the backdrop means the `v` flip is missing.

`entity_renderfx_duel1.png` must show three copies of the same model that differ from each other. The `RF_DISINTEGRATE1` copy must carry visible grey and black bands with a hole, the `RF_DISINTEGRATE2` copy must carry a hard white-to-transparent edge with the surviving shell visibly pushed out along its normals, and the `RF_VOLUMETRIC` copy must carry a rim that brightens away from the view axis. Three identical models means the short-circuit never fired. A `RF_DISINTEGRATE2` copy with no vertex displacement means the normals never reached the deform.

The alternative for the second image is a synthetic sprite scene, which needs no retail assets but has no normals, so it would gate neither the vert deform nor the volumetric fade. The disintegrate colour bands would still show on a sprite's four corners. That trade is the row's real content.

**Row 5 - `scene_renderfx_tint.png` under the deferral (user ruling, ratified as proposed).** The post-render list defers `RF_FORCE_ENT_ALPHA`, which is already live and already gated. `scene_renderfx_tint` submits the only two entities carrying it in any committed fixture (`scene_golden.rs:697-698`), four sprites that do not overlap, so the reordering may or may not move a pixel. **Ratified: the deferral lands exactly as the oracle writes it, and if the golden moves the lane STOPs and the image goes through the row-4 bless procedure before the commit lands.** The four sprites must still read as four sprites in the same four positions and colours. A sprite that changed colour or vanished is a defect, not a blessable golden, because a pure reordering of non-overlapping alpha quads cannot change either.

**Row 6 - the distortion stage's GL state (mechanical, ratified as amended per the audit challenge).** The premise of the drafted row was false. The oracle holds two separate chains in the single-texture branch: the bind chain at `tr_shade.cpp:2163-2175` and the state chain at `:2177-2206`. A plain `RF_DISTORTION` stage matches neither the stencil arm nor the force-alpha arm, so it reaches `else { GL_State( stateBits ); }` at `:2203-2206` and takes its own stage state. **Ratified: draw the distortion stage under its own shader state, which is plain parity.** No divergence, no site note. Divergence note 4 is deleted and the packet's stage-arm prose is corrected. A distortion entity that also carries `RF_FORCE_ENT_ALPHA` diverts into the force-alpha arm, which this backend already ports.

**Row 7 - the post-render partition, the LIFO order, and the replay shape (mechanical, ratified as amended per the audit challenge).** **Ratified as faithful: partition at the surface, not at the stage.** Every `StageDrawItem` a deferred surface produced moves together and keeps its own stage order, and the order of the deferred surfaces reverses to match the oracle's `while (g_numPostRenders > 0) { g_numPostRenders--; ... }` drain (`tr_backend.cpp:1008-1011`). A flat reversal of the item list would reverse the stages inside each surface too, which the oracle never does. The `MAX_POST_RENDERS` cap counts surfaces, not stages, and a surface past the cap draws in the main pass.

The amendment replaces the replay shape. **Ratified: replay per deferred surface with the oracle's capture-and-draw interleave (`tr_backend.cpp:1144-1212`), one render pass per deferred surface, so capture N sees deferred draws 1 through N-1.** The drafted all-captures-then-one-pass shape cannot reproduce that: every capture would read a frame with no deferred surface in it, and the one screen-image slot would leave every distortion stage sampling the last capture. Two cloaked players on screen is ordinary live surface, and the single committed distortion golden draws one entity and would not catch the fault. The capture gate is the run compare row 9 names, not a per-frame set.

**Row 8 - the `RF_NOSHADOW` disposition (user ruling, ratified as amended).** **Ratified: no port, a recorded disposition.** Both MP reads gate on `r_shadows->integer == 2`, the retail default is 1 (`oracle/codemp/renderer/tr_init.cpp:1139`), and this workspace has no shadow backend to add a surface to. The flag graduates with the stencil-shadow backend, per DEC-54's complement rule. The alternative is to port the whole stencil-shadow chain, which is a step of its own and much larger than this one.

The amendment corrects the conversion set and the asymmetry claim. The drafted pair pinned the asymmetry on `tr_ghoul2.rs:2459-2462`, which is the deferral note for the `RF_NOSHADOW` **write** (`oracle/codemp/renderer/tr_ghoul2.cpp:3525-3528`, the `bInShadowRange` suppression), and left the ghoul2 reads unmarked. **Ratified: four marker sites.** The write-site note at `tr_ghoul2.rs:2459-2462` converts and carries no asymmetry sentence. The reads are `tr_mesh.rs:512-514` for md3 and `tr_ghoul2.rs:840-842` and `:909-911` for ghoul2, and the asymmetry note goes on the ghoul2 reads alone: MP's ghoul2 projection arm does not test the flag (`oracle/codemp/renderer/tr_ghoul2.cpp:2614-2622`) and SP's does (`oracle/code/renderer/tr_ghoul2.cpp:2801`), while both trees agree at the md3 site that neither projection arm tests it (`oracle/codemp/renderer/tr_mesh.cpp:400-404`, `oracle/code/renderer/tr_mesh.cpp:416-420`).

**Row 9 - the capture dedup and its retry (mechanical, cleared by the audit with one wording correction).** Transcribe `lastPostEnt` as written. It is set only after a successful copy (`tr_backend.cpp:1207`), so an entity whose first deferred surface projects behind the eye retries on its next surface. Reset it per frame, matching the local declared inside the drain (`:1006`), and never per view. The correction: the gate is a compare against the last captured entity, not a set of every entity captured this frame, so a deferred order of A, B, A captures A twice. Write the run compare, not a set membership test.

**Row 10 - `MAX_POST_RENDERS`'s home (mechanical, cleared by the audit).** `mp_renderer::tr_backend`, beside the deferred `RB_RenderDrawSurfList` that owns Raven's `tr_backend.cpp` file statics. This is the step-009 precedent for `LIGHTNING_RECURSION_LEVEL` and `TrSurfaceShapeState`, which kept their canonical home in `mp_renderer::tr_surface` while the live arms landed in `pipeline3d.rs`.

## Pause triggers, named for this step

- Any committed fixture other than `scene_renderfx_tint.png` moves. STOP. Commits 1, 3, and 4 change no committed draw, and a moved pixel means something else changed.
- `scene_renderfx_tint.png` moves in any commit other than commit 2. STOP. Only the deferral may touch it.
- A committed fixture moves in commit 4. STOP. No committed scene submits `RF_DISTORTION`, so the stage arm must be inert on all of them.
- The capture appears to need the frame's depth buffer, a second colour attachment, or a resolve target. STOP. The oracle copies colour only.
- The distortion sprite draws white in the golden candidate. STOP before blessing. That is the capture failing, not a look to accept.
- The horizontal mirror looks like a bug worth fixing. STOP. It is Raven's behavior, and the packet's capture section is the instruction.
- A shape appears that captures every deferred surface before any deferred surface draws. STOP, per row 7. That shape is ruled out, and the interleave is the instruction.
- The `v` flip looks like it belongs in a WGSL shader. STOP. Every distortion item is an entity surface built through `build_dynamic_block`, so the flip lands on the CPU and no shader is in scope.
- The `RF_NOSHADOW` work appears to need a shadow surface, a `tr.shadowShader`, or a stencil pass. STOP, per row 8. This step ports no shadow path.
- Any arm seems to need `tr.distortionShader`, `tr_stencilled`, `RB_DistortionFill`, or the 2048-square capture. STOP. All four are dead or unreachable in the retail MP build.
- The post-render partition seems to need a mutable `refEntity_t` or a second entity walk. STOP. The capture reads the entity back by index out of the existing `entities` slice.
- Verification is `cargo build` or `cargo check` plus the golden suites, never rust-analyzer, which is stale in this workspace.

## Commit bundle

The full gate battery, named once and referenced per commit:

- `cargo build --workspace`. An intermediate commit may carry warnings, and the bundle's final state must build with zero warnings. Commit 1 leaves `target_texture` unread inside `Pipeline3d::draw`, because commit 3 is its first consumer, so one unused-parameter warning is expected there and needs no placeholder. Every commit from 3 onward builds clean.
- `cargo test --workspace -- --test-threads=1`.
- `cargo test -p mp_renderer_gpu --test world_golden -- --ignored --test-threads=1`, all four world goldens byte-identical.
- `cargo test -p mp_renderer_gpu --test scene_golden -- --test-threads=1`, every scene golden byte-identical.
- `cargo test -p mp_renderer_gpu --test entity_golden -- --ignored --test-threads=1`, byte-identical.
- `cargo test -p mp_renderer_gpu --test ghoul2_vertex_golden -- --ignored --test-threads=1`, byte-identical.
- `cargo test -p mp_renderer_gpu --test hud_golden -- --test-threads=1` and the same with `--ignored`, both byte-identical.

Every golden run is serial with `--test-threads=1`, each as one foreground command with a long timeout. Two engine boots in parallel threads crash in the GPU init path, and the world-golden pk3 inflate aborts without it.

1. **The target texture reaches the backend.** `Gpu::headless_texture`, the new parameter on `execute_package`, `execute_frame`, `render_world`, and `Pipeline3d::draw`, and all twelve sites updated to pass it. No draw changes. Files: `crates/mp/renderer-gpu/src/gpu.rs`, `frame_exec.rs`, `pipeline3d.rs`, `crates/mp/client-app/src/render_thread.rs`, the three harness binaries, and the five test files. Subject: `feat(gh#31 s010): the frame target texture reaches the backend`. Gates: the full battery. All eighteen committed fixtures byte-identical is the proof that the threading is pure.

2. **The post-render list.** `MAX_POST_RENDERS`, the two new `StageDrawItem` fields, the partition and the LIFO reversal in `Pipeline3d::draw`, and one render pass per deferred surface with `wgpu::LoadOp::Load` on both attachments. No capture and no screen bind yet, so a distortion entity still draws its own texture, just later. Files: `crates/mp/renderer/src/tr_backend.rs`, `crates/mp/renderer-gpu/src/pipeline3d.rs`. Subject: `feat(gh#31 s010): the post-render deferral`. Gates: the full battery. Seventeen fixtures byte-identical, and `scene_renderfx_tint.png` under row 5.

3. **The screen capture.** `ScreenImage`, the `Pipeline3d` field, the keep-alive vector, `world_coord_to_screen_coord_float`, `world_coord_to_screen_coord`, `screen_capture_rect`, `capture_screen_image`, and the encode of each capture immediately before its own deferred surface's pass, gated by the row-9 run compare. Nothing binds the captured texture yet. Files: `crates/mp/renderer-gpu/src/pipeline3d.rs`. Subject: `feat(gh#31 s010): the per-entity screen capture`. Gates: the full battery. No committed scene submits `RF_DISTORTION`, so all eighteen fixtures stay byte-identical.

4. **The distortion stage arm.** `GpuImages::view_bind_group`, the per-surface bind-group build against the captured view, the extended cache key, the `build_dynamic_block` `v` flip, the marker deletion, and the `Warned::Distortion` removal. Files: `crates/mp/renderer-gpu/src/gpu_images.rs`, `pipeline3d.rs`. Subject: `feat(gh#31 s010): the RF_DISTORTION stage arm`. Gates: the full battery, all eighteen fixtures byte-identical.

5. **The distortion golden.** `scene_distortion` and its PNG, after its own row-4 STOP. Files: `crates/mp/renderer-gpu/tests/scene_golden.rs` and the new PNG. Subject: `test(gh#31 s010): the distortion golden`. Gates: the full battery, with the new golden green at tolerance zero.

6. **The disintegrate and volumetric golden.** `golden_entity_renderfx_duel1` and its PNG, after its own row-4 STOP. Files: `crates/mp/renderer-gpu/tests/entity_golden.rs` and the new PNG. Subject: `test(gh#31 s010): the disintegrate and volumetric golden`. Gates: the full battery, with the new golden green at tolerance zero.

7. **The `RF_NOSHADOW` disposition.** The four marker conversions of row 8, comment text only. Files: `crates/mp/renderer/src/tr_mesh.rs`, `crates/mp/renderer/src/tr_ghoul2.rs`. Subject: `docs(gh#31 s010): the RF_NOSHADOW disposition`. Gates: the full battery, with every fixture byte-identical, which is what a comment-only change must produce.

8. **The finished file**, per the packet skill: assumptions and choices keyed to their commits, deviations or the word "none", the commit list with gate results, and open gaps. File: `.claude/packets/31/step-010/finished.md`. Subject: `process(gh#31 s010): finished file`.

Every commit uses `--no-gpg-sign`, a heading subject, an STE body, and no trailer of any kind: no `Co-Authored-By`, no generated-with footer. Gate results are written as plain sentences inside the body, so no line parses as a git trailer. The lockstep referee is not required, because no commit touches `mp_game`, the server, or any `jampded` link-set crate.

## Write scopes

Branch `gh31-step-010-renderfx`, cut from `gh31-step-009-fx-minirefents`. A worktree builder runs `git merge gh31-step-009-fx-minirefents --no-gpg-sign` as its first act, not a merge from master, because step-009 is not on master yet and its arms are this step's ground.

- `crates/mp/renderer-gpu/src/pipeline3d.rs` - the field, `ScreenImage`, the two `StageDrawItem` fields, the four free functions, the partition, the per-surface post-render passes, the bind, the `build_dynamic_block` `v` flip, the marker and warning deletions, the imports.
- `crates/mp/renderer-gpu/src/gpu.rs` - `headless_texture` only.
- `crates/mp/renderer-gpu/src/gpu_images.rs` - `view_bind_group` only.
- `crates/mp/renderer-gpu/src/frame_exec.rs` - the new parameter on the three functions only.
- `crates/mp/renderer/src/tr_backend.rs` - `MAX_POST_RENDERS` and one deferred-note line only.
- `crates/mp/renderer/src/tr_mesh.rs` - one marker conversion at `:512-514`, comment text only.
- `crates/mp/renderer/src/tr_ghoul2.rs` - three marker conversions at `:840-842`, `:909-911`, and `:2459-2462`, comment text only.
- `crates/mp/renderer-gpu/tests/scene_golden.rs` - `scene_distortion` and its test.
- `crates/mp/renderer-gpu/tests/entity_golden.rs` - `golden_entity_renderfx_duel1` and its scene.
- `crates/mp/renderer-gpu/tests/goldens/scene_distortion.png`, `entity_renderfx_duel1.png` - new, blessed under the row-4 STOP.
- `crates/mp/renderer-gpu/tests/goldens/scene_renderfx_tint.png` - re-blessed under the row-4 STOP, in commit 2 only, and only if row 5 fires.
- `crates/mp/client-app/src/render_thread.rs`, `crates/mp/renderer-gpu/src/bin/dev_harness.rs`, `ui_harness.rs`, `world_harness.rs`, `crates/mp/renderer-gpu/tests/world_golden.rs`, `ghoul2_vertex_golden.rs`, `hud_golden.rs` - edit-only, to pass the new `target_texture` argument. Any other caller `cargo check` shows broken by the same signature is in scope on the same edit-only terms.
- `.claude/packets/31/step-010/` for `finished.md`.

Everything else is read-only, including `oracle/`, every file under `crates/mp/engine/`, `crates/mp/cgame/`, `crates/mp/ui/`, `crates/mp/uishared/`, `crates/mp/renderer/src/tr_shade.rs`, `tr_surface.rs`, `tr_shadows.rs`, `crates/mp/renderer-gpu/src/stage2d.rs`, `pipeline2d.rs`, every WGSL shader, every other committed fixture, and `~/Developer/jka/` beyond read-only asset reads. Source files change through the Edit tool only.

## Disposition

After a clean lane-review: open the pull request from `gh31-step-010-renderfx` into the `wf/31-renderer-census` umbrella branch and merge it on GitHub with a merge commit. This follows the 2026-08-29 ruling that the census steps collect on the umbrella and reach master in one merge at the end, the same base step-009's pull request #48 uses. The pull request opens after #48 merges, so the diff carries this step's commits alone. Never squash, and never commit on master. The session never pushes or opens the pull request unprompted. It prepares the branch, asks, and the user rules on the push and on the merge.

## Amendments

**2026-08-29 - the ratification walk closed all eight open rows.** The audit is at `.claude/packets/31/step-010/audit.md` (`bdd3ee76`). Two rows were challenged there and three claims disputed, and the rulings below execute them. Every ruling is folded into the body above.

- Row 1, the `RF_DISTORTION` scope: ratified as proposed. The whole chain lands this step, the post-render list, the per-entity capture, and the stage bind.
- Row 2, the target texture reaches the backend: ratified as proposed, with the audit's call-site correction. Twelve sites change, five of them in test files, and three of those test files are otherwise unedited by this step.
- Row 3, the screen-image texture shape: ratified as amended. One square texture on `Pipeline3d`, rebuilt on a side-length change, stands. It fills per deferred surface, capture, draw that surface, capture the next, so one slot serves any number of distortion entities. The stage's texture coordinates carry a `v` flip for the opposite row order the two worlds store a copied rect in, and the "reproduces the sampling exactly" claim is rewritten around it.
- Row 4, the two goldens: ratified as amended. Both goldens land as drafted, and the synthetic scene places the distortion sprite off the horizontal centre so the capture rect's mirror is observable. The defect condition names the expected mirror displacement.
- Row 5, `scene_renderfx_tint.png` under the deferral: ratified as proposed. The deferral lands as written, and a moved golden is a STOP plus an eyes-on bless.
- Row 6, the distortion stage's GL state: ratified as amended per the audit challenge. The oracle's state chain is separate from its bind chain (`oracle/codemp/renderer/tr_shade.cpp:2177-2206`), and a plain `RF_DISTORTION` stage reaches `else { GL_State( stateBits ); }`, so drawing under the stage's own state is plain parity. Divergence note 4 and the "skips `:2203-2206`" sentence are deleted.
- Row 7, the post-render partition and the replay shape: ratified as amended per the audit challenge. The faithful partition, the per-surface stage order, the LIFO reversal, and the surface cap all stand. The replay adds the oracle's capture-and-draw interleave (`oracle/codemp/renderer/tr_backend.cpp:1144-1212`), one render pass per deferred surface, so capture N sees deferred draws 1 through N-1. The all-captures-then-one-pass text in the body and in commit 3 is rewritten to that shape.
- Row 8, the `RF_NOSHADOW` disposition: ratified as amended. No port. The marker set becomes four sites: the write-site note at `tr_ghoul2.rs:2459-2462` (oracle `:3525-3528`), and the read sites at `tr_mesh.rs:512-514`, `tr_ghoul2.rs:840-842`, and `:909-911`. The asymmetry note scopes to the ghoul2 site alone (`oracle/code/renderer/tr_ghoul2.cpp:2801`), because SP's md3 projection arm does not test the flag either.

Two rows the audit cleared were not walked and stand as drafted, with the audit's wording correction folded in.

- Row 9, the capture dedup: cleared, with the gate corrected from a per-frame set to the run compare the oracle writes.
- Row 10, `MAX_POST_RENDERS`'s home: cleared.

Three cite corrections from the audit's claim verification are folded in the same pass: `tr_types.h:17-54` holds twenty-two flag defines rather than twenty-one, `RF_FIRST_PERSON`'s live cull gate is `crates/mp/renderer/src/tr_main.rs:1887` rather than the light-logging gate the draft cited, and the out-of-scope line no longer forbids the row-2 argument change in the three otherwise-unedited test files.

**2026-08-30 - a mid-lane amendment: the horizontal mirror does not exist, and the distortion golden passed.**

The lane found that the packet's mirror claim is false, and the coordinator ratified the finding. `R_WorldCoordToScreenCoordFloat` projects onto `tr.refdef.viewaxis[1]`, and `AnglesToAxis` fills that row with the left vector (`oracle/codemp/game/q_math.c:530-536`, Raven's own comment "angle vectors returns right instead of y axis"). The GL view transform then maps our `y` to GL `-x` through `s_flipMatrix` (`oracle/codemp/renderer/tr_main.cpp:17-27`). So the helper's `x` counts from the right edge of the screen, and the entity renders at `vidWidth - x`. Raven's `cX = vidWidth - x - (rad/2)` converts the helper's value back into a left-edge column, so the capture square lands on the entity's own screen position. The lane confirmed this against a rendered frame: a backdrop sprite at world `y = -115` drew at screen x 252, not at 68.

- Divergence note 3, "The horizontal mirror is preserved", is struck. There is no mirror, so there is nothing to note at the site.
- Row 4 loses its mirror clause. The sentence requiring the sprite to show the colours at a mirrored screen position is struck, and so is the sentence naming a sprite that shows the colours directly behind it as a defect. The rationale for placing the sprite off the horizontal centre is struck with them, because the capture always centres on the entity, so a sprite that does not sit over the backdrop captures an empty frame.
- Row 4's `v`-flip defect reading inverts. `glCopyTexImage2D` puts `t = 0` at the captured rect's bottom row, so the oracle's own sprite draws the crop upside down against the screen behind it. The port's `v` flip reproduces that, and bands that read the right way up mean the flip is missing.
- No code changes. `screen_capture_rect` transcribes both terms as written and was already correct.

`scene_distortion.png` passed its eyes-on bless on 2026-08-30. The image shows the sprite carrying a magnified crop of the four-colour block behind it, with the two lower quadrant colours in a thin band across the sprite's upper edge, which is the row order the oracle produces.

**2026-08-30 - a mid-lane amendment: the disintegrate arms were dead, and the fix lands in this lane.**

The packet states that `RF_DISINTEGRATE1` and `RF_DISINTEGRATE2` are "already ported and live" and that "This step adds no code for them, only a gate". Both claims were false. The gate this step ordered exposed the fault on its first render.

`lighting_ref_entity` (`crates/mp/renderer-gpu/src/pipeline3d.rs`) built the `RefEntity` the colour short-circuits read, and it filled four fields and defaulted every other one. `renderfx`, `old_origin` and `end_time` all fell to zero. `RB_CalcDisintegrateColors` branches on `renderfx` (`oracle/codemp/renderer/tr_shade_calc.cpp:1545-1636`), so neither branch ran and the colour buffer kept the zeros it was allocated with. `RB_CalcDisintegrateVertDeform` tests the same flag (`:1640-1671`), so the vert deform never fired. In the render the `RF_DISINTEGRATE1` copy vanished, because its forced alpha test discards a zero alpha, and the `RF_DISINTEGRATE2` copy drew as a black silhouette. `RF_VOLUMETRIC` was unaffected, because it reads `shaderRGBA` alone.

The user ratified the one-hunk fix in this lane. `lighting_ref_entity` now carries `renderfx`, `old_origin` and `end_time`, and the change lands as its own commit ahead of the golden. All nineteen fixtures committed before it stayed byte-identical, because no committed scene submits either flag.

**2026-08-30 - a mid-lane amendment: the renderfx entity golden is blessed, with two of row 4's clauses struck.**

`entity_renderfx_duel1.png` passed its eyes-on bless on 2026-08-30. Row 4 asked the `RF_DISINTEGRATE1` copy for "visible grey and black bands with a hole" and the `RF_DISINTEGRATE2` copy for a "surviving shell visibly pushed out along its normals". The hole and the hard edge both landed. The band clause and the deform clause are struck, because the chosen model cannot show either at any threshold.

`RB_CalcDisintegrateColors` puts its three bands in a shell from `threshold` to `sqrt(threshold² + 180)`, which is about `90/threshold` units thick. `twinpodcc.md3` carries 702 vertices over a 315 unit span, so its vertices sit roughly 30 units apart. At the threshold of 120 that cuts a visible hole the shell is 0.75 units thick and catches almost no vertex. A lower threshold widens the shell and shrinks the sphere until it catches too few vertices to cut a hole at all, which a 12 unit attempt confirmed. `RB_CalcDisintegrateVertDeform` moves the vertices inside the sphere, and those are the ones that turn transparent, so the displacement has the same limit. The user ruled that the golden still gates all three arms, because it is the image that exposed the dead arms above.

The model also forced two scene changes, both recorded here as part of the ratified image.

- The three copies stand 600 units from the eye at side offsets `[340, 0, -340]`. The map object is 315 units deep, so at the sibling test's 260 units one copy alone spans about 700 pixels of the 800 pixel frame. At 600 units each copy is about 270 pixels and three fit side by side.
- The scene carries `RDF_NOWORLDMODEL`. At 600 units all three copies sit behind the duel1 wall. The sibling `golden_entity_duel1` keeps the world in frame, so the world and entity composition is still covered.

**2026-08-30 - the lane-review walk closed all ten rows.**

The vet is at `.claude/packets/31/step-010/vet.md`. It raised 18 findings over the range `61a5062b..gh31-step-010-renderfx`. The named base branch no longer exists, because step-009 merged into `wf/31-renderer-census` as pull request #48, and `61a5062b` is its tip. The user ruled on every row, and the rulings below are the ledger.

Five findings take a code fix, and they land in one commit.

- F7, the failed-capture fallback. The oracle binds `tr.screenImage` unconditionally (`oracle/codemp/renderer/tr_shade.cpp:2163-2169`), so a skipped capture samples the last successful capture or the 8 by 8 white placeholder (`oracle/codemp/renderer/tr_image.cpp:2776`). The port binds the stage's own diffuse. The fallback stays as written and now carries a divergence note. The Surface contract's clause "which is the oracle's behavior when `R_WorldCoordToScreenCoord` returns false" is struck, because those oracle lines do not support it.
- F8 and F15, the struck mirror sentence. The comment at the capture rectangle still asserted the mirror the 2026-08-30 amendment struck. It is replaced by the proven fact, one sentence per line. The `vid_height - y` sentence stands.
- F9, the all-zero colour buffer. The site now records that `RF_DISINTEGRATE1`'s first band writes alpha only and leaves rgb at the previous surface's `tess` scratch, that the port supplies zeros, and that the difference is confined to the alpha-test border blend at no more than a quarter vertex weight.
- F10, the unguarded `i32` arithmetic. `screen_capture_rect` widens its clamp arithmetic to `i64`, so a saturated projection cannot overflow in a debug build. Results are bit-identical for every in-range input, and the site note records the rule-19 pick against C's undefined out-of-range cast.
- F14, F16, F17, F18 and the row-8 rider. Eleven over-length comment lines break at clause boundaries. `lighting_ref_entity`'s doc names the seven fields the builder carries. The `entity_golden.rs` copy width reads 270 pixels, which is what the blessed image measures. Two `tr_ghoul2.rs` lines name the stencil-shadow backend and the gore backend instead of a wave. The doc-site marker takes the exact `//TODO: Port` form.

The remaining findings are closed by ruling, and the clauses below change with them.

- F1, F2 and F13. `Pipeline3d::draw_items`, the `(Vec<StageDrawItem>, Vec<Range<usize>>)` return on `collect_stage_items`, and the two parameterized path helpers in `entity_golden.rs` are accepted as forced surfaces. Row 7's per-surface replay needs two callers for one unchanged draw body, the surface boundaries cannot be rebuilt from `post_render_ent` alone because one entity contributes several adjacent draw surfs, and a second golden in one test file needs a stem parameter.
- F3. The `screen_image` contract clause now reads that the bit applies on the single-texture path, with `false` at the fog site, the two-texture site, and the unreachable non-dynamic site. The fog pass binds `tr.fogImage` and a multitextured stage diverts to `DrawMultitextured`, which has no distortion arm, so both are oracle-correct.
- F4. The PBR own-diffuse fallback joins the divergence list as ratified. A distortion stage under `BackendMode::Pbr` binds its own diffuse, because `view_bind_group` builds the two-texture group and the PBR pipeline takes a four-texture layout.
- F5. The marker set is five sites, not four. The `_G2_GORE` split is ratified, because that deferral shared the note the row-8 conversion replaced and would otherwise have lost its greppable subject.
- F6. Session-directed amendment appends to `packet.md` are inside the lane's write scope. The write-scope line granting the folder for `finished.md` alone did not anticipate a mid-lane ruling.
- F11. The eight landed `Gates:` bodies stand. A future gate paragraph opens with prose, so no final paragraph starts with a bare token-colon line.
- F12. The four unplanned commits are listed against their rulings. `9b47e5c9` is the disintegrate fix the user ratified in this lane, and `a09b8e38`, `c7da3493` and `035334af` are the three amendment appends F6 covers.

**2026-08-30 - the fix-round walk closed two minor rows.**

The vet walked the fix-round commits and returned two findings. The user ruled on both.

- M1, the three-line F9 note. The divergence note at the all-zero colour buffer runs to three lines where the packet's divergence section asks for two or fewer. It stands as ratified wording, and this site is the one exception to that cap. The F9 ruling named three facts: the oracle's first band writes alpha only and leaves rgb at the previous surface's `tess` scratch, the port supplies zeros, and the difference is confined to the alpha-test border blend at no more than a quarter vertex weight. Three facts do not fit two lines under the 150 column rule.
- M2, the step-folder write scope. The scope extends to `vet.md`, the lane-review skill's required artifact. The folder grant now reads: `finished.md`, session-directed `packet.md` tail appends, and the vet's `vet.md`.
