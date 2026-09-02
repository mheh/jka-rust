# Synopsis gh#56 step-001 - the dropped client shutdown

## Intent

The port dropped both of the oracle's client-shutdown calls on a map load, so `VM_Clear` wipes the VM table while `cl.uivm` still points at slot 0 and `VM_Create("jampgame")` reuses that slot. This step restores the calls through three new engine hooks and pins the slot aliasing with one unit test.

## Surface contract

- `EngineHooks::CL_ShutdownAll`, `CL_ShutdownCGame`, `CL_ShutdownUI`, `CIN_CloseAllVideos`
- their four null-build no-op bodies and four `null_dedicated` initializers
- four client-side adapters and installs in `install_client_engine_hooks`
- one new statement pair in `SV_SpawnServer`, three in `Hunk_Clear`, and one replaced comment
- `crates/mp/engine/core/tests/vm_slot_alias.rs` with one test and one syscall stub

Anything not on this list is out of scope.

## Commits

1. `feat(gh#56 s001): the client shutdown hooks` - four fields, four null bodies, four adapters.
2. `fix(gh#56 s001): a map load shuts the client modules down` - the two call sites and the truthful comment.
3. `test(gh#56 s001): a cleared VM slot is reused by the next module` - the aliasing pin.
4. `process(gh#56 s001): finished file`.

## Open rows

- **Row 1, mechanical.** `CIN_CloseAllVideos` is the third member of the same oracle block, fully ported with zero callers. Default: port it as a fourth hook in the same commits.
- **Row 2, mechanical.** `null_client.cpp` defines none of the four symbols, because the `#ifndef DEDICATED` guards remove the call sites rather than the definitions, so the brief's cite does not exist. Default: each field doc cites the real client definition, and each null body cites the guard that excises the call.
- **Row 3, mechanical.** A literal `VM_Create("ui")` in a unit test reaches `Sys_LoadDll` and the `.qvm` fallback, fails, and empties the slot again. Default: a new `crates/mp/engine/core/tests/vm_slot_alias.rs` that seats slot names directly after `demo_referee.rs:133-136`, drives `VM_Create`'s real name-match loop, clears, reseats as `jampgame`, and asserts the same slot address.
- **Row 4, mechanical.** The brief named 22 golden fixtures. Master carries 21, and the twenty-second arrives with the weather branch this step merges ahead of. Default: the battery names 21.

## Record-only

- SP is moot. `crates/sp/` has no `z_memman`, no `sv_init`, no `Hunk_Clear`, and no `SV_SpawnServer`.
- The hoth2 freeze stays unattributed. The wild `ClientDisconnect` read is a plausible cause and nobody reproduced it.
- Cite corrections carried: the stale comment is at `z_memman_pc.rs:811-814`, not `:815-818`. The UI and game export pairs are at `ui/exports.rs:33,37` and `game/exports.rs:38,42`.

## Gates

Per commit: `cargo build --workspace` and `cargo test --workspace -- --test-threads=1`. Commit 2 also runs the five golden suites serial at 21 fixtures, the lockstep referee at nine tests, and the end-to-end `devmap mp/ffa1` run where `draw screen without UI loaded` must appear and `unknown cmd connect` must not.

## Dispatch flags

- oracle ambiguity: **false**
- a new state home: **false**
- ABI or parity-gate surface: **false**
- a divergence proposal: **false**

## Disposition

After a clean lane-review, open a pull request to master and merge with a merge commit per DEC-67, ahead of the weather pull request #55.
