# Vet report - packet gh#56 step-001, range `c9a33698..gh56-step-001-vm-shutdown`

Vetted 2026-09-01. Four commits walked in order with `git show`, every hunk read, no sample. The oracle cites were read before any commit: `oracle/codemp/server/sv_init.cpp:509-527`, `oracle/codemp/qcommon/z_memman_pc.cpp:752-772`, `oracle/codemp/client/cl_main.cpp:657-682`, `oracle/codemp/client/cl_cgame.cpp:595-607`, `oracle/codemp/client/cl_ui.cpp:1444-1454`, `oracle/codemp/client/cl_cin.cpp:126-132`, and a symbol grep over `oracle/codemp/null/null_client.cpp`. The grep confirms `null_client.cpp` defines none of the four shutdown symbols, so the packet's Row 2 premise holds. The finished file was not opened, per the vet charter.

## 1. Letter violations

### Commit d9251729 (hooks)

None. The four fields sit after `Key_WriteBindings` with the contracted signatures. The four null bodies, the four `null_dedicated` initializers, the four adapters, and the four installs match the surface contract. The added section-header comments and the doc comments on the null bodies are repo-idiom requirements, not new surface.

### Commit 5b2d88a2 (call sites)

None. No new item in either file. Both hunks are hook reads plus calls, inside the two contracted functions.

### Commit 4b4591e2 (test)

**Finding 1 - the test file exceeds the contract's item list.** The surface contract reads: "One `#[test] fn a_cleared_slot_is_reused_by_the_next_module()` and one private `extern "C"` syscall stub. No production surface." The delivered file carries three extra private items beyond the test fn and the syscall stub:

```rust
extern "C-unwind" fn stub_vm_main(
```

```rust
fn seat_slot_zero(common: &mut Common, name: &str) {
```

Row 3's ratified default requires a seated "stub entry point", so `stub_vm_main` is authorized by the row even though the contract's one-stub count misses it. `seat_slot_zero` is a private convenience helper the contract does not list. Nothing is `pub`, and no production surface moves. I flag the mismatch and leave the disposition to the session.

### Commit 48c614f2 (finished file)

None on surface. The commit touches only `.claude/packets/56/step-001/finished.md`, inside the write scope.

## 2. Oracle divergences

None found.

- `Hunk_Clear` order after the change: `CL_ShutdownCGame`, `CL_ShutdownUI`, `SV_ShutdownGameProgs`, `CIN_CloseAllVideos`, `hunk_tag = TAG_HUNK_MARK1`, `Z_TagFree(MARK1)`, `Z_TagFree(MARK2)`, `R_HunkClearCrap`, `VM_Clear`. The oracle at `z_memman_pc.cpp:754-771` runs `CL_ShutdownCGame()` (755), `CL_ShutdownUI()` (756), `SV_ShutdownGameProgs()` (758), `CIN_CloseAllVideos()` (761), `hunk_tag`/`Z_TagFree` (764-766), `R_HunkClearCrap()` (768), `VM_Clear()` (771). The order matches line for line.
- `SV_SpawnServer` order after the change: the `CL_MapLoading` hook, the `CL_ShutdownAll` hook, `CM_ClearMap`, then `Hunk_Clear` further down. The oracle at `sv_init.cpp:511-527` runs `CL_MapLoading()` (511), `CL_ShutdownAll()` (515), `CM_ClearMap()` (518), `Hunk_Clear()` (527). The `#ifdef _XBOX` blocks between them are platform-dead and were already dropped before this step. The order matches.
- The no-op defaults are the dedicated build's true behavior. Raven's guards at `sv_init.cpp:513-516` and `z_memman_pc.cpp:754-757,760-762` remove the call sites in the dedicated build, and `null_client.cpp` defines no replacement bodies. A `Some(no_op)` field read through `.expect` reproduces "call site removed" exactly, and `jampded` never installs the client tier (`crates/mp/engine/core/src/host_view.rs:73-75` gates on `engine.cl.is_some()`).
- The client-build boot path also matches. Raven's client build runs the real `CL_ShutdownCGame`/`CL_ShutdownUI` inside `Com_InitHunkMemory`'s `Hunk_Clear` and both early-return on a null VM pointer. The port installs the real hooks before `com_init` (`crates/mp/client-app/src/sim.rs:96,99`), and the ported bodies early-return on `cl.cgvm.is_null()` / `cl.uivm.is_null()` (`crates/mp/engine/client/src/cl_cgame.rs:675-684`, `cl_ui.rs:749-759`), which match `cl_cgame.cpp:595-607` and `cl_ui.cpp:1444-1454` including `UI_MENU_RESET`.

## 3. The named hunks, verbatim

### The `SV_SpawnServer` call-site hunk, `crates/mp/engine/server/src/sv_init.rs`

```diff
+    // make sure all the client stuff is unloaded
+    // The hook answers with a no-op on `jampded`, which reproduces Raven's `#ifndef DEDICATED` guard at this call site.
+    let cl_shutdown_all = view
+        .common
+        .hooks
+        .CL_ShutdownAll
+        .expect("CL_ShutdownAll hook");
+    cl_shutdown_all(view);
+
```

Placed between the `CL_MapLoading` hook call and `CM_ClearMap`, as contracted. The first comment line is Raven's own comment from `sv_init.cpp:514`, preserved per porting rules. Nothing wrong found.

### The `Hunk_Clear` block, `crates/mp/engine/qcommon/src/z_memman_pc.rs`

```diff
-    // DEDICATED: this is the dedicated-server build (§20/§C10 precedent —
-    // the engine-fork-discovery rulings treat DEDICATED as the live
-    // configuration), so the `#ifndef DEDICATED` client blocks
-    // (CL_ShutdownCGame/CL_ShutdownUI/CIN_CloseAllVideos) are dropped.
+    // The three client hooks in this function answer with a no-op on `jampded`, which reproduces Raven's `#ifndef DEDICATED` guards.
+    // The two shutdown calls must run before `VM_Clear` below, because each one nulls the client's own VM handle.
+    // A handle left set across `VM_Clear` addresses whichever module `VM_Create` seats in that slot next.
+    let cl_shutdown_cgame = view
+        .common
+        .hooks
+        .CL_ShutdownCGame
+        .expect("CL_ShutdownCGame hook");
+    cl_shutdown_cgame(view);
+
+    let cl_shutdown_ui = view.common.hooks.CL_ShutdownUI.expect("CL_ShutdownUI hook");
+    cl_shutdown_ui(view);
+
     let sv_shutdown_game_progs = view
```

```diff
+    let cin_close_all_videos = view
+        .common
+        .hooks
+        .CIN_CloseAllVideos
+        .expect("CIN_CloseAllVideos hook");
+    cin_close_all_videos(view);
+
```

The stale DEDICATED comment is gone and the replacement is truthful. "The three client hooks" counts CGame, UI, and CIN correctly. Nothing wrong found.

### The four `EngineHooks` fields and no-op defaults, `crates/mp/engine/qcommon/src/common/engine_hooks.rs`

```diff
+    // ---- client tier: the `#ifndef DEDICATED` calls `null_client.cpp` never defines ----
+    /// Source: `oracle/codemp/client/cl_main.cpp:657`
+    pub CL_ShutdownAll: Option<fn(&mut EngineHostView)>,
+    /// Source: `oracle/codemp/client/cl_cgame.cpp:595`
+    pub CL_ShutdownCGame: Option<fn(&mut EngineHostView)>,
+    /// Source: `oracle/codemp/client/cl_ui.cpp:1444`
+    pub CL_ShutdownUI: Option<fn(&mut EngineHostView)>,
+    /// Source: `oracle/codemp/client/cl_cin.cpp:126`
+    pub CIN_CloseAllVideos: Option<fn(&mut EngineHostView)>,
```

```diff
+            CL_ShutdownAll: Some(CL_ShutdownAll_null),
+            CL_ShutdownCGame: Some(CL_ShutdownCGame_null),
+            CL_ShutdownUI: Some(CL_ShutdownUI_null),
+            CIN_CloseAllVideos: Some(CIN_CloseAllVideos_null),
```

```diff
+/// The dedicated build never calls `CL_ShutdownAll`.
+/// Source: `oracle/codemp/server/sv_init.cpp:513-516`
+#[allow(non_snake_case)]
+fn CL_ShutdownAll_null(_view: &mut EngineHostView) {}
+
+/// The dedicated build never calls `CL_ShutdownCGame`.
+/// Source: `oracle/codemp/qcommon/z_memman_pc.cpp:754-757`
+#[allow(non_snake_case)]
+fn CL_ShutdownCGame_null(_view: &mut EngineHostView) {}
+
+/// The dedicated build never calls `CL_ShutdownUI`.
+/// Source: `oracle/codemp/qcommon/z_memman_pc.cpp:754-757`
+#[allow(non_snake_case)]
+fn CL_ShutdownUI_null(_view: &mut EngineHostView) {}
+
+/// The dedicated build never calls `CIN_CloseAllVideos`.
+/// Source: `oracle/codemp/qcommon/z_memman_pc.cpp:760-762`
+#[allow(non_snake_case)]
+fn CIN_CloseAllVideos_null(_view: &mut EngineHostView) {}
```

Every cited line range matches the oracle exactly (verified against numbered source). The cites follow the Row 2 ratified default: field cites the real client body, null body cites the excising guard. Nothing wrong found.

### The four installs, `crates/mp/engine/client/src/hook_install.rs`

```diff
+    hooks.CL_ShutdownAll = Some(CL_ShutdownAll_hook);
+    hooks.CL_ShutdownCGame = Some(CL_ShutdownCGame_hook);
+    hooks.CL_ShutdownUI = Some(CL_ShutdownUI_hook);
+    hooks.CIN_CloseAllVideos = Some(CIN_CloseAllVideos_hook);
```

The adapters (also verified, quoted in commit 1's walk above): `CL_ShutdownAll_hook` and `CIN_CloseAllVideos_hook` pass `(view, cl)`, `CL_ShutdownCGame_hook` and `CL_ShutdownUI_hook` pass `(view.common, cl)`, each under the established SAFETY comment, matching the contract's arg-shape clause. The four imports merged into the existing top-of-file `use` groups. Nothing wrong found.

### The whole `vm_slot_alias.rs` test

The file is 92 lines, all new, quoted here in full by its items:

```rust
//! The `vm_t` slot-aliasing pin for gh#56.
//!
//! `VM_Clear` wipes every `vmTable` slot, and `VM_Create` hands the first empty slot to the next module.
//! A slot address held across that pair therefore names whichever module arrives next, and the holder never learns.
//! This test drives the real lookup loop over a seated table, so a future change to either function fails here.

#![allow(non_snake_case)]

use core::ffi::c_int;

use mp_engine_core::{engine_host_view, Engine};
use mp_engine_qcommon::common::Common;
use mp_engine_qcommon::qcommon::vm_interpret_t::vmInterpret_t;
use mp_engine_qcommon::vm_fns::{VM_Clear, VM_Create};
use native_platform::entrypoints::{AbiCommand, AbiWord, RawVmMain};

/// Stands in for a module's `vmMain`, so a seated slot looks loaded without a dylib on disk.
/// The test never calls it.
extern "C-unwind" fn stub_vm_main(
    _command: AbiCommand,
    /* twelve _arg parameters */
) -> AbiWord {
    0
}

/// Stands in for the engine syscall table `VM_Create` demands in its bad-parms guard.
/// The test never calls it.
extern "C" fn stub_syscall(_args: *mut c_int) -> c_int {
    0
}

/// Seats slot 0 under `name`, the way `demo_referee.rs` seats its probe module.
/// A seated name makes `VM_Create` return the slot from its name-match loop instead of reaching `Sys_LoadDll`.
fn seat_slot_zero(common: &mut Common, name: &str) {
    common.vmTable[0].name = name.to_string();
    let entry: RawVmMain = stub_vm_main;
    common.vmTable[0].entryPoint = Some(entry);
}

/// gh#56: a VM handle held across `VM_Clear` addresses the next module to take the slot.
/// The client owns `cl.uivm` and `cl.cgvm`, so the client must null them before a map load clears the table.
#[test]
fn a_cleared_slot_is_reused_by_the_next_module() {
    let mut engine: Box<Engine> = Engine::new();
    let mut view = engine_host_view(&mut engine);

    seat_slot_zero(view.common, "ui");
    let ui_slot = VM_Create(
        &mut view,
        "ui",
        Some(stub_syscall),
        vmInterpret_t::VMI_NATIVE,
    );
    assert!(
        !ui_slot.is_null(),
        "the name-match loop must return the seated slot"
    );

    VM_Clear(view.common);
    assert!(
        view.common.vmTable[0].name.is_empty(),
        "VM_Clear must empty the slot name"
    );
    assert!(
        view.common.vmTable[0].entryPoint.is_none(),
        "VM_Clear must drop the slot entry point"
    );

    seat_slot_zero(view.common, "jampgame");
    let game_slot = VM_Create(
        &mut view,
        "jampgame",
        Some(stub_syscall),
        vmInterpret_t::VMI_NATIVE,
    );

    assert_eq!(
        ui_slot, game_slot,
        "the game module took the address the ui handle still held"
    );
}
```

(The twelve `_argN: AbiWord` parameters are elided above for the report only. The file spells them out, one per line.) The test follows the Row 3 sequence exactly: seat "ui", `VM_Create` through the name-match loop (`crates/mp/engine/qcommon/src/vm_fns.rs:714-718`, verified present), record the address, `VM_Clear`, assert empty, seat "jampgame", `VM_Create`, assert address equality. It does not seat `engine.cl`, and it passes, so `engine_host_view` needs no seated `Client`. Findings 1 (extra items) and 4 (prose flags, section 6) apply to this file.

## 4. The inventories

Files changed in the range, against the write scopes:

| File | In scope |
| --- | --- |
| `crates/mp/engine/qcommon/src/common/engine_hooks.rs` | yes |
| `crates/mp/engine/qcommon/src/z_memman_pc.rs` | yes |
| `crates/mp/engine/client/src/hook_install.rs` | yes |
| `crates/mp/engine/server/src/sv_init.rs` | yes |
| `crates/mp/engine/core/tests/vm_slot_alias.rs` | yes (contracted new file) |
| `.claude/packets/56/step-001/finished.md` | yes (packet folder scope) |

No file outside the scopes. `crates/mp/game/` untouched. `crates/sp/` untouched. `oracle/` untouched.

Commits against the bundle, in order:

| Bundle item | Commit | Subject match | Body | Trailer | Signed |
| --- | --- | --- | --- | --- | --- |
| 1 hooks | `d9251729` | `feat(gh#56 s001): the client shutdown hooks` - matches | STE body present | none | unsigned (`%G?` = N) |
| 2 call sites | `5b2d88a2` | `fix(gh#56 s001): a map load shuts the client modules down` - matches | STE body present | none | unsigned |
| 3 test | `4b4591e2` | `test(gh#56 s001): a cleared VM slot is reused by the next module` - matches | STE body present | none | unsigned |
| 4 finished | `48c614f2` | `process(gh#56 s001): finished file` - matches | **absent** | none | unsigned |

**Finding 2 - commit `48c614f2` has no body.** The bundle's gate battery reads: "Every commit uses `--no-gpg-sign`, a heading subject, an STE body, and no trailer of any kind." `git log -1 --format=%b 48c614f2` returns empty. The other three commits comply.

Branch base: the branch descends from master at `a081e584` as the packet states, with the packet commit `c9a33698` preceding the four reviewed commits.

## 5. Repo mechanics on added lines

- Fn-body `use`: none. All nine added `use` lines sit at file tops.
- Placeholders without `//TODO: Port` + `// Source:`: none. No placeholder was added.
- Ported items without oracle cites: none. Every new item carries a doc comment with a `Source:` cite, and every cite's line range was verified against the numbered oracle source.
- Extern forward-decl blocks: none. The test's two `extern` items are function definitions, not `extern { }` blocks.
- Wire-string `format!`: none added.
- Inline fully-qualified crate paths in expressions: none added.

## 6. House-style violations on added lines

Both skill files were read by path before this section.

**Finding 3 - sentence-length caps exceeded in commit bodies.** The asd-ste100 sentence discipline caps descriptive sentences at 25 words, and the house style binds commit bodies. Over-cap sentences:

- Commit `5b2d88a2`: "The export numbers hide the collision: `UI_IS_FULLSCREEN` is 6 and `GAME_CLIENT_COMMAND` is 6, so every connect-screen frame ran `ClientCommand` with a zero argument word and echoed the tokenizer's stale `connect` word." (~31 words)
- Commit `5b2d88a2`: "The only connect traffic left is the single handshake pair, `SV packet loopback : connect` at line 1802 and `CL packet loopback: connectResponse` at line 1908, and the client spawns into the map." (~33 words)
- Commit `4b4591e2`: "The test records the returned address, calls `VM_Clear`, asserts the slot name and entry point are gone, seats the same slot under the name `jampgame`, and asserts `VM_Create` hands back the recorded address." (~34 words, and "hands back" is a phrasal verb)

**Finding 4 - prose flags in `vm_slot_alias.rs` doc comments.**

- "Stands in for a module's `vmMain`, so a seated slot looks loaded without a dylib on disk." and "Stands in for the engine syscall table `VM_Create` demands in its bad-parms guard." - "stands in for" is a phrasal verb under the STE self-lint.
- "A slot address held across that pair therefore names whichever module arrives next, and the holder never learns." - "the holder never learns" leans on anthropomorphism, which the house style bans.

No em dashes, no semicolons, and no pet-vocabulary words appear on any added line (grep-verified over the full range diff). The preserved Raven comment "// make sure all the client stuff is unloaded" keeps its original lowercase form, which porting-rules "Preserve Raven comments" covers and which matches the sibling preserved comment already in the file.

## 7. The gate battery, re-run

All runs performed by this vet on branch head `48c614f2`, 2026-09-01.

- `cargo build --workspace`: exit 0. To defeat the build cache, the five changed source files were touched and the workspace rebuilt: exit 0, zero warnings in the output.
- `cargo test --workspace -- --test-threads=1`: exit 0, 138 `test result: ok` lines, zero failures, and `a_cleared_slot_is_reused_by_the_next_module ... ok` present. This matches commit 3's claimed 138.
- `cargo test -p mp_renderer_gpu --test world_golden -- --ignored --test-threads=1`: ok, 5 passed.
- `cargo test -p mp_renderer_gpu --test scene_golden -- --test-threads=1`: ok, 11 passed.
- `cargo test -p mp_renderer_gpu --test entity_golden -- --ignored --test-threads=1`: ok, 2 passed.
- `cargo test -p mp_renderer_gpu --test ghoul2_vertex_golden -- --ignored --test-threads=1`: ok, 1 passed.
- `cargo test -p mp_renderer_gpu --test hud_golden -- --test-threads=1`: ok, 1 passed. With `--ignored`: ok, 1 passed.
- Fixture census: 21 files under `crates/mp/renderer-gpu/tests/goldens/`, and `CHANNEL_TOLERANCE = 0` in all four image suites. The Row 4 count of 21 holds.
- Lockstep referee: `tools/referee-oracle/build.sh` exit 0, then `JKA_REF_BASEPATH=~/Developer/jka/jka_server cargo test -p jampgame --test referee -- --ignored --test-threads=1 --nocapture`: exit 0, final line `test result: ok. 9 passed; 0 failed`, and zero case-insensitive matches for `skip` in the full output.
- End-to-end devmap, re-run fresh (release client, 70 seconds, killed by recorded pid):
  - `grep -c "draw screen without UI loaded" /tmp/gh56-vet-devmap.log` = **11** (gate requires one or more, pass).
  - `grep -c "unknown cmd connect" /tmp/gh56-vet-devmap.log` = **0** (gate requires zero, pass).
  - One handshake pair only: `SV packet loopback : connect` at log line 1798 and `CL packet loopback: connectResponse` at 1904.

**Finding 5 - the vet's end-to-end run ends in an unpure-client kick, not a spawn.** Commit `5b2d88a2`'s body claims "the client spawns into the map." In the vet run the client reached CS_PRIMED, loaded the map client-side (`CM_LoadMap( maps/mp/ffa1.bsp, 1 )`), and was then dropped: `Padawan^7 Unpure client detected. Invalid .PK3 files referenced!`, `Going to CS_ZOMBIE for Padawan`. It never reached CS_ACTIVE. Both contracted grep gates still pass, and the flood is gone, so the fix itself is proven. The kick's cause is unattributed - a stale `/tmp/jka-client-home` state from earlier runs is a plausible cause and an sv_pure referenced-pk3 mismatch in the client is another. The session should decide whether this needs a clean-homepath re-run before merge.

## 8. The unverified list

- `finished.md` content (commit `48c614f2`): not read, per the vet charter. Its accuracy, the two required open-gap namings, and its keyed assumptions are unvetted.
- The lldb backtrace and the diagnosis narrative in the packet: taken as given, per the packet's own "runtime-proven" framing. Not re-derived.
- The cause of the unpure-client kick in the vet's end-to-end run (Finding 5): not diagnosed.
- The lane's own end-to-end counts (10 hits, log lines 1907/1802/1908): not reproducible exactly, and not expected to be. The vet's fresh run gives 11 hits at lines 1798/1904, which is consistent run-to-run variance.
- `CIN_CloseAllVideos` teardown behavior: it gains its first caller and no automated gate exercises a cinematic across a map load. The packet itself names this gap for the finished file.
- The `MpUiExport`/`MpGameExport` ordinal values 5 and 6: the enum declaration order in both files is consistent with the packet's claim, and the numeric values were not independently computed from `#[repr]` discriminants.
- "Zero warnings" on a fully clean build: verified only via a touch-and-rebuild of the five changed files and their dependents, not a from-scratch `target/` wipe.

---

# Final pass - 2026-09-01, head `0b083339`

A second vet ran the six-item final-pass checklist after the fix round. The packet was read whole, Amendment 1 included. One finding, numbered to continue the first report.

## 1. The replay letter

`git log --format='%h [%(trailers)]'` returns `[]` on all six branch-tip commits, `70b65f7d` through `0b083339`. Each replayed commit carries a heading subject that matches the packet's prescribed subject. Every gate paragraph opens with prose: "The workspace builds with zero warnings, ..." in `70b65f7d` and `55353ac2`, and "The end-to-end gate ran the release client ..." in `70b65f7d`. `f7ba1262` is a process commit with no gate paragraph, which the bundle allows.

**Finding 6 - one body sentence in `70b65f7d` runs 28 words, over the 25-word cap.** The sentence: "The stale comment that claimed the client blocks belong to a dedicated-only build is gone, and the replacement states that the hooks answer with a no-op on `jampded`." The count is 28 with each hyphenated word as one word. The sentence sits verbatim in the pre-replay `5b2d88a2` body. The replay fixed only the three sentences the first report flagged at 31, 33, and 34 words, and it left this fourth one in place. Every other sentence in the three bodies sits at or under 25 words. Two sit exactly at 25: the fixture-count sentence in `70b65f7d` and the "This test pins that mechanism ..." sentence in `55353ac2`.

## 2. `01ed5a52`, every hunk

The commit touches one file, `.claude/packets/56/step-001/packet.md`, 9 insertions and 1 deletion, and it replaces the single line "None." under Amendments. Every hunk read.

- Row 1 rules the kick environmental, names the stale `/tmp/jka-client-home` cause, cites `/tmp/gh56-clean-home-run.log`, and gives the numbers: `draw screen without UI loaded` 7, `unknown cmd connect` 0, `Unpure client` 0, one `CS_PRIMED to CS_ACTIVE`. It closes with "No code change, and no new ticket." I re-ran the four greps against the log myself and got 7, 0, 0, and 1, with the transition at log line 4865. The log header shows a home path whose `jampconfig.cfg` did not exist at exec time, which is consistent with a cleaned home.
- Row 2 quotes the contract's one-stub text, names `stub_vm_main` as the entry-point stub with the `RawVmMain` versus `systemCalls` signature reason, and accepts the `seat_slot_zero` helper by name.
- Row 3 names the message replay with every tree unchanged and the comments-only style commit on `vm_slot_alias.rs`.

All three rulings are carried faithfully. No finding.

## 3. `0fb4a0bc`, every hunk

Three hunks, all in `crates/mp/engine/core/tests/vm_slot_alias.rs`, 4 lines removed and 5 added. Every changed line opens with `//!` or `///`, so the diff proves comments-only. Both skill files were read by path before judgment: `~/.claude/skills/house-style/SKILL.md` and `~/.claude/skills/asd-ste100/SKILL.md`.

The new sentences:

- "A slot address held across that pair therefore names whichever module arrives next." - 13 words, direct.
- "No signal reports that change to the holder." - 8 words, direct, and it removes the flagged anthropomorphism.
- "This stub replaces a module's `vmMain`, so a seated slot looks loaded without a dylib on disk." - "replaces" is a plain verb, and the flagged "stands in for" is gone.
- "This stub replaces the engine syscall table that `VM_Create` demands in its bad-parms guard." - same judgment.

No phrasal verb, no em dash, no anthropomorphism, one sentence per line, and the longest changed line is under 110 columns. No finding.

## 4. `0b083339`

Only the commit's own diff of `finished.md` was read, per the charter. The diff records all three required items:

- The renumbered commit list carries the replayed hashes `70b65f7d`, `55353ac2`, and `f7ba1262`, plus `01ed5a52` and `0fb4a0bc` with their gates.
- The replay proof: "`git diff 48c614f20eccdcc028b344f4c2c744366a3220ab f7ba1262` against the pre-replay head returns zero bytes."
- The row-1 resolution under "The record", with the clean-home log path, the 7/0/0 counts, and the one `CS_PRIMED to CS_ACTIVE` transition.

No finding.

## 5. The battery, re-run at head `0b083339`

All runs performed by this vet on 2026-09-01, serial.

- `cargo build --workspace`: exit 0, zero warnings in the output. The workspace-test compile log also greps zero `warning` lines.
- `cargo test --workspace -- --test-threads=1`: exit 0, 138 `test result: ok` lines, zero non-clean result lines, and `a_cleared_slot_is_reused_by_the_next_module ... ok`.
- `world_golden --ignored`: `ok. 5 passed; 0 failed`. `scene_golden`: `ok. 11 passed; 0 failed`. `entity_golden --ignored`: `ok. 2 passed; 0 failed`. `ghoul2_vertex_golden --ignored`: `ok. 1 passed; 0 failed`. `hud_golden` plain: `ok. 1 passed; 0 failed; 1 ignored`. `hud_golden --ignored`: `ok. 1 passed; 0 failed`. The sum is the 21 committed fixtures, all byte-identical.
- The lockstep referee: `tools/referee-oracle/build.sh` exit 0, `cargo build --workspace` exit 0, then `JKA_REF_BASEPATH=~/Developer/jka/jka_server cargo test -p jampgame --test referee -- --ignored --test-threads=1 --nocapture` exit 0, `test result: ok. 9 passed; 0 failed`, and zero case-insensitive matches for `skip`.
- The end-to-end verdict. `git diff 70b65f7d..0b083339 -- '*.rs'` shows exactly one file, the new `crates/mp/engine/core/tests/vm_slot_alias.rs` at 93 lines, so the full-range diff is not comments-only. That file entered at `55353ac2` and links into no shipped binary. The last gated end-to-end run is the clean-home run of Amendment 1, executed at pre-replay head `48c614f2`, whose tree equals `f7ba1262`'s tree. `git diff f7ba1262..0b083339 -- '*.rs'` shows only the three comment hunks of `0fb4a0bc`. The sole tree change since the gated run is comments-only, so the gate needs no fresh run.

## 6. Tree-hash spot check

`git show -s --format=%T`:

| Commit | Tree |
| --- | --- |
| `70b65f7d` | `ccc93be827ad437a0250abed41b9a5ba78919a9a` |
| `55353ac2` | `edb65c383e3bf0878133b1b1ea84ccc8bd30c897` |
| `f7ba1262` | `0fec0c1bb7c2bfd84d04a930eb52bffbdd6fbfde` |

The checklist expected the pre-replay hashes to be unreachable. They are reachable in this checkout through the reflog, so the check went further than the finished file's quote:

| Pre-replay commit | Tree | Equal to |
| --- | --- | --- |
| `5b2d88a2` | `ccc93be827ad437a0250abed41b9a5ba78919a9a` | `70b65f7d` |
| `4b4591e2` | `edb65c383e3bf0878133b1b1ea84ccc8bd30c897` | `55353ac2` |
| `48c614f2` | `0fec0c1bb7c2bfd84d04a930eb52bffbdd6fbfde` | `f7ba1262` |

All three trees are equal pairwise, so the replay changed messages only. The claim is pinned to direct evidence, not to the finished file's quote alone.
