# gh#31 step-004 - the draw-arm migration, finished

The lane ran on branch `gh31-step-004-draw-arms`, cut from master at `19a64fe7`, in the main working tree. Nothing merged, nothing pushed, and no pull request opened. The contract is `.claude/packets/31/step-004/packet.md`, with ruling A taken and the 2026-08-05 bundle Amendment applied.

## Commits

| Commit | Subject |
|---|---|
| `72403771` | `feat(gh#31 s004): the PublishedModel view helpers` |
| `b4fa7e5f` | `test(gh#31 s004): the harness publication drain` |
| `6309869b` | `process(gh#31 s004): amendment - the bundle restructures on compile reality` (session-written, not lane work) |
| `ae368895` | `refactor(gh#31 s004): the draw arms and decoders read the published entry` |
| `e75f6953` | `test(gh#31 s004): the entity image golden` |
| `71cb4b41` | `style(gh#31 s004): comment lint over the lane's own lines` |
| this file | `docs(gh#31 s004): the finished file` |

## Gate results per commit

| Commit | `cargo build --workspace` | `cargo test --workspace` | ghoul2 vertex fixture | world goldens | scene goldens | entity golden |
|---|---|---|---|---|---|---|
| `72403771` | green | green, zero failures | not run | not run | not run | did not exist |
| `b4fa7e5f` | green | green, 135 suites, zero failures | **byte-identical** | byte-identical | 7 green | did not exist |
| `ae368895` | green | green, zero failures | **byte-identical** | byte-identical | 7 green | did not exist |
| `e75f6953` | green | green | byte-identical at the parent | byte-identical at the parent | green at the parent | blessed, then green on a clean confirming run |
| `71cb4b41` | green | green, 136 suites, zero failures | **byte-identical** | not re-run | not re-run | not re-run |

The ghoul2 vertex fixture `tests/goldens/ghoul2_verts_stormtrooper.bin` is byte-identical at every code commit in this lane. It never entered a re-bless state, and the committed blob is unchanged from master. That is the lane's central evidence that the migration moved no behavior.

All golden runs used `--test-threads=1`. `dedicated` stayed `"0"` in every run. The scene goldens ran without `--ignored`, because those seven tests carry no `#[ignore]`.

## Assumptions and choices, keyed to commits

**`72403771`, the view helpers.** `md3_ptr` returns a raw pointer rather than a view, because every consumer (`r_cull_model`, `r_compute_fog_num`, the two surface walks) already takes one. The two helpers share one SAFETY argument, and `mdxm_view` cites `md3_ptr` rather than repeating it. `mark_block` fills `name` through the existing `read_qpath` helper, which the module already imports. The unit test writes a real `ofsEnd` into the test block so the `mdxm_view` assert reads a value back through the view, which proves the helper resolved the block base plus the stored offset rather than any pointer that happens to be inside the block. `mdxa_view` did not land, per the contract and survey section B.

**`b4fa7e5f`, the drain.** The drain pair sits before the frame-pinned `Arc::clone` at all four `execute_frame` callers, and the comment at each site states the ordering as a contract line. This commit adds no reader of `assets.models`, so it is behavior-neutral by construction, and the three golden batteries confirmed that.

**`ae368895`, the migration.** Three judgment calls beyond the contract's text:

1. `g2_compute_lod` takes `num_lods: i32` rather than the entry. The contract allowed either. `numLods` is the only field the oracle body reads, so the narrower parameter keeps the signature honest about what the function needs.
2. `decode_md3_surface` and `decode_ghoul2_surface` take `&ModelBlocks` rather than `&RenderAssets`. The contract allowed either. The narrower parameter keeps the two decoders free of the whole asset bundle, and the call sites pass `&assets.models`.
3. `r_compute_lod` folds Raven's `numLods < 2` test and the LOD-0 header resolve into one `match`. The single-LOD arm and the absent-header arm produce the same value, and the guard order preserves Raven's control-flow behavior.

**`e75f6953`, the entity golden.** The scene reuses two existing recipes without changing either locked test. The four stat asserts (both arms drew a surface, neither decoder failed) stand between a world-only render and a bless, which is what makes a blank frame fail loudly rather than bless.

**The lint commit.** Comment text only, no code. It rewrites every comment line this lane added or touched to the house line-break rule: one sentence per line, semantic breaks only, and a 150-column ceiling. Where the lane edited a legacy 72-column-wrapped block, the whole block was re-linted rather than left half-converted. Two pre-existing semicolons inside `r_compute_lod`'s doc comment were removed as part of that. Preserved Raven comments inside `r_compute_lod` were re-indented but not reworded, per porting-rules §"Preserve Raven comments".

## The entity golden's bless provenance

- Fixture: `crates/mp/renderer-gpu/tests/goldens/entity_duel1.png`
- SHA-256: `a1ccf23a291ddf5b4794c2bef49c23881ad2522042e71247b403fb460bd767d5`
- Byte size: 592713
- Blessed 2026-08-05 with `JKA_GOLDEN_BLESS=1`, on the client register path, with `dedicated` at `"0"`.
- Confirmed on a clean run without the bless variable, zero differing pixels at `CHANNEL_TOLERANCE = 0`.

**First image verdict: both arms draw.** The blessed image shows the `twinpodcc.md3` pod left of center and the stormtrooper in its base T-pose right of center, both fully inside the frame, both carrying real shaders and textures, against the textured duel1 room with correct depth ordering against the wall and floor. Neither entity culled and neither clipped an edge, so no origin adjustment was needed and the fixture was blessed once.

The chosen origin constants, all relative to the spawn-origin eye at `FROZEN_TIME_MS = 12345`:

| Constant | Value |
|---|---|
| `MD3_FORWARD_DIST` | `260.0` |
| `MD3_SIDE_OFFSET` | `110.0` |
| `MD3_DROP` | `60.0` |
| `GHOUL2_FORWARD_DIST` | `170.0` |
| `GHOUL2_SIDE_OFFSET` | `-70.0` |
| `GHOUL2_DROP` | `40.0` |

Both entities carry a zero radius, which pins the Ghoul2 LOD to 0. The constants are part of the fixture, and the module doc says so.

## Deviations

1. **The bundle restructure.** Contract commits 2 and 3 cannot compile separately: commit 2 dissolves `EntityWalkHost.models`, which is the exact field commit 3's `backend_models` reads. The lane split the drain out as its own behavior-neutral commit and landed the arm and decoder work as one. The session ruled this accepted and recorded it as the 2026-08-05 packet Amendment at `6309869b`. Scope is unchanged.
2. **`boot.rs` is a fifth `EntityWalkHost` construction site.** The contract lists four. `crates/mp/renderer-gpu/src/ui_host/boot.rs:774` builds a fifth for the `load_world_and_render` spike, which calls `R_RenderView` directly rather than `execute_frame`. Edited under the write-scope catch-all for compile fallout: the construction is replaced by `Some(&mut engine_view)` and one import drops. No drain landed there, because it is not an `execute_frame` caller and the spike draws no entity. Recorded in the same Amendment.
3. **`None` early-return skips in the migrated arms.** Where the old code read a raw `model_t` unconditionally, the published lookup returns `Option`, so `r_add_md3_surfaces` gains two early returns and `render_surfaces` gains one. Each is unreachable in a live frame and each is the defined behavior where a stale reference could reach it, which is the disposition survey section G anticipated. Recorded in the same Amendment.
4. **The drain commit's golden battery ran on its own tree, not per contract-commit boundary.** The contract gates commit 2 with the full battery. Because the drain became its own commit, the battery ran twice: once against the drain alone, and once against the migration. Both were clean.
5. **The lint commit is not in the contract's bundle.** The house style requires it, and the step-002 lane set the precedent.

## Open gaps

- **The live client now draws MD3 entities and still draws no Ghoul2 player.** `execute_package` passes no host, so after the ungate the render thread draws weapons and map objects from its published blocks while players stay dark until DEC-65 ruling 2 crosses the per-entity bone matrices. This lane did not exercise the live client, so that consequence is reasoned from the code, not measured.
- **The `MOD_BAD` null-axis fall-through now runs ungated render-side for the first time.** No golden covers it, because no fixture scene submits an `RT_MODEL` whose handle resolves `MOD_BAD` without a Ghoul2 token. The path is Raven's default-shader push and is unchanged in shape.
- **The Ghoul2 legs inside the `MOD_BAD` arm skip the whole entity when no host is present.** That reproduces today's behavior, where the entire match was gated. A render-side caller therefore draws neither the Ghoul2 model nor the default-shader fall-back for a token-carrying `MOD_BAD` entity. Ruling 2 closes this when the matrices cross.
- **The per-slot `re` aliasing ruling stays deferred.** Survey section E verified that this step adds no in-frame hook, no new cast of the seated slot, and no console-handler execution, so the two ghoul2 registration hooks remain the only in-frame `re`-slot casts. The ruling comes due with the first step that adds an in-frame hook or runs a console handler in the rig.
- **The entity golden runs on one GPU.** `CHANNEL_TOLERANCE` is 0, so a different driver may need the tolerance widened, the same caveat the world goldens carry.
- **`mdxa_view` still has no consumer** and did not land, per the contract.
