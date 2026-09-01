# Finished gh#56 step-001 - the dropped client shutdown

Branch `gh56-step-001-vm-shutdown`, cut from `master` at `a081e584`, merged with `master` as the lane's first act (already up to date). Three code commits plus this file. Nothing pushed, no pull request opened.

## Assumptions and choices

**Commit 1 - the four hook fields.** The four fields sit at the end of the client tier, after `Key_WriteBindings`, under their own section header, because `null_client.cpp` defines none of them and the header records that. Each field doc cites the real client body and each null body cites the guard that removes the call, which is row 2's default. `CL_ShutdownCGame_null` and `CL_ShutdownUI_null` share the cite `oracle/codemp/qcommon/z_memman_pc.cpp:754-757`, because one guard covers both calls. The four null bodies sit in a second section under the `null_client.cpp` block rather than inside it, so a reader does not take them for ports of a `null_client.cpp` body that does not exist.

The four adapters follow the established shape. `CL_ShutdownAll_hook` and `CIN_CloseAllVideos_hook` pass `view` beside the cast `cl`, and `CL_ShutdownCGame_hook` and `CL_ShutdownUI_hook` pass `view.common`, which the bodies' signatures fix. No borrow conflict appeared, so the named pause trigger never fired.

Row 1's default holds: `CIN_CloseAllVideos` landed as the fourth hook in the same commits.

**Commit 2 - the two call sites.** `SV_SpawnServer` keeps Raven's own comment, `// make sure all the client stuff is unloaded`, above the hook read. The replacement comment in `Hunk_Clear` states three facts: the client hooks answer with a no-op on `jampded`, the two shutdown calls must run before `VM_Clear`, and a handle left set across `VM_Clear` addresses the next module in that slot. The call order matches the oracle exactly, with `CIN_CloseAllVideos` after `SV_ShutdownGameProgs`.

The end-to-end gate needed no re-staging. The loader resolves `basei386.so` names under `fs_basepath`, and `shasum` showed the three staged modules byte-identical to the fresh `--release` build, because this step changes no module crate. Nothing under `~/Developer/jka/` was written.

**Commit 3 - the aliasing test.** Row 3's shape and home hold. `engine_host_view` does not need a seated `Client` for this test, so the fixture is `Engine::new()` and the view alone. `VM_Create` runs with `VMI_NATIVE` against a pre-seated slot name, so the name-match loop returns before `Sys_LoadDll`, and no dylib is read.

## Deviations

**The test file carries two private stubs, not one.** The surface contract names "one private `extern "C"` syscall stub". Row 3's own sequence also requires a stub entry point and asserts `VM_Clear` drops it, and `RawVmMain` and the `systemCalls` parameter have different signatures, so one function cannot serve both. The file therefore holds `stub_vm_main` and `stub_syscall`. Both are private to the test file and neither is production surface, which the contract's binding clause requires.

No other deviation. No file outside the write scopes was edited.

## Commits and gate results

1. `d9251729` **feat(gh#56 s001): the client shutdown hooks.** `cargo build --workspace` clean with zero warnings. `cargo test --workspace -- --test-threads=1` green, 137 result lines, no failure. The goldens and the referee were not required, because no call site reads the new fields yet and behavior does not move.

2. `5b2d88a2` **fix(gh#56 s001): a map load shuts the client modules down.** The full battery green. Build clean with zero warnings, and `cargo test --workspace -- --test-threads=1` green at 137 result lines. All 21 committed fixtures byte-identical over five serial runs: 5 world (`--ignored`), 11 scene, 2 entity (`--ignored`), 1 ghoul2 vertex (`--ignored`), and 2 hud (one plain run, one `--ignored`). The lockstep referee green at nine tests against the freshly built oracle dylib, with zero case-insensitive matches for `skip`.

   The end-to-end run started the release client with `+devmap mp/ffa1`, let it sit 70 seconds, and killed it by recorded process id. `grep -c "draw screen without UI loaded"` returned `10`, first at log line 1907, which proves `cl.uivm` went null across the map load. `grep -c "unknown cmd connect"` returned `0`. The only connect traffic left is the single handshake pair, `SV packet loopback : connect` at line 1802 and `CL packet loopback: connectResponse` at line 1908. The client spawned into the map and picked items up, so the map load completed rather than stalling.

3. `4b4591e2` **test(gh#56 s001): a cleared VM slot is reused by the next module.** `cargo build --workspace` clean with zero warnings. `cargo test --workspace -- --test-threads=1` green, 138 result lines, no failure. The new test is the one added line.

`rustfmt --check` is clean on all five touched files. No committed fixture moved at any point in the bundle.

## Open gaps

**The hoth2 freeze is unreproduced and unattributed.** The wild `ClientDisconnect` read the packet describes is a plausible cause of the terminal freeze the first gh#56 session saw, and nobody has reproduced it deliberately. This step does not claim that freeze. It removes the caller that made the read possible, which is all the evidence supports.

**`CIN_CloseAllVideos` gains its first caller with no automated gate on cinematic teardown.** The function had zero callers in the workspace before this step. No test in this workspace plays a cinematic, and the end-to-end `devmap` run starts no video, so the new call ran only against an empty `cinTable`. The oracle block and the ported body are the whole evidence for the call. Live play through a cinematic is the verification.

**The SP tree is moot, recorded for the reader.** `crates/sp/engine/` carries no `z_memman` and no `sv_init` source, `grep` finds no `Hunk_Clear` and no `SV_SpawnServer` under `crates/sp/`, and SP links its cgame and ui statically. There is nothing of this kind to fix there.
