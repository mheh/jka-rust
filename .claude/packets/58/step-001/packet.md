# Packet gh#58 step-001 - the referee rig repair

Drafted 2026-09-08 on branch `gh58-step-001-referee-rig`, cut from master `1cb6c525`. The shape reference is `.claude/packets/54/step-002/packet.md` on `gh54-step-002-cinematics`.

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

`libgcc_s.1.1.dylib` owns the key and the destructor. `nm -m` on it shows `_emutls_key`, `_emutls_destroy`, and an undefined `_pthread_key_create` from libSystem. The oracle dylib itself contains no `emutls` symbol, so the key comes from the gcc runtime and not from Raven's code.

### Who unloads it

`crates/jampgame/tests/referee.rs:308` ends `drive` with `drop(module)`. `LoadedModule` holds a `libloading::Library` (`crates/native/platform/src/module_loader/loaded_module.rs:18`), and dropping it calls `dlclose`. That drop releases the last reference to the oracle dylib, so dyld unloads it and its two gcc runtime images. The engine thread then exits, `_pthread_tsd_cleanup` calls `emutls_destroy` through the still-registered key, and the pointer is dangling.

The Rust child survives the same drop because its cdylib links no gcc runtime and registers no such key.

### Why now

The referee was last green on 2026-09-01. Homebrew `gcc` moved to 16.2.0 on 2026-09-04, and `brew list --versions gcc` reports `gcc 16.2.0`. `/opt/homebrew/bin/g++-16` is the only real g++ on this host, so `build.sh`'s auto-detect loop picks it.

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

### The candidates that mask, both rejected as defaults

Calling `_exit` after the reflog write in the oracle child hides the signal and leaves the dangling destructor in place. Comparing the snapshots before judging the exit status does the same, and it also blinds the rig to a real crash, which is the one finding the driver's crash message exists to report. Row 4 and row 5 carry them for the user.

Building the oracle with Apple clang is not available. `README.md`'s toolchain section records why: `FOFS(x) ((int)&(((gentity_t *)0)->x))` is a hard error in C++ on a 64-bit host, and Apple clang treats `-fpermissive` as a silent no-op.

## Surface contract

This step creates no `pub` item, no type, no constant, and no test. It changes four files.

- `tools/referee-oracle/build.sh`: the `CXXFLAGS`-adjacent link commands in the `case "$OS"` block, and the header comment that states the toolchain requirement.
- `tools/referee-oracle/README.md`: the toolchain section, which gains the static-link rationale.
- `crates/jampgame/tests/referee.rs`: the last two lines of `drive`, and the comment above them.
- `crates/jampgame/tests/common/mod.rs`: the tail of `run_lifecycle`, pending row 2.

Anything not on this list is out of scope, and the agent must not add it. No new `pub` item, no new crate, no new cvar, no new test, no ABI change, no change to any scenario, reflog, or committed fixture, and no edit under `oracle/` or `crates/mp/`.

## Commit bundle

Every commit uses `git commit --no-gpg-sign`, a heading subject, an STE body, and no trailer of any kind.

### Commit 1 - `fix(gh#58 s001): the referee oracle links the gcc runtime statically`

Files: `tools/referee-oracle/build.sh`, `tools/referee-oracle/README.md`.

The Darwin and Linux link commands both gain `-static-libgcc -static-libstdc++`. The header comment and the README toolchain section record why: the gcc runtime dylibs register a thread-specific-data key, and the harness unloads the module before the thread exits. The README's existing claim that the module's only undefined symbols are libc and libm becomes true again.

Gates:

1. `tools/referee-oracle/build.sh` runs clean and prints `compiled 89 TUs`.
2. `otool -L tools/referee-oracle/build/liboraclejampgame.dylib` names `/usr/lib/libSystem.B.dylib` and no other dependency.
3. `cargo build --workspace`
4. `cargo test -p jampgame --test referee -- --ignored --test-threads=1` passes 9 of 9 with no crash finding.
5. `cargo test --workspace -- --test-threads=1`

### Commit 2 - `fix(gh#58 s001): the referee children keep the module mapped to thread exit`

Files: `crates/jampgame/tests/referee.rs`, and `crates/jampgame/tests/common/mod.rs` if row 2 rules for it.

`drive` ends with `std::mem::forget(module)` instead of `drop(module)`, and `run_lifecycle` does the same with its module. A comment states the reason in one or two sentences: a dlclose before thread exit can strand a thread-specific-data destructor, the child process is short-lived, and `GAME_SHUTDOWN` still runs first.

Gates:

1. `cargo build --workspace`
2. `cargo test -p jampgame --test referee -- --ignored --test-threads=1` passes 9 of 9 with no crash finding.
3. `cargo test -p jampgame --test oracle_smoke -- --ignored --test-threads=1` passes.
4. `cargo test --workspace -- --test-threads=1`

### Commit 3 - `process(gh#58 s001): the finished file`

File: `.claude/packets/58/step-001/finished.md`. No gate beyond the packet skill's contract.

### Gate notes

The referee suite needs both artifacts first: `tools/referee-oracle/build.sh`, then `cargo build --workspace`. The rig runs one agent at a time, and the lane agent is the only one on it. The world goldens are not on this battery, because no commit touches the renderer. `cargo test --workspace` never runs without `--test-threads=1`, or the two world-golden tests abort in the pk3 inflate path.

Both batteries are known reachable. The drafter ran the full referee suite green 9 of 9 with the two fixes applied, and ran `cargo test --workspace -- --test-threads=1` green on the pristine tree.

## Write scopes

Branch: `gh58-step-001-referee-rig`, already checked out at `1cb6c525`. Never commit on master.

Writable:

- `crates/jampgame/tests/referee.rs`
- `tools/referee-oracle/build.sh` and `tools/referee-oracle/README.md`
- `.claude/packets/58/step-001/`
- `crates/jampgame/tests/common/mod.rs`, pending row 2

Everything else is read-only, including `oracle/`, every crate under `crates/mp/` and `crates/sp/`, `crates/native/`, every committed fixture and reflog, and `~/Developer/jka/` beyond the read-only asset reads the real-map scenarios already make. Source files change through the Edit tool only. `tools/referee-oracle/build/` is generated and gitignored, so the agent runs the build script but commits nothing from that directory.

There is a stash from another lane in the stash list. The agent must never touch it.

## Disposition

After a clean lane-review: open the pull request from `gh58-step-001-referee-rig` into master and merge it on GitHub with a merge commit, per DEC-67. Never squash. The session never pushes or opens the pull request unprompted. It prepares the branch, asks, and the user rules on the push and on the merge. The gh#54 step-002 lane merges master into its branch afterwards and reruns its commit-1 battery.

## Open rows

1. **The two fixes, mechanical.** Land both, commit 1 and commit 2. Proposed default: both. Commit 1 removes the key from the picture and repairs both broken tests. Commit 2 removes the unload, which is the other half of the precondition, and it protects the rig against a future toolchain that registers a key again.

2. **The write-scope extension to `crates/jampgame/tests/common/mod.rs`, user ruling.** `oracle_smoke` is broken by the same defect, and one line in `run_lifecycle` repairs it. The file is not in the scope the issue states. Proposed default: extend the scope to that one file and that one function tail, and fix it inside commit 2.

3. **The Linux link line, mechanical.** The flags are gcc-generic and apply to the `Linux)` arm as well. No Linux host tests them here, and CI runs no referee. Proposed default: apply the same two flags to both arms, so the script stays one rule.

4. **The `_exit` after the reflog write, user ruling.** Rejected as a default. It hides the signal and leaves the dangling destructor. Proposed default: do not add it.

5. **The driver comparing before it judges the exit status, user ruling.** Rejected as a default. It hides the signal and blinds the rig to a real crash. Proposed default: keep the crash check first, exactly as it stands at `crates/jampgame/tests/referee.rs:653-661`.

6. **A toolchain pin in `build.sh`, user ruling.** The auto-detect loop takes the newest `g++-1x` it finds. A pin would freeze the rig against the next Homebrew bump. Proposed default: no pin. The static link is what removes the coupling, and the README records the reason.

7. **A DEC entry, user ruling.** The invariant "the referee oracle links the gcc runtime statically" is durable and cross-session. Proposed default: no DEC. The README and this packet carry it, and the issue links the step folder.

## Amendments

None.
