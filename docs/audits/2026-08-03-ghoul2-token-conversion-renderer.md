# Logic audit - DEC-65 ruling 3, the ghoul2 token conversion (packet gh#31 step-001)

Read-only Fable audit, 2026-08-03. Subject: the plan to convert the module-visible ghoul2 `void*` slot from a `Box<CGhoul2Info_v>` heap pointer to the `Ghoul2Handle + 1` token, ahead of wave-3 renderer entity drawing. The packet under audit: `.claude/packets/31/step-001/packet.md` (draft commit `8829335a`). The follow-up amendment draft lands as the second part of this file.

## A. mItem mutation reachability - DEFECT

The packet claims only init, duplicate, and clean mutate `mItem`, and that all three are slot-address arms. The call graph refutes this in three places.

The complete mutator set on `CGhoul2Info_v` is `alloc`, `free`, `clear`, `deep_copy`, `assign`, `assign_item`, plus lazy allocs inside `resize` and `push_back` when `mItem == 0` (crates/mp/engine/ghoul2/src/shared/cghoul2_info_v.rs:49-186). An exhaustive grep of the ghoul2 crate finds every trap-reachable call: api_models.rs:310,354 (init), 348-355 (clean), 447,497,549 (remove family), 653 (copy instance), 697 (copy specific). The ragdoll and IK family (api_ragdoll.rs, ragdoll.rs), bones, bolts, surfaces, collision, and gore never call a mutator on the module cell. ragdoll.rs:833 builds a local wrapper from a handle and never writes it back. `g2api_set_ghoul2_model_indexes` (SETMODELS) is a compiled no-op (api_models.rs:582-589). The saveload free (api_saveload.rs:144) has no trap arm in any of the three dispatch files.

The three refutations:

1. **The remove family frees.** `g2api_remove_ghoul2_model` and `g2api_remove_ghoul2_models` call `ghoul2.free(g2)` when the vector empties (api_models.rs:497, 549), which zeroes `mItem`. The arms hold the slot address (`vma(...) as *mut *mut CGhoul2Info_v`: sv_game.rs:3389, 3395, cl_cgame.rs CG_G2_REMOVEGHOUL2MODEL, cl_ui.rs UI_G2_REMOVEGHOUL2MODEL), yet the packet lists them under the class-2 guard-asymmetry note with no write-back. Without write-back the module keeps a stale token. The concrete crash: remove-to-empty, then re-init through the same slot. `from_token` yields a stale nonzero `mItem`, `push_back` skips `alloc` (cghoul2_info_v.rs:182-185), the push lands in the `null_slot` scratch (info_array.rs:156-162), and `g2api_init_ghoul2_model`'s final read `ghoul2.get(g2, model)` indexes an empty slice and panics (api_models.rs:335, cghoul2_info_v.rs:154). Today the shared cell carries `mItem 0` forward and re-init works.
2. **`g2api_copy_ghoul2_instance` mutates the value-passed destination.** COPYGHOUL2INSTANCE passes both args by value (sv_game.rs:3342-3343, cl_cgame.rs:3138-3139, cl_ui.rs:2213-2214). The callee runs `g2_to.deep_copy` (api_models.rs:653), which frees the old handle and allocs a fresh one (cghoul2_info_v.rs:99-101). Under tokens the new `mItem` dies in the discarded stack cell, the module keeps a stale token, and the fresh arena slot leaks with no live holder. Mitigation: the arm is module-dead. No Rust module calls the trap wrapper, and the oracle's game, cgame, and ui sources contain no `trap_G2API_CopyGhoul2Instance` call site either.
3. **`g2api_copy_specific_g2_model` can alloc through the value-passed destination.** `ghoul2_to.resize(g2, model_to + 1)` (api_models.rs:697) allocs when the destination cell is `mItem 0`, a state that exists today after remove-to-empty (cell kept, `mItem` zeroed). Live callers: g_client.rs:1714, 3924, cg_weapons.rs:2526, 2541, 2597, cg_servercmds.rs:666, cg_players.rs:10880, 12746. At every site the destination is a live player or weapon instance, so the alloc path needs the remove-to-empty precondition first. See C for the Raven comparison.

The packet's own pre-flight audit (packet.md:34-36) would trip on items 2 and 3 and STOP. The classification error on the remove arms would not trip it, because the packet pre-sorted those arms into class 2 by hand.

## B. Aliasing semantics - CONFIRMED-SAFE

All instance content (bones, bolts, surfaces, gore) lives in the arena keyed by `mItem`, not in the 4-byte cell. Two slots that alias today share content through the same `mItem`, and two independent token snapshots decode to the same `mItem` and reach the same arena vector. Content aliasing is therefore preserved exactly. Only mutation of `mItem` itself breaks under snapshots, and that set is precisely the A-defect list.

The copy sites checked: cg_ents.rs:969 (item slot into refEntity), cg_ents.rs:415, cg_view.rs:878, cg_weapons.rs:141, cg_draw.rs:667 (cent slot into refEntity), cg_predict.rs:1024, 1041, 1500 (pmove), cg_players.rs:13123-13174 (frame_hold), g_client.rs:3579 (`precachedKyle` as duplicate source), cg_weapons.rs:2532-2543 (`ci.ghoul2Weapons` as copy source). All are value reads into value-consuming traps or into refEntity, none mutates `mItem` through the copy.

The one identity comparison, `frame_hold != centGhoul2` (cg_players.rs:13127), keeps its truth table: aliased copies compare equal as tokens, independently created instances get distinct handles and compare unequal. Clean through one alias then use through the other improves under tokens: today a dangling deref or a stale reused Box address, under tokens a generation-checked handle that `is_valid` (info_array.rs:132-137) rejects into empty-vector no-ops, which is also Raven's own semantics for a gone handle (the function-static null vector, mirrored at info_array.rs:139-151).

## C. Null-state round trip - CONFIRMED-SAFE, with one NEEDS-RULING edge

The unguarded sv trio (sv_game.rs:3383-3397): Raven binds `**ghlRemove` and calls `.size()`, which reads `mItem` through a null `this` and crashes (G2_API.cpp:783-885). The `mItem 0` stack cell answers `size 0`, so has returns qfalse and both removes return qfalse. That is the one defined behavior for Raven's crash, a clean porting-rules §19 divergence, and the packet already notes it. The client-side arms carry explicit null guards with the same answer (cl_cgame.rs CG_G2_HASGHOUL2MODELONINDEX, cl_ui.rs:2252-2275).

Clean and init round-trip correctly. Raven's clean nulls the slot, the plan's `mItem 0 -> to_token -> null` matches. A failed init still pushes a `-1` entry into a real arena slot (api_models.rs:298-323), so the token is non-null, which matches Raven's non-null empty-ish object.

The edge that needs a ruling: Raven's removes null the module slot (`delete *ghlRemove; *ghlRemove = NULL;`, G2_API.cpp:868-869, 956-957). Today's Rust arms do NOT null the slot and leave a live empty cell, an existing divergence. Amendment 1 below (write-back at remove) restores Raven's exact observable state. But it changes one today-behavior: today, copy-specific into a post-remove empty cell allocs and revives the instance (api_models.rs:695-697), while under write-back the slot is null and the arm no-ops on the null guard (sv_game.rs:3354-3356). Raven itself nulls the slot at remove, so Raven also no-ops at that follow-on copy. The write-back version therefore matches Raven and today's Rust is the outlier. This deserves one sentence of ruling in the packet, not a design change. The same reasoning covers duplicate-with-invalid-source, where Raven leaves a non-null empty object and the plan leaves null: distinguishable only through the same copy-specific path.

## D. Long-term retention - CONFIRMED-SAFE

- The FX system is the only engine subsystem that retains a ghoul2 reference across trap returns, and it already retains the `i32` `mItem`, not a pointer. The CG_FX_PLAY_BOLTED_EFFECT_ID arm decodes the slot value and passes `g2v.mItem` (cl_cgame.rs:2656-2667). Storage is `i_ghoul2: i32` (fx_scheduler.rs:482, clight.rs:77, cparticle.rs:108), and every later use runs the generation check (`Ghoul2IsValid` -> `info_array.is_valid`, fx_host.rs:644-651). Token conversion changes nothing here.
- `g2api_attach_instance_to_ent_num`, `clear_attached_instance`, and `clean_ent_attachments` are compiled no-ops (api_bolts.rs:552-579).
- sv_world.rs:785, 797 reads the gentity slot at trace time and holds nothing across calls.
- `CG_GET_GHOUL2` (cg_main.rs:5623-5634) returns the raw slot value to the engine, but no engine consumer exists in the Rust tree or the oracle client. Dead path.
- fx_host.rs:693 builds its stack cell from the retained `i32` handle, not from a module slot. It stays consistent, per packet.md:32.

## E. Scheme coherence end to end - CONFIRMED-SAFE

The refEntity copy sites are cg_ents.rs:415, 969, cg_view.rs:878, cg_weapons.rs:141, cg_draw.rs:667, and cg_players.rs:13135, 13160, 13174. The renderer already decodes that field as a token: tr_scene.rs:427 (`ghoul2_token_decode(ent.ghoul2)`), tr_main.rs:1715 and the encode at 1792, and the render-side struct is already `ghoul2: Option<Ghoul2Handle>` (render_state/placeholders.rs:175). No consumer assumes pointer-ness beyond null tests and the one equality at cg_players.rs:13127. No hashing, no ordering, no arithmetic exists in modules or engine. ui_saber.rs passes either the slot address (line 757, 1348) or the slot value (789, 1374) straight into traps, and the display-context `ghoul2Ptr` (ui_display_context.rs:649-701) is a pass-through parameter, not stored state. tr_scene.rs:379 only null-tests. The one raw-value export, `CG_GET_GHOUL2`, has no consumer (see D).

## F. Clean-through-value - DEFECT, folded into A

Exactly one trap path frees through a value argument: the COPYGHOUL2INSTANCE destination, whose `deep_copy` frees the old instance with no reachable slot to update (A item 2). Raven at the same site mutates the pointee, so the module slot stays correct. The token plan changes observable behavior there. The arm is dead in both trees, so the exposure is latent, but the arm body must still be ruled, since a converted decode-call-discard body silently leaks an arena slot if the trap is ever exercised. Clean and the remove family all receive the slot address, so no other free escapes write-back reach.

## Required packet amendments

1. Reclassify REMOVEGHOUL2MODEL and REMOVEGHOUL2MODELS (sv_game.rs:3386-3397, cl_cgame.rs, cl_ui.rs remove arms, 4 arms total) as write-back arms: decode, call, `to_token` back through the existing `pp`. This closes the stale-token re-init panic and restores Raven's `*ghlRemove = NULL` (G2_API.cpp:868-869, 956-957) that today's Rust omits. HASGHOUL2MODELONINDEX stays read-only, no write-back.
2. Rule the three COPYGHOUL2INSTANCE arms. They are value-passed `mItem` mutators, the pre-flight STOP condition, and they cannot write back. Evidence for the ruling: no module caller in the Rust tree, no `trap_G2API_CopyGhoul2Instance` caller in oracle game, cgame, or ui. Options are a documented dead-arm conversion with a leak note, or an error arm.
3. Add a §19-style note for the three COPYSPECIFICGHOUL2MODEL arms: the empty-destination alloc path (api_models.rs:695-697) becomes a null no-op after amendment 1. Raven no-ops there too because Raven nulls at remove, so today's Rust is the outlier, and the note should say so.
4. Reword the pre-flight audit so the has/remove family is audited as slot-address, not value-passed. The current guard-asymmetry sentence (packet.md:30) pre-sorts them wrongly.
5. Minor: at sv_world.rs:785 the `from_token` cell feeds `.get(g2, 0)`, which panics on an empty instance when `sv_showghoultraces` is on. Add a size guard note at the site.

## Why, and whether it is worth it

The collision is real and already latched. The renderer decodes `refEntity_t.ghoul2` as `handle + 1` today (tr_scene.rs:427, tr_main.rs:1715), while cgame fills that field with the Box pointer at six live copy sites. The moment wave 3 draws ghoul2 entities, decode of a pointer gives `pointer - 1` truncated into the masked `is_valid` lookup: almost always a silent no-draw, and on an index-plus-generation coincidence, a draw of the wrong instance. Doing nothing fails wave 3.

The two cheap outs both lose on inspection. Deref-decode (renderer reads `mItem` through the pointer at scene-add) is same-thread and one line, but the CLEANMODELS arm frees the Box (`Box::from_raw`, sv_game.rs:3407), so any refEntity copy that outlives a clean derefs freed memory, and the raw pointer becomes permanent module ABI against DEC-65's explicit ruling. Tokens-on-cgame-only splits one `void*` convention across three dispatch files while sv_world, FX, and ui still touch the same scheme, and saves little, because cgame alone is 46 of the 115 arms. The ruled fallback 3b (pointer-to-handle map at the render trap) keeps every stale-pointer hazard and adds a map that must be invalidated at every free, forever.

The evidence favors 3a strongly. The Box cell holds nothing but the 4-byte handle (cghoul2_info_v.rs:11-15), so the heap indirection buys only mutable-cell aliasing, and this audit shows that aliasing carries real semantics in exactly the arms that hold the slot address anyway, once the remove family is classified correctly. The g2api crate needs zero change. The FX and sv_world paths already speak `mItem`. Raven's own remove semantics null the slot, so write-back is the native shape of the API, not an imposition. The defects found reclassify four arms, add two short rulings, and add one site note. They raise the cost by hours, not by architecture, and amendment 1 makes the port more Raven-faithful than the current code. Verdict: the conversion is worth doing, and it should not start until the five amendments are in the packet, because the plan as written ships a reachable panic (remove-then-reinit) and a silent-loss arm.

## Post-audit ruling

The user ruled amendment 2 as option (c), 2026-08-03: in-place `deep_copy`. The destination keeps its handle when one exists and the arena contents are replaced in place, which matches Raven's `g2To.DeepCopy(g2From)` object-identity-preserving assignment (`G2_API.cpp:2239-2259`, verified against the oracle first-hand). The alloc path remains only for a `mItem: 0` destination.

## Follow-up: the amendment draft (applied to the packet verbatim)

The same auditor drafted the packet fold-in after the ruling. The replacement sections below now live in `.claude/packets/31/step-001/packet.md` - this is the frozen record of the draft as delivered. The class arithmetic: 6 (init/duplicate) + 3 (clean) + 4 (remove family) + 3 (has, read-only through the slot address) + 99 value-passed = 115 arms, consistent with the survey count after the seven slot-address arms leave the value-passed set. The `deep_copy` in-place change sits in commit 1 because it is a `mp_engine_ghoul2` crate change every dispatch commit depends on.

Key deltas the draft introduced over the original packet:

- A new write-back class 4 for the remove family, restoring Raven's `*ghlRemove = NULL` (`G2_API.cpp:868-869,956-957`).
- `CGhoul2Info_v::deep_copy` named in the surface contract as a behavior change: in-place, handle-preserving on a live destination, alloc only on `mItem: 0`, doc comment records the divergence from the old free-then-alloc.
- `COPYGHOUL2INSTANCE` x3: null-destination guard returning -1 without calling (§19 - Raven crashes on the null reference), module-dead note at the site.
- `COPYSPECIFICGHOUL2MODEL` x3: §19 site note that the post-remove no-op matches Raven and the current Rust realloc-through-kept-cell is the outlier.
- The pre-flight audit names the full mutator inventory (`alloc`, `free`, `clear`, `deep_copy`, `assign`, `assign_item`, lazy allocs in `resize`/`push_back`, `cghoul2_info_v.rs:49-186`) and audits the has/remove family as slot-address.
- `sv_world.rs:785` diagnostic print guarded with `size(g2) > 0`.
- Commit 1 gains the `deep_copy` change plus a unit test that a live destination keeps its handle across a copy.

## Follow-up: the landing (work lane, 2026-08-03)

The work lane executed the amended packet as the five-commit bundle. The commits are `7833dc68`, `49419369`, `92e136de`, `129cc053`, and `62b2e7af`, all on `master` and held local. The lane's own file is `.claude/packets/31/step-001/finished.md`, which carries the per-commit assumptions and the deviation note. The lane report follows verbatim, with em dashes normalized to plain hyphens for house style.

### Lane report

Lane complete. Five commits on `master`, local only, nothing pushed.

#### What landed

| # | Commit | Subject |
|---|---|---|
| 1 | `7833dc68` | token helpers move to `mp_engine_ghoul2`, `deep_copy` goes in-place |
| 2 | `49419369` | `cl_ui.rs` arms read the token |
| 3 | `92e136de` | `cl_cgame.rs` arms read the token |
| 4 | `129cc053` | `sv_game.rs` + `sv_world.rs` arms read the token |
| 5 | `62b2e7af` | DEC-65 ruling 3 close-out + finished file |

Raw `Box<CGhoul2Info_v>` pointers no longer cross any module boundary. `Box::new`/`into_raw`/`from_raw` are gone from all three dispatch files. All 13 write-back arms carry the slot address, and the remove family's write-back restores Raven's `*ghlRemove = NULL` that the pointer code omitted.

#### Gate results

- Every commit: `cargo build --workspace` and `cargo test --workspace` green, zero failures.
- Commits 1 and 3: world goldens byte-identical (`golden_world_duel1`, `golden_world_ffa2` pass under `--ignored`).
- Commit 4: **lockstep referee suite green** - all 4 mock scenarios, all 4 real-map scenarios, and `regenerate_logs`; 9 passed, 0 failed, no self-skips, committed reflogs unchanged.

#### Pre-flight class-2 audit - no STOP

No uncovered value-passed `mItem` mutator exists. The exhaustive grep of `mp_engine_ghoul2` found every mutator call site outside the type's own file; each is covered by a named class or rule. One addition over the packet's inventory: the two saveload mutators (`misc.rs:1676`/`:1959` via `g2_load_ghoul2_model`, and `api_saveload.rs:144`) have **no trap arm** in any dispatch file, so they are not trap-reachable.

#### One deviation, pre-existing

The `--ignored` leg of `ghoul2_vertex_golden` aborts in `re_from_view` (`crates/mp/renderer/src/hook_install.rs:58`) on a null `re` slot. **It does the same at HEAD before this work**, in both debug (SIGABRT) and release (SIGSEGV) - verified by stashing and re-running. It is a rig defect, and it blocks the image-golden half of the DEC-65 gate for later steps.

#### Judgement calls worth your eye

- **`deep_copy` empty-source arm.** The packet did not state it. The body keeps the destination handle with an empty vector where Raven frees outright; the sole caller guards on `g2From.IsValid()`, so no trap arm reaches it. Documented at the site.
- **`deep_copy` frees replaced bone caches.** Raven's `Free()` did this via `DeleteLow`; the in-place body would otherwise strand them. Inside the one method the contract names.

#### Open gaps (both out of write scope, deliberately untouched)

- `crates/mp/game/src/g_turret_G2.rs:189` - stale comment citing `CGhoul2Info_v **` at `sv_game.rs:3326`. Comment only, no behavior.
- `docs/decisions.md:1534` - the DEC-65 entry still describes the split in present tense; not marked executed.

### Standing items this landing leaves open

Three items outlive the lane. Two are stale comments the packet's write scope excluded: the `g_turret_G2.rs:189` cite and the present-tense DEC-65 entry in the ledger. The third is the `ghoul2_vertex_golden` rig defect, which predates this work and needs its own ticket, because it blocks the image-golden half of the DEC-65 gate that later gh#31 steps depend on.

## Follow-up: the conformance pass (lane-review clerk, 2026-08-03)

A conformance clerk read the packet with its Amendments, the finished file, and the whole diff `e9c7cf8c..d666bad3` hunk by hunk, then re-ran every gate the finished file claims. The clerk judges nothing and approves nothing. The report below is the evidence half of the lane-review, and the disposition stays with the reviewer.

The clerk re-ran the gates on `d666bad3` with a clean tree. All of them reproduce. The base-commit checks used a throwaway git worktree at `e9c7cf8c` with its own `CARGO_TARGET_DIR`, and the worktree was removed afterwards.

Em dashes in the clerk's own prose are normalized to plain hyphens for house style. Em dashes inside quoted source lines, diff hunks, and captured tool output are evidence, so they keep their original bytes.

### Clerk report

#### 1. Letter violations

**1.1 - File outside the write scopes: `crates/mp/renderer/src/render_state/placeholders.rs`.** The packet's Paths list names ten paths plus `.claude/packets/31/step-001/`. This file is not among them. Changed in commit `62b2e7af`:

```
--- a/crates/mp/renderer/src/render_state/placeholders.rs
+++ b/crates/mp/renderer/src/render_state/placeholders.rs
@@ -168,7 +168,7 @@ pub struct RefEntity {
     /// The entity's attached Ghoul2 instance list, decoded from the tier-1
-    /// `*mut c_void ghoul2` token (`ghoul2_token_decode`, `tr_scene.rs`). Raven
+    /// `*mut c_void ghoul2` token (`ghoul2_token_decode`, `mp_engine_ghoul2::token`). Raven
```

**1.2 - File outside the write scopes, in a commit outside the bundle: `docs/audits/2026-08-03-ghoul2-token-conversion-renderer.md`.** 48 lines appended in `d666bad3` (`audit: ghoul2 token conversion landing (DEC-65 ruling 3)`), the tip of the diff range. The packet's bundle has five commits and the finished file lists five. This is a sixth.

**1.3 - Behavior change to a dispatch arm that the Conversion rules do not describe: `sv_game.rs` `G_G2_DUPLICATEGHOUL2INSTANCE` no longer calls the callee on a non-null slot.** The packet's Class 1 rule reads "read the slot token into a stack cell with `from_token`, call the `g2api_*` function, write the cell back into the slot with `to_token`". The sv arm instead moved the call inside the null-slot branch and builds the cell literally, not through `from_token`:

```
-            let g2_from = &mut *(*args.offset(1) as *mut CGhoul2Info_v);
-            let pp = vma(view.common, args, 2) as *mut *mut CGhoul2Info_v;
-            // Raven `*g2To = new CGhoul2Info_v` (assert `!*g2To`) — the seam owns
-            // the box; the ported fn takes the deref'd handle.
+            let mut g2_from = CGhoul2Info_v::from_token(*args.offset(1) as *mut c_void);
+            let pp = vma(view.common, args, 2) as *mut *mut c_void;
+            // Raven `*g2To = new CGhoul2Info_v` (assert `!*g2To`) builds the destination object.
+            // The destination starts empty, so the copy allocates and the write-back carries the new handle out.
             if (*pp).is_null() {
-                *pp = Box::into_raw(Box::new(CGhoul2Info_v { mItem: 0 }));
+                let mut g2_to = CGhoul2Info_v { mItem: 0 };
+                g2api_duplicate_ghoul2_instance(g2, &mut g2_from, &mut g2_to);
+                *pp = g2_to.to_token();
             }
-            let g2_to = &mut **pp;
-            g2api_duplicate_ghoul2_instance(g2, g2_from, g2_to);
             return 0;
```

Before the change the sv arm called `g2api_duplicate_ghoul2_instance` on every dispatch. After it, a non-null slot reaches no call. The callee's own body is:

```rust
pub fn g2api_duplicate_ghoul2_instance(
    g2: &mut Ghoul2System,
    g2_from: &mut CGhoul2Info_v,
    g2_to: &mut CGhoul2Info_v,
) {
    if g2_to.is_valid(g2) {
        return;
    }
    let _ = g2api_copy_ghoul2_instance(g2, g2_from, g2_to, -1);
}
```

The two client twins already had the call inside the null branch before this lane, so only the sv arm moved.

**Nothing else exceeded the contract.** A mechanical scan of all 801 added lines found no added `#[repr]`, no added `offset_of!` or `size_of` assert, no added cvar call, no `FrameEvent` variant, no added `impl`, `trait`, `struct`, or `enum`, no added or reordered dispatch condition, and no dependency edit. `crates/mp/renderer-gpu/Cargo.toml` already carried `mp_engine_ghoul2 = { path = "../engine/ghoul2" }`, and no `Cargo.toml` is in the range. The complete added `pub` set is exactly the contract's list:

```
crates/mp/engine/ghoul2/src/lib.rs: pub mod token;
crates/mp/engine/ghoul2/src/shared/cghoul2_info_v.rs:     pub fn from_token(token: *mut c_void) -> CGhoul2Info_v {
crates/mp/engine/ghoul2/src/shared/cghoul2_info_v.rs:     pub fn to_token(&self) -> *mut c_void {
crates/mp/engine/ghoul2/src/token.rs: pub fn ghoul2_token_decode(token: *mut c_void) -> Option<Ghoul2Handle> {
crates/mp/engine/ghoul2/src/token.rs: pub fn ghoul2_token_encode(handle: Option<Ghoul2Handle>) -> *mut c_void {
```

The removed `pub` set is exactly the two `tr_scene.rs` definitions. Write-back sites number exactly 13, which matches the packet's arm set: `sv_game.rs:3151,3394,3416,3425,3436`, `cl_cgame.rs:2953,3040,3172,3199`, `cl_ui.rs:2065,2114,2269,2306`. `oracle/` is unmodified, and `git log origin/master..master` is 116 commits, so nothing was pushed.

#### 2. The named hunks

The clerk quoted all twelve named hunks verbatim. The full quotations live in the session record. The load-bearing ones follow.

`crates/mp/engine/ghoul2/src/token.rs`, the whole new file:

```rust
//! The module-visible Ghoul2 token (DEC-65 ruling 3).
//!
//! Every ghoul2 reference that leaves the engine crosses as `Ghoul2Handle + 1`, cast to pointer width.
//! Null round-trips to `None`, because handle `0` is the always-invalid arena id (`info_array.rs:132-137`).
//! One scheme serves the module `void*` slots and `refEntity_t.ghoul2`.
//! That is what lets cgame copy its slot value straight into the render entity, and the renderer decode it.
//! Raw ghoul2 pointers never leave the engine.
//!
//! The scheme is live at every seam as of 2026-08-03, which closes the split `tr_scene.rs` used to flag.
//! The render trap decoded tokens while `sv_game.rs` handed out `Box<CGhoul2Info_v>` pointers in the same `void*` slot.
//! All 115 ghoul2 trap arms in `sv_game.rs`, `cl_cgame.rs`, and `cl_ui.rs`, plus the two `sv_world.rs` slot readers, now decode this token.

use core::ffi::c_void;

use crate::info_array::Ghoul2Handle;

/// Decodes a module-visible ghoul2 token into a [`Ghoul2Handle`].
/// A null token reads as no instance.
pub fn ghoul2_token_decode(token: *mut c_void) -> Option<Ghoul2Handle> {
    if token.is_null() {
        None
    } else {
        Some(Ghoul2Handle(token as i32 - 1))
    }
}

/// Encodes a [`Ghoul2Handle`] back into the module-visible token.
/// The inverse of [`ghoul2_token_decode`], and `None` encodes as null.
pub fn ghoul2_token_encode(handle: Option<Ghoul2Handle>) -> *mut c_void {
    match handle {
        Some(h) => (h.0 + 1) as *mut c_void,
        None => core::ptr::null_mut(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_token_round_trips_to_none() {
        assert!(ghoul2_token_decode(ghoul2_token_encode(None)).is_none());
        assert!(ghoul2_token_encode(None).is_null());
    }

    #[test]
    fn handle_round_trips_through_the_token() {
        let handle = Ghoul2Handle(1024);
        let token = ghoul2_token_encode(Some(handle));
        assert!(!token.is_null());
        assert_eq!(ghoul2_token_decode(token), Some(handle));
    }

    /// Handle `0` is a real value on the arena side, and it must not collide with the null token.
    #[test]
    fn zero_handle_is_not_the_null_token() {
        let token = ghoul2_token_encode(Some(Ghoul2Handle(0)));
        assert!(!token.is_null());
        assert_eq!(ghoul2_token_decode(token), Some(Ghoul2Handle(0)));
    }
}
```

The two function bodies are byte-identical to the definitions deleted from `tr_scene.rs`. The module doc cites `info_array.rs:132-137` where the packet's Ground truth paragraph cited `info_array.rs:132-151`.

The `deep_copy` hunk, `crates/mp/engine/ghoul2/src/shared/cghoul2_info_v.rs`:

```
-    /// Raven `CGhoul2Info_v::DeepCopy` — frees this handle, then (if `other`
-    /// is non-null) allocates a fresh slot, copies `other`'s instance vector,
-    /// and zeroes each copied instance's runtime-only fields (`mBoneCache`,
-    /// `mTransformedVertsArray`, `mSkelFrameNum`, `mMeshFrameNum`) so no
-    /// runtime state aliases across the copy.
+    /// Raven `CGhoul2Info_v::DeepCopy` — replaces this handle's instance vector with a copy of `other`'s,
+    /// and zeroes each copied instance's runtime-only fields (`mBoneCache`, `mTransformedVertsArray`,
+    /// `mSkelFrameNum`, `mMeshFrameNum`) so no runtime state aliases across the copy.
+    ///
+    /// DIVERGENCE (DEC-65 ruling 3, the 2026-08-03 in-place ruling).
+    /// Raven does `Free()` then `Alloc()`, which gives the destination a new `mItem` while the destination
+    /// *object* keeps its address, so a module that holds that object still sees the copy.
+    /// Here the destination object is the module's own 4-byte token slot, passed by value into the trap,
+    /// so a new `mItem` would die in the discarded stack cell and leak the arena slot.
+    /// This body therefore keeps the destination handle when one exists and replaces the arena contents in place.
+    /// It allocates only for a `mItem: 0` destination, and it frees the replaced instances' bone caches,
+    /// which is the half of Raven's `Free()` that has an effect outside the destination handle.
+    /// A live destination with an empty source keeps its handle and ends up with an empty vector, where Raven ends up null.
+    /// The one caller guards on `g2From.IsValid()` (`api_models.rs:653`), so that state is unreachable from a trap arm.
     ///
-    /// Source: `oracle/codemp/ghoul2/ghoul2_shared.h:382-397`
+    /// Source: `oracle/codemp/ghoul2/ghoul2_shared.h:382-397`, `oracle/codemp/ghoul2/G2_API.cpp:2239-2259`
     pub fn deep_copy(&mut self, g2: &mut Ghoul2System, other: &CGhoul2Info_v) {
...
-        self.free(g2);
-        if other.mItem != 0 {
-            self.alloc(g2);
-            let copy = g2.info_array.get(other.mItem).to_vec();
-            let dest = g2.info_array.get_mut(self.mItem);
-            *dest = copy;
-            for info in dest.iter_mut() {
-                info.bone_cache = None;
-                info.transformed_verts_array = None;
-                info.skel_frame_num = 0;
-                info.mesh_frame_num = 0;
+        if self.mItem == 0 {
+            if other.mItem == 0 {
+                return;
             }
+            self.alloc(g2);
+        }
+
+        // Raven's `Free()` reaches `DeleteLow`, which frees every replaced instance's bone cache
+        // (`G2_API.cpp:319-326`) from the sibling arena. The in-place body keeps that half.
+        let stale: Vec<_> = g2
+            .info_array
+            .get_mut(self.mItem)
+            .iter_mut()
+            .filter_map(|info| info.bone_cache.take())
+            .collect();
+        for id in stale {
+            remove_bone_cache(g2, id);
+        }
+
+        let copy = if other.mItem == 0 {
+            Vec::new()
+        } else {
+            g2.info_array.get(other.mItem).to_vec()
+        };
+        let dest = g2.info_array.get_mut(self.mItem);
+        *dest = copy;
+        for info in dest.iter_mut() {
+            info.bone_cache = None;
+            info.transformed_verts_array = None;
+            info.skel_frame_num = 0;
+            info.mesh_frame_num = 0;
         }
     }
```

The signature is unchanged. Two unit tests were added where the packet asked for one.

The `sv_world.rs` diagnostic guard:

```
                 if view.cvar_integer("sv_showghoultraces") != 0 {
-                    mp_engine_qcommon::common::common::com_printf(
-                        view.common,
-                        &format!(
-                            "Ghoul2 trace   lod={:1}   length={:6.0}   to {}\n",
-                            (*clip).useLod,
-                            Distance((*clip).start, (*clip).end),
-                            (&*((*touch).ghoul2 as *mut CGhoul2Info_v))
-                                .get(&*(view.g2.as_raw() as *mut Ghoul2System), 0)
-                                .file_name
-                        ),
-                    );
+                    let trace_g2 = CGhoul2Info_v::from_token((*touch).ghoul2);
+                    let g2_read = &*(view.g2.as_raw() as *mut Ghoul2System);
+                    // The print indexes model 0, so an empty or stale cell skips it instead of panicking.
+                    if trace_g2.size(g2_read) > 0 {
+                        mp_engine_qcommon::common::common::com_printf(
+                            view.common,
+                            &format!(
+                                "Ghoul2 trace   lod={:1}   length={:6.0}   to {}\n",
+                                (*clip).useLod,
+                                Distance((*clip).start, (*clip).end),
+                                trace_g2.get(g2_read, 0).file_name
+                            ),
+                        );
+                    }
                 }
```

The clerk found nothing wrong with the three CLEANMODELS arms, the four remove-family write-back arms, the three COPYGHOUL2INSTANCE null-guard arms, the three COPYSPECIFICGHOUL2MODEL arms and their §19 notes, the unguarded sv trio, the `CG_FX_PLAY_BOLTED_EFFECT_ID` arm, the two import hunks, or the `placeholders.rs` cite change, beyond the items listed under sections 1, 3, and 6.

#### 3. Ledger mismatches

Behaviors visible in the diff that the finished file does not mention. A confessed choice is not a mismatch, and the finished file's confessed set is excluded.

1. **The sv `G_G2_DUPLICATEGHOUL2INSTANCE` call-skip**, quoted at 1.3. The finished file has no line about this arm, and the commit-4 body names it only as a write-back arm.
2. **The sv duplicate arm builds its destination cell literally**, as `let mut g2_to = CGhoul2Info_v { mItem: 0 };`, where the packet's Class 1 rule says `from_token`. The same literal appears in the two client twins.
3. **The "six sv_game arms" count is seven.** The finished file states "The six `sv_game` arms that only index through `get_mut` bind the cell without `mut`". Seven arms do: `sv_game.rs` lines 3156, 3296, 3558, 3567, 3742, 3772, 3778.
4. **Handle `0` does not survive the `from_token` and `to_token` pair, while the new unit test pins that it survives the free-function pair.** `to_token` maps `mItem == 0` to null. The finished file records the third test only as "a third case was added for handle `0` against the null token". The packet's Surface contract does state the `to_token` null encoding, so the letter holds. The two layers behave differently for handle `0`, and that is unstated.
5. **`cl_cgame.rs` `CG_G2_COPYSPECIFICGHOUL2MODEL` lost its `// SAFETY:` line outright** together with its `unsafe` block, where the finished file's Commit 3 note says such prefixes "became plain notes".
6. **`deep_copy`'s `Source:` cite gained a second file**, `oracle/codemp/ghoul2/G2_API.cpp:2239-2259`.
7. **The `token.rs` module doc cites `info_array.rs:132-137`** where the packet's ground truth cited `info_array.rs:132-151`.

#### 4. The inventories

Files changed against the write scopes. Twelve of fourteen are in scope. The two exceptions are `crates/mp/renderer/src/render_state/placeholders.rs` and this audit file itself.

Commits against the bundle. All five bundle items map one to one, with no split and no reorder.

| Bundle item | Commit | Match |
|---|---|---|
| 1. token helpers plus `deep_copy` | `7833dc68` | yes |
| 2. `cl_ui.rs` plus the audit report in the body | `49419369` | yes, and the audit report is present |
| 3. `cl_cgame.rs` | `92e136de` | yes |
| 4. `sv_game.rs` plus `sv_world.rs` | `129cc053` | yes |
| 5. doc close-out | `62b2e7af` | yes, plus the out-of-scope `placeholders.rs` |
| not in the bundle | `d666bad3` | the audit commit |

Commit messages against the rules. All six carry a heading subject in `scope(gh#31 wN): noun phrase` form, subject lengths 41 to 81. Bodies are unwrapped STE-flavored paragraphs. A mechanical lint of every body found 0 em dashes, 0 semicolons outside backticks, and 0 trailers of any kind. Each commit is unsigned, which matches the packet's `--no-gpg-sign`.

#### 5. Repo mechanics on added lines

- No `use` declaration inside a function body. All 16 added `use` lines sit at file top, except `use super::*;` in the `token.rs` `#[cfg(test)]` module, which porting-rules exempts.
- No `todo!()` and no other placeholder. Zero `todo!`, zero `unimplemented!`, zero `TODO` strings added.
- Four added items carry no oracle `Source:` cite: `ghoul2_token_decode`, `ghoul2_token_encode`, `from_token`, and `to_token`. The two free functions are relocated byte for byte from `tr_scene.rs`, whose deleted definitions also carried no cite. `from_token` and `to_token` are new, and they are the only members of that `impl` block without a `Source:` line.
- No new extern forward-declaration block. Zero `extern "` lines added.
- One `format!` on an added line, in `sv_world.rs`. It feeds `com_printf`, so it builds a console diagnostic and not a wire string. It is the pre-existing `sv_showghoultraces` print, re-indented one level by the new guard.

#### 6. House-style violations on added lines

1. **One em dash**, in the `deep_copy` doc: ``/// Raven `CGhoul2Info_v::DeepCopy` — replaces this handle's instance vector with a copy of `other`'s,``
2. **Two prose semicolons.** In code, `sv_world.rs`: `// design); \`(*touch).ghoul2\` is the module's ghoul2 token (DEC-65 ruling 3).` In the finished file: "which was done; the golden itself could not execute on either side of the change." Four further prose semicolons landed in this audit file under commit `d666bad3`. Semicolons inside backticked C source are code, not prose, and are not counted.
3. **Pet vocabulary on three added lines.** "seam" in the `token.rs` module doc and in the `cghoul2_info_v.rs` lifecycle block. "canonical" in the finished file.
4. **Comments that narrate mechanics.** Three families, each repeated once per dispatch file: `// The slot reads into a stack cell, and the write-back below carries the new handle out.`, `// The arm reads the slot but never writes it, because the call is a pure read.`, and `// Both sides come across by value, so this arm has no slot to write back through.`
5. **A doc comment off the content rules.** The `deep_copy` doc is 15 `///` lines, of which the DIVERGENCE block is 10, against the house rule of 1 to 2 lines for a method. The packet's Surface contract required a divergence record at this site, so the length is the part that exceeds the rule. Two of its lines also break inside a sentence at a point that is not a clause boundary, with joined lengths under 150.
6. Clean: no banned voice, no antithesis constructions, zero added comment lines over 150 columns, zero contractions, zero marketing adjectives.

#### 7. The gate claims, re-run

Every gate re-run by the clerk on `d666bad3` with a clean tree.

| Claim in the finished file | Command | Real output |
|---|---|---|
| `cargo build --workspace` green | `cargo build --workspace` | Green, exit 0. Warnings: `mp_engine_client` 26, `mp_renderer` 1, `mp_uishared` 2. |
| `cargo test --workspace` green, no failures | `cargo test --workspace` | Green, exit 0. No `FAILED`. |
| the new unit tests | `cargo test -p mp_engine_ghoul2 --lib` | 89 passed, 0 failed. `deep_copy_allocates_an_empty_destination` ok, `deep_copy_keeps_a_live_destination_handle` ok, and all three `token::tests` ok. |
| world goldens byte-identical under `--ignored` | `JKA_REF_BASEPATH=~/Developer/jka/jka_server cargo test -p mp_renderer_gpu --test world_golden -- --ignored --test-threads=1 --nocapture` | `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 73.78s`. `git status` clean afterwards, so no golden was rewritten. |
| `ghoul2_vertex_golden` default leg passes | `cargo test -p mp_renderer_gpu --test ghoul2_vertex_golden` | `ignored, needs retail assets and a GPU; run locally with --ignored`, 0 passed, 0 failed, 1 ignored. The file holds one test and it is `#[ignore]`d, so the default leg is a skip. |
| the `--ignored` leg aborts in `re_from_view` | same test with `-- --ignored --nocapture` | Reproduced: `panicked at crates/mp/renderer/src/hook_install.rs:58:5: null pointer dereference occurred`, `signal: 6, SIGABRT`. |
| the abort predates the work | worktree at `e9c7cf8c`, separate `CARGO_TARGET_DIR`, same command | Reproduced at the base commit with the same panic site and the same SIGABRT. Debug profile only. The clerk did not re-run the release profile. |
| lockstep referee suite green, 9 passed, no self-skip, reflogs unchanged | `JKA_REF_BASEPATH=~/Developer/jka/jka_server cargo test -p jampgame --test referee -- --ignored --test-threads=1 --nocapture`, after `cargo build --workspace`, with the oracle dylib present | `test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 18.07s`. Zero case-insensitive matches for "skip", so no self-skip. Real-map work confirmed by `===== referee PASS — scenario 'real-ffa1-items': 2501 frames byte-identical; 15006 client-states, 275856 entity-states, 850498 syscalls compared =====`. `regenerate_logs` rewrote all eight `.reflog` files, and `git status --short` afterwards is empty. |
| the crate warning count returns to its pre-change value | `cargo build -p mp_engine_client` at HEAD and at `e9c7cf8c` | 26 warnings at both. The claim holds. |

No environment piece was missing. `~/Developer/jka/jka_server/base/assets0.pk3` and `tools/referee-oracle/build/liboraclejampgame.dylib` both exist.

#### 8. The unverified list

Named plainly. None is assumed fine.

1. **The "115 ghoul2 trap arms" figure** in the `token.rs` module doc, and the packet's 42, 46, 27 split. A raw condition count gives 47, 54, and 35 hits, which over-counts multi-condition arms. The counting method could not be settled mechanically.
2. **The pre-flight class-2 audit result.** The clerk confirmed every cite the commit body names, and confirmed no save or load trap arm exists in the three dispatch files. The clerk did not re-derive the exhaustive mutator inventory across all 99 class-2 callees.
3. **Whether the arm-by-arm conversions preserve module-observable behavior.** The referee suite and the world goldens cover the server and render paths. Nothing in the range exercises the `cl_ui.rs` ghoul2 arms or the copy family under a differential harness.
4. **The `-1` return the three COPYGHOUL2INSTANCE null guards introduce.** No test and no oracle comparison covers it. The clerk did not grep the oracle tree for `trap_G2API_CopyGhoul2Instance`.
5. **The `deep_copy` bone-cache free.** Neither new unit test asserts that the stale bone caches leave `Ghoul2System.bone_caches`.
6. **The release-profile SIGSEGV** the finished file claims for the `ghoul2_vertex_golden` deviation. The clerk reproduced the debug-profile abort on both sides only.
7. **`crates/mp/game/src/g_turret_G2.rs:189` and `docs/decisions.md:1534`.** Nothing in the diff touches them, and the clerk did not read either to confirm the described staleness.
8. **Whether the sv duplicate call-skip changes anything observable.** That depends on whether a non-null slot can hold a stale or invalid token at that arm, which is a judgment about reachable state.
9. **`fx_host.rs:693`.** The file is unchanged and the site still reads `let mut handle = CGhoul2Info_v { mItem: ghoul2 };`, which builds a cell from an `i32` handle value and not from a module slot. Whether that is the intended consistency is a judgment.
