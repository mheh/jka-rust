# Vet report - gh#31 step-009, the FX mini-refent backend arms

Range `02e701e8..gh31-step-009-fx-minirefents`, eight commits, walked oldest to newest, every hunk. The oracle cites were read before any commit. `finished.md` was not opened, and its diff content was excluded from review. Finding count: 10.

Commit order as walked:

1. `476930c1` feat(gh#31 s009): the render-side RNG owner and the shape state
2. `fb868fb7` feat(gh#31 s009): the oriented-quad and cylinder arms
3. `4a5ffa0d` test(gh#31 s009): the oriented-quad golden
4. `abc7209c` test(gh#31 s009): the cylinder golden
5. `232b4d11` feat(gh#31 s009): the electricity arm
6. `4a1469da` test(gh#31 s009): the electricity golden
7. `bcc5bf76` feat(gh#31 s009): the saber-glow hilt radius
8. `d3c46ea9` process(gh#31 s009): finished file

## 1. Letter violations

No `pub` item, signature, `#[repr]` layout, trap or dispatcher arm, cvar, `FrameEvent` variant, engine hook, dependency, or out-of-scope file appears that the contract does not list. `TrSurfaceShapeState.f_count`, the `Default` derive, `LIGHTNING_RECURSION_LEVEL`, the two private `Pipeline3d` fields, the `build_entity_geometry` signature, and the four private free functions all match the surface contract verbatim.

**Finding 1 (minor, `fb868fb7` and `4a1469da`).** Two imports land that the packet's import list does not name. The packet's row 8 names `Q_random`, `RotatePointAroundVector`, `MakeNormalVectors`, `Q_crandom`, `Rng`, `TrSurfaceShapeState`, `LIGHTNING_RECURSION_LEVEL`, the three `RF_` flags, and `c_int` as "New imports". The lane also added:

```rust
    _DotProduct, _VectorAdd as VectorAdd, _VectorMA as VectorMA, _VectorScale as VectorScale,
```

(`crates/mp/renderer-gpu/src/pipeline3d.rs`, the `_VectorAdd as VectorAdd` alias, needed by the cylinder and bolt arms) and

```rust
use mp_qshared::common::mp::cgame::tr_types::{
    RF_DEPTHHACK, RF_FORCE_ENT_ALPHA, RF_RGB_TINT, RF_TAPERED,
};
```

(`crates/mp/renderer-gpu/tests/scene_golden.rs`, needed by the electricity scene the packet itself specifies). Both are private `use` declarations inside write-scope files and both follow the file's existing alias convention, so neither creates a contract surface. I flag them because the packet's import sentence reads as a complete list.

## 2. Oracle divergences

Every arm and helper was compared line-by-line against `oracle/codemp/renderer/tr_surface.cpp:177-220` (oriented quad), `:818-847` and `:853-953` (cylinder), `:658-710` (`DoLine2`), `:976-987` (`CreateShape`), `:990-1036` (`ApplyShape`), `:1039-1124` (`DoBoltSeg`), `:1127-1169` (`RB_SurfaceElectricity`), and `:579` (the hilt radius). Winding orders, evaluation orders, the nine per-component draws, the `&&` short-circuit, the float widths (`f32` on the seeded path, widen-the-constant `f64` in `create_shape`, `f32` on the hilt radius), and the clamp shapes all match. The two ratified divergences (the backend-owned `Rng` stream, the local `end` for the `oldorigin` write) carry their site notes.

**Finding 2 (low, `232b4d11`).** The row-6 local `end` has a consequence the site note does not name. Oracle `DoBoltSeg`'s fork branch reads `e->oldorigin` at `:1107` *after* `RB_SurfaceElectricity` wrote the grown endpoint into it at `:1159`, so under `RF_GROW` the oracle's fork aims at the grown endpoint. The Rust fork branch reads the entity field:

```rust
            let mut new_dest: vec3_t = [0.0; 3];
            VectorAdd(cur, e.oldorigin, &mut new_dest);
```

which now holds the un-grown submitter value. The branch is dead in MP (`f_count` never exceeds 0, three greps confirm), so no behavior differs today, and the gap sits inside row 6's accepted divergence. It is unnamed at the site.

**Observation (not a finding).** `s = sin(ang)` at `:200-201` runs C's double `sin` and narrows at the store, while the arm runs `f32::sin`:

```rust
                let ang = (PI * e.rotation as f64 / 180.0) as f32;
                let s = ang.sin();
                let c = ang.cos();
```

This is byte-identical to the established `RT_SPRITE` arm and `do_sprite` shape, which the packet names as the required shape ("the same sine and cosine shape the sprite arm already runs"), so it is the contracted transcription. A sub-ulp difference against MSVC remains possible and no differential gate observes it.

## 3. The named hunks

**The `Pipeline3d` fields (`476930c1`).** Nothing wrong found. Verbatim:

```rust
    /// The render-side C runtime stream, per DEC-66 ruling 1.
    /// It persists across frames, because a per-frame reset would replay the same jitter every frame and freeze the bolt shimmer.
    rng: Rng,

    /// Raven's `sh1`, `sh2` and `f_count` file statics, per DEC-66 ruling 1.
    shape: TrSurfaceShapeState,
```

**Their construction in `Pipeline3d::new` (`476930c1`).** Nothing wrong found. Verbatim:

```rust
            rng: Rng::new(),
            shape: TrSurfaceShapeState::default(),
```

`Rng::new()` seeds the MSVC CRT state at 1 (`crates/native/math/src/rng.rs:127-132`), which the packet's ruling-1 shape requires.

**The `build_entity_geometry` signature (`476930c1`).** Matches the contract exactly. Verbatim:

```rust
fn build_entity_geometry(
    e: &refEntity_t,
    view: &viewParms_t,
    refdef_time: i32,
    rng: &mut Rng,
    shape: &mut TrSurfaceShapeState,
) -> (Vec<WorldVertex>, Vec<u32>) {
```

Its one call site in `collect_entity_surface` passes `refdef_time, &mut self.rng, &mut self.shape` before the `warn_once` call, as contracted.

**The `RT_ORIENTED_QUAD` arm (`fb868fb7`).** Nothing wrong found. The MP body is transcribed (axis rows read directly, the SP `MakeNormalVectors` line preserved as Raven's comment), the two temporaries keep the load-bearing write order, and `add_quad_stamp_ext(..., 0.0, 0.0, 1.0, 1.0)` closes it as `RB_AddQuadStamp` expands. Verbatim:

```rust
        refEntityType_t::RT_ORIENTED_QUAD => {
            let radius = e.radius;
            //	MakeNormalVectors( backEnd.currentEntity->e.axis[0], left, up );
            let mut left = e.axis[1];
            let mut up = e.axis[2];

            if e.rotation == 0.0 {
                VectorScale(left, radius, &mut left);
                VectorScale(up, radius, &mut up);
            } else {
                let ang = (PI * e.rotation as f64 / 180.0) as f32;
                let s = ang.sin();
                let c = ang.cos();

                // Use a temp so we don't trash the values we'll need later
                let mut temp_left: vec3_t = [0.0; 3];
                VectorScale(left, c * radius, &mut temp_left);
                VectorMA(temp_left, -s * radius, up, &mut temp_left);

                let mut temp_up: vec3_t = [0.0; 3];
                VectorScale(up, c * radius, &mut temp_up);
                // no need to use the temp anymore, so copy into the dest vector ( up )
                VectorMA(temp_up, s * radius, left, &mut up);

                // This was copied for safekeeping, we're done, so we can move it back to left
                left = temp_left;
            }

            if view.isMirror != 0 {
                VectorSubtract(vec3_origin, left, &mut left);
            }

            add_quad_stamp_ext(
                &mut verts,
                &mut indices,
                e.origin,
                left,
                up,
                color,
                0.0,
                0.0,
                1.0,
                1.0,
            );
        }
```

`VectorMA(temp_up, s * radius, left, &mut up)` reads `left` while it still holds the unrotated copy, which is the oracle's `:208` order.

**The `RT_CYLINDER` arm (`fb868fb7`).** Nothing wrong found against `:853-953`. The radius mapping follows the code per the amendment (`e.radius` through `v1` to the `e.oldorigin` ring, `e.rotation` through `vu` to the `e.origin` ring), the LOD math and 8..=32 clamps match, the static arrays became owned locals with the packet's justification at the site, and `do_cylinder_part` carries the cylinder's own `vbase, +1, +2, +2, +3, vbase` winding, distinct from `DoLine`'s. Verbatim (the arm body):

```rust
        refEntityType_t::RT_CYLINDER => {
            // `#define NUM_CYLINDER_SEGMENTS 32`
            // Source: oracle/codemp/renderer/tr_surface.cpp:815
            const NUM_CYLINDER_SEGMENTS: i32 = 32;

            // Work out the detail level of this cylinder
            let mut midpoint: vec3_t = [0.0; 3];
            VectorAdd(e.origin, e.oldorigin, &mut midpoint);
            VectorScale(midpoint, 0.5, &mut midpoint); // Average start and end

            VectorSubtract(midpoint, view.ori.origin, &mut midpoint);
            let mut length = VectorNormalize(&mut midpoint);

            // this doesn't need to be perfect....just a rough compensation for zoom level is enough
            length *= view.fovX / 90.0;

            let mut detail = 1.0 - (length / 1024.0);
            let mut segments = (NUM_CYLINDER_SEGMENTS as f32 * detail) as i32;

            // 3 is the absolute minimum, but the pop between 3-8 is too noticeable
            if segments < 8 {
                segments = 8;
            }

            if segments > NUM_CYLINDER_SEGMENTS {
                segments = NUM_CYLINDER_SEGMENTS;
            }

            // Get the direction vector
            let mut vr: vec3_t = [0.0; 3];
            let mut vu: vec3_t = [0.0; 3];
            MakeNormalVectors(e.axis[0], &mut vr, &mut vu);

            let mut v1: vec3_t = [0.0; 3];
            VectorScale(vu, e.radius, &mut v1); // size1
            VectorScale(vu, e.rotation, &mut vu); // size2

            // Calculate the step around the cylinder
            detail = 360.0 / segments as f32;

            // Raven's two ring arrays are function-local statics, and every element is written before it is read on each call.
            // They are per-call scratch, so they become owned locals here.
            let mut upper_points = [[0.0f32; 3]; NUM_CYLINDER_SEGMENTS as usize];
            let mut lower_points = [[0.0f32; 3]; NUM_CYLINDER_SEGMENTS as usize];

            for i in 0..segments {
                // Upper ring
                let mut point: vec3_t = [0.0; 3];
                RotatePointAroundVector(&mut point, e.axis[0], vu, detail * i as f32);
                VectorAdd(point, e.origin, &mut upper_points[i as usize]);

                // Lower ring
                RotatePointAroundVector(&mut point, e.axis[0], v1, detail * i as f32);
                VectorAdd(point, e.oldorigin, &mut lower_points[i as usize]);
            }

            // Calculate the texture coords so the texture can wrap around the whole cylinder
            detail = 1.0 / segments as f32;

            let mut quad = [polyVert_t {
                xyz: [0.0; 3],
                st: [0.0; 2],
                modulate: [0; 4],
            }; 4];

            for i in 0..segments {
                let next_segment = if i + 1 < segments { i + 1 } else { 0 };

                quad[0].xyz = upper_points[i as usize];
                quad[0].st[1] = 1.0;
                quad[0].st[0] = detail * i as f32;
                quad[0].modulate = e.shaderRGBA;

                quad[1].xyz = lower_points[i as usize];
                quad[1].st[1] = 0.0;
                quad[1].st[0] = detail * i as f32;
                quad[1].modulate = e.shaderRGBA;

                quad[2].xyz = lower_points[next_segment as usize];
                quad[2].st[1] = 0.0;
                quad[2].st[0] = detail * (i + 1) as f32;
                quad[2].modulate = e.shaderRGBA;

                quad[3].xyz = upper_points[next_segment as usize];
                quad[3].st[1] = 1.0;
                quad[3].st[0] = detail * (i + 1) as f32;
                quad[3].modulate = e.shaderRGBA;

                do_cylinder_part(&mut verts, &mut indices, &quad);
            }
        }
```

**The `RT_ELECTRICITY` arm (`232b4d11`).** Nothing wrong found against `:1127-1169` beyond finding 4 below. Verbatim:

```rust
        refEntityType_t::RT_ELECTRICITY => {
            let radius = e.radius;

            let start = e.origin;

            let mut fwd: vec3_t = [0.0; 3];
            VectorSubtract(e.oldorigin, start, &mut fwd);
            let dis = VectorNormalize(&mut fwd);

            // see if we should grow from start to end
            let mut perc = 1.0f32;
            if (e.renderfx & RF_GROW) != 0 {
                perc = 1.0 - (e.axis[0][2] - refdef_time as f32) / e.axis[0][1];

                if perc > 1.0 {
                    perc = 1.0;
                } else if perc < 0.0 {
                    perc = 0.0;
                }
            }

            // The oracle writes the grown endpoint back into the shared entity array and reads it straight back.
            // The write lands in a local here, an accepted divergence a portal or a mirror view would make visible.
            // Source: oracle/codemp/renderer/tr_surface.cpp:1159
            let mut end: vec3_t = [0.0; 3];
            VectorMA(start, perc * dis, fwd, &mut end);

            // compute side vector
            let mut v1: vec3_t = [0.0; 3];
            let mut v2: vec3_t = [0.0; 3];
            VectorSubtract(start, view.ori.origin, &mut v1);
            VectorSubtract(end, view.ori.origin, &mut v2);
            let mut right: vec3_t = [0.0; 3];
            CrossProduct(v1, v2, &mut right);
            VectorNormalize(&mut right);

            // DEC-66 ruling 2 threads the entity's own seed as a local, because the oracle's seed write never outlives one draw chain.
            let mut seed: c_int = e.frame;
            do_bolt_seg(
                &mut verts,
                &mut indices,
                e,
                &mut seed,
                start,
                end,
                right,
                radius,
                color,
                shape,
                rng,
            );
        }
```

The local `end` divergence note is two lines plus a Source cite, names portals and mirrors, and words the write as an accepted divergence, which is the row-6 amendment's shape.

**`do_bolt_seg` whole (`232b4d11`).** Nothing wrong found against `:1039-1124` beyond findings 2 and 4. Verbatim:

```rust
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
) {
    let mut fwd: vec3_t = [0.0; 3];
    VectorSubtract(end, start, &mut fwd);
    let dis = VectorNormalize(&mut fwd);

    let mut rt: vec3_t = [0.0; 3];
    let mut up: vec3_t = [0.0; 3];
    MakeNormalVectors(fwd, &mut rt, &mut up);

    let mut old = start;
    let mut off: vec3_t = [10.0, 10.0, 10.0];

    let mut old_perc = 0.0f32;
    let mut old_radius = radius;
    let mut new_radius = radius;

    let mut i: c_int = 20;
    while i as f32 <= dis {
        // because of our large step size, we may not actually draw to the end.  In this case, fudge our percent so that we are basically complete
        let perc = if (i + 20) as f32 > dis {
            1.0
        } else {
            // percentage of the amount of line completed
            i as f32 / dis
        };

        // create our level of deviation for this point
        //
        // Raven writes these three lines as `VectorScale` and `VectorMA` macros, and each macro expands its scale argument once per component.
        // Every component therefore draws its own value, nine per step, and hoisting any draw into a local would change both the stream and the shape.
        let mut temp: vec3_t = [0.0; 3];
        // move less in fwd direction, chaos also does not affect this
        temp[0] = fwd[0] * (Q_crandom(seed) * 3.0);
        temp[1] = fwd[1] * (Q_crandom(seed) * 3.0);
        temp[2] = fwd[2] * (Q_crandom(seed) * 3.0);

        // move more in direction perpendicular to line, angles is really the chaos
        temp[0] += rt[0] * (Q_crandom(seed) * 7.0 * e.axis[0][0]);
        temp[1] += rt[1] * (Q_crandom(seed) * 7.0 * e.axis[0][0]);
        temp[2] += rt[2] * (Q_crandom(seed) * 7.0 * e.axis[0][0]);

        // move more in direction perpendicular to line
        temp[0] += up[0] * (Q_crandom(seed) * 7.0 * e.axis[0][0]);
        temp[1] += up[1] * (Q_crandom(seed) * 7.0 * e.axis[0][0]);
        temp[2] += up[2] * (Q_crandom(seed) * 7.0 * e.axis[0][0]);

        // track our total level of offset from the ideal line
        VectorAdd(off, temp, &mut off);

        // Move from start to end, always adding our current level of offset from the ideal line
        //	Even though we are adding a random offset.....by nature, we always move from exactly start....to end
        let mut cur: vec3_t = [0.0; 3];
        VectorAdd(start, off, &mut cur);
        VectorScale(cur, 1.0 - perc, &mut cur);
        VectorMA(cur, perc, end, &mut cur);

        if (e.renderfx & RF_TAPERED) != 0 {
            // This does pretty close to perfect tapering since apply shape interpolates the old and new as it goes along.
            //	by using one minus the square, the radius stays fairly constant, then drops off quickly at the very point of the bolt
            old_radius = radius * (1.0 - old_perc * old_perc);
            new_radius = radius * (1.0 - perc * perc);
        }

        // Apply the random shape to our line seg to give it some micro-detail-jaggy-coolness.
        apply_shape(
            verts,
            indices,
            cur,
            old,
            right,
            new_radius,
            old_radius,
            LIGHTNING_RECURSION_LEVEL,
            color,
            shape,
            rng,
        );

        // randomly split off to create little tendrils, but don't do it too close to the end and especially if we are not even of the forked variety
        //
        // MP never assigns `f_count`, so this branch is dead here. SP sets it to 3 right before its own `DoBoltSeg` call, which is what makes the fork live there.
        // The `&&` chain keeps its short-circuit, because an eager `Q_random` draw would advance the seed on every step.
        // Source: oracle/code/renderer/tr_surface.cpp:844
        if (e.renderfx & RF_FORKED) != 0
            && shape.f_count > 0.0
            && Q_random(seed) > 0.94
            && radius * (1.0 - perc) > 0.2
        {
            shape.f_count -= 1.0;

            // Pick a point somewhere between the current point and the final endpoint
            let mut new_dest: vec3_t = [0.0; 3];
            VectorAdd(cur, e.oldorigin, &mut new_dest);
            VectorScale(new_dest, 0.5, &mut new_dest);

            // And then add some crazy offset
            for t in 0..3 {
                new_dest[t] += Q_crandom(seed) * 80.0;
            }

            // we could branch off using OLD and NEWDEST, but that would allow multiple forks...whereas, we just want simpler brancing
            do_bolt_seg(
                verts, indices, e, seed, cur, new_dest, right, new_radius, color, shape, rng,
            );
        }

        // Current point along the line becomes our new old attach point
        old = cur;
        old_perc = perc;

        i += 20;
    }
}
```

The nine per-component draws stand as nine separate `Q_crandom(seed)` calls in the oracle's fwd/rt/up, 0/1/2 order, with no hoist. The `while` form reproduces `for ( i = 20; i <= dis; i += 20 )` including the int-to-float comparisons. The fork guard keeps the `&&` short-circuit, so the `Q_random` draw fires only after the two dead terms, and `f_count` starts at 0.0 through `Default` and is never assigned.

**The saber-glow hilt-radius expression (`bcc5bf76`).** Nothing wrong found. Verbatim:

```rust
            // Big hilt sprite
            //
            // The quarter-unit pulse draws from the backend's own C runtime stream, per DEC-66 ruling 1.
            // Retail runs one process-wide stream instead, and ruling 3 accepts the split as a divergence on cosmetic geometry.
            // Source: oracle/codemp/renderer/tr_surface.cpp:579
            do_sprite(
                &mut verts,
                &mut indices,
                e.origin,
                5.5 + rng.random() * 0.25,
                0.0,
                color,
                view,
            );
```

`Rng::random` returns `f32`, so the expression stays `f32` end to end, matching `5.5f + random() * 0.25f`. The `//TODO: Port RB_SurfaceSaberGlow's random hilt radius` marker and its stand-in note are deleted in the same hunk. The commit's radius claims verify arithmetically: draws 41 and 18467 off the fresh MSVC stream give 5.500313 and 5.640896.

**The census-complement module doc (`fb868fb7`).** Nothing wrong found in content, one style hit (finding 8). Verbatim:

```rust
/// Builds one generated entity surface's world-space vertices and triangle indices, the `RB_SurfaceEntity` dispatch restricted to the kinds this backend draws.
/// Those are the DEC-54 census set, `RT_SPRITE`, `RT_LINE` and `RT_SABER_GLOW`, plus the three the FX module submits, `RT_ORIENTED_QUAD`, `RT_CYLINDER` and `RT_ELECTRICITY`.
///
/// Every other `reType` returns empty, which the caller counts as a skip.
/// `RT_BEAM` and `RT_ORIENTEDLINE` are census-complement fog, so they stay unbuilt rather than guessed at.
/// The census never saw the other three.
/// The FX module builds their `refEntity_t` inside the engine and submits it from `FxHost::AddFxToScene`, behind the trap seam the census counts.
//TODO: Port RB_SurfaceBeam
// Source: oracle/codemp/renderer/tr_surface.cpp:478-528
//TODO: Port RB_SurfaceOrientedLine
// Source: oracle/codemp/renderer/tr_surface.cpp:792-807
```

Both corrected cites verify against the oracle: `RB_SurfaceBeam` sits at `:478-528` and `RB_SurfaceOrientedLine` at `:792-807`. The three markers for the now-live kinds are removed.

## 4. The inventories

**Files against write scopes.** The range touches exactly eight paths: `crates/mp/renderer-gpu/src/pipeline3d.rs`, `crates/mp/renderer/src/tr_surface.rs`, `crates/mp/renderer-gpu/tests/scene_golden.rs`, the three new PNGs, `scene_saber_glow.png`, and `.claude/packets/31/step-009/finished.md`. All are in scope. No FX file, no `frame_exec.rs`, no other test file, no other fixture moved. `scene_saber_glow.png` moved only in `bcc5bf76`, so the pause trigger held.

**Commits against the bundle.** Eight commits against a seven-commit bundle. Bundle commit 3 (the two shape goldens) split into `4a5ffa0d` and `abc7209c`, declared in the first split commit's body. Bundle commits 5 and 6 swapped: the saber-glow radius (`bcc5bf76`) landed after the electricity golden (`4a1469da`). Both are legal reshapes and no fixture rule broke under the reorder.

**Finding 3 (declared deviation, `abc7209c`).** The packet's cylinder scene asks for both cylinders "near enough for `segments` to clamp at 32". The commit body declares the scene runs 28 segments and states the clamp is unreachable in a readable image. The math supports the declaration: `segments = (32.0 * detail) as i32` reaches 32 only at `detail >= 1.0`, which needs a zero eye distance. The packet letter is unsatisfiable, the deviation is declared, and the session rules on it.

**Finding 4 (packet-internal contradiction, transcribed in `232b4d11`).** The `do_bolt_seg` doc line and the seed-hoist site note both read "because the oracle's seed write never outlives one draw chain". The row-6 amendment states the opposite fact: a second view of the frame reads the `frame` seed the first draw mutated (`oracle/codemp/renderer/tr_surface.cpp:1159` and the shared entities array). The lane transcribed the packet's own contracted sentence for `do_bolt_seg`, so the defect originates in the packet, not the lane. The two comments state parity where the amendment ruled an accepted, currently unobservable divergence.

**Finding 5 (minor inventory, `232b4d11`).** The `tr_surface.rs` edits go past the two comment edits the bundle names for commit 4. Besides dropping the overruled sentence and adding the live-arm line, the commit corrects two further stale sentences in the `DoBoltSeg` deferred note (the `f_count`-not-on-the-carrier sentence and the `LIGHTNING_RECURSION_LEVEL`-unported sentence), which commit `476930c1` had made false. Comment-only, inside the "comment edits only" write scope, and declared in the finished file. No body changed and no `todo!()` was removed in that file.

**Commit messages.** All eight carry a heading subject in the packet's contracted form, a prose body with gate results as plain sentences, and no trailer of any kind. `git log --format='%(trailers)'` over the range returns empty for every commit. One body-style hit is finding 10.

## 5. Repo mechanics on added lines

Swept every added line in the range. No `use` declaration inside a function body. No new `todo!()` or placeholder (the only `todo!` strings in the diff are prose inside `finished.md`, which is outside this review). No new extern forward-declaration block. No `format!` building a wire string. Every newly ported item carries a `Source:` cite: `do_line2`, `do_cylinder_part`, `create_shape`, `apply_shape`, `do_bolt_seg`, the three arms, the arm-local `NUM_CYLINDER_SEGMENTS`, `LIGHTNING_RECURSION_LEVEL`, `TrSurfaceShapeState`, and the three scene builders. The deleted `//TODO: Port` markers (three module-doc markers plus the saber-glow marker) are exactly the four the contract names. No findings.

## 6. House-style violations on added lines

Both skill files were read by path. Raven-preserved comments (`don't trash`, `we're done`, `doesn't need to be perfect`, `micro-detail-jaggy-coolness`, and the rest) are transcriptions the porting rules protect and are not counted.

**Finding 6 (`fb868fb7`, `232b4d11`, `4a5ffa0d`, `4a1469da`).** Seven added comment lines exceed the 150-column cap:

```
        // Every component therefore draws its own value, nine per step, and hoisting any draw into a local would change both the stream and the shape.
        // MP never assigns `f_count`, so this branch is dead here. SP sets it to 3 right before its own `DoBoltSeg` call, which is what makes the fork live there.
/// Builds one generated entity surface's world-space vertices and triangle indices, the `RB_SurfaceEntity` dispatch restricted to the kinds this backend draws.
/// Those are the DEC-54 census set, `RT_SPRITE`, `RT_LINE` and `RT_SABER_GLOW`, plus the three the FX module submits, `RT_ORIENTED_QUAD`, `RT_CYLINDER` and `RT_ELECTRICITY`.
/// The FX module's `RT_ORIENTED_QUAD` submissions, which the trap census never saw because `COrientedParticle::Draw` builds the entity inside the engine.
/// Three quads at one radius, each on its own orthonormal `axis[1]`/`axis[2]` pair and its own rotation, so the arm reads the entity's axis rather than the view's.
/// The FX module's `RT_ELECTRICITY` submissions, which the trap census sees only as the `fx/AddElectricity` row because `CElectricity::Draw` builds the entity inside the engine.
```

**Finding 7 (`fb868fb7`, `232b4d11`).** Column wraps at width, not at a sentence or clause boundary, on added lines. In `pipeline3d.rs`, three arm-head comments:

```rust
        // `RB_SurfaceOrientedQuad`: the quad spans the entity's own `axis[1]`
        // and `axis[2]`, not the view's, so it keeps its world orientation.
```

```rust
        // `RB_SurfaceCylinder`: two rings of points around `axis[0]`, joined
        // into a closed ring of quads. The segment count drops with distance.
```

```rust
        // `axis[0][1]` and `axis[0][2]` are not axis components here. Raven's own inline comments name them the duration and the end
        // time, and the FX submitter fills them that way.
```

The last one breaks inside the noun phrase "end time". In `tr_surface.rs`, the rewritten `DoBoltSeg` note lines re-flow into the legacy wrap of the surrounding paragraph instead of breaking per sentence:

```rust
/// DEC-66 ruling 2 closes that gap without a rewire: the dispatch stays immutable and the seed threads as a local, which is what the live arm does.
/// The `RF_FORKED`
```

```rust
/// comment), and `f_count` now sits on it as the `float` file-scope value
/// oracle declares alongside `sh1`/`sh2` (`static float f_count;`,
/// `tr_surface.cpp:956`). The loop also reads `e->renderfx & RF_TAPERED`
```

```rust
/// longer a blocker. `LIGHTNING_RECURSION_LEVEL` now sits beside
/// `TrSurfaceShapeState` in this file
```

Every joined sentence sits under 150 columns except the ones in finding 6, so width was the only reason for the breaks.

**Finding 8 (`232b4d11`, `fb868fb7`).** Two sentences on one comment line, against the one-sentence-per-line default:

```rust
        // MP never assigns `f_count`, so this branch is dead here. SP sets it to 3 right before its own `DoBoltSeg` call, which is what makes the fork live there.
```

```rust
        // `axis[0][1]` and `axis[0][2]` are not axis components here. Raven's own inline comments name them the duration and the end
```

```rust
        // into a closed ring of quads. The segment count drops with distance.
```

**Finding 9 (`fb868fb7`).** Pet vocabulary on an added doc line: the word "seam" in the module doc, "behind the trap seam the census counts" (quoted whole in section 3). The house-style list bans "seam". Mitigation for the session's ruling: "the ABI seam" is entrenched repo vocabulary, used by `CLAUDE.md` and the porting rules themselves, and the packet's own prose uses it.

**Finding 10 (`bcc5bf76`).** Pet vocabulary in a commit body: "The code and the re-blessed PNG ride in one commit, because either alone leaves the scene suite red." The house-style list bans "rides". The sentence is the packet's own contracted wording ("The code and the PNG ride in one commit, because either alone leaves the scene suite red"), so the origin is the packet.

No em dash, no semicolon in prose, no contraction outside preserved Raven comments, no banned voice, and no mechanics-narrating comment was found on any added line.

## 7. The gate battery, re-run

Run on the checked-out branch at `d3c46ea9`, each command as the packet gives it. Real output:

- `cargo build --workspace` after touching the three edited source files: recompiled with **zero warnings, zero errors**, `Finished dev profile`.
- `cargo test --workspace -- --test-threads=1`: every suite `test result: ok`, **0 failed** across the workspace (the scene suite ran its 10 tests inside this pass).
- `cargo test -p mp_renderer_gpu --test world_golden -- --ignored --test-threads=1`: `ok. 4 passed; 0 failed` in 68.24s (`golden_world_dlights_duel1`, `golden_world_duel1`, `golden_world_ffa2`, `golden_world_marks_duel1`).
- `cargo test -p mp_renderer_gpu --test scene_golden -- --test-threads=1`: `ok. 10 passed; 0 failed` in 5.76s, including `golden_scene_fx_oriented_quad`, `golden_scene_fx_cylinder`, `golden_scene_fx_electricity`, and `golden_scene_saber_glow`.
- `cargo test -p mp_renderer_gpu --test entity_golden -- --ignored --test-threads=1`: `ok. 1 passed; 0 failed` in 15.63s.
- `cargo test -p mp_renderer_gpu --test ghoul2_vertex_golden -- --ignored --test-threads=1`: `ok. 1 passed; 0 failed` in 15.44s.
- `cargo test -p mp_renderer_gpu --test hud_golden -- --test-threads=1`: `ok. 1 passed; 0 failed; 1 ignored`. With `--ignored`: `ok. 1 passed; 0 failed`.

`CHANNEL_TOLERANCE` is 0 in `scene_golden.rs:73` and the three new tests carry no `#[ignore]`. `git status` is clean after all runs, so every committed fixture is byte-identical to what the tests produce. The byte-identical claims are therefore re-proven, not trusted.

Two fixture claims verified beyond the gates. The re-blessed `scene_saber_glow.png` differs from the `02e701e8` copy in exactly 22 pixels, all inside x 123-132, y 131-140, one 10x10 box, matching the commit body's count. The four images were inspected against the packet's named defect conditions: three quads at three visibly different angles, two closed bodies with one straight and one tapered, two jagged fork-free bolts of visibly different jitter, and two blades with hilt draws present.

## 8. The unverified list

- `finished.md` content. Excluded by instruction. Its assumptions, deviations list, and gap list are unreviewed.
- The four eyes-on user rulings the commit bodies cite ("The user ruled the candidate a pass on 2026-08-28/29"). Nothing in the repo records them, and I did not assume them true.
- Whether each row-3 STOP actually happened before its PNG commit. The process is not reconstructable from git history.
- Exact float parity of `f32::sin`/`f32::cos` against MSVC's double-`sin`-then-narrow at `:200-201` and the sprite precedent. No differential gate observes these arms against the oracle binary. The goldens pin the Rust behavior only.
- The "visible ring silhouettes" wording of the cylinder pass condition. The flat-shaded fixture shows two closed outlines and a taper. Interior ring facets are not resolvable at fixture size, and I could not mechanically count segments in the image.
- The row-7 vertex-cap posture on a long bolt. The committed 100-unit bolts emit 60 vertices, far under any cap, and no cap or flush code was added. Behavior at a genuinely long bolt is untested by any committed gate.
- Cross-stream interleaving of the backend `Rng` against retail's process-wide stream (DEC-66 ruling 3's accepted divergence). No gate observes it, as the packet states.

# Fix-round vet - 2026-08-29, `d3c46ea9..gh31-step-009-fx-minirefents`

Two new commits, walked whole: `32db25ab` (fix: the lane-review comment and packet corrections) and `9df6cab7` (process: the fix-round record). The first eight hashes are unchanged (`476930c1`, `fb868fb7`, `4a5ffa0d`, `abc7209c`, `232b4d11`, `4a1469da`, `bcc5bf76`, `d3c46ea9`), so no history was rewritten. Finding count: 1.

## The ruled remedies, checked

**Finding 2's remedy is present.** The row-6 site note in `pipeline3d.rs` gains the fork-branch sentence, one sentence over two lines broken at the comma because the joined sentence passes 150 columns:

```rust
            // The oracle writes the grown endpoint back into the shared entity array and reads it straight back.
            // The write lands in a local here, an accepted divergence a portal or a mirror view would make visible.
            // The dead `RF_FORKED` branch would also read the un-grown `e.oldorigin`,
            // where the oracle's fork read at `:1107` sees the grown value the write at `:1159` left.
            // Source: oracle/codemp/renderer/tr_surface.cpp:1159
```

The `:1107`/`:1159` cites match the oracle lines I walked in the first round.

**Finding 4's remedy is present at all three sites.** The `do_bolt_seg` doc:

```rust
/// `seed` is `e.frame` hoisted into a local, per DEC-66 ruling 2.
/// The oracle's seed write is unobservable until portal or mirror views draw, because a second view of the same frame would read the mutated seed.
```

The seed-hoist site note:

```rust
            // DEC-66 ruling 2 threads the entity's own seed as a local.
            // The dropped write is unobservable until portal or mirror views draw,
            // because a second view of the same frame would read the mutated `oldorigin` and seed.
```

The packet body's contracted `do_bolt_seg` doc carries the same replacement, so the packet no longer contradicts its own row-6 amendment. All three now state the amended mechanism.

**Findings 6, 7 and 8's remedy is executed on every quoted line.** Each over-length or width-wrapped line I quoted in `pipeline3d.rs` and `scene_golden.rs` is re-broken to one sentence per line or at a clause boundary, no added line in `crates/` exceeds 150 columns (checked mechanically over the diff), the "end time" noun phrase is whole again, the fork note and the electricity head each hold one sentence per line, and a word-by-word comparison of each re-broken group against the first-round quotes shows no wording change outside the two finding-4 comments that were ruled to change.

**Ruling 6 is executed.** The packet's commit bundle item 5 now reads "The code and the PNG land in one commit", the word "ride" is gone from the reusable wording, and the commit history stands unrewritten. The packet also gains a dated 2026-08-29 Amendment recording all six verdicts, including the no-edit verdicts on findings 1 and 9.

**The commits are comment-and-doc only.** A mechanical filter over the `crates/` diff of `32db25ab` finds zero changed lines that are not `//` or `///` lines. No fixture moved, no `pub` item, signature, or other surface changed. `9df6cab7` touches only `.claude/packets/31/step-009/finished.md` and `vet.md`, and the committed `vet.md` is byte-identical to the report I wrote (verified with `diff`).

**Commit messages.** Both carry a heading subject, a plain-sentence body with gate results as prose, and `%(trailers)` returns empty for both.

## The finding

**Finding F1 (minor, `32db25ab`, `crates/mp/renderer/src/tr_surface.rs`).** Three fix-round lines in the `DoBoltSeg` deferred note still interlock with the untouched legacy wrap around them, so their sentences still break at width across unchanged neighbor lines:

```rust
/// The `RF_FORKED` branch's `f_count--` write is this packet's STATE HOMES row
/// `DoBoltSeg`/`f_count`: "per-subsystem owned state struct, NAMED BY THIS
```

(the changed first line ends mid noun phrase, and the unchanged second line continues the legacy wrap),

```rust
/// already names that carrier (`TrSurfaceShapeState`, `CreateShape`'s doc
/// comment), and `f_count` now sits on it as the `float` file-scope value oracle declares alongside `sh1`/`sh2`
```

(the changed second line begins mid-sentence off the unchanged first), and

```rust
/// crate's canonical flag home (`tr_public::ref_flags`), so the masks are no
/// longer a blocker.
```

(the changed `longer a blocker.` line is the tail of the legacy `no / longer` wrap). A full remedy needs a per-sentence re-flow of the unchanged legacy paragraph, which would widen the diff past the ruling's comment-only scope. The session rules whether the residual stands or the paragraph re-flows.

## The gate battery, re-run at `9df6cab7`

- `cargo build --workspace`: zero warnings, zero errors.
- `cargo test --workspace -- --test-threads=1`: every suite `test result: ok`, no suite reports a nonzero failed count.
- `cargo test -p mp_renderer_gpu --test world_golden -- --ignored --test-threads=1`: `ok. 4 passed; 0 failed` in 67.77s.
- `cargo test -p mp_renderer_gpu --test scene_golden -- --test-threads=1`: `ok. 10 passed; 0 failed` in 5.81s.
- `cargo test -p mp_renderer_gpu --test entity_golden -- --ignored --test-threads=1`: `ok. 1 passed; 0 failed` in 15.64s.
- `cargo test -p mp_renderer_gpu --test ghoul2_vertex_golden -- --ignored --test-threads=1`: `ok. 1 passed; 0 failed` in 16.19s.
- `cargo test -p mp_renderer_gpu --test hud_golden -- --test-threads=1`: `ok. 1 passed; 0 failed; 1 ignored`. With `--ignored`: `ok. 1 passed; 0 failed`.

`git status` is clean after all runs, so all eighteen committed fixtures are byte-identical.
