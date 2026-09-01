# Packet gh#56 step-001 - the dropped client shutdown

Drafted 2026-09-01 from the completed investigation on gh#56. The mechanism is runtime-proven by lldb backtrace, so this packet formalizes the fix and does not re-open the diagnosis.

## Scope

This step restores two client-shutdown calls the port dropped, so a `devmap` map load tears the `ui` and `cgame` VMs down before `VM_Clear` wipes the VM table. It closes the listen-server connect flood on gh#56.

The step delivers three engine hook fields, their client-side installers, two call sites, and one unit test that pins the VM slot aliasing. It changes no game module, no ABI type, no cvar, and no wire format. It adds no third-party crate. The game side of the flood is faithful and stays untouched.

The step does not change the client's connectionless dispatch, the server's connectionless fallback, the out-of-band gate in `CL_PacketEvent`, the reliable command ring, or `VM_Create`'s slot search. Every one of those is faithful to the oracle by read. The step does not touch `crates/sp/`.

## The proven mechanism

On a `devmap`, `SV_SpawnServer` never shuts the client modules down, and `Hunk_Clear` never shuts them down either. `VM_Clear` then wipes all three `vm_t` slots while `cl.uivm` still holds the address of slot 0. `VM_Create("jampgame")` takes the first free slot, which is slot 0, so `cl.uivm` now names the game module.

The export numbers make the collision silent instead of loud. `MpUiExport::UI_REFRESH` is 5 and `MpUiExport::UI_IS_FULLSCREEN` is 6 (`crates/mp/abi/src/ui/exports.rs:33,37`). `MpGameExport::GAME_CLIENT_DISCONNECT` is 5 and `MpGameExport::GAME_CLIENT_COMMAND` is 6 (`crates/mp/abi/src/game/exports.rs:38,42`). Both pairs line up exactly.

Every connect-screen frame runs `SCR_DrawScreenField`, which calls `VM_Call(cl.uivm, UI_IS_FULLSCREEN)` with an empty argument list (`crates/mp/engine/client/src/cl_scrn.rs:754-759`). That call lands on `GAME_CLIENT_COMMAND` with a zero-filled argument word, so `ClientCommand(ctx, 0)` runs and echoes the shared tokenizer's stale word (`crates/mp/game/src/g_cmds.rs:3623-3630`). The stale word is whatever the connectionless handshake tokenized last, which is `connect` and then `connectResponse`. That is the flood.

The sibling hazard is a wild read. `VM_Call(cl.uivm, UI_REFRESH, &[cl.cls.realtime])` (`crates/mp/engine/client/src/cl_scrn.rs:830-837`) lands on `GAME_CLIENT_DISCONNECT` with `cls.realtime` as the client index, and `ClientDisconnect` does `g_entities.as_mut_ptr().add(clientNum)` and dereferences it (`crates/mp/game/src/g_client.rs:3179-3182`). `cls.realtime` is a millisecond clock, so the add is unbounded. The game side is faithful, so this step fixes the caller and never the callee.

The lldb backtrace that proved it: `SCR_DrawScreenField` -> `VM_Call(cl.uivm, 6)` -> jampgame `vmMain`.

## The oracle, cited

### `SV_SpawnServer` shuts the client down before it clears the hunk

```c
	// if not running a dedicated server CL_MapLoading will connect the client to the server
	// also print some status stuff
	CL_MapLoading();

#ifndef DEDICATED
	// make sure all the client stuff is unloaded
	CL_ShutdownAll();
#endif

	CM_ClearMap();
```

`oracle/codemp/server/sv_init.cpp:509-518`. `Hunk_Clear()` follows at `:527`.

### `Hunk_Clear` shuts the two module VMs down before it clears the table

```c
void Hunk_Clear( void ) {

#ifndef DEDICATED
	CL_ShutdownCGame();
	CL_ShutdownUI();
#endif
	SV_ShutdownGameProgs();

#ifndef DEDICATED
	CIN_CloseAllVideos();
#endif
```

`oracle/codemp/qcommon/z_memman_pc.cpp:752-762`. `VM_Clear()` runs at `:771`, after `R_HunkClearCrap()` at `:768`.

The two blocks overlap on purpose. `CL_ShutdownAll` itself calls `CL_ShutdownCGame` and `CL_ShutdownUI` (`oracle/codemp/client/cl_main.cpp:657-682`), so on the `devmap` path the `Hunk_Clear` pair runs second and both early-return on a null VM pointer. `Hunk_Clear` has other callers, so it keeps its own pair.

### The three symbols

- `CL_ShutdownAll` - `oracle/codemp/client/cl_main.cpp:657`.
- `CL_ShutdownCGame` - `oracle/codemp/client/cl_cgame.cpp:595`.
- `CL_ShutdownUI` - `oracle/codemp/client/cl_ui.cpp:1444`.
- `CIN_CloseAllVideos` - `oracle/codemp/client/cl_cin.cpp:126`.

## The port as it stands

### The two gaps

`crates/mp/engine/server/src/sv_init.rs:553-558` calls the `CL_MapLoading` hook and goes straight to `CM_ClearMap`. The `CL_ShutdownAll` call is absent.

`crates/mp/engine/qcommon/src/z_memman_pc.rs:810-820` opens `Hunk_Clear` with a comment that says the `#ifndef DEDICATED` client blocks are dropped because this is the dedicated build, then calls the `SV_ShutdownGameProgs` hook. The comment sits at `:811-814`. The brief cited `:815-818`, which is the `.expect` chain below it, so this packet uses the corrected range.

### The four client bodies all exist and are complete

- `CL_ShutdownAll(view: &mut EngineHostView, cl: &mut Client)` - `crates/mp/engine/client/src/cl_main.rs:2654`.
- `CL_ShutdownCGame(common: &mut Common, cl: &mut Client)` - `crates/mp/engine/client/src/cl_cgame.rs:675`.
- `CL_ShutdownUI(common: &mut Common, cl: &mut Client)` - `crates/mp/engine/client/src/cl_ui.rs:749`.
- `CIN_CloseAllVideos(view: &mut EngineHostView, cl: &mut Client)` - `crates/mp/engine/client/src/cl_cin.rs:1220`.

`CL_ShutdownAll` already has two live callers, `CL_Shutdown` at `crates/mp/engine/client/src/cl_main.rs:2960` and `CL_FlushMemory` at `:3262`, so this step adds a third caller to an exercised body rather than waking a cold one. `CIN_CloseAllVideos` has zero callers today.

### The hook table and its installer

`EngineHooks` is the qcommon upcall table (`crates/mp/engine/qcommon/src/common/engine_hooks.rs:51-173`). The client tier holds eighteen fields, each `Option<fn(&mut EngineHostView, ..)>` with a `null_client.cpp` no-op default written by `EngineHooks::null_dedicated()` (`:180-223`).

`crates/mp/engine/client/src/hook_install.rs:45-64` swaps the real bodies in through `install_client_engine_hooks`. Each adapter follows one shape:

```rust
/// Raven `CL_MapLoading`. Source: `oracle/codemp/client/cl_main.cpp:778`
fn CL_MapLoading_hook(view: &mut EngineHostView) {
    // SAFETY: view-constructor slot, single-threaded, no other live cast.
    let cl = unsafe { cl_from_view(view) };
    CL_MapLoading(view, cl);
}
```

`crates/mp/engine/client/src/hook_install.rs:101-106`. An adapter whose body takes `&mut Common` passes `view.common` beside the cast `cl`, as `CL_MouseEvent_hook` does at `:135-139`.

`install_engine_hooks` runs in `main()` before `com_init` (`crates/mp/client-app/src/sim.rs:96,99`), and it calls `install_client_engine_hooks` only when `Engine.cl` is seated (`crates/mp/engine/core/src/host_view.rs:73-75`). So `jampded` keeps the null table and the client build gets the real bodies before any `Hunk_Clear` runs.

### Why the defaults must be no-ops rather than `None`

`Com_InitHunkMemory` calls `Hunk_Clear` (`crates/mp/engine/qcommon/src/z_memman_pc.rs:845-848`), and `jampded` calls `Hunk_Clear` on every map load and never installs the client tier. A `.expect` on the new fields would panic there. A no-op default reproduces the excised `#ifndef DEDICATED` call exactly, keeps every call site one unconditional line, and matches all eighteen existing client-tier fields.

`null_client.cpp` defines none of these four symbols, because the guards remove the call sites in the dedicated build rather than the definitions. Row 2 settles what the field docs cite instead.

### The VM slot machinery

`MAX_VM` is 3 (`crates/mp/engine/qcommon/src/vm/module_registry.rs:17`). `VM_Clear` unloads each dll and writes `vm_t::default()` over every slot, then nulls `currentVM` and `lastVM` (`crates/mp/engine/qcommon/src/vm_fns.rs:380-389`). It does not touch `cl.uivm` or `cl.cgvm`, and it cannot, because qcommon sits below the client crate.

`VM_Create` first walks the table for a case-insensitive name match and returns that slot address (`crates/mp/engine/qcommon/src/vm_fns.rs:713-718`), then walks for the first empty name (`:720-733`). Both loops start at index 0, so a cleared table hands slot 0 to the next creator.

`CL_ShutdownUI` nulls `cl.uivm` after `VM_Free` (`crates/mp/engine/client/src/cl_ui.rs:749-759`), and `CL_ShutdownCGame` nulls `cl.cgvm` (`crates/mp/engine/client/src/cl_cgame.rs:675-684`). That is the whole fix. The owner clears its own handle before the table is wiped.

### The test rig for the aliasing pin

`crates/mp/engine/core/tests/demo_referee.rs:126-137` shows how a test seats a VM slot without loading a dylib:

```rust
    assert!(i < MAX_VM, "no free vm_t slot for the probe module");
    view.common.vmTable[i].name = "cgame".to_string();
    let entry: RawVmMain = probe_vm_main;
    view.common.vmTable[i].entryPoint = Some(entry);
```

`crates/mp/engine/core/tests/demo_referee.rs:617-645` shows the lightweight fixture shape: `Engine::new()`, an optional seated `Client`, `engine_host_view(&mut engine)`, then plain asserts.

The test must live in `mp_engine_core`'s test directory, because `VM_Create` takes `&mut EngineHostView` and only `mp_engine_core` exports `engine_host_view`.

`VM_Create` cannot be called on an unseated name in a unit test, because it would reach `Sys_LoadDll` and then the `.qvm` fallback. Row 3 gives the shape that exercises the real lookup loop with no dll on disk.

## Surface contract

### `crates/mp/engine/qcommon/src/common/engine_hooks.rs`

Four new fields on `EngineHooks`, placed at the end of the client tier block, after `Key_WriteBindings`:

```rust
pub CL_ShutdownAll: Option<fn(&mut EngineHostView)>,
pub CL_ShutdownCGame: Option<fn(&mut EngineHostView)>,
pub CL_ShutdownUI: Option<fn(&mut EngineHostView)>,
pub CIN_CloseAllVideos: Option<fn(&mut EngineHostView)>,
```

Four new private no-op bodies beside the other null bodies:

```rust
fn CL_ShutdownAll_null(_view: &mut EngineHostView) {}
fn CL_ShutdownCGame_null(_view: &mut EngineHostView) {}
fn CL_ShutdownUI_null(_view: &mut EngineHostView) {}
fn CIN_CloseAllVideos_null(_view: &mut EngineHostView) {}
```

Four new initializers in `EngineHooks::null_dedicated`, each `Some(<name>_null)`.

### `crates/mp/engine/client/src/hook_install.rs`

Four new private adapters and four new assignments inside `install_client_engine_hooks`:

```rust
fn CL_ShutdownAll_hook(view: &mut EngineHostView);
fn CL_ShutdownCGame_hook(view: &mut EngineHostView);
fn CL_ShutdownUI_hook(view: &mut EngineHostView);
fn CIN_CloseAllVideos_hook(view: &mut EngineHostView);
```

Each casts `cl` with `cl_from_view` under the established SAFETY comment, then calls its body. `CL_ShutdownAll_hook` and `CIN_CloseAllVideos_hook` pass `view`. `CL_ShutdownCGame_hook` and `CL_ShutdownUI_hook` pass `view.common`.

The four imports join the existing `use` groups at the file top: `CL_ShutdownAll` from `crate::cl_main`, `CL_ShutdownCGame` from `crate::cl_cgame`, `CL_ShutdownUI` from `crate::cl_ui`, and `CIN_CloseAllVideos` from `crate::cl_cin`.

### `crates/mp/engine/server/src/sv_init.rs`

No new item. One new statement pair inside `SV_SpawnServer`, between the `CL_MapLoading` hook call at `:556` and `CM_ClearMap` at `:558`, reading the `CL_ShutdownAll` hook and calling it.

### `crates/mp/engine/qcommon/src/z_memman_pc.rs`

No new item. Inside `Hunk_Clear`, two hook reads and calls before the existing `SV_ShutdownGameProgs` block, and one hook read and call after it. The stale DEDICATED comment at `:811-814` is replaced.

### `crates/mp/engine/core/tests/vm_slot_alias.rs`

One new test file. One `#[test] fn a_cleared_slot_is_reused_by_the_next_module()` and one private `extern "C"` syscall stub. No production surface.

### The closed world

Anything not on this list is out of scope, and the agent must not add it.

## The open rows

### Row 1 - `CIN_CloseAllVideos` (mechanical)

The oracle block has three members, not two. `CIN_CloseAllVideos` is fully ported at `crates/mp/engine/client/src/cl_cin.rs:1220` with zero callers in the workspace, and its signature already matches the hook shape.

**Proposed default: port it as the fourth hook in the same commits.** It is the same `#ifndef DEDICATED` block, the body is complete, and leaving it out would close two thirds of one oracle block and leave a fresh gap of the same kind. The surface contract above assumes this default.

### Row 2 - what the four field docs cite (mechanical)

The brief said to cite `oracle/codemp/null/null_client.cpp` for the no-op defaults. That file defines none of the four symbols, because the `#ifndef DEDICATED` guards remove the call sites and no null body is ever needed. Every other client-tier field in the table cites a real `null_client.cpp` body, so this is a new case.

**Proposed default: each field doc cites the real client definition, and each null body cites the guard that excises the call.** So the field carries `/// Source: oracle/codemp/client/cl_main.cpp:657`, and `CL_ShutdownAll_null` carries a one-line note plus `/// Source: oracle/codemp/server/sv_init.cpp:513-516`. The `Hunk_Clear` trio cites `oracle/codemp/qcommon/z_memman_pc.cpp:754-757` and `:760-762`.

### Row 3 - the aliasing test's shape and home (mechanical)

`VM_Create` on an unknown name reaches `Sys_LoadDll` and then a `.qvm` file read, and both fail in a unit test, after which `VM_Free` empties the slot again. So the literal `VM_Create("ui")` the brief names cannot run without a dylib on disk.

**Proposed default: a new file `crates/mp/engine/core/tests/vm_slot_alias.rs` with one test that seats slot names directly and drives the real lookup loop.** The sequence: build `Engine::new()` and `engine_host_view`, seat `vmTable[0].name = "ui"` with a stub entry point after `demo_referee.rs:133-136`, call `VM_Create(&mut view, "ui", Some(stub), vmInterpret_t::VMI_NATIVE)` so the name-match loop at `vm_fns.rs:713-718` returns slot 0's address, record that address, call `VM_Clear(view.common)`, assert the slot's name is empty and its entry point is `None`, seat `vmTable[0].name = "jampgame"` the same way, call `VM_Create(&mut view, "jampgame", Some(stub), vmInterpret_t::VMI_NATIVE)`, and assert the returned pointer equals the recorded one. The test's doc comment names the hazard: a handle held across `VM_Clear` addresses the next module.

If `engine_host_view` needs a seated `Client`, the test seats `engine.cl = Some(Client::default())` as `demo_referee.rs:620` does. The lane confirms which with `cargo test`.

### Row 4 - the golden fixture count (mechanical)

The brief named 22 committed golden fixtures. Master carries 21 (`crates/mp/renderer-gpu/tests/goldens/`, five world goldens in `world_golden.rs`). The twenty-second arrives with the weather branch, and this step merges ahead of that pull request.

**Proposed default: the gate battery names 21 fixtures byte-identical.**

## Record-only findings

- **The SP tree is moot.** `crates/sp/engine/` holds `client`, `ghoul2`, `icarus`, `qcommon`, `rmg`, and `server`, and none of them carries a `z_memman` or an `sv_init` source. `grep` finds no `Hunk_Clear` and no `SV_SpawnServer` under `crates/sp/`. There is nothing to fix there, and SP links its cgame and ui statically in any case.
- **The hoth2 freeze.** The wild `ClientDisconnect` read described above is a plausible cause of the terminal freeze the first gh#56 session saw, and nobody has reproduced it deliberately. This step does not claim the freeze, and the finished file repeats that.
- **Cite corrections carried from the brief.** The stale `Hunk_Clear` comment is at `crates/mp/engine/qcommon/src/z_memman_pc.rs:811-814`, not `:815-818`. The `SV_SpawnServer` gap sits between `crates/mp/engine/server/src/sv_init.rs:556` and `:558`. `ClientCommand` opens at `crates/mp/game/src/g_cmds.rs:3623`. `ClientDisconnect`'s wild read is at `crates/mp/game/src/g_client.rs:3179-3182`.

## Pause triggers, named for this step

- `cargo check` shows a borrow conflict that the `view.common` plus cast `cl` shape cannot satisfy. Do not restructure a body to fix it. Stop and ask.
- The end-to-end run still shows `unknown cmd connect` after both call sites land. That contradicts the diagnosis. Stop and ask.
- The lockstep referee reports a divergence. Stop and ask, and never weaken the comparison.
- Any file outside the write scopes needs an edit.
- Any surface the contract does not list.

## Commit bundle

### The gate battery, named once

Every commit uses `--no-gpg-sign`, a heading subject, an STE body, and no trailer of any kind. Every golden run is serial with `--test-threads=1`, each as one foreground command with a long timeout, because two engine boots in parallel threads crash in the GPU init path and the world-golden pk3 inflate aborts without it.

- `cargo build --workspace`. The bundle's final state builds with zero warnings.
- `cargo test --workspace -- --test-threads=1`.
- `cargo test -p mp_renderer_gpu --test world_golden -- --ignored --test-threads=1`, all five world goldens byte-identical.
- `cargo test -p mp_renderer_gpu --test scene_golden -- --test-threads=1`, all eleven scene goldens byte-identical.
- `cargo test -p mp_renderer_gpu --test entity_golden -- --ignored --test-threads=1`, byte-identical.
- `cargo test -p mp_renderer_gpu --test ghoul2_vertex_golden -- --ignored --test-threads=1`, byte-identical.
- `cargo test -p mp_renderer_gpu --test hud_golden -- --test-threads=1` and the same with `--ignored`, both byte-identical.

Twenty-one committed fixtures, one per test, at `CHANNEL_TOLERANCE` zero in the four image suites.

**The lockstep referee is a gate here**, because this step changes `mp_engine_qcommon` and `mp_engine_server`, both on the `jampded` link set:

```sh
tools/referee-oracle/build.sh
cargo build --workspace
JKA_REF_BASEPATH=~/Developer/jka/jka_server cargo test -p jampgame --test referee -- --ignored --test-threads=1 --nocapture
```

Nine tests pass, and the output holds zero case-insensitive matches for `skip`.

**The end-to-end run** is the gate that proves the fix. Start the client in the background with its output captured, let it sit about sixty seconds, kill it by recorded process id, then read the log:

```sh
./target/release/mp_client_app +set fs_basepath "$HOME/Developer/jka/jamp-client" +set fs_homepath /tmp/jka-client-home +set r_fullscreen 0 +set r_mode 6 +set developer 1 +devmap mp/ffa1 > /tmp/gh56-devmap.log 2>&1 & echo $! > /tmp/gh56-devmap.pid
```

Kill it with `kill "$(cat /tmp/gh56-devmap.pid)"` and never by process name, because a live server must stay untouched. Then:

- `grep -c "draw screen without UI loaded" /tmp/gh56-devmap.log` returns one or more. That line proves `cl.uivm` went null across the map load (`crates/mp/engine/client/src/cl_scrn.rs:745-748`).
- `grep -c "unknown cmd connect" /tmp/gh56-devmap.log` returns zero.

The retail assets under `$HOME/Developer/jka/` are read-only. Build the release client first with `cargo build --release --workspace`.

### The commits

1. **The three shutdown hooks plus the cinematic hook.** Four fields, four null bodies, four `null_dedicated` initializers, and the four client-side adapters and installs. Files: `crates/mp/engine/qcommon/src/common/engine_hooks.rs` and `crates/mp/engine/client/src/hook_install.rs`. Surface: the field, null-body, and adapter lists above. Subject: `feat(gh#56 s001): the client shutdown hooks`. Gates: `cargo build --workspace` and `cargo test --workspace -- --test-threads=1`. No call site exists yet, so behavior does not move and the goldens and the referee are not required on this commit alone.

2. **The two call sites.** `SV_SpawnServer` calls the `CL_ShutdownAll` hook between the `CL_MapLoading` hook and `CM_ClearMap`. `Hunk_Clear` calls `CL_ShutdownCGame` and then `CL_ShutdownUI` before `SV_ShutdownGameProgs`, and `CIN_CloseAllVideos` after it, matching the oracle order at `z_memman_pc.cpp:754-762`. The stale DEDICATED comment is replaced with a truthful one that states the hooks answer no-op on `jampded`. Files: `crates/mp/engine/server/src/sv_init.rs` and `crates/mp/engine/qcommon/src/z_memman_pc.rs`. Surface: no new item. Subject: `fix(gh#56 s001): a map load shuts the client modules down`. Gates: the full battery, the lockstep referee green at nine tests, and the end-to-end `devmap mp/ffa1` run with both grep results as stated above.

3. **The aliasing unit test.** The row-3 test. Files: `crates/mp/engine/core/tests/vm_slot_alias.rs`. Surface: the one test and its stub. Subject: `test(gh#56 s001): a cleared VM slot is reused by the next module`. Gates: `cargo build --workspace` and `cargo test --workspace -- --test-threads=1`.

4. **The finished file**, per the packet skill: assumptions and choices keyed to their commits, deviations or the word "none", the commit list with gate results, and open gaps. The open gaps must name two: the hoth2 freeze is unreproduced and unattributed, and `CIN_CloseAllVideos` gains its first caller with no automated gate on cinematic teardown. File: `.claude/packets/56/step-001/finished.md`. Subject: `process(gh#56 s001): finished file`.

## Write scopes

Branch `gh56-step-001-vm-shutdown`, cut from master at `a081e584`.

- `crates/mp/engine/qcommon/src/common/engine_hooks.rs`
- `crates/mp/engine/qcommon/src/z_memman_pc.rs`
- `crates/mp/engine/client/src/hook_install.rs`
- `crates/mp/engine/server/src/sv_init.rs`
- `crates/mp/engine/core/tests/vm_slot_alias.rs` - new
- `.claude/packets/56/step-001/` - the packet, the synopsis, and the finished file

`oracle/` is read-only. `~/Developer/jka/` is read-only. Source files change through the Edit tool only. `crates/mp/game/` is out of scope, and the game side of the flood must not be touched.

## Repo idiom

- `cargo check` and `cargo build` are the ground truth. rust-analyzer is stale in this workspace.
- Imports are canonical short names at the file top. No `use` inside a function body, and no inline `crate::a::b::c` path in an expression.
- Every new item carries a doc comment and a `Source:` cite.
- Comments run one sentence per line, under 150 columns, and a line break is a semantic act rather than a width fix.
- Commit bodies are STE prose, and each gate paragraph opens with prose rather than a command.

## Disposition

After a clean lane-review this branch opens a pull request to master and merges on GitHub with a merge commit, per DEC-67. It goes ahead of the weather pull request #55. Never squash, and never commit on master.

## Amendments

### Amendment 1 - the lane-review walk, 2026-09-01

The Fable vet returned five findings. The user closed all three disposition rows on 2026-09-01. The verdicts follow.

**Row 1 - the "Unpure client detected" kick was environmental.** The vet saw the end-to-end client kick itself at `CS_PRIMED`. The cause was stale state in `/tmp/jka-client-home`, left by the mixed-binary era of the investigation. The session re-ran the gate with a clean home path, and the log sits at `/tmp/gh56-clean-home-run.log`. That run shows `draw screen without UI loaded` 7 times, `unknown cmd connect` zero times, and `Unpure client` zero times. It also shows one `CS_PRIMED to CS_ACTIVE` transition, so the client reached the live state. Commit 2's spawn claim stands verified. No code change, and no new ticket.

**Row 2 - the surface contract undercounted the test stubs.** The contract names "one private `extern "C"` syscall stub" for `vm_slot_alias.rs`. That count is wrong against ratified row 3's own default, which requires a stub entry point as well. `stub_vm_main` is that entry-point stub, and it is needed because `RawVmMain` and `VM_Create`'s `systemCalls` parameter have different signatures. The `seat_slot_zero` helper is accepted and named here too. All three items are private to the test file, and none is production surface. The lane's confessed deviation stands as written.

**Row 3 - the walk orders two more commits.** A message replay rewrites three commit bodies with every tree unchanged, and the finished file carries the empty-diff proof. A style commit rewords the phrasal-verb doc lines in `vm_slot_alias.rs`, and it changes comments only.

**Finding 6 - the 28-word sentence in `70b65f7d` stands by amendment.** The final vet pass found one more over-cap sentence in that commit body: "The stale comment that claimed the client blocks belong to a dedicated-only build is gone, and the replacement states that the hooks answer with a no-op on `jampded`." A second replay would rewrite hashes that three record files already pin, so the sentence stands as written. Future commit bodies comply with the 25-word cap.
