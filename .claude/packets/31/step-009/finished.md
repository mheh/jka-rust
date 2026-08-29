# Finished gh#31 step-009 - the FX mini-refent backend arms

Branch `gh31-step-009-fx-minirefents`, cut from the fold commit `02e701e8`. Seven commits, none pushed, no pull request opened.

## Assumptions and choices, keyed to their commits

**476930c1, the state home and the threading.** `Pipeline3d` owns `rng: Rng` and `shape: TrSurfaceShapeState`, both built in `Pipeline3d::new`, per row 4. `TrSurfaceShapeState` keeps its home in `mp_renderer::tr_surface` and gains `f_count` and a derived `Default`. `LIGHTNING_RECURSION_LEVEL` lands beside it as a `c_int`, which needed one new `core::ffi::c_int` import in that file. `RendererFrontend::rng` was not reused, because it sits on the frontend side of the DEC-50 split.

**fb868fb7, the oriented-quad and cylinder arms.** Both arms transcribe the MP body. The oriented quad reads `axis[1]` and `axis[2]` directly and keeps Raven's commented-out `MakeNormalVectors` line as a comment, because the SP body is the one that calls it. The rotation branch keeps both temporaries, because the second `VectorMA` still reads the unrotated `left`.

The cylinder follows the code, not Raven's ring names: `e.radius` scales the `v1` ring that translates to `e.oldorigin`, and `e.rotation` scales the `vu` ring that translates to `e.origin`. The site comment says exactly that and names the contradiction rather than repeating Raven's upper and lower names. Raven's two ring arrays are function-local statics that every call overwrites before it reads, so they become owned `[vec3_t; 32]` locals, and the quad becomes one owned `[polyVert_t; 4]`. `do_cylinder_part` carries its own index winding, `vbase, +1, +2, +2, +3, vbase`, which differs from `do_line`'s.

The ring loops write through a `point` local instead of reading and writing one array slot in the same call. The oracle reads `upper_points[i].xyz` back as the source of its own `VectorAdd`, and the local is the same computation without a borrow conflict.

The module doc above `build_entity_geometry` names the FX module as the submitter of the three engine-side kinds, keeps `RT_BEAM` and `RT_ORIENTEDLINE` in the census-complement list, and corrects the two surviving cites to `:478-528` and `:792-807`. All three matching `//TODO: Port` markers are gone.

**232b4d11, the electricity arm.** The two random families stay apart. `do_bolt_seg` draws from the entity's own 69069 seed, hoisted out of `e.frame` into a local `c_int` and threaded through the recursion per DEC-66 ruling 2, so the immutable entity borrow holds. `create_shape` draws from the backend's C runtime stream.

The three deviation lines are written one statement per component, nine `Q_crandom` draws per step, because Raven's `VectorScale` and `VectorMA` macros expand the scale argument once per component. No draw is hoisted anywhere in the chain.

Float widths follow the packet. The seeded family stays `f32` end to end. The six `create_shape` expressions widen their constants as `0.66f32 as f64` and narrow at the store, because `crandom()` is `double` in C. The hilt radius stays `f32`, because `random()` is `float`.

The `RF_FORKED` branch transcribes as MP writes it. `f_count` starts at zero and nothing assigns it, so the branch never runs, and the `&&` chain keeps its short-circuit so the guard's `Q_random` draw never advances the seed. A two-line site note records that SP sets `f_count` to 3, under porting rule 20.

The grown endpoint lands in a local `end` per row 6. The site note words the entity write as an accepted divergence a portal or a mirror view would make visible, not as parity.

The `mp_renderer` twin drops the sentence DEC-66 ruling 2 overrules and names the live arm on all three deferred blocks. Two further sentences in the `DoBoltSeg` note went stale the moment commit `476930c1` landed - the ones saying `f_count` is not on the carrier and `LIGHTNING_RECURSION_LEVEL` is unported - so they were corrected to match. No `todo!()` in that file was removed and no body in it was ported.

**bcc5bf76, the saber-glow hilt radius.** One expression, `5.5 + rng.random() * 0.25`, with the stand-in note deleted.

## Deviations

Three, all accepted by the user during the lane.

1. **The cylinder segment clamp is unreachable.** The packet asked for both cylinders near enough for `segments` to clamp at 32. `segments = 32 * (1 - length / 1024)` truncates, so 32 needs a zero distance from the eye to the midpoint, which is not a renderable image. `scene_fx_cylinder` runs 28 segments, a full smooth ring. Accepted by the user on 2026-08-28.

2. **Commit 3 split into two neighbors.** The packet bundles both shape goldens in one commit, and the ratified flow puts an eyes-on stop before each PNG. Committing both together would have parked an approved PNG behind an unreviewed one, so `4a5ffa0d` carries the oriented-quad test and PNG and `abc7209c` carries the cylinder test and PNG. No scope moved.

3. **Commits 5 and 6 swapped.** The packet orders the saber-glow re-bless before the electricity golden. The coordinator sequenced the electricity candidate as stop 3 and the saber-glow re-bless as stop 4, so the electricity golden landed first. Both are neighbors and neither depends on the other.

## Commits, with gate results

Every commit ran the full amended battery: `cargo build --workspace`, `cargo test --workspace -- --test-threads=1`, and the five golden suites each as its own serial foreground run.

| Commit | Subject | Warnings | Fixtures |
| --- | --- | --- | --- |
| `476930c1` | the render-side RNG owner and the shape state | 3 expected | 15, all byte-identical |
| `fb868fb7` | the oriented-quad and cylinder arms | 3 expected | 15, all byte-identical |
| `4a5ffa0d` | the oriented-quad golden | 3 expected | 15 identical, 1 new green |
| `abc7209c` | the cylinder golden | 3 expected | 16 identical, 1 new green |
| `232b4d11` | the electricity arm | 0 | 17, all byte-identical |
| `4a1469da` | the electricity golden | 0 | 17 identical, 1 new green |
| `bcc5bf76` | the saber-glow hilt radius | 0 | 17 identical, `scene_saber_glow.png` re-blessed |
| this file | finished file | - | - |

The three warnings on commits 1 through 3 are the `refdef_time`, `rng` and `shape` parameters, unread until commit 4 consumes the first two and commit 5 the third. The amended ruling allows them on an intermediate commit. The bundle's final state builds at zero warnings, confirmed on a forced rebuild of both touched crates.

The lockstep referee did not run. No commit touches `mp_game`, the server, or a `jampded` link-set crate.

Four goldens went through the row-3 stop. The user ruled `scene_fx_oriented_quad.png` and `scene_fx_cylinder.png` a pass on 2026-08-28, and `scene_fx_electricity.png` and the `scene_saber_glow.png` re-bless a pass on 2026-08-29. The harness writes a candidate only under `JKA_GOLDEN_BLESS=1`, because a missing golden aborts before it can write an `.actual.png`, so each candidate was blessed, re-run clean to prove it byte-stable, then held uncommitted until the ruling arrived.

The saber-glow re-bless moved 22 pixels, all inside one 10 by 10 box, all background turning red blade, forming one right-edge column and one bottom-edge row on the red hilt. The backend stream is fixed for that scene, so the two radii are `5.50031` and `5.64090` against the old `5.5`. The first rounds to the same pixels and the second is the one-pixel rim. Nothing else in any suite moved.

## Open gaps

**The vertex cap, per row 7.** The arms build into unbounded `Vec<WorldVertex>` and `Vec<u32>` with no cap and no flush, exactly as every other CPU-built arm in this backend does. The oracle flushes the `tess` batch at `SHADER_MAX_VERTEXES` through `RB_CheckOverflow`, and this backend has no flush point. A dead-fork bolt emits 12 vertices per 20-unit step, so a bolt past roughly 1,600 units would approach the cap. Neither committed bolt comes near it: `scene_fx_electricity` runs 100 units, five steps, 60 vertices per bolt. Recorded here rather than in code, as the row instructs.

**The independent render stream.** DEC-66 ruling 3 already accepts it. This step is where it first reaches a drawing arm, in `create_shape` and in the hilt radius. The observable effect is a different jitter phase on cosmetic geometry, and no gate observes cross-stream interleaving.

**The `oldorigin` write, per row 6.** The oracle's write survives past one draw chain inside a frame. A mirror or portal frame runs `R_RenderView` per view, both views index the one shared `backEnd.refdef.entities` array, and the second `RB_SurfaceElectricity` call reads the `oldorigin` and the `frame` seed the first call mutated. Nothing observes it today, because no committed golden and no current backend path draws a portal view. Whoever lands portal views should revisit this site.

**Two stale doc comments left alone.** `CreateShape` (`crates/mp/renderer/src/tr_surface.rs:108-116`) and `RB_SurfaceSaberGlow` (`:1217-1243`) still say the renderer has no RNG receiver, which DEC-66 ruling 1 superseded. The audit flagged them as optional and the packet's contract does not list them, so they stayed untouched rather than widening scope. They are a one-line fix for a later comment pass.

**`mp_renderer::tr_surface` is still the dead `tess` twin.** Every leaf its electricity, cylinder and oriented-quad bodies would call is a `todo!()`, and this step removed none of them. Row 1 ruled the live arms into `pipeline3d.rs` and left that file to the `tess` backend, if it is ever built.

## Fix round, 2026-08-29

The lane-review walk ruled on the vet report (`.claude/packets/31/step-009/vet.md`, ten findings). Two commits carry the result.

`32db25ab` executes the four verdicts that touch text and changes no code. The diff is comment and doc lines only, proven by a filter over the source hunks.

- Finding 2. The row-6 site note gains one line: the dead `RF_FORKED` branch would also read the un-grown `e.oldorigin`, where the oracle's fork read at `:1107` sees the grown value the write at `:1159` left.
- Finding 4. The two comments saying the seed write "never outlives one draw chain" now carry the row-6 amendment's mechanism. The packet body carried the same sentence and is corrected with them.
- Findings 6, 7 and 8. The quoted comment lines re-break to one sentence per line, all under 150 columns, with no wrap at width. The break inside the noun phrase "end time" is gone. No wording changed. One 153-column line was left alone: it is a preserved Raven comment, which the vet excluded and the porting rules protect.
- Finding 10. The word "ride" is struck from the packet's reusable wording at commit bundle item 5. The commit history stands.

Two verdicts needed no edit. Finding 1, the two unlisted imports, is accepted, because both are private `use` declarations inside write-scope files. Finding 9, "seam" in the module doc, stands as entrenched repo vocabulary. Finding 3, the segment clamp, and finding 5, the two extra stale sentences, were already declared above.

A dated Amendment in `packet.md` records all six verdicts.

Gates on the fix round: `cargo build --workspace` at zero warnings after a forced rebuild of both touched crates, `cargo test --workspace -- --test-threads=1` green, and all five golden suites green serially. Every one of the eighteen committed fixtures stayed byte-identical, which is what a comment-only change must produce.

The vet report itself is committed with this section, so the packet directory holds the whole record.
