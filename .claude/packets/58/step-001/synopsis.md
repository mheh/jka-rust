# Synopsis gh#58 step-001 - the referee rig repair

Ratified 2026-09-08. All seven rows closed as proposed, plus the vet's new row as row 8. The packet body carries the folded shape. Audit: `.claude/packets/58/step-001/audit.md` (`1f565841`), verdict GO WITH FIXES.

## Intent

This step repairs the lockstep referee rig, which fails 8 of 9 scenarios with no parity divergence. The gate is the full suite green, 9 of 9, with every oracle child exiting 0.

## The diagnosis, established

The oracle dylib links `/opt/homebrew/opt/gcc/lib/gcc/current/libstdc++.6.dylib`, which pulls `libgcc_s.1.1.dylib`. Both images register a thread-specific-data key whose destructor is `emutls_destroy`. The dangling one belongs to `libstdc++`: every fault address ends in `0x17a0`, that symbol's offset there, against `0x11a00` in `libgcc_s`. `crates/jampgame/tests/referee.rs:308` ends `drive` with `drop(module)`, which dlcloses the last reference and unloads all three images. The engine thread then exits and `_pthread_tsd_cleanup` calls a destructor in unmapped memory. The crash report shows the faulting thread named `referee-engine`, the frames `_pthread_tsd_cleanup` to `_pthread_exit` to `_pthread_start`, and a top frame with no owning image. The Rust child survives the same drop because it links no gcc runtime. Homebrew gcc moved to 16.2.0 on 2026-09-04, and the rig was last green on 2026-09-01. The coupling is the `gcc/current` symlink, not the build date, so an artifact built under gcc 15 loads the gcc 16 runtime at load time.

`crates/jampgame/tests/oracle_smoke.rs` fails the same way through `run_lifecycle` in `crates/jampgame/tests/common/mod.rs`. It is `#[ignore]`d and on no gate battery, so it hid.

The audit ran each fix alone. The static link repairs both tests and is the root fix. The `forget` repairs the referee suite only, and it costs nothing because each child runs one drive and exits. With both, the suite passed 9 of 9, every scenario byte-identical, so the link change moves no snapshot byte.

## Surface contract

- `tools/referee-oracle/build.sh` - the two link commands and the header comment.
- `tools/referee-oracle/README.md` - the toolchain section.
- `crates/jampgame/tests/referee.rs` - the tail of `drive`, plus a new comment above it.
- `crates/jampgame/tests/common/mod.rs` - the tail of `run_lifecycle`.

No new `pub` item, no new crate, no new cvar, no new test, no ABI change, and no edit under `oracle/`, `crates/mp/`, `tools/cgame-oracle/`, or `crates/cgame/`.

## Commits

1. `fix(gh#58 s001): the referee oracle links the gcc runtime statically` - `-static-libgcc -static-libstdc++` on both link arms, plus the two doc surfaces.
2. `fix(gh#58 s001): the referee children keep the module mapped to thread exit` - `std::mem::forget` in `drive` and in `run_lifecycle`.
3. `process(gh#58 s001): the finished file`.

Commits 1 and 2 gate on `cargo build --workspace`, `cargo test -p jampgame --test referee -- --ignored --test-threads=1` at 9 of 9, and `cargo test --workspace -- --test-threads=1`. Commit 1 also checks `otool -L` names libSystem alone. Commit 2 also runs `cargo test -p jampgame --test oracle_smoke -- --ignored --test-threads=1`. The world goldens are off this battery, because no commit touches the renderer. Measured timings: `build.sh` 40 s, referee 20 s, `oracle_smoke` 3 s, workspace tests 36 s.

## The rows, all closed

1. **The two fixes** - cleared. Land both. Commit 1 is the root fix, commit 2 is belt and braces at no cost.
2. **The write scope** - ratified. It takes `crates/jampgame/tests/common/mod.rs` by the `run_lifecycle` tail, repaired inside commit 2.
3. **The Linux link arm** - cleared. Same flags, untested, because CI never runs `build.sh`.
4. **An `_exit` after the reflog write** - ratified. Do not add it.
5. **The crash check stays first** - ratified, reason softened. A check after the diff would still report a crash. The order stands because it surfaced this defect.
6. **No toolchain pin** - ratified. The README says why: the `gcc/current` symlink, not the build date.
7. **No DEC entry** - ratified. The invariant spans two build scripts once the cgame sibling is repaired.
8. **The cgame oracle sibling** - ratified as a follow-up, filed as issue #59. Same link lines at `tools/cgame-oracle/build.sh:260-261`, same drop at `crates/cgame/tests/replay_referee.rs:153`, same signal 11. The lane records it under open gaps and touches neither path.

## Dispatch flags

- Oracle ambiguity: **false**. Nothing in `oracle/` is read, interpreted, or edited.
- New state home: **false**. No state moves.
- ABI or parity-gate surface: **true**. The step rebuilds the referee oracle dylib, which is the parity gate itself, and it touches the driver's crash-finding channel.
- Divergence proposal: **false**. No behavior diverges from Raven. The full suite is byte-identical with the static link.
