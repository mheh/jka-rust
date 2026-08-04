# Finished - packet gh#31 step-001, the ghoul2 token conversion

Lane complete. Five commits on `master`, all local, nothing pushed.

## Commit list and gate results

1. `7833dc68` - **ghoul2(gh#31 w1): token helpers move to mp_engine_ghoul2, deep_copy goes in-place**.
   Gates: `cargo build --workspace` green. `cargo test --workspace` green, no failures. World goldens byte-identical, `golden_world_duel1` and `golden_world_ffa2` both pass under `--ignored`. `ghoul2_vertex_golden` run in its own `cargo test` invocation, and its default (non-ignored) leg passes. See "Deviations" for the `--ignored` leg.
2. `49419369` - **client(gh#31 w2): cl_ui ghoul2 arms read the token, not a pointer**.
   Gates: `cargo build --workspace` green. `cargo test --workspace` green, no failures.
3. `92e136de` - **client(gh#31 w3): cl_cgame ghoul2 arms read the token, not a pointer**.
   Gates: `cargo build --workspace` green. `cargo test --workspace` green, no failures. World goldens byte-identical, both tests pass under `--ignored`.
4. `129cc053` - **server(gh#31 w4): sv_game and sv_world ghoul2 arms read the token, not a pointer**.
   Gates: `cargo build --workspace` green. `cargo test --workspace` green, no failures. Lockstep referee suite green: `referee_solo`, `referee_idle`, `referee_melee_brawl`, `referee_force_duel`, `referee_real_duel1_idle`, `referee_real_duel1_walk`, `referee_real_duel1_combat`, `referee_real_ffa1_items`, and `regenerate_logs`, 9 passed and 0 failed. No scenario self-skipped, because `JKA_REF_BASEPATH` pointed at the installed assets. The committed reflogs are unchanged after `regenerate_logs`.
5. This commit - **docs(gh#31 w5): DEC-65 ruling 3 close-out**.
   Gates: `cargo build --workspace` green. `cargo test --workspace` green, no failures.

## Assumptions and choices, keyed to their commits

**Commit 1.**

- The new file is `crates/mp/engine/ghoul2/src/token.rs`, a crate-root module. The two helpers are free functions, not a type, so the one-type-per-file rule does not select a `shared/` subfolder for them.
- `deep_copy`'s empty-source arm. The packet states the destination keeps its handle when one exists and allocates only for `mItem: 0`, and it does not state what an empty source does. The body keeps the destination handle and gives it an empty vector, where Raven frees the destination outright. The one caller guards on `g2From.IsValid()` (`api_models.rs:653`), so a trap arm cannot reach that state, and the doc comment records the divergence. A `mItem: 0` destination with an empty source returns without allocating, which matches Raven.
- `deep_copy` frees the replaced instances' bone caches before it overwrites them. Raven's `Free()` reaches `DeleteLow`, which frees them (`G2_API.cpp:319-326`), and the in-place body would otherwise strand them in the sibling arena. This is inside the one method the surface contract names, so it is in scope.
- The round-trip unit tests moved from `tr_scene.rs` to `token.rs` with the helpers, and a third case was added for handle `0` against the null token. DEC-32 gives one canonical home, so nothing stayed behind.

**Commit 2.**

- The pre-flight class-2 audit found no uncovered value-passed `mItem` mutator, so no arm hit the STOP condition. The full result is in the commit body. The two saveload mutators (`misc.rs:1676`, `misc.rs:1959` through `g2_load_ghoul2_model`, and `api_saveload.rs:144`) have no trap arm in any of the three dispatch files, which is the one addition the audit makes over the packet's own inventory.
- `UI_G2_INITGHOUL2MODEL` writes the slot back unconditionally, not only when the slot read null. The init path can allocate through `push_back` on a `mItem: 0` cell, and writing back an unchanged token is a no-op.

**Commit 3.**

- `CG_FX_PLAY_BOLTED_EFFECT_ID` converts as a class-2 value-passed arm. It reads `g2v.mItem` and hands the `i32` to `FX_PlayBoltedEffectID`, so FX keeps retaining the handle it always retained.
- The token decode is a safe call, so the arms whose only `unsafe` was the old pointer dereference lost their blocks, and the misleading `SAFETY:` prefixes on those lines became plain notes. The crate's warning count returns to its pre-change value.

**Commit 4.**

- `sv_world.rs` builds the diagnostic cell and takes a read-only `&Ghoul2System` for the `size(g2) > 0` guard and the `get(g2, 0)` read. The guard sits inside the existing `sv_showghoultraces` test, so a disabled diagnostic costs nothing.
- The six `sv_game` arms that only index through `get_mut` bind the cell without `mut`, because `CGhoul2Info_v::get_mut` takes `&self`.

**Commit 5.**

- The `tr_scene.rs` RECONCILE block was part of the `ghoul2_token_decode` doc comment, so it left `tr_scene.rs` with the helper in commit 1. Its replacement lives in the `token.rs` module doc and records DEC-65 ruling 3 as executed, with the arm count.
- `placeholders.rs:171` cited `tr_scene.rs` as the helper's home. It now cites `mp_engine_ghoul2::token`.

## Deviations

One, and it is a pre-existing defect, not a change this lane made.

The `--ignored` leg of `ghoul2_vertex_golden` aborts in `re_from_view` (`crates/mp/renderer/src/hook_install.rs:58`) on a null `re` slot. It does the same at `HEAD` before any of this work, in both the debug profile (null-pointer-dereference panic, SIGABRT) and the release profile (SIGSEGV), verified by stashing the work and re-running both. The test's default leg, which is what `cargo test --workspace` runs, passes as an `#[ignore]`d skip. The packet's gate for commit 1 says to run the test in its own invocation, which was done; the golden itself could not execute on either side of the change.

## Open gaps

- `crates/mp/game/src/g_turret_G2.rs:189` carries a stale comment: "the engine derefs arg1 as `CGhoul2Info_v **` (`sv_game.rs:3326`)". The engine now reads a token slot there, and the line number moved. The file is outside this packet's write scope, so it was left alone. It is a comment only, with no behavior attached.
- `docs/decisions.md:1534` (the DEC-65 entry) still describes the split in the present tense and cites `sv_game.rs:3116-3125`. The DEC ledger is outside this packet's write scope, so the entry was not edited to mark ruling 3 as executed.
- The `ghoul2_vertex_golden` rig defect above needs its own ticket. It blocks the image-golden half of the DEC-65 gate for later steps.
