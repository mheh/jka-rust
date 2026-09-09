# Synopsis gh#58 step-001 - the referee rig repair

Drafted 2026-09-08. Seven open rows. The packet body is `.claude/packets/58/step-001/packet.md`.

## Intent

This step repairs the lockstep referee rig, which fails 8 of 9 scenarios with no parity divergence. The gate is the full suite green, 9 of 9, with every oracle child exiting 0.

## The diagnosis, established

The oracle dylib links `/opt/homebrew/opt/gcc/lib/gcc/current/libstdc++.6.dylib`, which pulls `libgcc_s.1.1.dylib`. That runtime registers a thread-specific-data key whose destructor is `emutls_destroy`, proved by `nm -m` and by `DYLD_PRINT_LIBRARIES=1` on the child. `crates/jampgame/tests/referee.rs:308` ends `drive` with `drop(module)`, which dlcloses the last reference and unloads all three images. The engine thread then exits and `_pthread_tsd_cleanup` calls a destructor in unmapped memory. The crash report `referee-...-2026-09-08-203851.ips` shows the faulting thread named `referee-engine`, the frames `_pthread_tsd_cleanup` to `_pthread_exit` to `_pthread_start`, and a top frame with no owning image. The Rust child survives the same drop because it links no gcc runtime. Homebrew gcc moved to 16.2.0 on 2026-09-04, and the rig was last green on 2026-09-01.

`crates/jampgame/tests/oracle_smoke.rs` fails the same way through `run_lifecycle` in `crates/jampgame/tests/common/mod.rs`. It is `#[ignore]`d and on no gate battery, so it hid.

Both fixes were tried and reverted. Static-linking the gcc runtime leaves the dylib depending on libSystem alone, creates no key, and repairs both tests. Forgetting the module instead of dropping it repairs the referee suite with the pristine dylib. With both applied the suite passed 9 of 9 in 17.49 seconds, every scenario byte-identical. `cargo test --workspace -- --test-threads=1` is green on the pristine tree.

## Surface contract

- `tools/referee-oracle/build.sh` - the two link commands and the header comment.
- `tools/referee-oracle/README.md` - the toolchain section.
- `crates/jampgame/tests/referee.rs` - the tail of `drive` and its comment.
- `crates/jampgame/tests/common/mod.rs` - the tail of `run_lifecycle`, pending row 2.

No new `pub` item, no new crate, no new cvar, no new test, no ABI change, and no edit under `oracle/` or `crates/mp/`.

## Commits

1. `fix(gh#58 s001): the referee oracle links the gcc runtime statically` - `-static-libgcc -static-libstdc++` on both link arms, plus the two doc surfaces.
2. `fix(gh#58 s001): the referee children keep the module mapped to thread exit` - `std::mem::forget` in `drive`, and in `run_lifecycle` if row 2 rules for it.
3. `process(gh#58 s001): the finished file`.

Commits 1 and 2 gate on `cargo build --workspace`, `cargo test -p jampgame --test referee -- --ignored --test-threads=1` at 9 of 9, and `cargo test --workspace -- --test-threads=1`. Commit 1 also checks `otool -L` names libSystem alone. Commit 2 also runs `cargo test -p jampgame --test oracle_smoke -- --ignored --test-threads=1`. The world goldens are off this battery, because no commit touches the renderer.

## The open rows

1. **The two fixes** - mechanical. Land both. Commit 1 removes the key, commit 2 removes the unload.
2. **The write-scope extension to `crates/jampgame/tests/common/mod.rs`** - user ruling. Extend it by one function tail and repair `oracle_smoke` inside commit 2.
3. **The Linux link arm** - mechanical. Apply the same two flags there, so the script stays one rule.
4. **An `_exit` after the reflog write** - user ruling. Do not add it. It hides the signal.
5. **Comparing snapshots before judging the exit status** - user ruling. Do not do it. It blinds the rig to a real crash.
6. **A toolchain pin in `build.sh`** - user ruling. No pin. The static link is what removes the coupling.
7. **A DEC entry for the static-link invariant** - user ruling. No DEC. The README and the packet carry it.

## Dispatch flags

- Oracle ambiguity: **false**. Nothing in `oracle/` is read, interpreted, or edited.
- New state home: **false**. No state moves.
- ABI or parity-gate surface: **true**. The step rebuilds the referee oracle dylib, which is the parity gate itself, and it touches the driver's crash-finding channel.
- Divergence proposal: **false**. No behavior diverges from Raven. The full suite is byte-identical with the static link.
