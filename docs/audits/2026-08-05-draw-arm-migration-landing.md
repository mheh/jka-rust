# The draw-arm migration, the landing (gh#31 step-004)

This record freezes the work lane that migrated the entity draw arms and their decoders onto the published model registry, and the two-sitting lane-review that closed it. One opus work lane ran the bundle in `.claude/packets/31/step-004/packet.md` on branch `gh31-step-004-draw-arms`, pausing once for a session ruling (the bundle restructure) and resuming to completion. The landed range is `72403771..6118f834`, merged to master at `a26a8258`. The packet was drafted by a Fable survey-and-draft agent on `docs/audits/2026-08-04-step-004-draw-arm-migration-survey.md`, with ruling A (the entity image golden) taken at the packet audit and rulings 3 and 4 inherited from step-003. Follow-up work goes to gh#31 step-005, DEC-65 ruling 2, the bone matrices.

## What landed

- `PublishedModel` gains `name`, `md3_ptr(lod)`, and `mdxm_view()`, the one SAFETY home for the base-plus-offset casts (`72403771`).
- The harness publication drain mirrors the `RE_EndFrame` pair before the frame pin at all five `execute_frame` callers, closing the empty-registry hazard the drafting survey found (`b4fa7e5f`).
- The arms and decoders migrate as one commit, because the packet's split could not compile: `r_add_md3_surfaces` and `r_add_ghoul_surfaces` read the published entry, the `tr_main.rs` dispatch splits per ruling 4 (brush test first, `model_type` from the published entry, `BModelEntry.model_type` deleted), `EntityWalkHost` dissolves into `Option<&mut EngineHostView>`, and the whole `models: &RenderModels` plumbing deletes end to end (`ae368895`).
- The entity image golden lands: `tests/entity_golden.rs` and the blessed `entity_duel1.png`, the first committed image of entities drawn from the published registry, the `twinpodcc.md3` pod and the stormtrooper both in frame on the first bless (`e75f6953`).
- Two lint commits (`71cb4b41`, `6118f834`) and the finished file (`067545f2`).

The consequence pair: the MD3 arm now runs un-gated on the live client, drawing weapons and map objects from package-carried blocks, while the ghoul2 legs keep their host gate until DEC-65 ruling 2 crosses the bone matrices. The ghoul2 vertex fixture stayed byte-identical through the entire migration, its fourth configuration without a byte moving.

## The lane's account

The finished file `.claude/packets/31/step-004/finished.md` is the lane's own record: assumptions keyed to commits, per-commit gate results, the bless provenance with the six origin constants, and five confessed deviations. Its headline gate table: build and workspace tests green at every commit, the ghoul2 tripwire byte-identical at every code commit, and the entity golden blessed once then confirmed on a clean run at zero channel tolerance.

The lane paused once, mid-work, on the bundle defect: the packet's commits 2 and 3 could not compile separately, because commit 2 dissolves the field commit 3 reads. The session accepted the restructure as an Amendment (drain first as its own behavior-neutral commit, arms and decoders combined), which is the shape that landed.

## The review, two sittings

The conformance clerk's first gate battery died overnight against a pathological `target/debug/deps` holding 1,122,970 files, the environment failure recorded in the memory ledger as `target-deps-bloat-trap` and repaired with a `cargo clean` (deps fell to about 4,700 files, and the full golden battery fell from stalled to about 83 seconds of test time). The clerk's diff walk survived, and its resumed sitting re-ran every gate itself:

| Gate | The clerk's run |
|---|---|
| `cargo build --workspace` | 10.08s, zero warnings |
| `cargo test --workspace` | 517 passed, 0 failed, 19 ignored |
| ghoul2 vertex golden | 1 passed, 15.64s |
| world goldens | 2 passed, 39.28s |
| scene goldens | 7 passed, 3.86s |
| entity golden | 1 passed, 17.52s |

Fixture verification: the ghoul2 tripwire SHA-256 `897352f5e7e8982a2816f5e6af4120a50b722cf12c16a32df5081caee5d51c3d` identical on master, the branch, and the worktree after all runs; `entity_duel1.png` at 592713 bytes, SHA-256 `a1ccf23a291ddf5b4794c2bef49c23881ad2522042e71247b403fb460bd767d5`, an 800x600 RGBA PNG; every other fixture untouched by the diff.

The findings and their dispositions live in the packet's 2026-08-05 close-out Amendment: the four under-enumerated pass-through signatures (packet imprecision, the mandatory chain between `R_RenderView` and the walk), four equivalence-checked ledger mismatches, the column-wrapped new test file (fixed at `6118f834`, goldens re-confirmed), and the accepted unverified list. Two items carry to step-005: the `boot.rs` spike resolves entities against a registry no drain fills (inert while the spike draws no entity), and the per-slot `re` aliasing ruling stays deferred until the rig grows an in-frame hook.

## Ruling, 2026-08-05

The user ruled the merge after the dispositioned review, and ruled the batch push in the same breath: the accumulated master history since `e46e7ef7`, three merged census steps plus the gh#35 close, goes to origin with CI watched.
