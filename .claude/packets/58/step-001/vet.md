# Vet gh#58 step-001 - the referee rig repair

Range `af279749..gh58-step-001-referee-rig`, three commits, five files. The vet walked the packet whole, then each commit in order with `git show`, every hunk. The vet did not open `finished.md`. The vet does not approve.

Finding count: 7. Two are letter violations against the surface contract, one is a correctness defect in the added prose, one is a repo-mechanics conflict, two are style defects, and one is an inventory note.

## 1. Letter violations

**F1 - a comment site the surface contract does not list.** The contract at packet line 105 names two write sites in `build.sh`.

> `tools/referee-oracle/build.sh`: the `CXXFLAGS`-adjacent link commands in the `case "$OS"` block, and the header comment that states the toolchain requirement.

Commit `78a1c76e` writes a third site, a new two-line comment above the `case` block.

```
 # --- link the loadable module -------------------------------------------------
+# The two static flags keep the gcc runtime images out of the artifact.
+# The header comment above states why.
 case "$OS" in
```

The site is next to the link commands and the text is on subject. The contract still does not list it.

**F2 - the comment length exceeds the clause.** Commit 2's clause at packet line 136 sets the size.

> A new comment states the reason in one or two sentences: a dlclose before thread exit can strand a thread-specific-data destructor, each child process runs one drive and exits, and `GAME_SHUTDOWN` still runs first.

Commit `ea136782` writes three sentences at each of the two sites.

```rust
+    // `GAME_SHUTDOWN` runs above, so the module is finished with its work.
+    // A `dlclose` before the engine thread exits can strand a thread-specific-data destructor in unmapped memory.
+    // Each child process runs one drive and exits, so the mapping costs nothing.
```

```rust
+    // `GAME_SHUTDOWN` ran above, so the module is finished with its work.
+    // A `dlclose` before the engine thread exits can strand a thread-specific-data destructor in unmapped memory.
+    // The whole lifecycle runs once per process, so the mapping costs nothing.
```

No other letter violation is present. The branch creates no `pub` item, no type, no constant, no test, no cvar, no `FrameEvent` variant, no engine hook, no dependency, and no `#[repr]` change. The five changed files all sit inside the write scopes. No file under `oracle/`, `crates/mp/`, `crates/sp/`, `crates/native/`, `tools/cgame-oracle/`, or `crates/cgame/` changed. The crash check in `spawn_child` is untouched and still runs before the diff, at `crates/jampgame/tests/referee.rs:656`. No `_exit` call was added. The stash `stash@{0}: On gh54-step-002-cinematics: gh54-s002-commit1-wip` is intact.

## 2. Oracle divergences

The packet cites no Raven `oracle/**` line. Its only `oracle/` mentions are the read-only scope statement, the path of the built artifact, and the `oracle/codemp/game/*.c` set that `build.sh` already compiles. No commit changes ported logic, a float width, an operator, an evaluation order, a macro argument, a constant, or a side effect. There is no oracle ground to walk for this step, so this section is empty by the packet's own shape, not by sampling.

One behavior-adjacent check ran instead. The oracle artifact is byte-comparable to the recorded one by size, and every scenario compares byte-identical under the re-run battery in section 7.

## 3. The named hunks

**Hunk A - the two link commands in `tools/referee-oracle/build.sh`.**

```
 case "$OS" in
-	Darwin) "$CXX" -dynamiclib -o "$LIBOUT" build/obj/*.o -lm;;
-	Linux)  "$CXX" -shared -o "$LIBOUT" build/obj/*.o -lm;;
+	Darwin) "$CXX" -dynamiclib -o "$LIBOUT" build/obj/*.o -lm -static-libgcc -static-libstdc++;;
+	Linux)  "$CXX" -shared -o "$LIBOUT" build/obj/*.o -lm -static-libgcc -static-libstdc++;;
 esac
```

Both arms take the same pair, as row 3 ratified. The Darwin arm is proven by the gate battery. The Linux arm is untested here.

**Hunk B - the header comment in `tools/referee-oracle/build.sh`.**

```
+#
+# The link line adds `-static-libgcc -static-libstdc++`.
+# The Homebrew gcc runtime images are `libstdc++.6.dylib` and `libgcc_s.1.1.dylib`.
+# Each one registers a thread-specific-data key whose destructor is `emutls_destroy`.
+# The referee harness unloads the module before its engine thread exits, so the child calls that destructor in unmapped memory and dies with SIGSEGV.
+# The static link removes both images from the artifact, and the module then depends on libSystem alone.
```

The comment names both images and never names `libgcc_s` alone, as the attribution amendment requires. See F3 for the tense defect in the fourth line.

**Hunk C - the new subsection in `tools/referee-oracle/README.md`.**

```
+### The gcc runtime links statically
+
+The link line carries `-static-libgcc -static-libstdc++`. The Homebrew gcc runtime images are `libstdc++.6.dylib` and `libgcc_s.1.1.dylib`, and each one registers a thread-specific-data key whose destructor is `emutls_destroy`. The referee harness unloads the module before its engine thread exits, so the child calls that destructor in unmapped memory and dies with SIGSEGV. With the static link the artifact depends on `/usr/lib/libSystem.B.dylib` alone, and nothing registers the key. Check it with `otool -L build/liboraclejampgame.dylib`.
+
+A gcc version pin does not help here. The install name the linker records goes through the `gcc/current` symlink, so an artifact built under gcc 15 loads the gcc 16 runtime at load time.
```

The pin sentence that row 6 ordered is present. The paragraphs are unwrapped. See F3 for the tense defect in the third sentence.

**Hunk D - the tail of `drive` in `crates/jampgame/tests/referee.rs`.**

```rust
     referee_vm_call(vm, MpGameExport::GAME_SHUTDOWN, &[0]);
-    drop(module);
+    // `GAME_SHUTDOWN` runs above, so the module is finished with its work.
+    // A `dlclose` before the engine thread exits can strand a thread-specific-data destructor in unmapped memory.
+    // Each child process runs one drive and exits, so the mapping costs nothing.
+    std::mem::forget(module);
     snaps
 }
```

`GAME_SHUTDOWN` still runs on the line above, and `snaps` still returns.

**Hunk E - the tail of `run_lifecycle` in `crates/jampgame/tests/common/mod.rs`.**

```rust
         eprintln!("=================================\n");
     });
+
+    // `GAME_SHUTDOWN` ran above, so the module is finished with its work.
+    // A `dlclose` before the engine thread exits can strand a thread-specific-data destructor in unmapped memory.
+    // The whole lifecycle runs once per process, so the mapping costs nothing.
+    std::mem::forget(module);
 }
```

The old code dropped `module` at the closing brace with no `drop` line, so the hunk removes no line. That matches the surface-wording amendment.

## 4. The inventories

**Files against the write scopes.**

| file | commit | scope status |
| --- | --- | --- |
| `tools/referee-oracle/build.sh` | `78a1c76e` | writable, see F1 for the site |
| `tools/referee-oracle/README.md` | `78a1c76e` | writable |
| `crates/jampgame/tests/referee.rs` | `ea136782` | writable |
| `crates/jampgame/tests/common/mod.rs` | `ea136782` | writable, `run_lifecycle` tail only, satisfied |
| `.claude/packets/58/step-001/finished.md` | `c9abae2d` | writable |

No file outside the scopes changed. `tools/referee-oracle/build/` stays untracked, and the working tree is clean after the full gate battery.

**Commits against the bundle.** Three planned, three landed, in order, with the exact planned subjects. No split, no reorder, no widened commit, no unplanned commit, no bundled commit. Commit 1 carries the two `build.sh` and `README.md` files the bundle names. Commit 2 carries the two test files the bundle names. Commit 3 carries `finished.md` alone.

**Commit messages.** Each subject is a heading in the planned form. Each body is unwrapped STE prose with no semicolon, no em dash, and no contraction. No commit carries a trailer of any kind, and no commit carries a `Co-Authored-By` line or a generated-with footer. All three commits report `N` for the signature field, which is the expected result of `--no-gpg-sign`.

**Note N1 - a packet line reference does not match the file.** The write scope at packet line 164 reads "`crates/jampgame/tests/common/mod.rs`, the `run_lifecycle` tail only" and the surface contract at line 108 reads "the tail of `run_lifecycle`, at `:922` through its closing brace". Line 922 is the `fn run_lifecycle(dylib: PathBuf) {` signature, and the function's closing brace is at line 1159. The edit sits at lines 1151 to 1156, inside the tail. The named constraint holds. The packet's line number is the item that is wrong, not the commit.

## 5. Repo mechanics on added lines

The branch adds 72 lines. Of those, 47 are `finished.md`, which the vet did not open. The remaining 25 added lines are 20 comment or prose lines, two blank lines, one heading line, and the two link-command lines.

- No `use` declaration inside a function body. The added Rust lines declare no import.
- No `todo!()` and no other placeholder. There is no marker to check for a `//TODO: Port` subject or a `// Source:` cite.
- No newly ported item, so no oracle `Source:` cite is owed.
- No extern forward-declaration block.
- No `format!` call, so no wire string is built.

**F4 - an inline fully-qualified path in an expression.** `docs/porting-rules.md` states the rule without a `std` carve-out.

> **No inline fully-qualified crate paths in expressions.** Reference a symbol by its short name (imported at file top), not by an inline `crate::a::b::c` / `mp_qshared::x::y` path spelled out at the call site.

Both added statements spell the path inline.

```rust
    std::mem::forget(module);
```

The rule's examples are crate paths, and `crates/jampgame/tests/referee.rs` already writes `std::env::var`, `std::process::Command`, and `std::process::ExitStatus` inline on unchanged lines. The vet reports the conflict and does not waive it.

## 6. House-style violations on added lines

The vet read `~/.claude/skills/house-style/SKILL.md` and `~/.claude/skills/asd-ste100/SKILL.md` by path before this section.

Clean on the mechanical checks: no em dash on any added line, no semicolon in any added prose sentence, no pet vocabulary, no contraction, no banned-voice construction, no lowercase sentence start, no `(???)` marker, and no line over 150 columns. The longest added comment line is 149 columns. Every added comment line ends a sentence, so no line is a column wrap. The added comments are why-lines and disposition lines, not mechanics narration.

**F3 - the added prose states a fact that the same branch makes false.** Commit 2 stops the referee harness from unloading the module. Commit 1's header comment and the README section both state the unload in the present tense, and both keep standing after commit 2 lands.

`tools/referee-oracle/build.sh`:

```
+# The referee harness unloads the module before its engine thread exits, so the child calls that destructor in unmapped memory and dies with SIGSEGV.
```

`tools/referee-oracle/README.md`:

```
+The referee harness unloads the module before its engine thread exits, so the child calls that destructor in unmapped memory and dies with SIGSEGV.
```

After `ea136782`, `drive` and `run_lifecycle` both call `std::mem::forget(module)`, and no path in `crates/jampgame/tests/` calls `dlclose` on the oracle module. A reader of the merged tree finds no unload. The sentence needs a past tense or a conditional form, for example "unloaded" or "can unload". This is a content defect under the /// docs and prose content rules, not a form defect.

**F5 - a commit-body sentence that does not carry a meaning.** Commit `ea136782`, body paragraph 1:

> `GAME_SHUTDOWN` still runs first, so the module finishes its work before the mapping stays.

"before the mapping stays" is not a state a reader can resolve. STE requires one meaning per word and a sentence that makes good sense.

**F6 - the same fact in two tenses.** The two comment blocks open with the same sentence in two forms, "runs above" in `referee.rs:308` and "ran above" in `common/mod.rs:1154`. STE requires one name for one thing. The two sites describe the identical relation to the line above them.

## 7. The gate battery, re-run

The vet ran every gate the bundle names, with the packet's exact invocation, on the branch head `c9abae2d`, alone on the rig.

**Commit 1, gate 1.** `tools/referee-oracle/build.sh`

```
referee-oracle: compiled 89 TUs
referee-oracle: linked build/liboraclejampgame.dylib
referee-oracle: OK — dllEntry + vmMain exported
```

**Commit 1, gate 2.** `otool -L tools/referee-oracle/build/liboraclejampgame.dylib`

```
tools/referee-oracle/build/liboraclejampgame.dylib:
	build/liboraclejampgame.dylib (compatibility version 0.0.0, current version 0.0.0)
	/usr/lib/libSystem.B.dylib (compatibility version 1.0.0, current version 1356.0.0)
```

The first line is the artifact's own install name, and `/usr/lib/libSystem.B.dylib` is the only dependency. `nm -m` on the artifact finds no `emutls` symbol and no `pthread_key_create`. The artifact has 26 undefined symbols, all libc or libm, which restores the README's claim at line 23. The built size is 1,715,656 bytes, the number commit 1's body states.

**Commit 1, gate 3 and commit 2, gate 1.** `cargo build --workspace`

```
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.69s
```

**Commit 1, gate 4 and commit 2, gate 2.** `cargo test -p jampgame --test referee -- --ignored --test-threads=1`, run twice.

```
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 18.39s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 17.99s
```

Nine of nine, both runs. No output line in either run contains "signal", "SIGSEGV", "crash", "diverg", or "FAILED". The byte-identity claim is re-run and not taken on trust. A third run under `--nocapture` prints the diff summary:

```
===== referee PASS — scenario 'idle': 101 frames byte-identical; 202 client-states, 1599 entity-states, 17177 syscalls compared =====
```

The crash check is intact and still precedes the diff. `spawn_child` at `crates/jampgame/tests/referee.rs:656` reads `if !status.success()` and `describe_exit` still reports a signal-killed child as a hard crash. The tree is clean after the runs, so `regenerate_logs` rewrote no committed reflog.

**Commit 2, gate 3.** `cargo test -p jampgame --test oracle_smoke -- --ignored --test-threads=1`

```
test oracle_smoke_init_frames_shutdown ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

**Commit 1, gate 5 and commit 2, gate 4.** `cargo test --workspace -- --test-threads=1`

Exit status 0. 138 result blocks, all `ok`, 530 tests passed, 0 failed. The block count matches the audit's measurement.

Commit 3 names no gate.

## 8. The unverified list

1. `.claude/packets/58/step-001/finished.md`, all 47 lines. The vet is barred from opening it. Its content, its style, and its open-gaps section, including the naming of issue #59, are unchecked. The vet confirmed only the file path, the line count, and the commit that carries it.
2. The Linux link arm. `Linux)  "$CXX" -shared -o "$LIBOUT" build/obj/*.o -lm -static-libgcc -static-libstdc++;;` never ran. No Linux host is available here, and `.github/workflows/build.yml` never invokes `build.sh`. The arm ships untested, which row 3 ratified.
3. The ablation. The vet ran the two fixes together only. It did not build a pristine dylib and rerun each fix alone, so the packet's claim that commit 1 alone repairs both tests and commit 2 alone repairs the referee suite only is unverified here.
4. The size comparison. The vet measured the static artifact at 1,715,656 bytes. It did not build the dynamic artifact, so "the same size as before" and the inference "nothing was used from libstdc++" rest on the packet's measurement, not on this vet's.
5. The attribution of the dangling destructor to `libstdc++.6.dylib`. The crash no longer reproduces after the fix, so the vet could not re-derive the `0x17a0` offset evidence from a live fault. The static link removes both images, so the fix holds under either attribution.
6. The claim in commit 2's body that "the smoke lifecycle runs once per process because of the module's own singletons". The vet confirmed that `run_lifecycle` has one call site on one spawned thread. It did not test a second in-process lifecycle, so the singleton reason is unchecked.
7. The long-run leak posture of `std::mem::forget(module)`. `LoadedModule` holds a `libloading::Library` and a function pointer, so the leak is the mapping alone. The vet did not measure a process that drives many modules, because no such path exists in these two tests today.
8. The `~/Developer/jka/` asset reads the real-map scenarios make. The vet ran them and they passed. It did not inspect that directory.

---

# Rewalk 2026-09-08 - after the fix round

The branch was rewritten in place. The rewalk covers `a2168bf9` (commit 2, body only), `329e47e3`, `d890bcc6`, and `c2b8f7f1`, with the same eight duties. The vet did not open `finished.md` at either revision. The vet does not approve.

Finding count for the rewalk: 2. Both are style findings on added prose. F3, F4, F5, and F6 are repaired. F1, F2, and N1 are closed by the packet amendment.

**Commit 2 moved no tree.** `git diff ea136782 a2168bf9 --stat` prints nothing. The rewrite is a message rewrite, so every gate result in the first walk still applies to that tree.

## 1. Letter violations

None. The packet's current Amendments, added by `c2b8f7f1`, close F1 and F2 as they stand and correct the `:922` cite to `crates/jampgame/tests/common/mod.rs:1151-1156`. The fix round adds no `pub` item, no type, no constant, no test, no cvar, no `FrameEvent` variant, no engine hook, no trap or dispatcher arm, and no `#[repr]` layout. It adds one dependency-free import per file, `use std::mem::forget;`, which is a module-scope import of an existing `std` symbol and not a new crate dependency. Every changed file sits inside the write scopes, `.claude/packets/58/step-001/` included.

## 2. Oracle divergences

None. The packet still cites no Raven `oracle/**` line, and the fix round changes two comments, two call expressions, one README paragraph, and one commit message. No ported logic, float width, operator, evaluation order, macro argument, constant, or side effect changes. `forget(module)` and `std::mem::forget(module)` are the same call.

## 3. The named hunks

**Hunk F - the two `forget` call sites and their imports.**

```rust
 mod common;
 
+use std::mem::forget;
 use std::path::{Path, PathBuf};
```

```rust
     // Each child process runs one drive and exits, so the mapping costs nothing.
-    std::mem::forget(module);
+    forget(module);
     snaps
```

```rust
 use std::ffi::{c_char, c_int, c_short, c_void, CStr, CString};
+use std::mem::forget;
 use std::path::{Path, PathBuf};
```

```rust
-    // `GAME_SHUTDOWN` ran above, so the module is finished with its work.
+    // `GAME_SHUTDOWN` runs above, so the module is finished with its work.
     // A `dlclose` before the engine thread exits can strand a thread-specific-data destructor in unmapped memory.
     // The whole lifecycle runs once per process, so the mapping costs nothing.
-    std::mem::forget(module);
+    forget(module);
```

Each import merges into the file's existing `std` group in alphabetical order, and neither file gains a second use block. No import sits inside a function body.

**Hunk G - the `build.sh` header correction.**

```
-# The referee harness unloads the module before its engine thread exits, so the child calls that destructor in unmapped memory and dies with SIGSEGV.
+# The referee harness once dropped the module on its engine thread, which unloaded both images with the destructor still registered.
+# The child then called that destructor in unmapped memory and died with SIGSEGV.
 # The static link removes both images from the artifact, and the module then depends on libSystem alone.
+# The harness also keeps the module mapped to thread exit now, which is belt and braces.
+# The static link is the root fix.
```

**Hunk H - the README correction.** The subsection's first paragraph is replaced whole. Its new third and fourth sentences read "The referee harness once dropped the module on its engine thread, which unloaded both images with the destructor still registered. The child then called that destructor in unmapped memory and died with SIGSEGV." Its new last two sentences read "The harness also keeps the module mapped to thread exit now, which is belt and braces. The static link is the root fix." See F7.

**Hunk I - the packet amendment.** `c2b8f7f1` appends one dated block with four bullets to `packet.md`. It edits no earlier line, so the packet's earlier `:922` cite still stands in the body and the amendment corrects it below, which is the ledger's own pattern.

## 4. The inventories

**Files changed by the fix round.**

| file | commit | scope status |
| --- | --- | --- |
| `crates/jampgame/tests/referee.rs` | `329e47e3` | writable |
| `crates/jampgame/tests/common/mod.rs` | `329e47e3` | writable |
| `tools/referee-oracle/README.md` | `329e47e3` | writable |
| `tools/referee-oracle/build.sh` | `329e47e3` | writable |
| `.claude/packets/58/step-001/finished.md` | `d890bcc6` | writable, not opened |
| `.claude/packets/58/step-001/packet.md` | `c2b8f7f1` | writable |

The whole range now touches seven files. No file outside the write scopes changed at any revision. `oracle/`, `crates/mp/`, `crates/sp/`, `crates/native/`, `tools/cgame-oracle/`, and `crates/cgame/` are untouched. The other lane's stash is intact. The working tree is clean after the full battery.

**Commits against the bundle.** The bundle plans three commits. The branch carries seven. The four extra commits are the lane-review ceremony: the vet, the fix round, the finished-file update, and the packet amendment. The current Amendments name every one of them, so the widening is authorized by the ratified packet and is not a finding. The two `fix` commits keep their planned subjects, and the code content of the bundle is unchanged in kind: commit 1 is untouched, and commit 2's tree is byte-identical to the walked one.

**Commit messages.** Each of the four new commits carries a heading subject in the lane's form, an unwrapped STE body, no semicolon, no em dash, no contraction, and no trailer of any kind. All six rewritten or new commits report `N` for the signature field, which `--no-gpg-sign` produces. `f9dfa968` is the earlier vet report, and `git diff f9dfa968 gh58-step-001-referee-rig -- .claude/packets/58/step-001/vet.md` prints nothing, so the rebase did not alter the report's text.

## 5. Repo mechanics on added lines

- No `use` declaration inside a function body. Both new imports sit at the file top, in the existing `std` group. F4 is repaired.
- No `todo!()` and no other placeholder.
- No newly ported item, so no oracle `Source:` cite is owed.
- No extern forward-declaration block.
- No `format!` call, so no wire string is built.
- No inline fully-qualified path in an expression on any added line. Both call sites now read `forget(module);`.
- `cargo test --no-run -p jampgame` prints no warning, so neither import is unused in any test binary that includes `common/mod.rs`.

## 6. House-style violations on added lines

Clean on the mechanical checks. No em dash on any added line, no semicolon in any added prose sentence, no contraction, no banned-voice construction, no lowercase sentence start, and no line over 150 columns. The longest added comment line is 132 columns. Every added comment line ends a sentence.

**F7 - the README paragraph passes the six-sentence cap.** STE's STRUCTURE rule sets it: "One topic per paragraph, max six sentences." The rewritten paragraph carries eight sentences, from "The link line carries `-static-libgcc -static-libstdc++`." through "The static link is the root fix." The reader also meets the instruction "Check it with `otool -L build/liboraclejampgame.dylib`." in position six, with two more statements after it. The added material belongs in a second paragraph, or after the pin paragraph.

**F8 - an idiom on four added lines.** "which is belt and braces" appears in `tools/referee-oracle/build.sh` and in `tools/referee-oracle/README.md`, and the phrase carries a figurative meaning that STE's WORDS rule does not admit. A plain form is "which is a second guard", the term commit 2's own body already uses. The packet uses "belt and braces" in row 1 and in its Amendments, so the lane took the word from its brief. The vet reports the conflict and does not waive it.

## 7. The gate battery, re-run

The vet re-ran commit 2's four gates with the packet's exact invocations, on the branch head `c2b8f7f1`, alone on the rig.

**Gate 1.** `cargo build --workspace`

```
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.41s
```

Both test files were touched first, so the two changed targets rebuilt. No warning appeared.

**Gate 2.** `cargo test -p jampgame --test referee -- --ignored --test-threads=1`

```
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 18.04s
```

Nine of nine. No crash finding.

**Gate 3.** `cargo test -p jampgame --test oracle_smoke -- --ignored --test-threads=1`

```
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

**Gate 4.** `cargo test --workspace -- --test-threads=1`

Exit status 0. 138 result blocks, all `ok`, 530 tests passed, 0 failed. The tree is clean after the run.

Commit 1's gates were not re-run in this rewalk. `329e47e3` changes only comment text in `build.sh`, and the link commands are byte-identical to the walked ones.

## The finding dispositions

- **F1**, ratified as it stands by the amendment. The pointer comment above the `case "$OS"` block is an accepted third site. Closed.
- **F2**, ratified as it stands by the amendment. The three-sentence comment blocks stand. Closed.
- **F3**, repaired in `329e47e3`. Both sentences are in the past tense and both are true of the merged tree, and both files also record the present behavior. Verified against the source at `tools/referee-oracle/build.sh:27-31` and `tools/referee-oracle/README.md:58`.
- **F4**, repaired in `329e47e3`. `use std::mem::forget;` sits in each file's top `std` group, and both call sites read `forget(module);`. No unused-import warning appears.
- **F5**, repaired in `a2168bf9`. The body now reads "`GAME_SHUTDOWN` still runs on the line above. The module therefore completes its work, and only its mapping remains." The sentence carries a meaning.
- **F6**, repaired in `329e47e3`. Both comment blocks now open with "`GAME_SHUTDOWN` runs above, so the module is finished with its work."
- **N1**, corrected in the packet's Amendments to `crates/jampgame/tests/common/mod.rs:1151-1156`.
- **F7** and **F8** are open and new.

## 8. The unverified list

1. `.claude/packets/58/step-001/finished.md`, at both `8fed1292` and `d890bcc6`, now 77 lines. The vet is barred from opening it. Its fix-round section, its commit list, its gate table, and its open-gaps section are unchecked. The vet confirmed only the path, the line counts, and the two commits that carry it.
2. The Linux link arm, still untested. No Linux host is available here.
3. The ablation of the two fixes, still not run separately by this vet.
4. The size comparison against a pristine dynamic artifact, still not built by this vet.
5. The attribution of the dangling destructor to `libstdc++.6.dylib`, still not re-derived from a live fault.
6. The claim that the smoke lifecycle runs once per process because of the module's own singletons, carried over unchanged into `a2168bf9`'s body. The vet confirmed one call site on one spawned thread and did not test a second in-process lifecycle.
7. Commit 1's five gates in this rewalk. Only the comment text in `build.sh` changed, and the vet compared the link commands rather than rebuilding the artifact.
8. The reflog and fixture set. The vet observed a clean tree after every run and did not hash the committed reflogs.
