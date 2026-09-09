# Audit gh#58 step-001 - the referee rig packet

Audited 2026-09-08 on branch `gh58-step-001-referee-rig` at `d87450ff`. The tree was clean before and after the audit, apart from this file. The stash from the gh#54 lane was not touched.

## Verdict

GO WITH FIXES. The diagnosis holds, both fixes work alone, and the packet lands the right pair. Two facts in the packet need correction before the lane runs, and one sibling defect outside the issue's scope needs its own step.

## The diagnosis, verified before the packet was read

The pristine dylib was rebuilt with `tools/referee-oracle/build.sh` (39.9 s wall, 89 TUs). `otool -L` names `/opt/homebrew/opt/gcc/lib/gcc/current/libstdc++.6.dylib` and `/usr/lib/libSystem.B.dylib`. `libstdc++.6.dylib` names `@rpath/libgcc_s.1.1.dylib`. `nm -m` on the oracle dylib finds no `emutls`, `__tlv`, or `pthread_key` symbol. `nm -m` on `libstdc++.6.dylib` finds `_emutls_key`, `_emutls_destroy` at `0x17a0`, and an undefined `_pthread_key_create`. `nm -m` on `libgcc_s.1.1.dylib` finds its own `_emutls_key`, `_emutls_destroy` at `0x11a00`, and an undefined `_pthread_key_create`.

`cargo test -p jampgame --test referee referee_idle -- --ignored --test-threads=1 --nocapture` on the pristine tree fails at `crates/jampgame/tests/referee.rs:655` with "the 'oracle' module was killed by signal 11". The newest report `referee-64bc2153f45e86af-2026-09-08-204918.ips` shows `KERN_INVALID_ADDRESS at 0x00000001073b97a0`, the faulting thread `referee-engine`, the frames `_pthread_tsd_cleanup`, `_pthread_exit`, `_pthread_start`, and a top frame with no owning image. No gcc image is in the report's image list at crash time.

`crates/jampgame/tests/referee.rs:308` reads `drop(module);` after `GAME_SHUTDOWN`. `crates/native/platform/src/module_loader/loaded_module.rs:18` holds `pub(crate) lib: libloading::Library,` and that drop is a `dlclose`. `crates/jampgame/tests/common/mod.rs:1435-1443` runs the drive on a spawned thread named `referee-engine` and joins it, so the thread exits after the unload.

The causal chain is CONFIRMED: the gcc runtime images register a thread-specific-data key with `emutls_destroy` as its destructor, `drop(module)` unloads the module and its two runtime images, and the engine thread's exit calls the destructor in unmapped memory.

## The fixes, each run alone

| fix | result | evidence |
|---|---|---|
| Commit 1 alone, `-static-libgcc -static-libstdc++` on both link arms, pristine driver | CONFIRMED, 9 of 9 | `otool -L` names `/usr/lib/libSystem.B.dylib` alone. `nm -m` finds no `emutls`, `pthread_key`, `_Unwind`, or `__cxa` symbol. Suite 40.8 s wall while the cgame probe ran beside it. `oracle_smoke` green. |
| Commit 2 alone, `std::mem::forget(module)` at `referee.rs:308`, pristine dylib | CONFIRMED, 9 of 9 | Every scenario line reads `byte-identical`, for example `real-ffa1-items: 2501 frames byte-identical; 850498 syscalls compared`. Suite 19.6 s wall. `oracle_smoke` still dies by signal 11, as the packet says. |
| Land both | CONFIRMED | Commit 1 repairs both tests and removes the key from the process. Commit 2 is belt-and-braces for the referee, and it is the only fix that reaches `run_lifecycle` without a rebuild of the oracle. Either alone repairs the referee suite. |

The static dylib is 1,715,656 bytes, the same size as the pristine one, so the static link absorbs no libstdc++ code. The packet's claim "Nothing was actually used from libstdc++" is CONFIRMED.

The cost of `std::mem::forget`: none in the referee. `run_referee` at `referee.rs:587-612` always runs each drive in a child process that exits right after its one drive, and `REFEREE_SELFTEST` at `referee.rs:575-585` takes the same child path. No in-process reload path exists. In `run_lifecycle` the cost is also none: `abi_smoke.rs:19` states the whole lifecycle runs in one test per process because of process singletons, and `oracle_smoke.rs` has one test. The one mapping stays until process exit.

## The open rows

| row | tag | judgment | evidence |
|---|---|---|---|
| 1. Land both fixes | mechanical | CLEARED | The two single-fix runs above. |
| 2. Extend the write scope to `common/mod.rs` | user ruling | CONFIRMED | `run_lifecycle` at `common/mod.rs:922` lets `module` fall out of scope at the closing brace on thread `abi-smoke-engine` (`common/mod.rs:913-919`). `oracle_smoke` dies by signal 11 with commit 2 alone and passes with commit 1 alone. The one-line extension is the honest fix for that file. Without it, commit 1 alone already repairs `oracle_smoke`. |
| 3. The Linux link arm | mechanical | CLEARED | `build.sh:431` is `Linux)  "$CXX" -shared -o "$LIBOUT" build/obj/*.o -lm;;`. The flags are gcc-generic. No Linux host here runs it, and `.github/workflows/build.yml` never invokes `build.sh`, so the arm stays untested either way. |
| 4. No `_exit` after the reflog write | user ruling | CONFIRMED | `child_drive` at `referee.rs:615-631` writes the snapshot and returns to `run_on_engine_thread_fn`. An `_exit` before the thread exit would skip the destructor call and hide the defect. |
| 5. Keep the crash check first | user ruling | CONFIRMED, with one nuance | `spawn_child` at `referee.rs:633-661` panics on `!status.success()` before it reads the file. A reorder would still catch a crash if the check stayed after the diff, so "blinds" overstates it. The proposed default is still the right one, because a crash that happens after a clean reflog is a finding the rig must report. |
| 6. No toolchain pin | user ruling | CONFIRMED | `build.sh:33` loops `g++-16 g++-15 g++-14 g++-13 g++-12 g++`. `/opt/homebrew/bin/g++-16` is the only real g++ on the host, and `/usr/bin/g++` is Apple clang. `brew list --versions gcc` reports `gcc 16.2.0`. |
| 7. No DEC entry | user ruling | CONFIRMED as stated, with one addition | The README carries the reason. The sibling defect below puts the same invariant into `tools/cgame-oracle/build.sh:260-261` as well, so the invariant now spans two build scripts. That is a fact for the ruling, not a challenge. |

## The dispatch flags

| flag | packet | judgment | evidence |
|---|---|---|---|
| Oracle ambiguity | false | CONFIRMED | No file under `oracle/` is read or changed. `build.sh` copies the tree and patches the copy. |
| New state home | false | CONFIRMED | No state moves. The one Rust change is the module's drop site. |
| ABI or parity-gate surface | true | CONFIRMED | The oracle dylib is the parity gate. The link change is verified byte-identical across all nine scenarios. |
| Divergence proposal | false | CONFIRMED | No Raven behavior changes. |

## The surface contract against the world

- `build.sh` link arms at `tools/referee-oracle/build.sh:430-431`: CONFIRMED, two lines, one per platform.
- `README.md` toolchain section: CONFIRMED at `tools/referee-oracle/README.md:33-55`. The claim at `README.md:23` that the undefined symbols are libc and libm becomes true again with commit 1.
- `drive` tail at `referee.rs:307-308`: CONFIRMED. No comment stands above `drop(module)` today, so the lane adds one rather than edits one.
- `run_lifecycle` tail at `common/mod.rs:922` to its closing brace: CONFIRMED, pending row 2.
- Other consumers of the referee oracle dylib: `tools/lockstep-referee/run.sh:41` links it as the secondary server's module. `crates/mp/app` spawns no thread, so the dedicated server unloads on the main thread. That path was not run, because the live rig hosts servers. It stays out of this step.

## The gates

| gate | real | wall time |
|---|---|---|
| `tools/referee-oracle/build.sh` | yes, prints `compiled 89 TUs` | 39.9 s |
| `otool -L` names libSystem alone | yes, the first line is the install name and the second is libSystem | under 1 s |
| `cargo build --workspace` | yes | 1.6 s incremental |
| `cargo test -p jampgame --test referee -- --ignored --test-threads=1` | yes, 9 passed | 19.6 s alone, 40.8 s beside the cgame probe |
| `cargo test -p jampgame --test oracle_smoke -- --ignored --test-threads=1` | yes | about 3 s |
| `cargo test --workspace -- --test-threads=1` | yes, pristine tree | 36 s, 138 result blocks green |

## What the packet missed

### DISPUTED: which image owns the dangling destructor

The packet's "Who registers the destructor" section says `libgcc_s.1.1.dylib` owns the key and the destructor. Both runtime images carry their own `_emutls_key` and `_emutls_destroy`. The fault address in the packet's report ends in `0x17a0` (`0x10ada17a0`), and so does the one in the newest report (`0x1073b97a0`) and the cgame report (`0x10aae97a0`). `_emutls_destroy` sits at `0x17a0` in `libstdc++.6.dylib` and at `0x11a00` in `libgcc_s.1.1.dylib`. The dangling destructor belongs to `libstdc++`, so `-static-libstdc++` is the load-bearing flag. The fix does not change. The header comment and the README should name libstdc++, or name both images, and not libgcc_s alone.

### The sibling defect in the cgame oracle

`tools/cgame-oracle/build.sh:260-261` carries the same two link lines, and `crates/cgame/tests/replay_referee.rs:153` drops the module on the thread `cgame-replay-engine` (`replay_referee.rs:121-122`). `tools/cgame-oracle/build/liboraclecgame.dylib` was built on Jul 30 against gcc 15 and still links `/opt/homebrew/opt/gcc/lib/gcc/current/libstdc++.6.dylib`, and that symlink now resolves to 16.2.0. `cargo test -p cgame --test replay_referee replay_oracle_self_check -- --ignored --test-threads=1 --nocapture` with the trace at `~/Developer/jka/trace-swoop1.bin` dies by signal 11 after 4 min 24 s. The report `replay_referee-d66a98d33f46b087-2026-09-08-210351.ips` shows the same signature. The issue scopes this step to `referee.rs` and `tools/referee-oracle/`, so this is a follow-up step or issue, not a widening of this one. The test is ignored and trace-gated, so no gate battery hides it today.

### The coupling is the `current` symlink, not the build date

An oracle artifact built under gcc 15 loads the gcc 16 runtime through `gcc/current`. The "Why now" section is right about the date, and this fact explains why a rebuild was not needed to expose the defect. The README's static-link rationale should say this in one sentence, because it is the reason a pin (row 6) would not help.

### CI

`.github/workflows/build.yml` never runs `build.sh` and never runs the referee. Its `gcc-multilib` installs at lines 101, 165, and 254 serve the i686 lane. The static link changes nothing CI does.

### A cheaper or more correct fix

None cheaper. A more correct shape for commit 2 would return the `LoadedModule` from the engine thread and drop it on the parent after `join()`, which keeps RAII and unloads after the destructor ran. That needs a signature change to `run_on_engine_thread_fn` and a wider scope for no gain in a one-shot child, so `std::mem::forget` is the right call here.
