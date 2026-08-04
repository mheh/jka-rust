# Packet gh#31 step-001 - DEC-65 ruling 3: the ghoul2 token conversion

## Scope

This step converts the module-visible ghoul2 `void*` slot from a raw `Box<CGhoul2Info_v>` heap pointer to the `Ghoul2Handle + 1` token, in all three trap-dispatch files plus the two engine-side slot readers in `sv_world.rs`. Raw ghoul2 pointers never leave the engine after this step. The step does not touch `ModelPool`, block publication, bone matrices, or any draw arm - those are later steps under DEC-65 rulings 1 and 2.

Ground truth: `CGhoul2Info_v` is `#[repr(C)] { mItem: i32 }` (`crates/mp/engine/ghoul2/src/shared/cghoul2_info_v.rs:11-15`), where `mItem` is the packed `slot | generation` arena id, `0` always invalid (`info_array.rs:132-151`). Every `g2api_*` function keeps its `&CGhoul2Info_v` / `&mut CGhoul2Info_v` signature, so the ghoul2 crate API shape does not change. One crate behavior changes under the 2026-08-03 ruling: `CGhoul2Info_v::deep_copy` becomes in-place (see Surface contract).

The token scheme stays the ratified one: `handle + 1` cast pointer-width, null round-trips to `None` (`tr_scene.rs:320-350`). One scheme serves both the module slots and `refEntity_t.ghoul2`, which is what lets cgame copy its slot value into `refEntity_t.ghoul2` and the renderer decode it.

## Surface contract

- `mp_engine_ghoul2` gains one new file holding the relocated token helpers, signatures unchanged:
  - `pub fn ghoul2_token_decode(token: *mut c_void) -> Option<Ghoul2Handle>`
  - `pub fn ghoul2_token_encode(handle: Option<Ghoul2Handle>) -> *mut c_void`
- `impl CGhoul2Info_v` gains two inherent methods over those helpers:
  - `pub fn from_token(token: *mut c_void) -> CGhoul2Info_v` - decoded handle or `mItem: 0`.
  - `pub fn to_token(&self) -> *mut c_void` - `mItem: 0` encodes to null.
- `CGhoul2Info_v::deep_copy` (`cghoul2_info_v.rs:90-112`) changes behavior: it keeps the destination handle when one exists and replaces the arena contents in place, and it allocs only when the destination cell is `mItem: 0`. This preserves the module-visible identity the way Raven's `DeepCopy` preserves the destination object (`G2_API.cpp:2239-2259`), so a value-passed copy no longer changes `mItem` on a live destination. The `DUPLICATEGHOUL2INSTANCE` arms are unaffected: their destination starts empty, so `deep_copy` allocs, and the slot address carries the write-back. The method's doc comment records the divergence from Raven's internal free-then-alloc.
- `mp_renderer::tr_scene` loses the two helper definitions. All importers move to the `mp_engine_ghoul2` path: `tr_scene.rs`, `tr_main.rs`, `renderer-gpu/tests/ghoul2_vertex_golden.rs`, `renderer-gpu/src/bin/world_harness.rs`, and any other current importer `cargo check` reveals. No re-export shim stays behind (DEC-32 one canonical home).
- The 115 trap arms in `sv_game.rs`, `cl_cgame.rs`, `cl_ui.rs` and the two derefs in `sv_world.rs:785,797` change body-only: decode instead of deref, stack cell instead of heap cell. The write-back arm set is exactly: `INITGHOUL2MODEL` x3, `DUPLICATEGHOUL2INSTANCE` x3, `CLEANMODELS` x3, `REMOVEGHOUL2MODEL` x3, `REMOVEGHOUL2MODELS` x1 - 13 arms, each holding the slot address. The `HASGHOUL2MODELONINDEX` x3 arms decode through the slot address read-only. The remaining 99 arms decode the argument value and discard. No trap number, no arm order, no dispatch signature changes.
- No `#[repr]` layout, no assert, no cvar, no `FrameEvent` variant changes. The slot stays `*mut c_void` in every module-visible struct.

Anything not on this list is out of scope, and the agent must not add it.

## Conversion rules

- Class 1 (`INITGHOUL2MODEL` x3, `DUPLICATEGHOUL2INSTANCE` x3): read the slot token into a stack cell with `from_token`, call the `g2api_*` function, write the cell back into the slot with `to_token`. No `Box::new`, no `Box::into_raw`.
- Class 3 (`CLEANMODELS` x3): same read, call `g2api_clean_ghoul2_models` (which zeroes `mItem`), write the cell back - a zeroed cell encodes to null. No `Box::from_raw`.
- Class 4 (`REMOVEGHOUL2MODEL` x3, `REMOVEGHOUL2MODELS` x1): these arms hold the slot address today (`sv_game.rs:3389,3395` and the guarded client twins), and the callees free the handle when the vector empties (`api_models.rs:497,549`). Decode from the slot address, call, write the cell back. A `mItem: 0` cell encodes to null, which restores Raven's `*ghlRemove = NULL` (`G2_API.cpp:868-869,956-957`) that the current pointer arms omit.
- `HASGHOUL2MODELONINDEX` x3: decode the pointee through the slot address into a stack cell, call, no write-back. The client null guards stay, and the sv arm's null pointee decodes to a `mItem: 0` cell - qfalse either way.
- Class 2 (the 99 remaining value-passed arms): decode the argument into a stack cell, call, discard the cell. There is no slot to write back.
- `COPYGHOUL2INSTANCE` x3: value-passed on both sides. Under in-place `deep_copy` a live destination keeps its handle, so the discard is correct. Add a null-destination guard that returns -1 without calling: Raven crashes on the null reference (porting-rules §19), and calling would alloc into a discarded cell and leak the arena slot. The arm is module-dead in both trees (no Rust or oracle module caller), noted at the site.
- `COPYSPECIFICGHOUL2MODEL` x3: the existing null guards stay. §19 site note: after a remove empties a slot, the slot is now null and this arm no-ops, where the current pointer code reallocs through the kept empty cell (`api_models.rs:695-697`). Raven also no-ops there because Raven nulls the slot at remove, so the current Rust is the outlier.
- Guard asymmetry stays: the sv arms without null tests (`sv_game.rs:3383-3397`) keep their behavior. A null token decodes to a `mItem: 0` cell instead of a null deref - the one defined behavior for Raven's crash, noted at the site per porting-rules §19.
- `sv_world.rs:785,797`: replace the derefs with `from_token` stack cells. The `:785` diagnostic print indexes model 0, so guard it with `size(g2) > 0` - an empty or stale cell must skip the print, not panic.
- `fx_host.rs:693` already builds a stack cell from a handle value - verify it stays consistent with the token scheme and change nothing unless it reads a module slot.

## Pre-flight audit (part of commit 2, before any class-2 conversion)

The mutator inventory on `CGhoul2Info_v` is: `alloc`, `free`, `clear`, `deep_copy`, `assign`, `assign_item`, plus the lazy allocs inside `resize` and `push_back` (`cghoul2_info_v.rs:49-186`). The trap-reachable mutator calls sit only in `g2api_init_ghoul2_model`, `g2api_clean_ghoul2_models`, the remove family (`api_models.rs:497,549`), `g2api_copy_ghoul2_instance` (`api_models.rs:653`), and `g2api_copy_specific_g2_model` (`api_models.rs:697`) - each covered by a named class or rule above. For every class-2 arm, confirm against this inventory that the called `g2api_*` function cannot mutate `mItem` through the value argument. Audit the has/remove family as slot-address arms, not value-passed. If any uncovered value-passed callee can mutate `mItem`, STOP without converting that arm and report - the write-back question becomes a ruling.

## Commit bundle

1. **Token helpers move to `mp_engine_ghoul2`, and `deep_copy` goes in-place** - the new file, the two inherent methods, importer path updates, delete the old definitions, plus the `deep_copy` in-place change with its doc-comment update and a unit test that a live destination keeps its handle across a copy. The crate change lands here so every dispatch commit below builds on it. Gates: `cargo build --workspace`, `cargo test --workspace`, world goldens byte-identical, `ghoul2_vertex_golden` run with its own `cargo test` invocation.
2. **`cl_ui.rs` conversion** (27 arms) plus the class-2 audit report in the commit body, run per the pre-flight section with the has/remove family audited as slot-address arms. Gates: `cargo build --workspace`, `cargo test --workspace`.
3. **`cl_cgame.rs` conversion** (46 arms). Gates: `cargo build --workspace`, `cargo test --workspace`, world goldens byte-identical.
4. **`sv_game.rs` conversion** (42 arms) plus the `sv_world.rs` derefs and the `:785` size guard. Gates: `cargo build --workspace`, `cargo test --workspace`, the lockstep referee suite.
5. **Doc close-out** - the `tr_scene.rs` RECONCILE block updates to cite DEC-65 as executed, stale line refs corrected, `cghoul2_info_v.rs` lifecycle doc gains the token sentence. Gates: `cargo build --workspace`, `cargo test --workspace`.

Every commit uses `--no-gpg-sign`, a heading subject, an STE body, and no trailer of any kind: no `Co-Authored-By`, no generated-with footer.

## Write scopes

- Branch: `master`, direct commits.
- Paths: `crates/mp/engine/ghoul2/src/`, `crates/mp/engine/server/src/sv_game.rs`, `crates/mp/engine/server/src/sv_world.rs`, `crates/mp/engine/client/src/cl_cgame.rs`, `crates/mp/engine/client/src/cl_ui.rs`, `crates/mp/renderer/src/tr_scene.rs`, `crates/mp/renderer/src/tr_main.rs`, `crates/mp/renderer-gpu/tests/ghoul2_vertex_golden.rs`, `crates/mp/renderer-gpu/src/bin/world_harness.rs`, plus `.claude/packets/31/step-001/` for the finished file.
- `oracle/` is read-only. `~/Developer/jka/` is untouchable. Source edits go through the Edit tool only. No pushes.

## Disposition

The commits land on master and hold locally with the batch. Lane-review follows completion. The next step (block publication, DEC-65 ruling 1) does not start from this packet.

## Amendments

**2026-08-03 - Fable logic audit: five amendments plus ruling (c) on the copy arms.**

The read-only audit confirmed the token scheme end to end (renderer decode, FX `mItem` retention with generation checks, content aliasing through the arena) and found the class map wrong in three places. The audit record: `docs/audits/2026-08-03-ghoul2-token-conversion-renderer.md`. The five amendments:

1. The remove family frees the handle when the vector empties (`api_models.rs:497,549`) and already holds the slot address, so `REMOVEGHOUL2MODEL` x3 and `REMOVEGHOUL2MODELS` x1 move from class 2 to the new write-back class 4. This restores Raven's `*ghlRemove = NULL` (`G2_API.cpp:868-869,956-957`), which the current pointer arms omit, and closes a stale-token re-init panic.
2. `g2api_copy_ghoul2_instance` mutated the value-passed destination through `deep_copy` (`api_models.rs:653`). The user ruled option (c): `deep_copy` becomes in-place, keeping the destination handle when one exists and replacing the arena contents, matching Raven's preservation of the module-visible destination object (`G2_API.cpp:2239-2259`). The arms add a null-destination guard and a module-dead note.
3. `g2api_copy_specific_g2_model` can alloc through an empty destination cell (`api_models.rs:697`). With amendment 1 that state is a null slot and the arm no-ops, matching Raven. A §19 site note records that the current Rust is the outlier.
4. The pre-flight audit now names the full mutator inventory and audits the has/remove family as slot-address arms, not value-passed.
5. The `sv_world.rs:785` diagnostic print gains a `size(g2) > 0` guard so an empty or stale cell skips the print instead of panicking.
