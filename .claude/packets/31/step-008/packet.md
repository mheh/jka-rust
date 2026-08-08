# Packet gh#31 step-008 - the 2D closure

## Scope

This step closes the census 2D group (DEC-54). Three of the group's four rows already run end to end: `2d/SetColor`, `2d/DrawStretchPic` and `2d/Font_DrawString` all reach the batch, because a font string is laid out at trap time and recorded as one `SetColor`/`DrawStretchPic` pair per glyph (`RE_Font_DrawString_body`, `crates/mp/renderer/src/tr_font.rs:2772-2779`). The one dark row is `2d/DrawRotatePic` (547 submissions in trace-swoop1, `docs/plans/2026-07-24-client-port/scene-trap-census.md:11`), which the census recorder counts for `R_DRAWROTATEPIC` and `R_DRAWROTATEPIC2` alike (`crates/cgame/tests/scene_census.rs:279-287`).

The chain above the gap is already live. The cgame traps enter at `CG_R_DRAWROTATEPIC` and `CG_R_DRAWROTATEPIC2` (`crates/mp/engine/client/src/cl_cgame.rs:2300-2337`), the frontends record the events (`RE_RotatePic`, `crates/mp/renderer/src/tr_cmds.rs:180-208`; `RE_RotatePic2`, `:214-242`), and both `FrameEvent` variants carry every field (`crates/mp/renderer/src/render_state/frame_event.rs:152-178`). The executor is where the chain stops: one arm skips both events with a warn-once (`crates/mp/renderer-gpu/src/frame_exec.rs:527-530`). On the live client this leaves three HUD elements blank: the radar icons (`CG_DrawRadar`, `CG_DrawRotatePic2` at `crates/mp/cgame/src/cg_draw.rs:2455,2584`), the disruptor zoom insert and its ticks (`:3182,3241`), and the rocket-lock wedge (`CG_DrawRocketLocking`, `CG_DrawRotatePic` at `:4448`).

The step also gives the 2D group its first goldens. No committed fixture draws a single 2D quad today: `world_golden.rs`, `scene_golden.rs`, `entity_golden.rs` and `ghoul2_vertex_golden.rs` all exercise 3D paths only, and nothing under `crates/mp/renderer-gpu/tests/` references `pipeline2d` or `stage2d`. This step adds one new test file with two tests: a synthetic asset-free 2D scene, and a retail-dependent font string.

The step does not touch the frontend (`tr_cmds.rs` records both events correctly), `stage2d.rs`, `blend.rs`, the world or entity paths, any existing test file, or any existing fixture. It adds no cvar, no `FrameEvent` variant, and no ABI surface.

**Carried from step-007, and directly relevant here.** The per-surface vertex cap against the oracle's `tess` batching is a standing backend architecture fact. The 2D path has the same fact in a sharper form, and it drives open row 1: the oracle's `RB_StretchPic` accumulates into `tess` and flushes only when the shader changes or at `RB_SwapBuffers` (`oracle/codemp/renderer/tr_backend.cpp:1429-1440,1841-1843`), while `RB_RotatePic` draws immediately through `qglBegin`/`qglEnd` with no flush of its own. A rotate pic therefore draws ahead of the stretch pics recorded before it, in whatever GL state the last flush left. This backend batches every 2D quad in command order and picks a blend per quad, so neither the interleave nor the inherited state reproduces exactly. Row 1 is the ruling on that.

## The oracle, cited

**`RE_RotatePic` / `RE_RotatePic2`** (`oracle/codemp/renderer/tr_cmds.cpp:244-265` and `:270-291`): each fills a `rotatePicCommand_t` with the ten fields the Rust frontends already record. Nothing else happens frontend-side.

**`RB_RotatePic`** (`oracle/codemp/renderer/tr_backend.cpp:1498-1541`):

- `image = &shader->stages[0].bundle[0].image[0]`. In this tree `textureBundle_t::image` is a single `image_t *` (`oracle/codemp/renderer/tr_local.h:372-389`), so `&image[0]` is the bundle's own image pointer, and `if (image)` is a real null gate. No animation index, no `R_BindAnimatedImage`, no video-map handling. For an `animMap` stage that same field holds the base of an `image_t *` array (`oracle/codemp/renderer/tr_shader.cpp:1441-1442`), so the oracle binds a type-confused pointer there.
- No `numUnfoggedPasses` gate. Stage 0 is read whatever the shader is, and `GeneratePermanentShader` always allocates at least one zero-filled stage (`oracle/codemp/renderer/tr_shader.cpp:2782-2783`), so a zero-pass shader reads a NULL image and the `if (image)` gate skips the draw.
- No `GL_State` call. The pass draws in the state the last `GL_State` left, which is `GLS_DEPTHTEST_DISABLE | GLS_SRCBLEND_SRC_ALPHA | GLS_DSTBLEND_ONE_MINUS_SRC_ALPHA` right after `RB_SetGL2D` (`:1282-1284`).
- `qglColor4ubv(backEnd.color2D)` - the `RE_SetColor` register, byte-quantized at `RB_SetColor` (`:1409-1412`).
- Geometry: `qglTranslatef(x + w, y, 0)`, `qglRotatef(a, 0, 0, 1)`, then the four corners `(-w, 0)`, `(0, 0)`, `(0, h)`, `(-w, h)` with texture coordinates `(s1,t1)`, `(s2,t1)`, `(s2,t2)`, `(s1,t2)`. The pivot is the quad's top-right corner, and the unrotated quad is the same rectangle a stretch pic would draw.

**`RB_RotatePic2`** (`oracle/codemp/renderer/tr_backend.cpp:1547-1602`):

- Gates on `shader->numUnfoggedPasses` first, then on the same stage-0 image pointer.
- `GL_State(shader->stages[0].stateBits)` - this arm does install its own blend.
- Same color register. Geometry: `qglTranslatef(x, y, 0)` then the rotation, with corners `(-w/2, -h/2)`, `(w/2, -h/2)`, `(w/2, h/2)`, `(-w/2, h/2)`. The `x`/`y` pair is the quad's center here, not its top-left corner.
- The tail resets state to `GLS_DEPTHTEST_DISABLE | GLS_SRCBLEND_SRC_ALPHA | GLS_DSTBLEND_ONE_MINUS_SRC_ALPHA` (`:1600-1602`), which is `GLS_2D_DEFAULT`.

Both arms run from `RB_ExecuteRenderCommands` at `RC_ROTATE_PIC` and `RC_ROTATE_PIC2` (`oracle/codemp/renderer/tr_backend.cpp:1929-1934`).

## Surface contract

**`QuadBatch` gains one method** (`crates/mp/renderer-gpu/src/pipeline2d.rs`, beside `push_quad_st` at `:179`):

```rust
/// Appends one screen-space quad with independent per-corner positions, the shape a rotated pic needs.
/// `xy` and `st` are both in `RB_StretchPic`'s vertex order: top-left, top-right, bottom-right, bottom-left.
///
/// Source: `oracle/codemp/renderer/tr_backend.cpp:1519-1533`
pub fn push_quad_xy(
    &mut self,
    xy: [[f32; 2]; 4],
    st: [[f32; 2]; 4],
    color: [f32; 4],
    blend: BlendState,
    image: Option<ImageHandle>,
)
```

`push_quad_st` becomes a wrapper: it derives the four corners from its `Rect` in the order it already writes them (`:187-209`) and calls `push_quad_xy`. The run-merge tail (`:221-231`) moves into `push_quad_xy` unchanged. This is a pure refactor, and its binding expectation is byte-identity on every committed golden.

**`FrameStats` (`crates/mp/renderer-gpu/src/frame_exec.rs:98`) replaces one counter.** `skipped_rotate_pics` (`:113`) is deleted and `rotate_pics` takes its place:

```rust
/// Quads batched from `DrawRotatePic` and `DrawRotatePic2` events. A rotate pic whose stage 0 binds no image draws nothing and is not counted.
pub rotate_pics: u32,
```

`skipped_events()` (`:134-136`) drops the `skipped_rotate_pics` term. The field has no reader outside this file, so nothing else changes.

**`Warned` (`:142-158`) loses its `RotatePic` variant**, with the `slot()` indices (`:161-172`), the `describe()` arm (`:177`) and `COUNT` renumbered mechanically.

**Two private methods on `FrameExecutor`**, modelled on `draw_stretch_pic` (`:880-937`):

```rust
/// `RB_RotatePic`: one quad rotated `angle` degrees about its own top-right corner, textured from stage 0's bundle image and colored by the `RE_SetColor` register.
/// Returns the quads batched: one, or zero when stage 0 binds no image.
///
/// Source: `oracle/codemp/renderer/tr_backend.cpp:1498-1541`
fn draw_rotate_pic(
    &mut self,
    shader: ShaderHandle,
    rect: Rect,
    uv: UvRect,
    angle: f32,
    color: [f32; 4],
    assets: &RenderAssets,
    gpu_images: &GpuImages,
) -> u32

/// `RB_RotatePic2`: the same quad rotated about `rect.x`/`rect.y`, which is its center here, drawn with stage 0's own blend bits.
/// The pass gates on `num_unfogged_passes` before it reads stage 0, exactly as the oracle does.
///
/// Source: `oracle/codemp/renderer/tr_backend.cpp:1547-1602`
fn draw_rotate_pic2(
    &mut self,
    shader: ShaderHandle,
    rect: Rect,
    uv: UvRect,
    angle: f32,
    color: [f32; 4],
    assets: &RenderAssets,
    gpu_images: &GpuImages,
) -> u32
```

Both bodies:

1. `assets.shaders.get(shader)`. On `None`, warn `Warned::UnknownShader` and push the white quad at the rotated corners with `blend_state_from_gls(GLS_2D_DEFAULT)` and no image, returning 1. This mirrors `draw_stretch_pic`'s fallback (`:891-895`) and the oracle's `R_GetShaderByHandle` default-shader return.
2. `draw_rotate_pic2` only: return 0 when `asset.num_unfogged_passes == 0`.
3. `asset.stages.first()`. On `None`, return 0. This matches the oracle: its zero-pass shader carries one zero-filled stage whose NULL image skips the draw, so both trees draw nothing.
4. The image is the stage's `bundle[0].image` field read directly, filtered by `gpu_images.contains`, never through `stage_image`. The oracle indexes `image[0]` on a single pointer, so no animation frame and no video map is resolved here. `None` returns 0. A `≤2`-line rule-19 note at the site names the `animMap` case: the oracle binds the animation array's base as if it were an image, and this read draws nothing instead.
5. The corners come from `rotated_corners`, the texture coordinates from `uv.corners()`, the color from the `SetColor` register unchanged, and the image from step 4. `draw_rotate_pic` blends per open row 1; `draw_rotate_pic2` blends with `blend_state_from_gls(stage.state_bits)`.

**One free fn in `frame_exec.rs`:**

```rust
/// The four screen-space corners of a quad whose local corners `local` sit around `pivot`, rotated `angle` degrees.
/// The rotation runs in the 640x480 virtual space `qglRotatef` ran in, before the ortho transform, so the sense matches the oracle.
///
/// Source: `oracle/codemp/renderer/tr_backend.cpp:1516-1518`
fn rotated_corners(pivot: [f32; 2], local: [[f32; 2]; 4], angle: f32) -> [[f32; 2]; 4]
```

The body is `let (sin, cos) = angle.to_radians().sin_cos();` then `[pivot[0] + lx * cos - ly * sin, pivot[1] + lx * sin + ly * cos]` per corner. Every value is `f32`, matching `qglRotatef`'s float matrix.

**The two executor arms** replace the skip arm at `:527-530`:

- `FrameEvent::DrawRotatePic { .. }` calls `draw_rotate_pic` with `Rect { x, y, w, h }`, `UvRect { s1, t1, s2, t2 }`, `angle`, and the current `color`; `stats.rotate_pics += drawn`.
- `FrameEvent::DrawRotatePic2 { .. }` calls `draw_rotate_pic2` with the same shape.

**The new test file `crates/mp/renderer-gpu/tests/hud_golden.rs`**, with the two goldens `tests/goldens/hud_2d.png` and `tests/goldens/hud_font_ocr_a.png`. Each renderer test file in this crate carries its own copy of `golden_path`, `actual_path`, `write_png`, `read_png`, `compare` and `coverage`; this file follows that, because a Rust integration test is its own binary and the crate shares no test helper module.

- **`golden_hud_2d`** - no `#[ignore]`, so `cargo test --workspace` runs it. It takes `scene_golden.rs`'s GPU-skip idiom (`Gpu::try_new_headless` returning `None` prints a skip line and returns, `scene_golden.rs:365-369`) and its synthetic boot: the same temp tree with `mpdefault.cfg`, `productid.txt` and one `.shader` script, written through the atomic-rename helper (`scene_golden.rs:188-253`), then `boot::boot_renderer` and `registered = true`. The scene is 2D only, with no `RE_RenderScene` call at all, recorded through the real frontends:
  - `RE_SetColor` white, then `RE_StretchPic(64, 64, 128, 128, 0, 0, 1, 1, pic_shader)` as the unrotated reference.
  - `RE_SetColor([1.0, 0.5, 0.25, 1.0])`, then `RE_RotatePic(256, 64, 128, 128, 0, 0, 1, 1, 45.0, rot_shader)`.
  - `RE_SetColor([0.25, 0.5, 1.0, 1.0])`, then `RE_RotatePic2(160, 320, 128, 128, 0, 0, 1, 1, 30.0, rot_shader)`.

  `pic_shader` registers the script's own name, and `rot_shader` registers a name the script does not define, so it resolves to the default shader and the rotate pics carry the procedural checkerboard `tr.defaultImage`. A checkerboard shows the rotation; a flat white square does not. Asserts before any bless: `stats.rotate_pics == 2`, `stats.quads > 0`, `stats.draw_calls > 0`, and `coverage(&actual) > 0`. `CHANNEL_TOLERANCE` is 0.

- **`golden_hud_font`** - `#[ignore = "needs retail assets and a GPU; run locally with --ignored"]`, the retail idiom of `world_golden.rs:420`. It boots `BootConfig::default()` with the `JKA_BASEPATH` override (`world_golden.rs:235-238`), uses `Gpu::new_headless`, registers `ocr_a` through `RE_RegisterFont` (`crates/mp/renderer/src/tr_font.rs:1936-1949`; `ocr_a` is the cgame small font, `crates/mp/cgame/src/cg_main.rs:5267`), and draws one string through `RE_Font_DrawString` (`tr_font.rs:3051-3072`) at a frozen clock. The string is `"jka-rust ^1font ^7golden"` at `(32, 64)`, scale `1.0`, `iMaxPixelWidth` `-1`, color white, so the color-code path runs inside the layout. Asserts before any bless: the font handle is nonzero, `stats.quads > 0`, `stats.draw_calls > 0`, and `coverage(&actual) > 0`.

Anything not on this list is out of scope, and the agent must not add it. No new third-party crate (a DEC-49-class dependency ruling comes from the user only). No change to `tr_cmds.rs`, `stage2d.rs`, `blend.rs`, `pipeline2d.wgsl`, `frame_event.rs`, or any 3D path. No new `Warned` variant, no new cvar, no new `FrameEvent` variant, no ABI change. No edit to `world_golden.rs`, `scene_golden.rs`, `entity_golden.rs`, `ghoul2_vertex_golden.rs`, or any committed fixture: every existing golden is read-only, and the two new PNGs are the only fixtures this step may create.

## Divergence notes, each ≤2 lines at its site

- **The 2D color register stays float.** `RB_SetColor` truncates to bytes (`tr_backend.cpp:1409-1412`); this backend keeps the `SetColor` register in floats, as the stretch-pic arm already does. The step introduces no new quantization and no new divergence here.
- **The rotate-pic interleave, the one accepted divergence.** The oracle draws a rotate pic immediately, ahead of any pending `tess` batch; this backend appends it in command order. The site note names the concrete face: the disruptor zoom draws its mask as a stretch pic and its insert as a rotate pic (`oracle/codemp/cgame/cg_draw.c:349-350,365`), so the oracle flushes the mask over the insert and this backend layers the insert over the mask.

**Parity, not a divergence: the zero-stage shader.** `GeneratePermanentShader` allocates one zero-filled stage even at zero passes (`oracle/codemp/renderer/tr_shader.cpp:2782-2783`), and its NULL image makes the oracle skip the draw. The Rust `stages.first()` read draws nothing for the same input, so the two trees agree and the code plan is unchanged.

## Open rows

The 2026-08-07 ratification walk closed every row. Rows 1, 3, 4 and 7 are ratified as proposed, row 8 is ratified as amended, and the mechanical rows stand. Each row below keeps its text as the lane's instruction.

**Row 1 - `RB_RotatePic`'s blend state (user ruling, ratified as proposed).** The oracle issues no `GL_State` for this arm, so it inherits the last one issued: `GLS_2D_DEFAULT` right after `RB_SetGL2D`, or the last flushed stretch-pic stage's bits otherwise. Two options. **Proposed default: draw every `DrawRotatePic` with `blend_state_from_gls(GLS_2D_DEFAULT)` and record the inheritance in a two-line note at the site.** That is the state `RB_SetGL2D` installs, it is defined, and it needs no new machinery. The alternative is a frame-local `state_2d` register beside `color` in `execute_frame`, updated by each stretch-pic stage push and reset to `GLS_2D_DEFAULT` after each `RenderScene`. The register would be closer but still not exact, because the oracle's state comes from the last *flushed* shader run and this backend has no flush point, so it would trade a named divergence for an unnamed approximation. `DrawRotatePic2` is unaffected either way: it installs stage 0's bits explicitly.

**Row 2 - the golden file and its name (mechanical).** Proposed default: one new file `crates/mp/renderer-gpu/tests/hud_golden.rs` holding both tests, with goldens `hud_2d.png` and `hud_font_ocr_a.png`. Both tests share the boot and compare helpers, and their different asset gates live on the tests, not on separate files.

**Row 3 - the retail font golden's asset gate (user ruling, ratified as proposed).** The workspace has two idioms. The renderer-gpu golden family uses `#[ignore]` plus a hard assert on load failure (`world_golden.rs:243,420`), and the cgame/jampgame tests self-skip with an `eprintln!("SKIP: ...")` and an early return (`crates/jampgame/tests/referee.rs:905-912`, `crates/cgame/tests/replay_referee.rs:181-191`). **Proposed default: idiom A, `#[ignore = "needs retail assets and a GPU; run locally with --ignored"]` plus a hard assert if the font fails to register**, matching every other retail golden in this crate. A self-skip inside an already-ignored test hides a real font-path break behind a green run.

**Row 4 - the bless flow for the two new PNGs (user ruling, ratified as proposed).** **Proposed default: the step-007 procedure, once per golden.** Run the test with `JKA_GOLDEN_BLESS=1` to write the PNG, re-run without it to confirm the byte-identical pass, then STOP before the commit that adds the PNG so the user looks at the image. `hud_2d.png` must show one axis-aligned reference quad plus two visibly rotated quads at different angles. `R_CreateDefaultImage` builds the rotated quads' image as a white-bordered box with a dark alpha-32 interior, so the border edges are what show the rotation. A picture with three axis-aligned quads is a defect, not a blessable golden. `hud_font_ocr_a.png` must show legible glyphs with the `^1` run in red; a blank or single-color image is a defect.

**Row 5 - the font, the string and the placement (mechanical).** Proposed default: `ocr_a`, the string `"jka-rust ^1font ^7golden"` at `(32, 64)`, scale `1.0`, `iMaxPixelWidth` `-1`, color white, at the frozen clock the other goldens use.

**Row 6 - the stat counter (mechanical).** Proposed default: `skipped_rotate_pics` is replaced by `rotate_pics`, counting quads batched, and `Warned::RotatePic` is deleted. The field has no reader outside `frame_exec.rs`.

**Row 7 - the unknown-shader fallback (mechanical, ratified as proposed).** Both rotate arms mirror `draw_stretch_pic` - warn `Warned::UnknownShader` once and push a white rotated quad at `GLS_2D_DEFAULT`. The oracle's `R_GetShaderByHandle` would return the checkerboard default shader instead; the white quad is this backend's accepted stretch-pic idiom, and this step keeps it rather than opening a second shape.

**Row 8 - the stage-0 image read (mechanical, ratified as amended).** Read `bundle[0].image` directly, never `stage_image`. The oracle indexes `image[0]` on a single pointer, so it resolves no animation frame and no video map. The amended rationale: for an `animMap` shader that pointer is the base of an `image_t *` array (`oracle/codemp/renderer/tr_shader.cpp:1441-1442`), so the oracle binds type-confused garbage there, and this read draws nothing instead. A `≤2`-line rule-19 note at the site names the `animMap` case.

## Pause triggers, named for this step

- Any committed golden moves - the three world goldens, the seven scene goldens, `entity_duel1.png`, or `ghoul2_verts_stormtrooper.bin`. STOP: the `push_quad_st` refactor is a pure refactor and the rotate arms touch no existing draw.
- The synthetic golden's two rotate pics look axis-aligned at bless time. STOP: the rotation sense or the pivot is wrong, and the user decides which reading is right.
- `RE_RegisterFont("ocr_a")` returns zero, or the font string draws no quad. STOP: that is a retail font-path break outside this step's scope, and the user rules on it.
- A rotate pic seems to need a `stage2d` evaluation (a `tcMod`, an `rgbGen`, an animation frame) to look right. STOP: the oracle runs none of them here, so the need means the reading is wrong.
- Row 1's alternative looks necessary mid-lane. STOP: the blend choice is a user ruling and lands as an Amendment.
- Verification is `cargo build` or `cargo check` plus the golden suites, never rust-analyzer, which is stale in this workspace.

## Commit bundle

The full gate battery, named once and referenced per commit:

- `cargo build --workspace`, zero warnings.
- `cargo test --workspace -- --test-threads=1`.
- `cargo test -p mp_renderer_gpu --test world_golden -- --ignored --test-threads=1`, all four world goldens byte-identical (`world_duel1`, `world_ffa2`, `world_marks_duel1`, `world_dlights_duel1`).
- `cargo test -p mp_renderer_gpu --test scene_golden -- --test-threads=1`, all seven scene goldens byte-identical.
- `cargo test -p mp_renderer_gpu --test entity_golden -- --ignored --test-threads=1`, byte-identical.
- `cargo test -p mp_renderer_gpu --test ghoul2_vertex_golden -- --ignored --test-threads=1`, byte-identical.

Every golden run is serial with `--test-threads=1`, each as one foreground command with a long timeout: two engine boots in parallel threads crash in the GPU init path, and the world-golden pk3 inflate aborts without it.

1. **The rotate-pic arms.** `push_quad_xy` with the `push_quad_st` refactor, `rotated_corners`, `draw_rotate_pic`, `draw_rotate_pic2`, the two executor arms, the `rotate_pics` counter, and the `Warned::RotatePic` removal. Subject: `feat(gh#31 s008): the rotate-pic executor arms`. Gates: the full battery. The refactor touches every 2D quad in the workspace, so byte-identity on all thirteen existing fixtures is the proof.
2. **The synthetic 2D golden.** `hud_golden.rs` with `golden_hud_2d`, plus `tests/goldens/hud_2d.png` after the row-4 STOP. Subject: `test(gh#31 s008): the synthetic 2D golden`. Gates: the full battery, plus `cargo test -p mp_renderer_gpu --test hud_golden -- --test-threads=1`.
3. **The retail font-string golden.** `golden_hud_font` plus `tests/goldens/hud_font_ocr_a.png` after its own row-4 STOP. Subject: `test(gh#31 s008): the retail font-string golden`. Gates: the full battery, plus `cargo test -p mp_renderer_gpu --test hud_golden -- --ignored --test-threads=1`.
4. **The finished file**, per the packet skill: assumptions and choices keyed to their commits, deviations or the word "none", the commit list with gate results, and open gaps. Subject: `process(gh#31 s008): finished file`.

Every commit uses `--no-gpg-sign`, a heading subject, an STE body, and no trailer of any kind: no `Co-Authored-By`, no generated-with footer. Gate results are written as plain sentences inside the body, so no line parses as a git trailer (the step-005 lesson). The lockstep referee is not required: no commit touches `mp_game`, the server, or any `jampded` link-set crate.

## Write scopes

Branch `gh31-step-008-2d-closure`, cut from master. A worktree builder runs `git merge master --no-gpg-sign` as its first act.

- `crates/mp/renderer-gpu/src/frame_exec.rs` - the two arms, the two methods, `rotated_corners`, the stat, the `Warned` variant.
- `crates/mp/renderer-gpu/src/pipeline2d.rs` - `push_quad_xy` and the `push_quad_st` refactor.
- `crates/mp/renderer-gpu/tests/hud_golden.rs` - new.
- `crates/mp/renderer-gpu/tests/goldens/hud_2d.png` and `hud_font_ocr_a.png` - new, blessed under the row-4 STOP.
- Any caller `cargo check` shows broken by the stat rename, edit-only to pass the new shape.
- `.claude/packets/31/step-008/` for `finished.md`.

Everything else is read-only, including `oracle/`, `crates/mp/renderer/src/tr_cmds.rs`, `stage2d.rs`, `blend.rs`, `shaders/pipeline2d.wgsl`, every existing test file under `crates/mp/renderer-gpu/tests/`, every committed fixture under `tests/goldens/`, and `~/Developer/jka/` beyond read-only asset reads. Source files change through the Edit tool only.

## Disposition

After a clean lane-review: merge to master locally. No push, and no pull request.

## Amendments

**2026-08-07 - the draft awaits the ratification walk.** Rows 1, 3 and 4 need user rulings; rows 2, 5, 6, 7 and 8 are mechanical with the defaults above.

**2026-08-07 - the ratification walk closed all six open rows.** The rulings are folded into the body above.

- Row 1, the `RB_RotatePic` blend state: ratified as proposed. Draw at `GLS_2D_DEFAULT` with the two-line note.
- Row 3, the retail font asset gate: ratified as proposed. `#[ignore]` plus a hard assert.
- Row 4, the bless flow: ratified as proposed. The step-007 procedure per golden, a STOP before each PNG commit, and named defect conditions for both images.
- Row 8, the stage-0 image read: ratified as amended. The direct `bundle[0].image` read stands. The corrected rationale is the `animMap` case: the oracle reads type-confused garbage (`tr_shader.cpp:1441-1442`), and this read maps that to draw-nothing under a `≤2`-line rule-19 note at the site.
- Divergence note 1, the zero-stage shader: ratified as amended. It leaves the divergence list. The oracle allocates at least one zero-filled stage (`tr_shader.cpp:2782-2783`) whose NULL image skips the draw, so draw-nothing is parity in both trees. The code plan is unchanged.
- Divergence note 3, the command-order interleave: ratified with the auditor's naming. Command-order append stands as the one accepted divergence, and the site note names the disruptor zoom layering flip (`cg_draw.c:349-350,365`) as its concrete face.

Mechanical folds from the audit: the gate battery counts four world goldens, not three. Two cosmetic cite drifts are fixed: `RB_SetGL2D` at `:1282-1284` and the `RB_RotatePic2` tail at `:1600-1602`. The cgame-site description now names the radar icons, the disruptor zoom insert and its ticks, and the rocket-lock wedge. Row 7 keeps the white-quad fallback, with the default-shader difference recorded as this backend's accepted stretch-pic idiom.

**2026-08-07 - the vet-disposition walk closed all seven findings.** The lane-review vet returned seven findings (`vet.md`), and the walk ratified all seven.

1. The commit `ccea53c9` `docs: README refreshed to the house style` is a session-caused rider: the session's parallel README rewriter committed while the shared checkout sat on the lane branch. This amendment authorizes it after the fact. It touches `README.md` only and rides to master with the lane merge.
2. The two added error strings in `hud_golden.rs` with semicolons: one fix round rewrites them to the STE form. The sibling files stay untouched.
3. The added module-doc sentence and boot comment with semicolons: the same fix round rewrites them. The semicolon as a multi-cite `Source:` separator stays, because a cite line is a reference list, not prose.
4. The column-wrapped added comments in `hud_golden.rs`: the fix round unwraps them to one sentence per line and splits the `rotate_pics` doc onto two lines.
5. "seam" stands as the repo's domain term, exempt from the pet-vocabulary ban.
6. The rule-19 `animMap` site note: the fix round completes it with the ratified second half, "and this read draws nothing instead".
7. Row 4's bless criterion misnamed the default image. `R_CreateDefaultImage` builds a white-bordered box with a dark alpha-32 interior, not a checkerboard. The row-4 text is corrected in place. The blessed `hud_2d.png` meets the rotation-visibility intent, so no code change and no re-bless.

The fix round covers findings 2, 3, 4 and 6 in one commit on the lane branch. The vet walks the fix commit, and a clean walk proceeds to the packet's disposition.
