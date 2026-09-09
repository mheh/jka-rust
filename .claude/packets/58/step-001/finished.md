# Finished gh#58 step-001 - the referee rig repair

Branch `gh58-step-001-referee-rig`, cut from master `1cb6c525` and checked out at `af279749`. Two code commits and this file. Nothing pushed, no pull request opened. The gh#54 stash in the stash list was not touched.

## Assumptions and choices

**Commit 1 - the static link.** The two flags sit at the end of both link commands, after `-lm`, so the Darwin and Linux arms stay one rule. The rationale lives in two places. The script header carries four sentences after the existing toolchain paragraph, and the link block carries a two-line pointer back to it, so a reader at the `case` statement is not left guessing. Both texts name `libstdc++.6.dylib` and `libgcc_s.1.1.dylib` together, per the packet's attribution amendment, and neither names `libgcc_s` alone.

The README gains a `### The gcc runtime links statically` subsection under the toolchain heading. It states the defect, the fix, the check command `otool -L build/liboraclejampgame.dylib`, and the pin question. The existing claim at `README.md:22-24`, that the module's only undefined symbols are libc and libm, is true again, so it needed no edit.

**Commit 2 - the module lifetime.** Both sites take the same three-line comment, worded for their own process shape. `drive` says each child process runs one drive and exits. `run_lifecycle` says the whole lifecycle runs once per process. The wider fix the audit named, returning the `LoadedModule` from the engine thread and dropping it on the parent after `join()`, was not taken. It needs a signature change to `run_on_engine_thread_fn`, which the surface contract does not carry.

No pause trigger fired. No unlisted surface was needed, no signature moved, and no fact contradicted the packet.

## Deviations

None. No file outside the write scopes was edited. `oracle/` was not read or written by hand, and `tools/cgame-oracle/` and `crates/cgame/` were not touched.

## Commits and gate results

1. `78a1c76e` **fix(gh#58 s001): the referee oracle links the gcc runtime statically.** Files: `tools/referee-oracle/build.sh`, `tools/referee-oracle/README.md`.

   - `tools/referee-oracle/build.sh` clean, printed `referee-oracle: compiled 89 TUs` and `OK - dllEntry + vmMain exported`. 38.2 s.
   - `otool -L tools/referee-oracle/build/liboraclejampgame.dylib` names the install name and `/usr/lib/libSystem.B.dylib`, and nothing else. `nm -m` finds zero `emutls` and zero `pthread_key` symbols. The artifact is 1,715,656 bytes, the same size the audit measured on the pristine build. Under 1 s.
   - `cargo build --workspace` clean. 3.0 s.
   - `cargo test -p jampgame --test referee -- --ignored --test-threads=1` passed 9 of 9, zero failed, no crash finding. 64.1 s wall, of which 57.2 s was the test run. This run built the test binaries first, so it is slower than commit 2's.
   - `cargo test --workspace -- --test-threads=1` green, 138 result blocks, every one `0 failed`. 135.9 s.

2. `ea136782` **fix(gh#58 s001): the referee children keep the module mapped to thread exit.** Files: `crates/jampgame/tests/referee.rs`, `crates/jampgame/tests/common/mod.rs`.

   - `cargo build --workspace` clean. 2.4 s.
   - `cargo test -p jampgame --test referee -- --ignored --test-threads=1` passed 9 of 9, zero failed, no crash finding. 20.8 s. Every scenario ran both children, and `spawn_child` at `referee.rs:633-661` panics on a non-zero child status, so the nine passes prove every oracle child exited 0.
   - `cargo test -p jampgame --test oracle_smoke -- --ignored --test-threads=1` passed. 0.6 s. A `--nocapture` rerun confirms the test really loads `liboraclejampgame.dylib` and drives `GAME_INIT` through `GAME_SHUTDOWN`, so the fast pass is not a skip.
   - `cargo test --workspace -- --test-threads=1` green, 138 result blocks, every one `0 failed`. 58.9 s.
   - `rustfmt --check` clean on both touched files.

3. **process(gh#58 s001): the finished file.** This commit. No gate beyond the packet skill's contract.

No committed fixture, reflog, or snapshot moved at any point. The static link changes no parity byte, which the nine byte-identical scenarios in both suite runs confirm.

## Open gaps

**The cgame oracle carries the same defect, filed as issue #59.** `tools/cgame-oracle/build.sh:260-261` holds the same two link lines, and `crates/cgame/tests/replay_referee.rs:153` drops the module on the thread `cgame-replay-engine`. `replay_oracle_self_check` dies by signal 11 with the same signature. This lane did not touch either path, as the packet requires. The invariant now spans two build scripts, and only one of them states it.

**The Linux link arm is untested.** No Linux host here runs `build.sh`, and `.github/workflows/build.yml` never invokes it. The flags are gcc-generic, and the arm keeps the same rule as Darwin, but nothing proved it.

**One build script states the invariant, the DEC ledger does not.** Row 7 ruled no DEC entry, and the README carries the reason. A reader of `tools/cgame-oracle/build.sh` finds nothing until issue #59 lands.
