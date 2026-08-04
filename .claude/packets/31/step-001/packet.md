# Packet gh#31 step-001 - DEC-65 ruling 3: the ghoul2 token conversion

## Scope

This step converts the module-visible ghoul2 `void*` slot from a raw `Box<CGhoul2Info_v>` heap pointer to the `Ghoul2Handle + 1` token, in all three trap-dispatch files plus the two engine-side slot readers in `sv_world.rs`. Raw ghoul2 pointers never leave the engine after this step. The step does not touch `ModelPool`, block publication, bone matrices, or any draw arm - those are later steps under DEC-65 rulings 1 and 2.

Ground truth: `CGhoul2Info_v` is `#[repr(C)] { mItem: i32 }` (`crates/mp/engine/ghoul2/src/shared/cghoul2_info_v.rs:11-15`), where `mItem` is the packed `slot | generation` arena id, `0` always invalid (`info_array.rs:132-151`). Every `g2api_*` function already takes `&CGhoul2Info_v` / `&mut CGhoul2Info_v`, so the ghoul2 crate API does not change.

The token scheme stays the ratified one: `handle + 1` cast pointer-width, null round-trips to `None` (`tr_scene.rs:320-350`). One scheme serves both the module slots and `refEntity_t.ghoul2`, which is what lets cgame copy its slot value into `refEntity_t.ghoul2` and the renderer decode it.

## Surface contract

- `mp_engine_ghoul2` gains one new file holding the relocated token helpers, signatures unchanged:
  - `pub fn ghoul2_token_decode(token: *mut c_void) -> Option<Ghoul2Handle>`
  - `pub fn ghoul2_token_encode(handle: Option<Ghoul2Handle>) -> *mut c_void`
- `impl CGhoul2Info_v` gains two inherent methods over those helpers:
  - `pub fn from_token(token: *mut c_void) -> CGhoul2Info_v` - decoded handle or `mItem: 0`.
  - `pub fn to_token(&self) -> *mut c_void` - `mItem: 0` encodes to null.
- `mp_renderer::tr_scene` loses the two helper definitions. All importers move to the `mp_engine_ghoul2` path: `tr_scene.rs`, `tr_main.rs`, `renderer-gpu/tests/ghoul2_vertex_golden.rs`, `renderer-gpu/src/bin/world_harness.rs`, and any other current importer `cargo check` reveals. No re-export shim stays behind (DEC-32 one canonical home).
- The 115 trap arms in `sv_game.rs`, `cl_cgame.rs`, `cl_ui.rs` and the two derefs in `sv_world.rs:785,797` change body-only: decode instead of deref, stack cell instead of heap cell, write-back through `to_token` where the arm holds the slot address. No trap number, no arm order, no dispatch signature changes.
- No `#[repr]` layout, no assert, no cvar, no `FrameEvent` variant changes. The slot stays `*mut c_void` in every module-visible struct.

Anything not on this list is out of scope, and the agent must not add it.

## Conversion rules

- Class 1 (`INITGHOUL2MODEL` x3, `DUPLICATEGHOUL2INSTANCE` x3): read the slot token into a stack cell with `from_token`, call the `g2api_*` function, write the cell back into the slot with `to_token`. No `Box::new`, no `Box::into_raw`.
- Class 3 (`CLEANMODELS` x3): same read, call `g2api_clean_ghoul2_models` (which zeroes `mItem`), write the cell back - a zeroed cell encodes to null. No `Box::from_raw`.
- Class 2 (the 106 value-passed arms): decode the argument into a stack cell, call, discard the cell. There is no slot to write back.
- Guard asymmetry stays: the sv arms that skip the null test (`HASGHOUL2MODELONINDEX`, `REMOVEGHOUL2MODEL`, `REMOVEGHOUL2MODELS`, `sv_game.rs:3383-3397`) keep their behavior. A null token decodes to a `mItem: 0` cell instead of a null deref - that is the one defined behavior for Raven's UB, noted at the site per porting-rules §19.
- `sv_world.rs:785,797`: replace the `(*touch).ghoul2 as *mut CGhoul2Info_v` derefs with `from_token` stack cells.
- `fx_host.rs:693` already builds a stack cell from a handle value - verify it stays consistent with the token scheme and change nothing unless it reads a module slot.

## Pre-flight audit (part of commit 2, before any class-2 conversion)

For every class-2 arm, confirm the called `g2api_*` function cannot mutate `mItem` (only alloc, free, and duplicate paths do today, and those are all slot-address arms). If any value-passed callee can mutate `mItem`, STOP without converting that arm and report - the write-back question becomes a ruling.

## Commit bundle

1. **Token helpers move to `mp_engine_ghoul2`** - the new file, the two inherent methods, importer path updates, delete the old definitions. Gates: `cargo build --workspace`, `cargo test --workspace`, world goldens byte-identical, `ghoul2_vertex_golden` run with its own `cargo test` invocation.
2. **`cl_ui.rs` conversion** (27 arms) plus the class-2 audit report in the commit body. Gates: `cargo build --workspace`, `cargo test --workspace`.
3. **`cl_cgame.rs` conversion** (46 arms). Gates: `cargo build --workspace`, `cargo test --workspace`, world goldens byte-identical.
4. **`sv_game.rs` conversion** (42 arms) plus the `sv_world.rs` derefs. Gates: `cargo build --workspace`, `cargo test --workspace`, the lockstep referee suite.
5. **Doc close-out** - the `tr_scene.rs` RECONCILE block updates to cite DEC-65 as executed, stale line refs corrected, `cghoul2_info_v.rs` lifecycle doc gains the token sentence. Gates: `cargo build --workspace`, `cargo test --workspace`.

Every commit uses `--no-gpg-sign`, a heading subject, an STE body, and no trailer of any kind: no `Co-Authored-By`, no generated-with footer.

## Write scopes

- Branch: `master`, direct commits.
- Paths: `crates/mp/engine/ghoul2/src/`, `crates/mp/engine/server/src/sv_game.rs`, `crates/mp/engine/server/src/sv_world.rs`, `crates/mp/engine/client/src/cl_cgame.rs`, `crates/mp/engine/client/src/cl_ui.rs`, `crates/mp/renderer/src/tr_scene.rs`, `crates/mp/renderer/src/tr_main.rs`, `crates/mp/renderer-gpu/tests/ghoul2_vertex_golden.rs`, `crates/mp/renderer-gpu/src/bin/world_harness.rs`, plus `.claude/packets/31/step-001/` for the finished file.
- `oracle/` is read-only. `~/Developer/jka/` is untouchable. Source edits go through the Edit tool only. No pushes.

## Disposition

The commits land on master and hold locally with the batch. Lane-review follows completion. The next step (block publication, DEC-65 ruling 1) does not start from this packet.

## Amendments

(none yet)
