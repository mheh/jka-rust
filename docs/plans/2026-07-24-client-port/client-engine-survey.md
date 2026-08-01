# Client engine island — survey (wayfinder ticket #3)

Survey date 2026-08-01, oracle submodule commit `4bebb8ec`. This digest covers the MP client engine: everything the `jamp` client binary needs that is not the ui/cgame modules and not the renderer. All line counts come from `wc -l` on the real files. Every seam claim carries an `oracle/...:line` cite verified by grep or by reading. The link set below comes from `oracle/codemp/jk2mp.vcproj` (the jamp client project), so linked-versus-dead is project-file ground truth, not inference.

## Sizing summary

| island | .cpp lines | .h lines | note |
|---|---:|---:|---|
| `client/` link set (23 TUs) | 34,080 | 3,200 | includes the 412-line dead `0_SH_Leak.cpp` |
| `win32/` link set (10 TUs) | 12,553 | 218 | large parts superseded or already ported, see below |
| `mp3code/` (21 TUs) | 8,380 | ~1,800 | DEC-03 replaces it with a decoder crate, not a port |
| client-only `qcommon/` leftovers | 2,583 | - | `cm_draw.cpp` + `cm_terrainmap.cpp` + dead `CNetProfile.cpp`/`hstring.cpp` |
| dead surface in `client/` + `win32/` | ~19,300 | - | console/Xbox/dx8 lineage, itemized in the dead-surface section |

The port-relevant core is the `client/` link set minus dead files plus two thin platform seams. The sound stack (11,213 lines with headers) and the FX system (9,550 lines with headers) are the two largest single subsystems.

## Client subsystems (`oracle/codemp/client/`)

| subsystem | TUs | .cpp lines | role |
|---|---|---:|---|
| connection + session | cl_main, cl_parse, cl_net_chan | 4,893 | connect state machine, server-message parse, demo record/play, client netchan wrap |
| input + keys | cl_input, cl_keys | 3,601 | usercmd build and packet pacing, key bindings, key catchers |
| module hosts | cl_cgame, cl_ui | 3,627 | VM create/dispatch for cgame and ui, snapshot/gamestate copy-out |
| console + screen | cl_console, cl_scrn | 1,455 | console buffer and draw, `SCR_UpdateScreen` frame orchestration |
| cinematics | cl_cin | 1,494 | RoQ decode and playback, `CIN_*` trap backend |
| FX system | FxScheduler, FxPrimitives, FxTemplate, FxUtil, FxSystem, FXExport | 7,945 | client-side effect engine behind the `CG_FX_*` traps |
| sound stack | snd_dma, snd_mem, snd_mix, snd_mp3, snd_music, snd_ambient | 10,653 | software mixer, sample cache, MP3 decode wrap, dynamic music, ambient sets |
| dead in dir | 0_SH_Leak | 412 | SmartHeap leak tracker, `MEM_DEBUG`-gated |

Header lines: `client.h` 622, `keys.h` 66, `keycodes.h` 347, FX headers 1,605 (incl. `fffx.h` 129), sound headers 560.

### Connection + session (cl_main 3,685 / cl_parse 1,033 / cl_net_chan 175)

`cl_main.cpp` owns the three key globals: `clientActive_t cl`, `clientConnection_t clc`, `clientStatic_t cls` (`oracle/codemp/client/cl_main.cpp:105-107`, externs at `client.h:139,236,382`). `cl` is wiped per gamestate and holds snapshots, usercmds, baselines, and parse entities (`client.h:75-137`). `clc` is wiped per disconnect and holds the netchan, reliable command buffers, download state, and demo state (`client.h:166-234`). `cls` is never wiped and holds the connection state enum, server browser lists, and the renderer handles (`client.h:295-349`). `cl_main` also owns demo record/play: `CL_Record_f` writes `demos/*.dm_26` files and `CL_PlayDemo_f` replays them (`cl_main.cpp:295,330,554`). Demo playback is a deterministic input path into the whole client, which matters for the validation-rig ruling. `cl_parse.cpp` decodes server messages into snapshots through the ported msg/huffman layer. `cl_net_chan.cpp` is the client twin of the ported `sv_net_chan` wrapper.

The renderer seam is the `refexport_t re` function table (`client.h:388`, instance `cl_main.cpp:111`) filled by `CL_InitRef` from the statically linked `GetRefAPI` (`cl_main.cpp:2480-2483`, `oracle/codemp/renderer/tr_public.h:116`). The renderer stays out of this island per the renderer plan.

### Input + keys (cl_input 1,897 / cl_keys 1,704)

`cl_input.cpp` state: `frame_msec`/`old_com_frameTime` and the `kbutton_t` set `in_left ... in_buttons[16]` (`cl_input.cpp:11-51`). It builds usercmds from key/mouse/joystick state plus the cgame-fed values in `cl` (`client.h:100-110`), and paces outgoing packets. `cl_keys.cpp` owns `keyGlobals_t kg` with `qkey_t keys[MAX_KEYS]` (`keys.h:32-45`). Seams: key catcher flags route input to console, ui, or cgame (`cls.keyCatchers`, `client.h:297`), and `CL_KeyEvent`/`CL_MouseEvent`/`CL_CharEvent`/`CL_JoystickEvent` are upcalls from the engine event loop.

### Module hosts (cl_cgame 2,108 / cl_ui 1,519)

Facts in the hosting-seam section below.

### Console + screen (cl_console 831 / cl_scrn 624)

`cl_console.cpp` owns `console_t` (`client.h:358-380`, 32 KB text ring). `cl_scrn.cpp` owns `SCR_UpdateScreen`, which brackets the frame with `re` calls and drives `VM_Call(cgvm, CG_DRAW_ACTIVE_FRAME, ...)` (`cl_cgame.cpp:1840-1842`) or the ui equivalent by connection state.

### Cinematics (cl_cin 1,494)

RoQ video decode plus the `CIN_PlayCinematic`/draw trap backend for both modules (10 `CG_CIN_*`/`UI_CIN_*` case arms across the two dispatchers). Output goes through `re.DrawStretchRaw`. `BinkVideo.cpp` is the Xbox replacement and is not in the link set.

### FX system (7,945 .cpp + 1,605 .h)

The client-side effect engine that serves the `CG_FX_*` traps (36 case labels in `cl_cgame.cpp`, gated by `#ifndef DEBUG_DISABLEFXCALLS`, `cl_cgame.cpp:1098`). `FXExport.cpp` (105 lines) is the flat `FX_*` function surface the dispatcher calls. This is a C++ class subsystem on the porting-rules §F track: `CFxScheduler` (`FxScheduler.h:373`), `CPrimitiveTemplate` (`FxScheduler.h:152`), a primitive class hierarchy in `FxPrimitives.h`, and the `SFxHelper` service wrapper `theFxHelper` (`FxSystem.h:49,221`) that reaches renderer, CM, and sound. The type-port campaign already landed FX enum/struct types in `crates/mp/engine/client/src/fx/` with one open `//TODO: Port CPrimitiveTemplate` marker (`fx/seffect_template.rs:24`).

### Sound stack (10,653 .cpp + 560 .h + mp3code 8,380)

`snd_dma.cpp` (6,325) is the front end: channels, sfx cache, spatialization, and both output arms. The software-mixer arm goes `snd_mix.cpp` -> `SNDDMA_*` device seam (`snd_local.h:157-167`, implemented by `win32/win_snd.cpp`). The OpenAL/EAX arm is gated by `s_UseOpenAL` (`snd_dma.cpp:213`, cvar read at `snd_dma.cpp:480-481`) with EAX state at `snd_dma.cpp:255-265` and vendored headers in `client/OpenAL/` (1,203 lines) and `client/eax/` (1,733 lines). DEC-03 rules the port: faithful software mixer through cpal, MP3 decode via minimp3, EAX and force feedback dropped. So the OpenAL/EAX arm is do-not-port, which removes a large share of `snd_dma.cpp` (approximate, not measured per-arm). `snd_mem.cpp` loads and resamples samples, `snd_mp3.cpp` wraps the `mp3code/` decoder (21 TUs, 8,380 lines, replaced not ported per DEC-03), `snd_music.cpp` (1,153) is the dynamic-music state machine, and `snd_ambient.cpp` (1,137) is the ambient-set system behind the three `CG_AS_*` traps (`cl_cgame.cpp:849-857`). MP `snd_music` does not use `hstring` (that dependency is SP-only). Sound type skeletons already exist in `crates/mp/engine/client/src/snd/` and `snd_ambient/` from the type port.

## Platform layer (`oracle/codemp/win32/`)

Link-set TUs and their disposition. "Replaced" cites the Rust home that already covers the role for `jampded`. "Superseded" means the winit/wgpu/cpal stack replaces the role wholesale, per the world-harness precedent.

| TU | lines | role | disposition |
|---|---:|---|---|
| win_main.cpp | 1,609 | WinMain loop, `Sys_QueEvent`/`Sys_GetEvent` ring (`win_main.cpp:1178,1211`), dll load, error boxes | ring + loop already ported: `SysEventQueue` (`crates/mp/engine/qcommon/src/common/sys_event_queue.rs`), `sys_engine.rs`, `crates/native/platform/src/sys_main.rs` |
| win_net.cpp | 1,222 | WinSock UDP/IPX | replaced: `crates/native/platform/src/net.rs`, `crates/mp/engine/qcommon/src/sys_net.rs` |
| win_shared.cpp | 547 | `Sys_Milliseconds`, cpuid, user dirs | replaced: `sys_engine.rs` + `timing/` in mp qcommon |
| win_syscon.cpp | 574 | Windows GUI console window | superseded: terminal console I/O (`net.rs:202 sys_console_input`) serves `jampded`; no GUI console on the mac target |
| win_wndproc.cpp | 547 | window messages -> event ring | superseded: winit `window_event`/`device_event` (`crates/mp/renderer-gpu/src/bin/world_harness.rs:999-1010`) |
| win_glimp.cpp | 2,095 | GL context, display modes | superseded: wgpu surface + winit window (`world_harness.rs:961-975`) |
| win_qgl.cpp | 4,271 | GL function-pointer loader | superseded entirely by wgpu |
| win_input.cpp | 1,141 | DirectInput mouse/kb/joystick pump, `IN_Frame` (`win_input.cpp:714`) | superseded: winit device events already pump mouse/keyboard in the harness; joystick support is a design point for the platform ruling |
| win_snd.cpp | 382 | DirectSound DMA buffer behind `SNDDMA_*` | superseded per DEC-03: cpal is the device seam |
| win_gamma.cpp | 165 | hardware gamma ramp | superseded: gamma is renderer-side under wgpu |

The load-bearing platform fact: the engine consumes input only through the `Sys_QueEvent` ring, and that ring plus its consumer loop are already ported and frozen (LIFE-D1, `sys_event_queue.rs:1`). A winit pump therefore feeds an existing seam. No mac/SDL lineage exists in this oracle tree; `win32/` is the only platform directory.

## Hosting seam (cl_cgame.cpp / cl_ui.cpp)

- cgame VM: `cgvm = VM_Create("cgame", CL_CgameSystemCalls, interpret)` (`cl_cgame.cpp:1771`), dispatcher `CL_CgameSystemCalls` (`cl_cgame.cpp:644`). Inbound `VM_Call` sites include `CG_INIT` (`cl_cgame.cpp:1780`), `CG_DRAW_ACTIVE_FRAME` (`cl_cgame.cpp:1840-1842`), `CG_CONSOLE_COMMAND` (`cl_cgame.cpp:1820`), `CG_SHUTDOWN` (`cl_cgame.cpp:601`). The client also calls the ui module from here: `VM_Call(uivm, UI_SET_ACTIVE_MENU, ...)` (`cl_cgame.cpp:984`).
- ui VM: `uivm = VM_Create("ui", CL_UISystemCalls, interpret)` (`cl_ui.cpp:1478`), dispatcher `CL_UISystemCalls` (`cl_ui.cpp:813`), `UI_INIT` call at `cl_ui.cpp:1494`.
- Trap counts, measured: `cgameImport_t` has 217 entries (`oracle/codemp/cgame/cg_public.h:56-341`) and the dispatcher serves 215 of them. The two unserved entries are `CG_TESTPRINTINT`/`CG_TESTPRINTFLOAT` (fall to the error default). Five additional case labels sit inside comment blocks (camera + SP-string arms, `cl_cgame.cpp:1083-1092,1264,1655`). `uiImport_t` has 150 entries (`oracle/codemp/ui/ui_public.h:17-192`) and the dispatcher serves 133. The 17 unserved entries are `UI_CM_LOADMODEL` plus the 16 QVM float/string shims (`UI_MEMSET ... UI_ASIN`, `UI_TESTPRINT*`). `UI_G2_REMOVEBOLT` is commented out (`cl_ui.cpp:1354`) and `UI_G2_ADDSKINGORE` is `#ifdef _SOF2` (`cl_ui.cpp:1361-1365`). This corrects the scoping.md census figures of "216 cgame / 124 ui".
- Export counts confirm scoping.md: `cgameExport_t` 32 entries (`cg_public.h:344-440`), `uiExport_t` 12 entries (`ui_public.h:194-251`).
- Trap groups by case-label count in the dispatchers: cgame G2 55, renderer 39, FX 36, sound 16, CM 13, AS 3; ui G2 37 (2 dead per above), renderer 18, sound 4.
- The one Class-A retained pointer, confirmed: `case CG_SET_SHARED_BUFFER: cl.mSharedMemory = ((char *)VMA(1))` (`cl_cgame.cpp:1682-1683`), field `client.h:136`. This is the engine-side twin of `sv.mSharedMemory` (`oracle/codemp/server/sv_game.cpp:940`, `server.h:87`), which the Rust server already models. No ui shared buffer exists (scoping.md census, re-confirmed: no other `VMA` pointer is stored in `cl_ui.cpp`).
- Copy-out seam functions (Class B shapes): `CL_GetGameState` (`cl_cgame.cpp:78`), `CL_GetUserCmd` (`cl_cgame.cpp:97`), `CL_GetSnapshot` translating `clSnapshot_t` to the module `snapshot_t` with entity-count truncation (`cl_cgame.cpp:157-192`), `GetClientState` filling `uiClientState_t` (`cl_ui.cpp:41,1045`).
- The Rust hook seam already exists and names the exact client entry points: `EngineHooks` (`crates/mp/engine/qcommon/src/common/engine_hooks.rs:51-92`) carries `CL_Shutdown`, `CL_Disconnect`, `CL_FlushMemory`, `CL_Init`, `CL_StartHunkUsers`, `CL_MapLoading`, `CL_PacketEvent`, `CL_Frame`, `CL_InitKeyCommands`, `CL_JoystickEvent`, `CL_MouseEvent`, `CL_CharEvent`, `CL_KeyEvent`, `CL_ForwardCommandToServer`, `CL_GameCommand`, `UI_GameCommand`, `Key_WriteBindings`, `SND_FreeOldestSound`, `SND_RegisterAudio_LevelLoadEnd`, plus `VM_CallSlot`. `null_dedicated()` fills them with the ported null-client no-ops (`crates/mp/engine/client/src/null/`). The oracle call sites these mirror sit in the ported common loop (`oracle/codemp/qcommon/common.cpp:936,1254,1395,1711`). The client island is, structurally, the second filler of this existing hook table.

## Already-ported overlap

This section names each client-engine dependency that already exists in Rust, with its crate home. The four design rulings read this first.

| oracle dependency | status | Rust home |
|---|---|---|
| common loop, cvar, cmd, FS, unzip, md4, zlib | ported | `crates/mp/engine/qcommon/src/{common,cvar,cmd,files,unzip.rs,md4,zlib_seam.rs}` |
| huffman, msg, netchan | ported | `crates/mp/engine/qcommon/src/{qcommon/huff.rs,msg.rs,net_chan.rs}` |
| CM (load, trace, patch, terrain, shader) | ported | `crates/mp/engine/qcommon/src/cm_*.rs` |
| vm.cpp loader/dispatch | ported | `crates/mp/engine/qcommon/src/vm_fns.rs` + `crates/mp/host-interface/src/vm_slot.rs` |
| `Sys_QueEvent` ring + `Sys_GetEvent` loop | ported | `sys_event_queue.rs`, `sys_engine.rs` |
| platform: net, timers, console I/O, module load, mem | ported | `crates/native/platform/src/{net.rs,sys_main.rs,sys_shared.rs,module_loader,mem.rs}` |
| server (listen-server half of jamp) | ported, live | `crates/mp/engine/server` |
| ghoul2 G2_API/bones/bolts/misc/surfaces (the 5 TUs jamp links) | ported | `crates/mp/engine/ghoul2` — the 55 cgame + 35 live ui G2 traps route here |
| ICARUS, RMG, NPCNav, StringEd, ROFF, GP2 | ported | `crates/mp/engine/{icarus,rmg}`, qcommon `stringed`/`roff`/`gp2` |
| null client (dedicated no-op CL surface) | ported | `crates/mp/engine/client/src/null/` |
| client/snd/fx/mp3/keys TYPE skeletons | types only, 4 `TODO: Port` markers | `crates/mp/engine/client/src/{client,snd,snd_ambient,fx,mp3,keys,fffx}` incl. `clientActive_t`/`clientConnection_t`/`clientStatic_t`/`console_t` |
| `Engine.cl`/`Engine.snd` island placeholders | stubs | `crates/mp/engine/client/src/client_host.rs:9-23` (`Client`, `SoundSystem`) |
| windowing + input pump + wgpu surface precedent | exists | `crates/mp/renderer-gpu/src/bin/world_harness.rs:71-75,961-1010,1341` |
| jpeg/png decode | policy-replaced | zune-jpeg per DEC-49; renderer scope |

Not yet ported, and needed by the client island: everything in the client subsystem table above, plus `cm_draw.cpp` (1,490, `CDraw32` pixel raster) and `cm_terrainmap.cpp` (497) — the RMG automap image path consumed via `CDraw32` from `cm_terrainmap.cpp:52-280` and reachable only through client traps (`CG_RMG_INIT` at `cl_cgame.cpp:1689`, `CG_CM_REGISTER_TERRAIN` at `cl_cgame.cpp:1686`, the wireframe-automap arm at `cl_cgame.cpp:1080`).

## Dead surface (do not port)

Per porting rule §20, no call site in the MP client path. Link-set membership verified against `jk2mp.vcproj`.

- `client/BinkVideo.cpp/.h` (597) — Xbox cinematics, not in the link set.
- `client/cl_cin_console.cpp` (332), `client/snd_dma_console.cpp` (2,933), `client/snd_mem_console.cpp` (355), `client/snd_local_console.h` (121) — console/Xbox twins, not in the link set.
- `client/0_SH_Leak.cpp` (412) — SmartHeap `MEM_DEBUG` leak tracker; the Rust allocator replaces SmartHeap entirely.
- `client/eax/` (1,733) + `client/OpenAL/` (1,203) headers and the `s_UseOpenAL`/EAX arms of `snd_dma.cpp` — dropped by DEC-03.
- `win32/` non-link-set files, 12,087 lines total: `win_qgl_dx8.cpp` (6,495), `win_qal_xbox.cpp` (1,321), `win_input_rumble.cpp` (705), `win_input_console.cpp` (648), `win_main_console.cpp` (615), `win_filecode.cpp` (345), `win_main_common.cpp` (331), `win_input_xbox.cpp` (309), `win_stream_dx8.cpp` (290), `win_glimp_console.cpp` (281), `dbg_console_xbox.cpp/.h` (206), `win_file_xbox.cpp` (171), `snd_fx_img.h` (85), `win_gamma_console.cpp` (73), `glw_win_dx8.h` (180), `win_file.h` (32).
- `qcommon/CNetProfile.cpp` (96) — the whole TU is `#ifdef _DONETPROFILE_` (`CNetProfile.cpp:4-97`); never enabled.
- `qcommon/hstring.cpp` MP twin (500) — its only MP consumer is the dead `CNetProfile.cpp`; the live hstring users are SP-only (SP twin already ported at `crates/sp/engine/qcommon/src/hstring/`).
- Seam-level dead surface: the 16 ui QVM shim traps and `UI_CM_LOADMODEL` (no dispatch arm), `CG_TESTPRINTINT`/`CG_TESTPRINTFLOAT`, the commented camera/SP-string cgame arms, `UI_G2_REMOVEBOLT`, and the `_SOF2`-gated `UI_G2_ADDSKINGORE`.

## State classification hints (scoping.md A-D vocabulary)

- Class A: `cl.mSharedMemory` (`client.h:136`) is the single engine-retained pointer into module memory, set at `cl_cgame.cpp:1682-1683`. The engine reads and writes the typed `TCG*` structs through it (census in scoping.md, `cg_public.h:442-591`). The A-lite trajectory case (`CG_GET_*_TRAJECTORY` + ROFF writes) also lands on this island's dispatcher.
- Class B: the copy-out shapes at the module seam stay layout-frozen: `gameState_t`, `snapshot_t` (built from `clSnapshot_t`, `cl_cgame.cpp:157-192`), `usercmd_t`, `uiClientState_t` (`cl_ui.cpp:41`), plus the renderer-bound `refEntity_t`/`refdef_t` family and `glconfig_t` (`cls.glconfig`, `client.h:341`). The wire types embedded in `cl` (`entityState_t` baselines and parse entities, `playerState_t` in snapshots) are already ported frozen types.
- Class C: the interiors of `clientActive_t`, `clientConnection_t`, `clientStatic_t`, `console_t`, `keyGlobals_t`, the `kbutton_t` set, FX scheduler internals, and the sound channel/sfx state are engine-private. No module or engine peer retains pointers into them across calls. They are free for idiomatic shapes, subject only to the embedded Class-B/wire members above.
- The FX system is the island's one §F C++-track subsystem (class hierarchy + scheduler). The sound stack, cinematics, and all `cl_*` TUs are C-track.
- The `clientSnapshot_t`-to-`snapshot_t` truncation (`MAX_ENTITIES_IN_SNAPSHOT`, `cl_cgame.cpp:192`) and the reliable-command ring copy-out (`CL_GetServerCommand`) are behavior seams the validation rig can observe directly.

## Corrections to scoping.md

- Trap totals: measured `cgameImport_t` = 217 entries / 215 dispatched (doc said 216) and `uiImport_t` = 150 entries / 133 dispatched (doc said 124). The export counts (32 / 12) confirm.
- The doc's `cl_cgame.cpp:1682` cite for the shared-buffer store points at the case label; the assignment is line 1683.
