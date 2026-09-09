# Packet gh#58 step-001 - the referee rig repair

Ratified 2026-09-08 on branch `gh58-step-001-referee-rig`, cut from master `1cb6c525`. All seven open rows are closed and the vet's new row is closed with them. The audit is at `.claude/packets/58/step-001/audit.md` (`1f565841`), and the folds below carry its one correction and its two notes. The shape reference is `.claude/packets/54/step-002/packet.md` on `gh54-step-002-cinematics`.

## Scope

This step repairs the lockstep referee rig. The rig fails 8 of its 9 scenarios on the development Mac, and no failure is a parity divergence. The oracle child writes a reflog byte-identical to the Rust child's, then dies with SIGSEGV at process exit, and the driver judges that exit status before it compares the two logs.

The step delivers the root-cause repair. The oracle dylib stops loading the Homebrew gcc runtime dylibs, and the referee children stop unloading a module whose thread-specific-data destructor is still registered. The gate is the full suite green, 9 of 9, with every oracle child exiting 0.

The step does not weaken the comparison, does not reorder the driver's crash check to hide a signal, does not add an `_exit` call, does not touch `oracle/`, does not touch `crates/mp/`, and does not add a crate. The gh#54 step-002 cinematics lane resumes after this merges.

## The diagnosis, with evidence

### The crash

The faulting thread is the harness engine thread, and the stack is thread-specific-data cleanup.

Report: `~/Library/Logs/DiagnosticReports/referee-64bc2153f45e86af-2026-09-08-203851.ips`.

```
exception: EXC_BAD_ACCESS (SIGSEGV), KERN_INVALID_ADDRESS at 0x000000010ada17a0
faultingThread: 2, name "referee-engine"
  <no owning image> + 0x10ada17a0
  libsystem_pthread.dylib  _pthread_tsd_cleanup + 488
  libsystem_pthread.dylib  _pthread_exit + 84
  libsystem_pthread.dylib  _pthread_start + 148
  libsystem_pthread.dylib  thread_start + 8
```

The top frame carries no image index, and no entry in the report's image list covers the address. The destructor that `_pthread_tsd_cleanup` calls lives in memory that is no longer mapped.

### Who registers the destructor

`tools/referee-oracle/build.sh` links the oracle module with `g++-16`, and the produced dylib pulls in the Homebrew gcc runtime.

```
$ otool -L tools/referee-oracle/build/liboraclejampgame.dylib
	/opt/homebrew/opt/gcc/lib/gcc/current/libstdc++.6.dylib
	/usr/lib/libSystem.B.dylib
```

`DYLD_PRINT_LIBRARIES=1` on the oracle child shows the transitive third image:

```
dyld: .../tools/referee-oracle/build/liboraclejampgame.dylib
dyld: /opt/homebrew/Cellar/gcc/16.2.0/lib/gcc/current/libstdc++.6.dylib
dyld: libstdc++.6.dylib has weak-def (or flat lookup) symbol used by libgcc_s.1.1.dylib
```

Both runtime images carry their own key and destructor. `nm -m` on each shows `_emutls_key`, `_emutls_destroy`, and an undefined `_pthread_key_create` from libSystem. The oracle dylib itself contains no `emutls` symbol, so the key comes from the gcc runtime and not from Raven's code.

The dangling destructor is the one in `libstdc++.6.dylib`. `_emutls_destroy` sits at `0x17a0` in `libstdc++.6.dylib` and at `0x11a00` in `libgcc_s.1.1.dylib`, and every fault address ends in `0x17a0`: `0x10ada17a0` in the report above, `0x1073b97a0` in the audit's rerun, and `0x10aae97a0` in the cgame sibling. `-static-libstdc++` is therefore the load-bearing flag. The pair stays, because `-static-libgcc` closes the same hazard in the second image at no cost.

### Who unloads it

`crates/jampgame/tests/referee.rs:308` ends `drive` with `drop(module)`. `LoadedModule` holds a `libloading::Library` (`crates/native/platform/src/module_loader/loaded_module.rs:18`), and dropping it calls `dlclose`. That drop releases the last reference to the oracle dylib, so dyld unloads it and its two gcc runtime images. The engine thread then exits, `_pthread_tsd_cleanup` calls `emutls_destroy` through the still-registered key, and the pointer is dangling.

The Rust child survives the same drop because its cdylib links no gcc runtime and registers no such key.

### Why now

The referee was last green on 2026-09-01. Homebrew `gcc` moved to 16.2.0 on 2026-09-04, and `brew list --versions gcc` reports `gcc 16.2.0`. `/opt/homebrew/bin/g++-16` is the only real g++ on this host, so `build.sh`'s auto-detect loop picks it.

The coupling is the `gcc/current` symlink, not the build date. The install name recorded in the artifact is `/opt/homebrew/opt/gcc/lib/gcc/current/libstdc++.6.dylib`, so an artifact built under gcc 15 loads the gcc 16 runtime at load time. A rebuild was never needed to expose the defect, and a toolchain pin would not have prevented it.

### The second victim

`crates/jampgame/tests/oracle_smoke.rs` fails the same way. `run_lifecycle` in `crates/jampgame/tests/common/mod.rs` loads the oracle dylib and drops the module at the end of the function body, on a thread named `abi-smoke-engine`. The test is `#[ignore]`d and is not on any gate battery, so it hid.

```
$ cargo test -p jampgame --test oracle_smoke -- --ignored --test-threads=1
test oracle_smoke_init_frames_shutdown ... error: test failed
  process didn't exit successfully: ... (signal: 11, SIGSEGV: invalid memory reference)
```

## The two candidate fixes, both tried and both proven

The drafter tried each candidate in the working tree and reverted every experiment. The tree is clean at the packet commit, and the defect reproduces.

**Candidate B, the link line.** Add `-static-libgcc -static-libstdc++` to `build.sh`'s Darwin link command. The rebuilt dylib links `/usr/lib/libSystem.B.dylib` and nothing else, and `nm -m` finds no `emutls` symbol and no `pthread_key_create` in it. No key is created, so no destructor can dangle. Nothing was actually used from libstdc++, because the linked artifact loses the dependency entirely rather than absorbing a copy.

Result: `referee_idle` green with the driver untouched, the whole suite green, and `oracle_smoke` green.

**Candidate A, the module lifetime.** Replace `drop(module)` at `crates/jampgame/tests/referee.rs:308` with `std::mem::forget(module)`. The image stays mapped for the rest of the child's short life, so a registered destructor always has code behind it. `GAME_SHUTDOWN` still runs on the line above, and each child drives exactly one module once, so nothing accumulates.

Result: `referee_idle` green with the pristine oracle dylib, and the whole suite green.

Candidate A alone does not repair `oracle_smoke`, because that load and drop live in `crates/jampgame/tests/common/mod.rs`.

**The combined run.** With both applied, the full suite passed 9 of 9 in 17.49 seconds, every oracle child exited 0, and every scenario reported byte-identical. The static link changes no snapshot byte.

**The audit reran each candidate alone.** Candidate B alone repairs both tests: the referee suite at 9 of 9, and `oracle_smoke` green. Candidate A alone repairs the referee suite at 9 of 9, and `oracle_smoke` still dies by signal 11. Candidate B is therefore the root fix and candidate A is belt and braces. Candidate A costs nothing, because `run_referee` (`crates/jampgame/tests/referee.rs:587-612`) runs each drive in a child process that exits right after its one drive, and no in-process reload path exists. The static dylib is 1,715,656 bytes, the same size as the pristine one, so the static link absorbs no libstdc++ code and nothing was used from it.

### The candidates that mask, both rejected as defaults

Calling `_exit` after the reflog write in the oracle child hides the signal and leaves the dangling destructor in place. Comparing the snapshots before judging the exit status keeps the signal visible, because a check that sits after the diff still reports a crash. The check-first order stands anyway. It is the order that surfaced this defect, and a crash after a clean reflog is a finding the rig must report. Row 4 and row 5 carry both for the user.

Building the oracle with Apple clang is not available. `README.md`'s toolchain section records why: `FOFS(x) ((int)&(((gentity_t *)0)->x))` is a hard error in C++ on a 64-bit host, and Apple clang treats `-fpermissive` as a silent no-op.

## Surface contract

This step creates no `pub` item, no type, no constant, and no test. It changes four files.

- `tools/referee-oracle/build.sh`: the `CXXFLAGS`-adjacent link commands in the `case "$OS"` block, and the header comment that states the toolchain requirement.
- `tools/referee-oracle/README.md`: the toolchain section, which gains the static-link rationale.
- `crates/jampgame/tests/referee.rs`: the last two lines of `drive`, plus a new comment above them. No comment stands above `drop(module)` at `:307-308` today, so the lane adds one and edits none.
- `crates/jampgame/tests/common/mod.rs`: the tail of `run_lifecycle`, at `:922` through its closing brace.

Anything not on this list is out of scope, and the agent must not add it. No new `pub` item, no new crate, no new cvar, no new test, no ABI change, no change to any scenario, reflog, or committed fixture, and no edit under `oracle/` or `crates/mp/`.

## Commit bundle

Every commit uses `git commit --no-gpg-sign`, a heading subject, an STE body, and no trailer of any kind.

### Commit 1 - `fix(gh#58 s001): the referee oracle links the gcc runtime statically`

Files: `tools/referee-oracle/build.sh`, `tools/referee-oracle/README.md`.

The Darwin and Linux link commands both gain `-static-libgcc -static-libstdc++`. The header comment and the README toolchain section record why: `libstdc++.6.dylib` registers a thread-specific-data key whose destructor is `emutls_destroy`, `libgcc_s.1.1.dylib` registers its own, and the harness unloads the module before the thread exits. Name `libstdc++` as the owning image, or name both images. Do not name `libgcc_s` alone. The README also gains the one sentence that explains the pin question: the recorded install name goes through `gcc/current`, so an artifact built under gcc 15 loads the gcc 16 runtime at load time. The README's existing claim that the module's only undefined symbols are libc and libm becomes true again.

The Linux arm takes the same two flags, so the script stays one rule. That arm is untested. No Linux host here runs it, and `.github/workflows/build.yml` never invokes `build.sh`.

Gates:

1. `tools/referee-oracle/build.sh` runs clean and prints `compiled 89 TUs`.
2. `otool -L tools/referee-oracle/build/liboraclejampgame.dylib` names `/usr/lib/libSystem.B.dylib` and no other dependency.
3. `cargo build --workspace`
4. `cargo test -p jampgame --test referee -- --ignored --test-threads=1` passes 9 of 9 with no crash finding.
5. `cargo test --workspace -- --test-threads=1`

### Commit 2 - `fix(gh#58 s001): the referee children keep the module mapped to thread exit`

Files: `crates/jampgame/tests/referee.rs` and `crates/jampgame/tests/common/mod.rs`.

`drive` ends with `std::mem::forget(module)` instead of `drop(module)`, and `run_lifecycle` does the same with its module. A new comment states the reason in one or two sentences: a dlclose before thread exit can strand a thread-specific-data destructor, each child process runs one drive and exits, and `GAME_SHUTDOWN` still runs first.

Gates:

1. `cargo build --workspace`
2. `cargo test -p jampgame --test referee -- --ignored --test-threads=1` passes 9 of 9 with no crash finding.
3. `cargo test -p jampgame --test oracle_smoke -- --ignored --test-threads=1` passes.
4. `cargo test --workspace -- --test-threads=1`

### Commit 3 - `process(gh#58 s001): the finished file`

File: `.claude/packets/58/step-001/finished.md`. No gate beyond the packet skill's contract.

### Gate notes

The referee suite needs both artifacts first: `tools/referee-oracle/build.sh`, then `cargo build --workspace`. The rig runs one agent at a time, and the lane agent is the only one on it. The world goldens are not on this battery, because no commit touches the renderer. `cargo test --workspace` never runs without `--test-threads=1`, or the two world-golden tests abort in the pk3 inflate path.

Both batteries are known reachable. The drafter ran the full referee suite green 9 of 9 with the two fixes applied, and ran `cargo test --workspace -- --test-threads=1` green on the pristine tree. The audit reran every gate and measured it: `build.sh` 40 s, the referee suite 20 s, `oracle_smoke` 3 s, and the workspace tests 36 s over 138 green result blocks.

## Write scopes

Branch: `gh58-step-001-referee-rig`, already checked out at `1cb6c525`. Never commit on master.

Writable:

- `crates/jampgame/tests/referee.rs`
- `tools/referee-oracle/build.sh` and `tools/referee-oracle/README.md`
- `.claude/packets/58/step-001/`
- `crates/jampgame/tests/common/mod.rs`, the `run_lifecycle` tail only

Everything else is read-only, including `oracle/`, every crate under `crates/mp/` and `crates/sp/`, `crates/native/`, `tools/cgame-oracle/`, `crates/cgame/`, every committed fixture and reflog, and `~/Developer/jka/` beyond the read-only asset reads the real-map scenarios already make. Source files change through the Edit tool only. `tools/referee-oracle/build/` is generated and gitignored, so the agent runs the build script but commits nothing from that directory.

There is a stash from another lane in the stash list. The agent must never touch it.

## Disposition

After a clean lane-review: open the pull request from `gh58-step-001-referee-rig` into master and merge it on GitHub with a merge commit, per DEC-67. Never squash. The session never pushes or opens the pull request unprompted. It prepares the branch, asks, and the user rules on the push and on the merge. The gh#54 step-002 lane merges master into its branch afterwards and reruns its commit-1 battery.

## The rows, all closed

Every row was ratified as proposed on 2026-09-08.

1. **The two fixes, cleared.** Land both. The audit ran each alone: commit 1 alone repairs the referee suite and `oracle_smoke`, and commit 2 alone repairs the referee suite only. Commit 1 is the root fix. Commit 2 is belt and braces at no cost, because each referee child runs one drive and exits (`crates/jampgame/tests/referee.rs:587-612`).

2. **The write-scope extension, ratified.** The scope takes `crates/jampgame/tests/common/mod.rs` by the `run_lifecycle` tail, and commit 2 repairs `oracle_smoke` there.

3. **The Linux link arm, cleared.** It takes the same two flags. The arm stays untested, because CI never runs `build.sh` and no Linux host here runs it.

4. **The `_exit` after the reflog write, ratified.** Do not add it. It hides the signal and leaves the dangling destructor.

5. **The crash check stays first, ratified.** The reason softens. A check that sits after the diff still reports a crash, so a reorder would not blind the rig. The check-first order stands because it is the order that surfaced this defect.

6. **No toolchain pin, ratified.** The README gains the sentence that explains it: an artifact built under gcc 15 loads the gcc 16 runtime through the `gcc/current` symlink at load time, so a pin would not help.

7. **No DEC entry, ratified.** The README and this packet carry the invariant. The invariant spans two build scripts once the cgame sibling is repaired.

8. **The cgame oracle sibling defect, the vet's new row, ratified as a follow-up.** It is filed as issue #59 and is not a widening of this step. `tools/cgame-oracle/build.sh:260-261` carries the same two link lines, `crates/cgame/tests/replay_referee.rs:153` drops the module on the thread `cgame-replay-engine`, and `replay_oracle_self_check` dies by signal 11 with the same signature. The test is ignored and trace-gated, so no gate battery hides it today. The lane must not touch `tools/cgame-oracle/` or `crates/cgame/`. It records the follow-up under open gaps in the finished file, naming issue #59.

## Amendments

**2026-09-08 - the ratification walk closed every row.** The audit is at `.claude/packets/58/step-001/audit.md` (`1f565841`), verdict GO WITH FIXES. Rows 1 through 7 are ratified as proposed, and the vet's new row lands as row 8 above. Each row above carries its folded text.

**2026-09-08 - the attribution corrects.** The draft named `libgcc_s.1.1.dylib` as the owner of the dangling destructor. Both runtime images carry their own `_emutls_key` and `_emutls_destroy`, and every fault address ends in `0x17a0`, which is the offset of `_emutls_destroy` in `libstdc++.6.dylib`. The same symbol sits at `0x11a00` in `libgcc_s.1.1.dylib`. `-static-libstdc++` is the load-bearing flag, and the pair of flags stays. Commit 1's comment and the README name `libstdc++`, or both images, and never `libgcc_s` alone. The fix does not change.

**2026-09-08 - the surface wording corrects.** No comment stands above `drop(module)` at `crates/jampgame/tests/referee.rs:307-308` today, so commit 2 adds one rather than edits one.

**2026-09-08 - the gate timings, for the lane's planning.** `build.sh` 40 s, the referee suite 20 s, `oracle_smoke` 3 s, and the workspace tests 36 s.

**2026-09-08 - the lane review, seven findings closed.** The vet is at `.claude/packets/58/step-001/vet.md`, now `f9dfa968` after a rebase. The user ruled every finding on 2026-09-08.

- **F1, ratified as it stands.** The two-line pointer comment above the `case "$OS"` block in `build.sh` is an accepted third comment site. It points a reader at the case statement back to the header rationale.
- **F2, ratified as it stands.** The two three-sentence comment blocks in commit 2 stand, because each sentence carries content.
- **N1, this packet's own cite corrects.** The `run_lifecycle` tail sits at `crates/jampgame/tests/common/mod.rs:1151-1156`. The `:922` cite above names the signature, not the tail.
- **F3, F4, F5, and F6, repaired in a fix round.** Commit 2's body was rewritten in place as `a2168bf9` with an identical tree. `329e47e3` corrects the two present-tense sentences, moves `forget` to a top-of-file import, and aligns the comment tenses.
