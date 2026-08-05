# The RendererFrontend absorption, the landing (gh#31 step-003)

This record freezes the work lane that gave `UiHost` a real `RendererFrontend`, seated the view's `re` slot across the harness, and ran Raven's client register path in the rig for the first time. One opus work lane ran the three-commit bundle in `.claude/packets/31/step-003/packet.md` on branch `gh31-step-003-absorption`, and the lane's own account is `.claude/packets/31/step-003/finished.md`. The landed range is `0def8283..1de794bb`, merged to master at `cd00313b` after a clean clerk lane-review. The packet was drafted by a Fable survey-and-draft agent on its survey record `docs/audits/2026-08-04-step-003-draw-arm-survey.md`, and the user took the four packet rulings at audit on 2026-08-04. Follow-up work goes to gh#31 step-004, the draw-arm migration.

## The lane report, verbatim

Lane complete. All three commits are on `gh31-step-003-absorption`, unmerged and unpushed.

### Commits

1. **`0def8283`** - `refactor(gh#31 s003): UiHost owns a real RendererFrontend, the re slot seated`. The 12 twins collapse into `re: RendererFrontend`, `models` stays a sibling, all 15 destructure/construction sites go two-level, the 56 dot accesses read `host.re.*`, `host_view` gains `re: *mut RendererFrontend` at all 16 call sites, `with_dc` passes null, and the frame-pinned `Arc::clone` lands at the four `execute_frame` sites. The A1 override still stands. Gates: build green, `cargo test --workspace` green, ghoul2 golden byte-identical, both world goldens byte-identical, 7 scene goldens green.
2. **`849251f9`** - `refactor(gh#31 s003): the client register path, and the ghoul2 re-bless`. Override removed, `a2-attempt.patch` deleted, fixture re-blessed, module-doc provenance updated. Gates: build green, `cargo test --workspace` green (516 passed, 0 failed, 18 ignored), ghoul2 golden green on a clean confirming run, world goldens byte-identical, scene goldens green.
3. **`1de794bb`** - the finished file.

### Multiset verdict

**The re-bless reproduced the committed fixture byte for byte.** 22 surfaces, 95472 bytes, multiset digest `f3551f6abb73953d71b6d293306a8889914e34c6a467cb11d44fc632fbef7264`, equal to the gh#35 record, and the draw-surf order is the identity permutation. I first re-derived the digest recipe against the pre-bless fixture and reproduced all 22 recorded prefixes, so the comparison used a checked recipe.

**The reason the order did not move is the model.** Every one of the 81 `mdxmSurfHierarchy_t` entries in `models/players/stormtrooper/model.glm` carries an empty shader name (read straight out of `assets1.pk3`). `R_FindShader` returns slot zero for an empty name, so the client leg writes `shaderIndex = 0`, the same value the server leg forced. The 22 sort keys stay tied and the tie order holds. The client leg definitely ran: `dedicated` is `"0"`, the harness holds no VM, and with the override gone a null slot there is exactly the gh#35 abort.

### Deviations

Two, both procedural, no scope deviation.

1. The brief's scene-golden command uses `--ignored`, but those 7 tests carry no `#[ignore]`, so it reports `0 passed, 7 filtered out` and gates nothing. Every run in this lane used the command without `--ignored`.
2. The re-bless ran and produced identical bytes, so commit 2 carries no fixture diff.

### What a reviewer should look at first

- The survey's section C prediction ("real indices untie the 22 keys, the order moves") is wrong in outcome for this model, and the gh#31 record should carry the correction. Step-004 must not assume the ghoul2 order is now free to move.
- The pin sites, `crates/mp/renderer-gpu/tests/ghoul2_vertex_golden.rs`, `tests/world_golden.rs`, `tests/scene_golden.rs`, and `src/bin/world_harness.rs`, plus the exempt `load_world_and_render` in `crates/mp/renderer-gpu/src/ui_host/boot.rs`.
- The open gap in the finished file on per-field aliasing through the seated slot: the pin closes the one real data write behind a live read (`sim.published`); the rest is the pre-existing `rm`-slot class under the per-slot rule at `crates/mp/renderer/src/hook_install.rs:7-11`, and it is worth a ruling before the harness grows a second in-frame hook.

## The clerk's verification

Lane-review ran with a conformance clerk on opus. It walked all 10 files and all 65 hunks and found no hunk that creates or changes a surface the contract does not list: every surface change maps to a contract line, no dependency, `#[repr]`, cvar, trap arm, or `FrameEvent` variant moved, and `oracle/` is untouched. Its gate re-runs, all its own:

| Gate | The clerk's run |
|---|---|
| `cargo build --workspace` | exit 0, 1m 17s |
| `cargo test --workspace` | exit 0. 516 passed, 0 failed, 18 ignored |
| the ghoul2 golden, serial | exit 0, 1 passed, 72.63s |
| the world goldens, serial | exit 0, 2 passed, 80.98s |
| the scene goldens, serial, no `--ignored` | exit 0, 7 passed, 41.21s |

Fixture verification, three ways: the fixture diff between master and the branch is empty, SHA-256 `897352f5e7e8982a2816f5e6af4120a50b722cf12c16a32df5081caee5d51c3d` on master, branch, and worktree, and blob size 95472 on both. The clerk re-derived the 22 per-surface digests and the multiset digest from a re-checked recipe and matched the recorded values entry for entry. It read the stormtrooper `.glm` out of the retail `assets1.pk3` itself, walked all 81 hierarchy entries with Raven's `childIndexes[numChildren]` stride, and confirmed zero entries carry a shader name. It checked the three supporting cites (`tr_shader.rs:5590-5592` slot-zero on empty, `tr_ghoul2.rs:2116-2121` client write, `server_load.rs:508` server force) and the bone-remap gate (53 bones, remap off).

Its ledger notes, all accepted at close-out without reopening the lane, with dispositions in the packet's close-out Amendment: the finished file's `pub` inventory sentence omits the contract-listed `pub re` field; `re.automap` joins the inert list; four load-time `boot.rs` sites seat the `re` pointer while an `Arc::make_mut` borrow is live, conforming to the contract's seat-everywhere rule and the pre-existing per-slot cast discipline; `ShaderHashTableExists` also casts the slot, unreachable without a VM; the `screenshot` console handlers change from a guaranteed null crash to a live cast, unexercised; one comment rewrite and three call reformats unconfessed. Accepted unverified: the commit-1-only byte-identity claim was not re-run at `0def8283`, corroborated by composition since the tip passes byte-identical on the client leg and the gh#35 control run proved the server leg against the same bytes.

## Rulings, 2026-08-04

The four packet rulings, taken at the packet audit on the recommendations: (1) `frame_data` stays per-call construction and `re.frame_data` sits inert; (2) the frame-pinned registry clone at the four `execute_frame` sites; (3) step-004 reads the published copy through view helpers on `PublishedModel`; (4) step-004 resolves entity `model_type` from the published entry. Rulings 3 and 4 bind the step-004 draft.

The clean-pass merge executed on the packet's disposition. Two items carry forward to step-004: the correction that the ghoul2 draw order is NOT free to move (the fixture has now matched under three configurations), and the open per-slot aliasing ruling for the seated `re` slot before the harness grows a second in-frame hook.
