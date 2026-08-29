# Packet gh#31 step-009 - the FX mini-refent backend arms

## Scope

This step lights the three generated entity kinds the FX module submits engine-side: `RT_ORIENTED_QUAD`, `RT_CYLINDER`, and `RT_ELECTRICITY`. It also closes the one render-side random draw DEC-66 named but left at a stand-in, `RB_SurfaceSaberGlow`'s hilt radius.

The trap census never saw these three kinds. It counts what a cgame trace submits through `CG_R_ADDREFENTITYTOSCENE`, and the FX system builds its own `refEntity_t` inside the engine and submits it from `FxHost::AddFxToScene`. The three submitters are `CElectricity::Draw` (`crates/mp/engine/client/src/fx/celectricity.rs:81-84`), `COrientedParticle::Draw` (`crates/mp/engine/client/src/fx/coriented_particle.rs:86-95`), and `CCylinder::Draw` (`crates/mp/engine/client/src/fx/ccylinder.rs:123-128`), all reached from `FX_AddElectricity`, `FX_AddOrientedParticle`, and `FX_AddCylinder` (`crates/mp/engine/client/src/fx/fx_util.rs:494-589`, `:1024-1129`, `:715-836`). The census does record the trap that starts the chain: `fx/AddElectricity` at 87 calls and `fx/AddPrimitive` at 32,656 (`docs/plans/2026-07-24-client-port/scene-trap-census.md:145-148`). Each of those primitives can become one of these mini refents.

The backend is where the chain stops. `build_entity_geometry` (`crates/mp/renderer-gpu/src/pipeline3d.rs:4551-4653`) builds `RT_SPRITE`, `RT_LINE`, and `RT_SABER_GLOW`, and its wildcard arm returns empty for everything else. `collect_entity_surface` counts an empty index block as a skip and warns once (`:1762-1766`). So every FX quad, cylinder, and lightning bolt draws nothing on the live client today.

The module doc above `build_entity_geometry` states the wrong reason for that. It reads `RT_BEAM`, `RT_ORIENTED_QUAD`, `RT_ORIENTEDLINE`, `RT_ELECTRICITY`, and `RT_CYLINDER` are census-complement fog (`:4536-4538`). Three of those five are not complement at all. They are engine-side submissions the census tool could not observe. This step corrects that sentence and leaves `RT_BEAM` and `RT_ORIENTEDLINE` in the fog list where they belong.

The step also gives the three kinds their first goldens, and it re-blesses `scene_saber_glow.png` once the hilt radius becomes a real draw.

The step does not touch the FX module. Every field the three arms read is already filled by the submitters listed above. It does not touch the frontend, `frame_exec.rs`, the world path, the 2D path, or any shader stage evaluator. It adds no cvar, no `FrameEvent` variant, and no ABI surface.

**Carried from step-008, and relevant here.** The per-surface vertex cap against the oracle's `tess` batching is a standing backend architecture fact. It bites harder in this step than in any earlier one, because one electricity bolt emits three tapered quads per `ApplyShape` call and one `ApplyShape` call per 20 units of bolt length. A long bolt is the first CPU-built surface in this backend that can approach the cap, and the oracle answers overflow by flushing the batch (`RB_CheckOverflow`, `oracle/codemp/renderer/tr_surface.cpp:30-52`) where this backend has no flush point. Open row 7 carries the instruction.

## The oracle, cited

### `RB_SurfaceOrientedQuad` (`oracle/codemp/renderer/tr_surface.cpp:177-220`)

The simplest of the three. It copies `left` from `e->axis[1]` and `up` from `e->axis[2]`, scales both by `e->radius` when `e->rotation` is zero, and otherwise rotates the pair by `rotation` degrees through the same sine and cosine shape the sprite arm already runs. The `MakeNormalVectors` call the SP tree uses is commented out here (`:184`), and the MP tree reads the two axis rows directly. It flips `left` under `backEnd.viewParms.isMirror` and emits one `RB_AddQuadStamp` (`:219`), which is `RB_AddQuadStampExt` with the full texture rectangle (`:132-134`). It draws no random numbers.

The rotation branch uses two temporaries and the oracle's own comment explains why (`:203`, `:208`). `tempUp` is the base of the second `VectorMA`, and `left` is still the unrotated copy at that point, so the order of the three writes is load-bearing.

SP differs: `code/renderer/tr_surface.cpp:185` derives both vectors from `axis[0]` through `MakeNormalVectors`. Port the MP body (porting-rules §20).

### `RB_SurfaceCylinder` and `DoCylinderPart` (`:853-953` and `:818-847`)

`e->origin` is the bottom point, `e->oldorigin` the top, `e->radius` the bottom ring radius, and `e->rotation` the top ring radius (`:892-893`, and the oracle's own header comment at `:849-851`). `NUM_CYLINDER_SEGMENTS` is 32 (`:815`).

The LOD block (`:865-887`) is already transcribed in the dead `mp_renderer` twin and reads the same way here: average the two endpoints, take the distance to `backEnd.viewParms.ori.origin`, scale by `fovX / 90.0f`, and derive `segments` from `1 - length / 1024`, clamped into 8..=32. The two ring loops call `RotatePointAroundVector` around `e->axis[0]` at `360.0f / segments` degree steps and translate each ring to its own endpoint (`:898-907`). The texture-coordinate loop then builds four `polyVert_t` per segment, wrapping the last segment back to index 0 (`:912-951`), and hands each quad to `DoCylinderPart`.

`DoCylinderPart` emits four vertices and six indices in the order `vbase, vbase+1, vbase+2, vbase+2, vbase+3, vbase`. Note that this winding differs from `DoLine`'s (`vbase, vbase+1, vbase+2, vbase+2, vbase+1, vbase+3`). Transcribe each one as written.

The oracle's `lower_points`, `upper_points`, and `verts` are function-local `static` arrays (`:855`). Every element is written before it is read on each call, and the read loop is bounded by the freshly computed `segments`, so they are rotating per-call scratch and become owned locals. They are not state.

It draws no random numbers. SP differs on almost every constant (`NUM_CYLINDER_SEGMENTS` 40, divisor 2048, `backlerp` as the second radius, a cone early-out). Port the MP body.

### The electricity chain

**`RB_SurfaceElectricity` (`:1127-1169`).** It reads `e->radius`, copies `e->origin` into `start`, and takes the normalized direction and distance to `e->oldorigin`. Under `RF_GROW` it computes `perc` from `(e->axis[0][2] - tr.refdef.time) / e->axis[0][1]` and clamps it into 0..=1 (`:1145-1157`). Those two axis slots are not axis components. The oracle's own inline comments name them `endTime` and `duration`, and the FX submitter fills them that way (`crates/mp/engine/client/src/fx/celectricity.rs:57-58,83-84`). The real `refEntity_t::endTime` field is never read here. It then writes the grown endpoint back into `e->oldorigin` (`:1159`), reads it straight back as `end`, builds the side vector as the normalized cross product of the two eye-to-endpoint vectors, and calls `DoBoltSeg` once.

**`DoBoltSeg` (`:1039-1124`).** It normalizes `start` to `end`, builds `rt` and `up` through `MakeNormalVectors`, seeds a running offset `off` at `{10,10,10}`, and steps the segment in 20-unit chunks. Each iteration adds a random deviation to `off`, interpolates the current point, optionally tapers the radius under `RF_TAPERED`, and calls `ApplyShape` at `LIGHTNING_RECURSION_LEVEL`. The MP tree has no `dis` clamp, unlike SP's 2000-unit cap.

**`ApplyShape` (`:990-1036`).** At `count < 1` it emits one tapered quad through `DoLine2` and returns. Otherwise it calls `CreateShape`, computes two interpolated points offset by the shape vectors, and recurses three times at `count - 1`. `LIGHTNING_RECURSION_LEVEL` is 1 (`:958`), so every recursion lands in the base case and one `ApplyShape` call produces exactly one `CreateShape` and three `DoLine2` quads.

**`CreateShape` (`:976-987`).** Six `crandom()` draws in source order: `sh1[0]`, `sh1[1]`, `sh1[2]`, then `sh2[0]`, `sh2[1]`, `sh2[2]`. `VectorSet` evaluates each argument once (`oracle/codemp/game/q_shared.h:1399`), so there is no multi-eval here. `sh2[1]` and `sh2[2]` read `sh1[1]` and `sh1[2]` back, which is why the two vectors share one home.

**`DoLine2` (`:658-710`).** The same shape as `DoLine`, with an independent span width for the `end` pair. Colors come from `backEnd.currentEntity->e.shaderRGBA` per vertex, texture coordinates are the fixed 0/1 square, and the index order is `vbase, vbase+1, vbase+2, vbase+2, vbase+1, vbase+3`.

### The two random families, and which owner each one gets

They are different generators and DEC-66 rules them separately.

**The seeded family.** `Q_crandom(&e->frame)` and `Q_random(&e->frame)` (`oracle/codemp/game/q_math.c:126-137`) run `*seed = 69069 * *seed + 1` and mutate `e->frame` in place. Every call in `DoBoltSeg` uses this family. DEC-66 ruling 2 rules the shape: hoist `e.frame` into a local `mut seed: c_int` and thread `&mut seed` through the recursion. The commented-out copies of these three functions at `oracle/codemp/renderer/tr_surface.cpp:961-973` are dead. The live definitions are the `q_math.c` ones, already ported as `native_math::qmath::{Q_rand, Q_random, Q_crandom}` with a `*mut c_int` seed parameter.

**The process stream.** `crandom()` and `random()` (`oracle/codemp/game/q_shared.h:1591-1592`) wrap the C runtime `rand()`. `CreateShape` uses this family, and so does `RB_SurfaceSaberGlow`'s hilt radius (`oracle/codemp/renderer/tr_surface.cpp:579`). DEC-66 ruling 1 gives it the backend's own persistent `Rng`, seeded `Rng::new()` at construction, and ruling 3 accepts the split from retail's one process-wide stream as a divergence.

### The macro multi-eval trap, at its exact sites

`VectorScale` and `VectorMA` are macros in this build. The `#if 1` at `oracle/codemp/game/q_shared.h:1352` selects the macro forms, not the `_VectorScale` and `_VectorMA` function forms of the `#else` branch:

```c
#define	VectorScale(v, s, o)			((o)[0]=(v)[0]*(s),(o)[1]=(v)[1]*(s),(o)[2]=(v)[2]*(s))
#define	VectorMA(v, s, b, o)			((o)[0]=(v)[0]+(b)[0]*(s),(o)[1]=(v)[1]+(b)[1]*(s),(o)[2]=(v)[2]+(b)[2]*(s))
```

The `s` argument appears three times in each body. Three lines in `DoBoltSeg` pass a random draw as `s`:

```c
VectorScale( fwd, Q_crandom(&e->frame) * 3.0f, temp );				// :1075
VectorMA( temp, Q_crandom(&e->frame) * 7.0f * e->axis[0][0], rt, temp );	// :1076
VectorMA( temp, Q_crandom(&e->frame) * 7.0f * e->axis[0][0], up, temp );	// :1077
```

Each line draws three values, one per component, and each component gets a different scale. Nine draws per loop iteration from these three lines. A transcription that hoists the call into a local changes both the stream length and the geometry, and it is the single highest-risk point in this step. Write each component as its own statement with its own `Q_crandom(&mut seed)` call.

Two more draw sites in the same function, neither a macro. The fork guard at `:1100` draws once, and only when `(e->renderfx & RF_FORKED) && f_count > 0` already holds, because C short-circuits `&&`. The fork offset loop at `:1111-1114` draws three times.

### Float widths

The seeded family returns `float` in C and `f32` in Rust (`crates/native/math/src/qmath.rs:289`). Every expression on that path stays `f32`. `Q_crandom(&mut seed) * 3.0` in `f32` is the faithful transcription, and no widening belongs there.

The process stream is the opposite. `crandom()` is `2.0 * (random() - 0.5)` with double literals, so the C expression is `double` until it narrows at the store, and `Rng::crandom` returns `f64` to match. The six `CreateShape` expressions are the only place in this step where the widen-the-constant rule applies:

```rust
shape.sh1[0] = (0.66f32 as f64 + rng.crandom() * 0.1f32 as f64) as f32;
```

Never a bare `f64` literal. `random()` is `float` in C and `f32` in Rust, so the saber-glow hilt radius `5.5f + random() * 0.25f` stays entirely in `f32`.

### What the oracle leaves dead or unclear

**`f_count` is never assigned.** `static float f_count;` (`:956`) zero-initializes and the only write in the whole MP tree is the decrement at `:1104`. A grep over `oracle/codemp/` returns exactly three hits, the declaration, the guard, and the decrement. So `f_count > 0` is false forever and the `RF_FORKED` tendril branch never executes in MP. SP sets `f_count = 3` right before its `DoBoltSeg` call (`oracle/code/renderer/tr_surface.cpp:844`), which is what makes the branch live there. Porting-rules §20 says duplicate, do not unify: transcribe the MP branch as written and let it stay dead. Open row 5 carries the note.

**`RB_SurfaceElectricity` writes its own input entity.** The `oldorigin` write at `:1159` mutates `backEnd.currentEntity->e`, which points into the per-frame entity array. Open row 6 rules the disposition.

**A zero `RF_GROW` duration divides by zero.** `e->axis[0][1]` is the divisor at `:1147`, and nothing guards it. The result is an infinity or a NaN, the two clamps catch the infinity and let the NaN through, and the bolt geometry becomes NaN. Rust float arithmetic does the same thing for the same input, so this is parity and needs no rule-19 note.

**`e->frame` is an overloaded field.** Its declared meaning is the MODEL_BEAM diameter (`oracle/codemp/cgame/tr_types.h:163`). Electricity reuses it as LCG state, and the FX submitter fills it that way, `frame = (frame_draw * 1265536.0) as i32` (`crates/mp/engine/client/src/fx/celectricity.rs:55`).

## Surface contract

### `crates/mp/renderer/src/tr_surface.rs`

`TrSurfaceShapeState` (`:50-53`) gains the fork counter and a derived constructor. The two existing fields and the type's `Source:` cite are unchanged, and its doc text is rewritten to name the backend as the owner:

```rust
/// Raven's `sh1`, `sh2` and `f_count` file statics, the electricity chain's shared home.
/// The render-side backend owns one instance beside its own frame-to-frame state, per DEC-66 ruling 1.
///
/// Source: `oracle/codemp/renderer/tr_surface.cpp:955-956`
#[derive(Default)]
pub struct TrSurfaceShapeState {
    pub sh1: vec3_t,
    pub sh2: vec3_t,
    pub f_count: f32,
}
```

One new const beside it:

```rust
/// Raven `LIGHTNING_RECURSION_LEVEL`, the `ApplyShape` recursion depth `DoBoltSeg` passes.
/// At 1 every recursive call lands in the base case, so one `ApplyShape` call emits three tapered quads.
///
/// Source: `oracle/codemp/renderer/tr_surface.cpp:958`
pub const LIGHTNING_RECURSION_LEVEL: c_int = 1;
```

Nothing else in this file changes except comment text. The deferred note at `:1071-1100` loses the sentence DEC-66 ruling 2 overrules, and the three deferred blocks for `RB_SurfaceElectricity`, `RB_SurfaceOrientedQuad` and `RB_SurfaceCylinder` each gain one line naming `build_entity_geometry` as the live arm. No body in this file is ported, and no `todo!()` in it is removed.

### `crates/mp/renderer-gpu/src/pipeline3d.rs`

`Pipeline3d` (`:907-954`) gains two private fields, built in `Pipeline3d::new`:

```rust
    /// The render-side C runtime stream, per DEC-66 ruling 1.
    /// It persists across frames, because a per-frame reset would replay the same jitter every frame and freeze the bolt shimmer.
    rng: Rng,

    /// Raven's `sh1`, `sh2` and `f_count` file statics, per DEC-66 ruling 1.
    shape: TrSurfaceShapeState,
```

`build_entity_geometry` (`:4551`) gains three parameters:

```rust
fn build_entity_geometry(
    e: &refEntity_t,
    view: &viewParms_t,
    refdef_time: i32,
    rng: &mut Rng,
    shape: &mut TrSurfaceShapeState,
) -> (Vec<WorldVertex>, Vec<u32>)
```

Its one call site is `collect_entity_surface` (`:1761`), which already carries `refdef_time: i32` as a parameter (`:1747`) and can pass `&mut self.rng` and `&mut self.shape` before its `self.warn_once` call. The module doc drops `RT_ORIENTED_QUAD`, `RT_ELECTRICITY` and `RT_CYLINDER` from the fog sentence, keeps `RT_BEAM` and `RT_ORIENTEDLINE` there, states that the FX module submits the three engine-side, removes the three matching `//TODO: Port` markers, and corrects the two stale cites that stay: `RB_SurfaceBeam` at `:478-528` and `RB_SurfaceOrientedLine` at `:792-807`.

Four new private free functions, placed beside `do_line` (`:4499`):

```rust
/// Raven `DoLine2` - one tapered quad from `start` to `end`, `span_width` wide at the start edge and `span_width2` at the end edge.
///
/// Source: `oracle/codemp/renderer/tr_surface.cpp:658-710`
#[allow(clippy::too_many_arguments)]
fn do_line2(
    verts: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
    start: vec3_t,
    end: vec3_t,
    up: vec3_t,
    span_width: f32,
    span_width2: f32,
    color: [u8; 4],
)

/// Raven `DoCylinderPart` - one four-corner ring segment, emitted with the cylinder's own index winding.
///
/// Source: `oracle/codemp/renderer/tr_surface.cpp:818-847`
fn do_cylinder_part(verts: &mut Vec<WorldVertex>, indices: &mut Vec<u32>, quad: &[polyVert_t; 4])

/// Raven `CreateShape` - redraws the two fractal offset vectors from the backend stream.
/// The six draws run in source order, and `sh2` reads `sh1` back, so the two vectors share one home.
///
/// Source: `oracle/codemp/renderer/tr_surface.cpp:976-987`
fn create_shape(shape: &mut TrSurfaceShapeState, rng: &mut Rng)

/// Raven `ApplyShape` - splits one bolt segment into three jagged sub-segments, or emits it as a tapered quad at the base case.
///
/// Source: `oracle/codemp/renderer/tr_surface.cpp:990-1036`
#[allow(clippy::too_many_arguments)]
fn apply_shape(
    verts: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
    start: vec3_t,
    end: vec3_t,
    right: vec3_t,
    sradius: f32,
    eradius: f32,
    count: c_int,
    color: [u8; 4],
    shape: &mut TrSurfaceShapeState,
    rng: &mut Rng,
)

/// Raven `DoBoltSeg` - steps `start` to `end` in 20-unit chunks, jitters each step off the entity's own seed, and shapes each chunk.
/// `seed` is `e.frame` hoisted into a local, per DEC-66 ruling 2, because the oracle's seed write never outlives one draw chain.
///
/// Source: `oracle/codemp/renderer/tr_surface.cpp:1039-1124`
#[allow(clippy::too_many_arguments)]
fn do_bolt_seg(
    verts: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
    e: &refEntity_t,
    seed: &mut c_int,
    start: vec3_t,
    end: vec3_t,
    right: vec3_t,
    radius: f32,
    color: [u8; 4],
    shape: &mut TrSurfaceShapeState,
    rng: &mut Rng,
)
```

Three new arms inside `build_entity_geometry`'s match, each written in the file's established shape: build into the two local vectors and cite the oracle at the arm.

- `refEntityType_t::RT_ORIENTED_QUAD` transcribes `:177-220` and closes with `add_quad_stamp_ext(..., 0.0, 0.0, 1.0, 1.0)`, which is what `RB_AddQuadStamp` expands to.
- `refEntityType_t::RT_CYLINDER` transcribes `:853-953` with owned `[vec3_t; 32]` ring arrays and one owned `[polyVert_t; 4]` quad.
- `refEntityType_t::RT_ELECTRICITY` transcribes `:1127-1169` and calls `do_bolt_seg` once.

One change inside the existing `RT_SABER_GLOW` arm: the hilt sprite radius becomes `5.5 + rng.random() * 0.25` (`:579`), and the `//TODO: Port RB_SurfaceSaberGlow's random hilt radius` marker at `:4639-4645` is deleted with its stand-in note.

New imports, merged into the file's existing use groups: `Q_random` and `RotatePointAroundVector` join the `mp_qshared::shared::q_math` group, `MakeNormalVectors` and `Q_crandom` come from `native_math::qmath` (they are not re-exported through `mp_qshared`), `Rng` from `native_math::rng`, `TrSurfaceShapeState` and `LIGHTNING_RECURSION_LEVEL` from `mp_renderer::tr_surface`, `RF_FORKED`, `RF_GROW` and `RF_TAPERED` from the group at `:44-47` if they are absent there, and `c_int` from `core::ffi`. No `use` inside a function body, and no inline fully qualified path in an expression.

### `crates/mp/renderer-gpu/tests/scene_golden.rs`

Three new scene builders and three new tests, each in the shape of `scene_sprites` and `golden_scene_sprites` (`:514-534`, `:752-760`):

```rust
fn scene_fx_oriented_quad(host: &mut UiHost, frame_data: &mut FrameData, shader: qhandle_t)
fn scene_fx_cylinder(host: &mut UiHost, frame_data: &mut FrameData, shader: qhandle_t)
fn scene_fx_electricity(host: &mut UiHost, frame_data: &mut FrameData, shader: qhandle_t)

#[test] fn golden_scene_fx_oriented_quad()
#[test] fn golden_scene_fx_cylinder()
#[test] fn golden_scene_fx_electricity()
```

None carries `#[ignore]`, matching every sibling in this file, so `cargo test --workspace` runs all three. `CHANNEL_TOLERANCE` stays 0.

- **`scene_fx_oriented_quad`** submits three quads at `radius` 16, with `rotation` 0, 30 and 45, and with `axis[1]` and `axis[2]` set to three different orthonormal pairs so the arm's own axis read is exercised rather than the view's. Colors differ per quad.
- **`scene_fx_cylinder`** submits two cylinders. One is a straight tube, `radius` and `rotation` both 8. One is a cone, `radius` 12 and `rotation` 2, tilted off the view axis, so the two ring radii and the `RotatePointAroundVector` step are both visible. Both sit near enough for `segments` to clamp at 32.
- **`scene_fx_electricity`** submits two bolts with different `frame` seeds, one plain and one with `RF_TAPERED`, both without `RF_GROW`, so no arm depends on `refdef_time`. `axis[0][0]` is the chaos multiplier and is set to 1.0.

### Fixtures

Three new PNGs under `crates/mp/renderer-gpu/tests/goldens/`: `scene_fx_oriented_quad.png`, `scene_fx_cylinder.png`, `scene_fx_electricity.png`. One re-blessed PNG: `scene_saber_glow.png`.

Anything not on this list is out of scope, and the agent must not add it. No new third-party crate, because a dependency of the DEC-49 kind is a user ruling and this packet may never grant one. No change to any file under `crates/mp/engine/client/src/fx/`, to `frame_exec.rs`, `tr_cmds.rs`, `tr_scene.rs`, `stage2d.rs`, `blend.rs`, any WGSL shader, or any world or 2D path. No new `Warned` variant, no new stat field, no new cvar, no new `FrameEvent` variant, no ABI change, and no `WorldVertex` change. No edit to `world_golden.rs`, `entity_golden.rs`, `ghoul2_vertex_golden.rs`, or `hud_golden.rs`. Every committed fixture except `scene_saber_glow.png` is read-only, and the three new PNGs are the only fixtures this step may create.

## Divergence notes, each ≤2 lines at its site

- **The render stream is independent.** DEC-66 ruling 3 already accepts this, and this step is where it first reaches a drawing arm. The note at the `Pipeline3d::rng` field names the observable effect: a different jitter phase on cosmetic geometry, with no gate observing cross-stream interleaving.
- **The `oldorigin` write becomes a local.** The oracle writes the grown endpoint back into the entity and reads it straight back. See open row 6.

## Open rows

**Row 1 - where the three live arms land (user ruling).** DEC-66 ruling 2 says the dispatch shape stays `Option<&RefEntity>` and names the deferred note at `crates/mp/renderer/src/tr_surface.rs:1083`, which reads as if the port lands in that file. It cannot. That file is the R3 transcription of Raven's `tess` backend, `tess` has no carrier anywhere in the workspace, nothing calls `RB_SurfaceEntity` there, and its leaf emitters `DoLine`, `DoLine2`, `DoCylinderPart`, `RB_CheckOverflow` and `RB_AddQuadStampExt` are all `todo!()` for exactly that reason. **Proposed default: the live arms land in `build_entity_geometry` (`crates/mp/renderer-gpu/src/pipeline3d.rs`), the backend every census step from 005 through 008 has fed and the only path a golden gates.** Ruling 2's real instruction survives intact under this reading: do not rewire the dispatch to a mutable entity borrow, thread the seed as a local instead. The `mp_renderer` file is touched only for the two comment edits in the contract, and its `todo!()` bodies stay. The alternative is to complete the `tess` backend first, which is a different and much larger step.

**Row 2 - the saber-glow hilt radius (user ruling).** DEC-66 names `RB_SurfaceSaberGlow` in the chain the rulings unblock, and its stand-in is still live at `pipeline3d.rs:4639-4645`. Closing it changes `scene_saber_glow.png`, a committed fixture at tolerance zero, so it needs a re-bless. **Proposed default: close it in this step and re-bless `scene_saber_glow.png` under the row-3 STOP.** Leaving a TODO that a landed DEC explicitly unblocked would leave the census row half dark, and the change is one expression. The alternative is to defer it to step-010 with the renderfx closure. The re-blessed image must show the same two blades with a hilt blob of visibly the same size, because the draw spans a quarter of a unit on a 5.5 unit radius. A hilt that moves or disappears is a defect, not a blessable golden.

**Row 3 - the four goldens and the bless flow (user ruling).** **Proposed default: three new scene goldens plus the one re-bless, each through the step-007 procedure that step-008 ratified.** Run once with `JKA_GOLDEN_BLESS=1`, re-run without it to confirm the byte-identical pass, then STOP before the commit that carries the PNG so the user looks at the image. Named defect conditions, one per image. `scene_fx_oriented_quad.png` must show three quads at three visibly different angles. Three axis-aligned quads means the rotation branch is wrong. `scene_fx_cylinder.png` must show two closed tubes with visible ring silhouettes, one straight and one tapered. A flat quad or an open fan means the ring wrap or the winding is wrong. `scene_fx_electricity.png` must show two jagged branching-free bolts of visibly different jitter. A straight line means the deviation loop is wrong, and two identical bolts mean the per-entity seed is not threading. `scene_saber_glow.png` is row 2's re-bless.

**Row 4 - the state home (mechanical).** Proposed default: `Pipeline3d` owns `rng: Rng` and `shape: TrSurfaceShapeState`, both built in `Pipeline3d::new`. `TrSurfaceShapeState` keeps its canonical home in `mp_renderer::tr_surface` and gains `f_count`, with `LIGHTNING_RECURSION_LEVEL` as a const beside it, which is DEC-66 ruling 1's letter. `RendererFrontend::rng` (`crates/mp/renderer/src/renderer_frontend.rs:117`) stays the frontend's own instance and is not reused, because it lives on the other side of the DEC-50 thread split.

**Row 5 - the dead fork branch (mechanical).** Proposed default: transcribe the `RF_FORKED` branch exactly as MP writes it, with `f_count` starting at 0.0 and never assigned, so the branch never runs. A `≤2`-line note at the site records that MP never sets `f_count` while SP sets it to 3 (`oracle/code/renderer/tr_surface.cpp:844`), under porting-rules §20. Keep the `&&` short-circuit, because the guard's `Q_random` draw must not advance the seed when the earlier terms are false.

**Row 6 - the `oldorigin` write (mechanical).** Proposed default: compute the grown endpoint into a local `end` and never write the entity. The backend receives a by-value entity list per scene under DEC-50, and `backEnd.currentEntity` in the oracle points into the per-frame `tr.refdef.entities` array that the frontend refills every frame, so no oracle read outlives the frame either. This is the same disposition the `RT_SABER_GLOW` arm already carries for its radius write (`pipeline3d.rs:4615-4621`), and it takes the same `≤2`-line note. It is parity, not a divergence.

**Row 7 - the vertex cap on a long bolt (mechanical).** Proposed default: build the geometry with no cap and no flush, exactly as the other CPU-built arms do, and record the fact in the finished file rather than in code. The oracle flushes at `SHADER_MAX_VERTEXES` through `RB_CheckOverflow`. This backend has no flush point and never has had one. Nothing in this step changes that architecture. The lane must not invent a cap, a split, or a truncation. If a golden bolt exceeds the cap, that is a pause trigger, not a fix.

**Row 8 - the imports and the corrected census comment (mechanical).** Proposed default: `Q_random` and `RotatePointAroundVector` join the existing `mp_qshared::shared::q_math` group, and `Q_crandom` and `MakeNormalVectors` come from `native_math::qmath` because `mp_qshared` does not re-export them (`crates/mp/qshared/src/shared/q_math.rs:21-34`). The module doc above `build_entity_geometry` names the FX module as the submitter of the three kinds, keeps `RT_BEAM` and `RT_ORIENTEDLINE` in the fog list, and its two surviving `//TODO: Port` cites are corrected to `:478-528` and `:792-807`.

## Pause triggers, named for this step

- Any committed fixture other than `scene_saber_glow.png` moves. STOP. The threading commit and the two random-free arms touch no existing draw, and a moved pixel means something else changed.
- `scene_saber_glow.png` moves before commit 5. STOP. Only the hilt-radius commit may move it.
- A bolt's vertex count approaches `SHADER_MAX_VERTEXES`. STOP, per row 7. Do not add a cap.
- The oriented-quad arm looks like it needs `MakeNormalVectors` to draw correctly. STOP. That is the SP body, and the MP tree reads `axis[1]` and `axis[2]` directly.
- `f_count` looks like it needs an initial value to make the fork branch run. STOP, per row 5. Making the MP branch live is an SP behavior.
- A random draw looks like it should be hoisted out of a per-component statement. STOP. That is the multi-eval trap, and the packet's macro section is the instruction.
- Any arm seems to need a mutable `refEntity_t`. STOP. DEC-66 ruling 2 rules the seed as a threaded local, and row 6 rules the `oldorigin` write.
- Verification is `cargo build` or `cargo check` plus the golden suites, never rust-analyzer, which is stale in this workspace.

## Commit bundle

The full gate battery, named once and referenced per commit:

- `cargo build --workspace`, zero warnings.
- `cargo test --workspace -- --test-threads=1`.
- `cargo test -p mp_renderer_gpu --test world_golden -- --ignored --test-threads=1`, all four world goldens byte-identical.
- `cargo test -p mp_renderer_gpu --test scene_golden -- --test-threads=1`, every scene golden byte-identical.
- `cargo test -p mp_renderer_gpu --test entity_golden -- --ignored --test-threads=1`, byte-identical.
- `cargo test -p mp_renderer_gpu --test ghoul2_vertex_golden -- --ignored --test-threads=1`, byte-identical.
- `cargo test -p mp_renderer_gpu --test hud_golden -- --test-threads=1` and the same with `--ignored`, both byte-identical.

Every golden run is serial with `--test-threads=1`, each as one foreground command with a long timeout. Two engine boots in parallel threads crash in the GPU init path, and the world-golden pk3 inflate aborts without it.

1. **The state home and the threading.** `TrSurfaceShapeState` gains `f_count` and `Default`, `LIGHTNING_RECURSION_LEVEL` lands beside it, `Pipeline3d` gains `rng` and `shape`, and `build_entity_geometry` gains its three parameters. No arm is added and no draw changes. Files: `crates/mp/renderer/src/tr_surface.rs`, `crates/mp/renderer-gpu/src/pipeline3d.rs`. Subject: `feat(gh#31 s009): the render-side RNG owner and the shape state`. Gates: the full battery. All fifteen committed fixtures byte-identical is the proof that the threading is pure.

2. **The oriented-quad and cylinder arms.** The two arms, `do_cylinder_part`, the corrected module doc with its two surviving TODO cites, and the deferred-note lines in `tr_surface.rs` for these two kinds. Files: the same two. Subject: `feat(gh#31 s009): the oriented-quad and cylinder arms`. Gates: the full battery. Neither arm draws a random number and no committed scene submits either kind, so all fifteen fixtures stay byte-identical.

3. **The two shape goldens.** `scene_fx_oriented_quad` and `scene_fx_cylinder` with their two PNGs, each after its own row-3 STOP. Files: `crates/mp/renderer-gpu/tests/scene_golden.rs` and the two new PNGs. Subject: `test(gh#31 s009): the oriented-quad and cylinder goldens`. Gates: the full battery, with the two new goldens green at tolerance zero.

4. **The electricity arm.** `create_shape`, `apply_shape`, `do_line2`, `do_bolt_seg`, the `RT_ELECTRICITY` arm, and the `tr_surface.rs` note edits DEC-66 ruling 2 calls for. Files: `crates/mp/renderer-gpu/src/pipeline3d.rs`, `crates/mp/renderer/src/tr_surface.rs`. Subject: `feat(gh#31 s009): the electricity arm`. Gates: the full battery. No committed scene submits `RT_ELECTRICITY`, so all seventeen fixtures stay byte-identical.

5. **The saber-glow hilt radius.** The one expression at `pipeline3d.rs:4646`, the marker deletion, and the re-blessed `scene_saber_glow.png`, after its own row-3 STOP. The code and the PNG ride in one commit, because either alone leaves the scene suite red. Files: `crates/mp/renderer-gpu/src/pipeline3d.rs` and `crates/mp/renderer-gpu/tests/goldens/scene_saber_glow.png`. Subject: `feat(gh#31 s009): the saber-glow hilt radius`. Gates: the full battery, with `scene_saber_glow.png` the only fixture that moved.

6. **The electricity golden.** `scene_fx_electricity` and its PNG, after its own row-3 STOP. Files: `crates/mp/renderer-gpu/tests/scene_golden.rs` and the new PNG. Subject: `test(gh#31 s009): the electricity golden`. Gates: the full battery, with the new golden green at tolerance zero.

7. **The finished file**, per the packet skill: assumptions and choices keyed to their commits, deviations or the word "none", the commit list with gate results, and open gaps. File: `.claude/packets/31/step-009/finished.md`. Subject: `process(gh#31 s009): finished file`.

Every commit uses `--no-gpg-sign`, a heading subject, an STE body, and no trailer of any kind: no `Co-Authored-By`, no generated-with footer. Gate results are written as plain sentences inside the body, so no line parses as a git trailer. The lockstep referee is not required, because no commit touches `mp_game`, the server, or any `jampded` link-set crate.

## Write scopes

Branch `gh31-step-009-fx-minirefents`, cut from master. A worktree builder runs `git merge master --no-gpg-sign` as its first act.

- `crates/mp/renderer-gpu/src/pipeline3d.rs` - the two fields, the three arms, the four new free functions, the saber-glow expression, the module doc, the imports.
- `crates/mp/renderer/src/tr_surface.rs` - `TrSurfaceShapeState`, `LIGHTNING_RECURSION_LEVEL`, and the comment edits only.
- `crates/mp/renderer-gpu/tests/scene_golden.rs` - the three scene builders and the three tests.
- `crates/mp/renderer-gpu/tests/goldens/scene_fx_oriented_quad.png`, `scene_fx_cylinder.png`, `scene_fx_electricity.png` - new, blessed under the row-3 STOP.
- `crates/mp/renderer-gpu/tests/goldens/scene_saber_glow.png` - re-blessed under the row-3 STOP, in commit 5 only.
- Any caller `cargo check` shows broken by the `build_entity_geometry` signature, edit-only to pass the new shape.
- `.claude/packets/31/step-009/` for `finished.md`.

Everything else is read-only, including `oracle/`, every file under `crates/mp/engine/client/src/fx/`, `crates/mp/renderer-gpu/src/frame_exec.rs`, every other test file under `crates/mp/renderer-gpu/tests/`, every other committed fixture, and `~/Developer/jka/` beyond read-only asset reads. Source files change through the Edit tool only.

## Disposition

After a clean lane-review, per DEC-67: open the pull request from `gh31-step-009-fx-minirefents` to master and merge it on GitHub with a merge commit. Never squash, and never commit on master. The session never pushes or opens the pull request unprompted. It prepares the branch, asks, and the user rules on the push and on the merge.

## Amendments

None yet.
