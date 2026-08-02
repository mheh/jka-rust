//! Shared mock-engine + lifecycle-drive machinery for the ABI smoke tests.
//!
//! Both `abi_smoke.rs` (our Rust `jampgame` cdylib) and `oracle_smoke.rs`
//! (Raven's UNMODIFIED jampgame built by `tools/referee-oracle/build.sh`) drive a
//! loadable module through the exact engine/module contract — `dllEntry(syscall)`
//! then `vmMain(command, arg0..arg11)` — against ONE mock engine, asserting the
//! module survives `GAME_INIT` -> warm-up `GAME_RUN_FRAME`s -> `CLIENT_CONNECT` ->
//! `CLIENT_BEGIN` -> connected frames -> `CLIENT_COMMAND` -> `CLIENT_DISCONNECT`
//! -> `GAME_SHUTDOWN` and produces the structural side effects the engine relies
//! on. Raven's DLL calling our mock through the SEAM-D11 trampoline is the
//! referee acceptance proof.
//!
//! Transport wiring (all real, no shims of our own):
//!   * the module is loaded through `native_platform::module_loader::sys_load_dll`
//!     — the ported libloading loader; it resolves and invokes `dllEntry`, handing
//!     the module the engine syscall trampoline;
//!   * the syscall trampoline handed over is `mp_engine_qcommon`'s real C-variadic
//!     `game_syscall_trampoline` (SEAM-D11); the module's `trap_*` wrappers call
//!     it variadically and it forwards a flat 16-word frame to
//!     `game_syscall_trampoline_words`, which reads the armed `EngineSlot` and
//!     dispatches into our fixed-arity `mock_syscall`;
//!   * `arm_game_slot` injects `(ctx, mock_syscall)` into that slot before the
//!     first `vmMain`.
//!
//! The oracle DLL's `vmMain`/syscall use Raven's original 32-bit `int` widths
//! while the loader's `AbiWord` is `isize`; on the arm64 host this is benign for
//! the small non-negative lifecycle values (writes to a 32-bit result register
//! zero-extend), and the variadic syscall path is width-agnostic (pointers pass
//! at their natural width). So the identical trampoline drives both modules.

#![allow(non_snake_case)]
#![allow(dead_code)]

pub mod reflog;

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::ffi::{c_char, c_int, c_short, c_void, CStr, CString};
use std::path::{Path, PathBuf};

use mp_abi::game::exports::MpGameExport;
use mp_abi::game::imports::MpGameImport;
use mp_engine_qcommon::vm::{arm_game_slot, game_syscall_trampoline};
use mp_qshared::common::mp::qcommon::usercmd_t;
use mp_qshared::common::mp::trace_t::trace_t;
use mp_qshared::shared::cvar::vmCvar_t;
use mp_qshared::shared::limits::ENTITYNUM_NONE;
use mp_qshared::shared::vec3_t;
use native_platform::entrypoints::{AbiCommand, AbiWord, RawSyscall, RawVmMain};
use native_platform::module_loader::{
    sys_load_dll, LoadedModule, ModuleNaming, ModuleSearchPolicy, SearchStep,
};

// ---- Referee swap (plan §3c): the real engine island behind the mock -------
// A real BSP + the real `SV_Trace`/`SV_LinkEntity`/`sv_world` back the spatial
// arms on real-map scenarios; the rest of the mock stays trusted.
use mp_engine_core::engine::Engine;
use mp_engine_core::host_view::engine_host_view;
use mp_engine_qcommon::cm_load::{CM_EntityString, CM_LoadMap};
use mp_engine_qcommon::cmd_common::{Cbuf_Init, Cmd_Init};
use mp_engine_qcommon::common::engine_host_view::EngineHostView;
use mp_engine_qcommon::common::opaque_slots;
use mp_engine_qcommon::cvar_fns::{Cvar_Get, Cvar_Init};
use mp_engine_qcommon::files_common::FS_InitFilesystem;
use mp_engine_qcommon::vm::vm_s::vm_t;
use mp_engine_qcommon::z_memman_pc::{Com_InitHunkMemory, Com_InitZoneMemory};
use mp_engine_server::server::server_state_t::serverState_t;
use mp_engine_server::sv_game::SV_GameSystemCalls;
use mp_engine_server::sv_world::SV_ClearWorld;
use mp_qshared::shared::cvar::CVAR_INIT;
use mp_qshared::shared::{qfalse, qtrue};

// The referee differential driver (`tests/referee.rs`) reuses this mock verbatim
// as its deterministic engine; the pub surface below (`referee_*` fns) is its
// entry into the same MOCK the smoke lifecycle drives. Kept in one module so the
// oracle and Rust runs go through byte-identical engine behavior.

// ---------------------------------------------------------------------------
// Import-number constants (wire values). Used as `match` patterns in the mock.
// ---------------------------------------------------------------------------

const G_PRINT: isize = MpGameImport::G_PRINT as isize;
const G_ERROR: isize = MpGameImport::G_ERROR as isize;
const G_MILLISECONDS: isize = MpGameImport::G_MILLISECONDS as isize;
const G_CVAR_REGISTER: isize = MpGameImport::G_CVAR_REGISTER as isize;
const G_CVAR_UPDATE: isize = MpGameImport::G_CVAR_UPDATE as isize;
const G_CVAR_SET: isize = MpGameImport::G_CVAR_SET as isize;
const G_CVAR_VARIABLE_INTEGER_VALUE: isize = MpGameImport::G_CVAR_VARIABLE_INTEGER_VALUE as isize;
const G_CVAR_VARIABLE_STRING_BUFFER: isize = MpGameImport::G_CVAR_VARIABLE_STRING_BUFFER as isize;
const G_ARGC: isize = MpGameImport::G_ARGC as isize;
const G_ARGV: isize = MpGameImport::G_ARGV as isize;
/// The one file the mock FS serves (see `G_FS_FOPEN_FILE`): the committed
/// synthetic humanoid animation.cfg parity fixture.
const ANIMCFG: &[u8] =
    include_bytes!("../../../mp/game/tests/oracle/fixtures/pmove_saber/animation.cfg");
const ANIMCFG_HANDLE: c_int = 0x41_4E_49; // "ANI"

const G_FS_FOPEN_FILE: isize = MpGameImport::G_FS_FOPEN_FILE as isize;
const G_FS_READ: isize = MpGameImport::G_FS_READ as isize;
const G_FS_WRITE: isize = MpGameImport::G_FS_WRITE as isize;
const G_FS_FCLOSE_FILE: isize = MpGameImport::G_FS_FCLOSE_FILE as isize;
const G_FS_GETFILELIST: isize = MpGameImport::G_FS_GETFILELIST as isize;
const G_GET_SERVERINFO: isize = MpGameImport::G_GET_SERVERINFO as isize;
const G_GET_USERINFO: isize = MpGameImport::G_GET_USERINFO as isize;
const G_SET_CONFIGSTRING: isize = MpGameImport::G_SET_CONFIGSTRING as isize;
const G_GET_CONFIGSTRING: isize = MpGameImport::G_GET_CONFIGSTRING as isize;
const G_GET_ENTITY_TOKEN: isize = MpGameImport::G_GET_ENTITY_TOKEN as isize;
const G_LOCATE_GAME_DATA: isize = MpGameImport::G_LOCATE_GAME_DATA as isize;
const G_SET_USERINFO: isize = MpGameImport::G_SET_USERINFO as isize;
const G_GET_USERCMD: isize = MpGameImport::G_GET_USERCMD as isize;
const G_SEND_SERVER_COMMAND: isize = MpGameImport::G_SEND_SERVER_COMMAND as isize;
const G_SEND_CONSOLE_COMMAND: isize = MpGameImport::G_SEND_CONSOLE_COMMAND as isize;
const G_TRACE: isize = MpGameImport::G_TRACE as isize;
const G_TRACECAPSULE: isize = MpGameImport::G_TRACECAPSULE as isize;
const G_G2TRACE: isize = MpGameImport::G_G2TRACE as isize;
// Spatial/world arms routed to the REAL engine on real-map scenarios (referee
// swap, plan §3c). The already-declared G_LOCATE_GAME_DATA/G_TRACE/
// G_TRACECAPSULE/G_G2TRACE/G_GET_ENTITY_TOKEN complete the REAL set.
const G_POINT_CONTENTS: isize = MpGameImport::G_POINT_CONTENTS as isize;
const G_LINKENTITY: isize = MpGameImport::G_LINKENTITY as isize;
const G_UNLINKENTITY: isize = MpGameImport::G_UNLINKENTITY as isize;
const G_ENTITIES_IN_BOX: isize = MpGameImport::G_ENTITIES_IN_BOX as isize;
const G_ENTITY_CONTACT: isize = MpGameImport::G_ENTITY_CONTACT as isize;
const G_ENTITY_CONTACTCAPSULE: isize = MpGameImport::G_ENTITY_CONTACTCAPSULE as isize;
const G_IN_PVS: isize = MpGameImport::G_IN_PVS as isize;
const G_IN_PVS_IGNORE_PORTALS: isize = MpGameImport::G_IN_PVS_IGNORE_PORTALS as isize;
const G_AREAS_CONNECTED: isize = MpGameImport::G_AREAS_CONNECTED as isize;
const G_ADJUST_AREA_PORTAL_STATE: isize = MpGameImport::G_ADJUST_AREA_PORTAL_STATE as isize;
const G_SET_BRUSH_MODEL: isize = MpGameImport::G_SET_BRUSH_MODEL as isize;

/// True when `n` is in the REAL set — the spatial/world syscall arms the referee
/// swap routes to the real engine crates (real BSP + `SV_Trace`/`SV_LinkEntity`/
/// `sv_world`). Everything else keeps the trusted mock behavior. `G_G2TRACE` is
/// included: its real arm (`sv_game.rs` G_G2TRACE) is a plain `SV_Trace` with no
/// ghoul2 dereference, so it is safe with zero ghoul2 instances (G2 model init
/// stays mocked to 0). Source: `sv_game.rs` REAL-set dispatch arms.
fn is_real_set(n: isize) -> bool {
    matches!(
        n,
        G_LOCATE_GAME_DATA
            | G_TRACE
            | G_TRACECAPSULE
            | G_G2TRACE
            | G_POINT_CONTENTS
            | G_LINKENTITY
            | G_UNLINKENTITY
            | G_ENTITIES_IN_BOX
            | G_ENTITY_CONTACT
            | G_ENTITY_CONTACTCAPSULE
            | G_IN_PVS
            | G_IN_PVS_IGNORE_PORTALS
            | G_AREAS_CONNECTED
            | G_ADJUST_AREA_PORTAL_STATE
            | G_SET_BRUSH_MODEL
            | G_GET_ENTITY_TOKEN
    )
}

/// Realistic userinfo string for client 0, exercising the keys
/// `ClientUserinfoChanged` reads (g_client.c:1888). A drop-in of what a real
/// client submits: name, rate/snaps, model + team_model, colors, handicap, sex,
/// item prediction, teamtask, an empty password, and a full `forcepowers` config.
///
/// The `forcepowers` value is Raven's own canonical `<rank>-<side>-<18 power
/// digits>` string (its `BG_LegalizedForcePowers` fallback, bg_misc.c:457).
/// NUM_FORCE_POWERS is 18, so the 18 digits fully populate the force-power array.
/// This is REQUIRED, not decorative: a real client always sends `forcepowers`,
/// and Raven's `WP_InitForcePowers`/`BG_LegalizedForcePowers` (w_force.c:277,
/// bg_misc.c:439) leaves its `int final_Powers[NUM_FORCE_POWERS]` stack array
/// UNINITIALIZED when the string is empty — then reads garbage as a force-power
/// level and indexes `bgForcePowerCost[i][countDown]` out of bounds, segfaulting
/// (ClientBegin -> WP_InitForcePowers). The Rust port survives an empty string
/// only because Rust zero-inits the array; the unmodified oracle does not.
const CLIENT0_USERINFO: &str = "\\name\\Padawan\\rate\\25000\\snaps\\20\\model\\kyle/default\
\\team_model\\kyle/default\\color1\\4\\color2\\4\\handicap\\100\\sex\\male\
\\cg_predictItems\\1\\teamtask\\0\\forcepowers\\7-1-032330000000001333\\password\\";

// ---------------------------------------------------------------------------
// Mock engine state
// ---------------------------------------------------------------------------

/// What `G_LOCATE_GAME_DATA` handed the engine — the core structural side effect
/// of `GAME_INIT` (`g_public.h:145`: the game tells the server where and how big
/// the entity/client arrays are).
#[derive(Clone, Copy, Debug)]
pub struct LocateData {
    pub g_ents: *mut c_void,
    pub num_g_entities: c_int,
    pub sizeof_g_entity_t: c_int,
    pub clients: *mut c_void,
    pub sizeof_g_client: c_int,
}

// ---------------------------------------------------------------------------
// Referee swap (plan §3c): the real engine island behind the spatial arms.
// ---------------------------------------------------------------------------

/// A never-invoked native `vmMain` stand-in. The real engine's `VM_ArgPtrWord`
/// (`vm_fns.rs:213`) only checks `currentVM.entryPoint.is_some()` to pick the
/// native identity-cast branch (dataBase 0 → the arg word already IS the real
/// pointer); it never CALLS `entryPoint`. A `Some(_)` holding any valid fn
/// pointer is all that path needs, so the routed arms resolve the module's real
/// `trace_t`/`vec3`/`sharedEntity_t` pointer args unchanged.
extern "C-unwind" fn real_world_native_marker(
    _command: AbiCommand,
    _a0: AbiWord,
    _a1: AbiWord,
    _a2: AbiWord,
    _a3: AbiWord,
    _a4: AbiWord,
    _a5: AbiWord,
    _a6: AbiWord,
    _a7: AbiWord,
    _a8: AbiWord,
    _a9: AbiWord,
    _a10: AbiWord,
    _a11: AbiWord,
) -> AbiWord {
    0
}

/// The real engine island backing the referee's spatial/world syscall arms.
/// Owns a `Box<Engine>` booted just far enough to load a real BSP and answer
/// `SV_Trace`/`SV_LinkEntity`/`sv_world`/PVS/pointcontents/brushmodel/entity-
/// token traps. The rest of the engine (VM load, spawn, frames) is deliberately
/// NOT run — the MOCK owns the game slot and drives the module itself.
struct RealWorld {
    engine: Box<Engine>,
    /// Kept alive for the island's lifetime: `engine.common.currentVM` points
    /// at it so `VM_ArgPtrWord` takes the native identity-cast branch.
    _vm: Box<vm_t>,
}

impl RealWorld {
    /// Boot the real island: a minimal FS bring-up (the ordered subset of
    /// `Com_Init` FS/CM need), `CM_LoadMap`, then the `SV_SpawnServer` tail that
    /// the routed arms read (`SV_ClearWorld` worldSectors + `entityParsePoint`
    /// from the real entity string + `sv.state`).
    ///
    /// `map_bsp` is `"maps/<map>.bsp"`; `basepath` is the assets dir (its `base/`
    /// holds `assets0.pk3`). Panics (a loud finding) if the map or assets are
    /// missing — callers gate on the assets dir first.
    fn new(map_bsp: &str, basepath: &str) -> RealWorld {
        let mut engine = Engine::new();

        // Install the server + renderer hook tables — but NOT the game-slot
        // arm. `mp_engine_core::install_engine_hooks` also builds the
        // GameDispatchCtx note and calls `arm_game_slot`; here the MOCK owns the
        // slot (`referee_arm`), so we replicate that fn's body MINUS the arm
        // step (host_view.rs:47-70).
        mp_engine_server::hook_install::install_engine_hooks(&mut engine.common.hooks);
        mp_renderer::hook_install::install_engine_hooks(&mut engine.common.hooks);

        // ---- Phase A: FS bring-up + CM_LoadMap (view-scoped) --------------
        // Minimal ordered subset of `Com_Init` (`common_fns.rs:1535-1602`):
        // Cvar_Init → Cbuf_Init → Com_InitZoneMemory (FS pack loads Z_Malloc) →
        // Cmd_Init (FS_Startup registers commands) → FS_InitFilesystem, then
        // `dedicated` (CM_LoadMap dereferences `com_dedicated`, common_fns:1595)
        // → Com_InitHunkMemory (CM_LoadMap loads geometry onto the hunk). NET/
        // console/journaling/config-exec and the later subsystem inits are
        // provably unneeded by FS+CM, so they are skipped.
        {
            let mut view = engine_host_view(&mut engine);
            Cvar_Init(&mut view);
            Cbuf_Init(view.common);
            Com_InitZoneMemory(&mut view);
            Cmd_Init(&mut view);

            // Point fs_basepath at the assets dir BEFORE FS_Startup reads it.
            // Cvar_Get keeps this value when FS_Startup re-registers the cvar
            // with its platform default (cvar_fns.rs:230-272).
            Cvar_Get(&mut view, "fs_basepath", basepath, CVAR_INIT);
            FS_InitFilesystem(&mut view);

            let ded = Cvar_Get(&mut view, "dedicated", "0", 0);
            view.common.com_dedicated = Some(ded);
            // The trace path dereferences `com_terrainPhysics` directly
            // (cm_trace.rs:1216); an unregistered (`None`) field aborts.
            // Register it exactly as `Com_Init` does (common_fns.rs:1662). The
            // routed arms' other `com_*` reads are null-safe
            // (`com_optvehtrace` via name lookup; `com_RMG` behind a
            // registered-guard), so only this one is required.
            view.common.com_terrainPhysics = Some(Cvar_Get(
                &mut view,
                "com_terrainPhysics",
                "1",
                mp_qshared::shared::cvar::CVAR_CHEAT,
            ));
            Com_InitHunkMemory(&mut view);

            let mut checksum: c_int = 0;
            CM_LoadMap(&mut view, map_bsp, qfalse, &mut checksum);
        }

        // ---- Phase B: the SV_SpawnServer tail the routed arms read --------
        // `SV_GetEntityToken` takes the main-BSP path only when
        // `mLocalSubBSPIndex == -1` (sv_game.cpp:210) — the alloc_zeroed default
        // is 0, so set it (a real server's SV_SetActiveSubBSP(-1) equivalent).
        engine.sv.sv.mLocalSubBSPIndex = -1;
        let eps = CM_EntityString(&mut engine.cm);
        engine.sv.sv.entityParsePoint = eps;
        SV_ClearWorld(&mut engine.cm, &mut engine.sv);
        engine.sv.sv.state = serverState_t::SS_GAME;

        // A native `currentVM` marker so `vma`/`VM_ArgPtrWord` in the routed
        // dispatcher resolve the module's real pointer args by identity
        // (dataBase 0, entryPoint Some). The real boot's `VM_Create` sets this;
        // we skip loading a VM (the MOCK drives the module), so wire it by hand.
        // MUST come AFTER Phase A: `Com_InitHunkMemory` → `Hunk_Clear` →
        // `VM_Clear` (vm_fns.rs:382) nulls `currentVM`, so setting it earlier
        // would be wiped.
        let mut vm: Box<vm_t> = Box::new(vm_t::default());
        vm.entryPoint = Some(real_world_native_marker);
        vm.dataBase = core::ptr::null_mut();
        engine.common.currentVM = &mut *vm as *mut vm_t;
        // `qtrue` is only pulled in for symmetry with the `qfalse` clientload
        // arg above; touch it so the shared import is never flagged unused.
        let _ = qtrue;

        RealWorld { engine, _vm: vm }
    }

    /// Route one syscall frame to the real dispatcher, rebuilding the
    /// `EngineHostView` + icarus/nav/roff sidecars from disjoint engine-field
    /// borrows (mirrors `server_host.rs::game_system_calls_shim`).
    fn dispatch(&mut self, args: *const isize) -> isize {
        let engine: &mut Engine = &mut self.engine;
        let cl_raw = match engine.cl.as_mut() {
            Some(cl) => cl as *mut _ as *mut (),
            None => core::ptr::null_mut(),
        };
        let mut view = EngineHostView {
            sv: opaque_slots::Server::from_raw(&mut engine.sv as *mut _ as *mut ()),
            cl: opaque_slots::Client::from_raw(cl_raw),
            // The game module never reaches the mixer, so the sound slot is NULL.
            snd: opaque_slots::SoundSystem::from_raw(core::ptr::null_mut()),
            bot: opaque_slots::BotLib::from_raw(&mut engine.bot as *mut _ as *mut ()),
            rm: opaque_slots::RenderModels::from_raw(
                &mut engine.render_models as *mut _ as *mut (),
            ),
            re: opaque_slots::Renderer::from_raw(core::ptr::null_mut()),
            rmg: opaque_slots::RmManager::from_raw(&mut engine.rmg as *mut _ as *mut ()),
            g2: opaque_slots::Ghoul2System::from_raw(&mut engine.g2 as *mut _ as *mut ()),
            // The game module never reaches the client FX system.
            fx: opaque_slots::FxSystem::from_raw(core::ptr::null_mut()),
            common: &mut engine.common,
            cm: &mut engine.cm,
        };
        // The raw slot casts alias disjoint engine fields that
        // `SV_GameSystemCalls` reborrows exactly as the boot shim does (DEC-23
        // slot-cast discipline, single-threaded referee); `icarus`/`nav`/`roff`
        // are disjoint from `common`/`cm`, so these field borrows never overlap.
        // `args` is the trampoline's 16-word frame (pointer args already real).
        // `SV_GameSystemCalls` is a safe fn — its own `unsafe` is internal.
        SV_GameSystemCalls(
            &mut view,
            &mut engine.icarus,
            &mut engine.nav,
            &mut engine.roff,
            args as *mut isize,
        )
    }
}

struct MockEngine {
    /// Monotonic `trap_Milliseconds` clock.
    millis: c_int,
    /// In-memory cvar table keyed by name (the engine's persisted values).
    cvars: BTreeMap<String, String>,
    /// `vmCvar_t.handle` → cvar name, so `G_CVAR_UPDATE` (which passes only the
    /// vmCvar pointer) can re-serve the right value.
    handle_names: BTreeMap<c_int, String>,
    next_handle: c_int,
    /// Configstrings the game set (`G_SET_CONFIGSTRING`), recorded by index.
    configstrings: BTreeMap<c_int, String>,
    /// Per-client userinfo strings served by `G_GET_USERINFO` and mutated by
    /// `G_SET_USERINFO` (the game rewrites userinfo on illegal name changes).
    userinfos: BTreeMap<c_int, String>,
    /// Minimal BSP entity token stream served by `G_GET_ENTITY_TOKEN`.
    tokens: Vec<CString>,
    token_idx: usize,
    /// Current command tokens served by `G_ARGC`/`G_ARGV` (the engine's parsed
    /// `Cmd_Argv` view) while a `GAME_CLIENT_COMMAND`/`GAME_CONSOLE_COMMAND` runs.
    cmd_args: Vec<CString>,
    /// Captured `G_LOCATE_GAME_DATA` payload (None until INIT calls it).
    locate: Option<LocateData>,
    /// `G_PRINT` capture.
    prints: Vec<String>,
    /// Set iff `G_ERROR` fired (a real `Com_Error` — a test failure).
    g_error: Option<String>,
    /// Per-import call counts for the coverage report.
    counts: BTreeMap<isize, u32>,
    /// Imports hit that the mock does not model (served the permissive `0`).
    logged_only: BTreeMap<isize, u32>,
    /// Per-client `usercmd_t` served by `G_GET_USERCMD` (the referee injects the
    /// replay log's input here each frame; unset clients get an all-zero cmd).
    usercmds: BTreeMap<c_int, usercmd_t>,
    /// Import-number sequence issued since the last `referee_begin_frame`
    /// (pointer-free — just the wire numbers, in call order).
    frame_imports: Vec<isize>,
    /// String payloads of the string-bearing syscalls issued this frame
    /// (`SET_CONFIGSTRING`/`SEND_SERVER_COMMAND`/`SEND_CONSOLE_COMMAND`/`PRINT`),
    /// captured by pointed-to DATA so the digest never hashes a raw address.
    frame_texts: Vec<(isize, c_int, String)>,
    /// The real engine island (referee swap, plan §3c). `Some` only on real-map
    /// scenarios (`sc.map` starting with `"mp/"`); when armed, the spatial/world
    /// syscall arms in [`is_real_set`] route to it instead of the mock arm.
    real: Option<RealWorld>,
}

impl MockEngine {
    fn new() -> Self {
        // Serve a plain FFA, single-server-slice environment. g_gametype=0
        // (FFA), bot_enable/dedicated=0, sv_maxclients modest. The module's own
        // cvar table defaults are echoed back by `G_CVAR_REGISTER` when a name
        // is not pre-seeded here, so only overrides need listing.
        let mut cvars = BTreeMap::new();
        for (k, v) in [
            ("g_gametype", "0"),
            ("bot_enable", "0"),
            ("dedicated", "0"),
            ("sv_maxclients", "16"),
            ("com_buildScript", "0"),
        ] {
            cvars.insert(k.to_string(), v.to_string());
        }

        // A minimal playable map: a `worldspawn` followed by one FFA spawn point.
        // G_ParseSpawnVars expects `{`, then key/value pairs, then `}`; the first
        // entity must be `worldspawn` (SP_worldspawn errors otherwise). The
        // `info_player_deathmatch` is required so ClientSpawn's SelectSpawnPoint
        // finds a spot — with none, Raven's `G_Error("Couldn't find a spawn
        // point")` drops the game (g_client.c SelectSpawnPoint). After the stream
        // `G_GET_ENTITY_TOKEN` returns qfalse (end of entity string).
        // Contract: `qboolean trap_GetEntityToken( char *buffer, int bufferSize )`
        // — Source: oracle/codemp/game/g_public.h:221.
        let tokens = [
            "{",
            "classname",
            "worldspawn",
            "}", //
            "{",
            "classname",
            "info_player_deathmatch",
            "origin",
            "0 0 100",
            "angle",
            "0",
            "}",
        ]
        .into_iter()
        .map(|s| CString::new(s).unwrap())
        .collect();

        // Client 0 arrives with a realistic userinfo; other slots are empty
        // until a (hypothetical) connect populates them.
        let mut userinfos = BTreeMap::new();
        userinfos.insert(0, CLIENT0_USERINFO.to_string());

        MockEngine {
            millis: 0,
            cvars,
            handle_names: BTreeMap::new(),
            next_handle: 1,
            configstrings: BTreeMap::new(),
            userinfos,
            tokens,
            token_idx: 0,
            cmd_args: Vec::new(),
            locate: None,
            prints: Vec::new(),
            g_error: None,
            counts: BTreeMap::new(),
            logged_only: BTreeMap::new(),
            usercmds: BTreeMap::new(),
            frame_imports: Vec::new(),
            frame_texts: Vec::new(),
            real: None,
        }
    }

    /// Install the tokenized command the engine's `Cmd_Argc`/`Cmd_Argv` view
    /// serves for the duration of one `ClientCommand`/`ConsoleCommand` dispatch.
    fn set_cmd(&mut self, tokens: &[&str]) {
        self.cmd_args = tokens.iter().map(|t| CString::new(*t).unwrap()).collect();
    }

    /// Clear the command view once the dispatch returns.
    fn clear_cmd(&mut self) {
        self.cmd_args.clear();
    }
}

// The mock is reached only through the single-threaded engine slot; a
// thread-local `RefCell` gives the fixed-arity syscall fn access to it without a
// `static mut`.
thread_local! {
    static MOCK: RefCell<MockEngine> = RefCell::new(MockEngine::new());
}

// ---------------------------------------------------------------------------
// Raw-frame helpers
// ---------------------------------------------------------------------------

/// Read argument word `i` from the trampoline's flat frame. `args[0]` is the
/// import number; `args[1..]` are the syscall's own words in encode order.
///
/// # Safety
/// `args` is the shim's 16-word frame; `i` must be < 16.
unsafe fn word(args: *const isize, i: usize) -> isize {
    *args.add(i)
}

/// Borrow a NUL-terminated C string argument as an owned `String` (lossy).
unsafe fn c_str(ptr: isize) -> String {
    if ptr == 0 {
        return String::new();
    }
    CStr::from_ptr(ptr as *const c_char)
        .to_string_lossy()
        .into_owned()
}

/// Write `value` (NUL-terminated, truncated to `size`) into a C char buffer.
///
/// The trap ABI declares `bufferSize` as a 32-bit `int` (e.g.
/// `trap_GetEntityToken( char *buffer, int bufferSize )`, g_public.h:221). On
/// this 64-bit host the C-variadic trampoline widens every arg to `intptr_t`
/// via `va_arg(ap, intptr_t)` (vm/game_syscall_trampoline.c:37), but an `int`
/// variadic arg is NOT promoted past 32 bits — the high 32 bits of the slot are
/// whatever garbage the module's register/stack held at the call. Reading the
/// full `isize` therefore sometimes yields a huge or NEGATIVE size (codegen- and
/// layout-dependent), and a negative one used to make this fn skip the write,
/// leaving the module's `com_token` buffer stale — the source of the flaky
/// `G_ParseSpawnVars: found 0 when expecting {` / `G_FindConfigstringIndex:
/// overflow` oracle crashes. Read only the ABI-defined low 32 bits.
unsafe fn write_c_buffer(buf: isize, size: isize, value: &str) {
    let size = size as c_int;
    if buf == 0 || size <= 0 {
        return;
    }
    let cap = size as usize;
    let dst = buf as *mut c_char;
    let bytes = value.as_bytes();
    let n = bytes.len().min(cap.saturating_sub(1));
    for (k, &b) in bytes.iter().take(n).enumerate() {
        *dst.add(k) = b as c_char;
    }
    *dst.add(n) = 0;
}

// ---------------------------------------------------------------------------
// The mock engine syscall entrypoint (matches `SlotSyscall`)
// ---------------------------------------------------------------------------

/// `extern "C-unwind" fn(ctx, args) -> isize` — the injected engine dispatch
/// target the inbound trampoline forwards to (`vm.cpp:471-472,506`). `args[0]` is
/// the `MpGameImport` number; the rest are the syscall's words.
///
/// Strategy: PERMISSIVE DEFAULT — an unmodeled syscall is logged and returns 0,
/// so gaps are visible (in the coverage report), never fatal. Only the syscalls
/// `GAME_INIT`/frame/shutdown actually need substance are implemented.
extern "C-unwind" fn mock_syscall(_ctx: *mut c_void, args: *const isize) -> isize {
    let n = unsafe { word(args, 0) };

    MOCK.with(|m| {
        let mut m = m.borrow_mut();
        *m.counts.entry(n).or_insert(0) += 1;
        // Pointer-free syscall-stream record for the referee digest: the wire
        // number in call order. String/scalar payloads are added per arm below.
        m.frame_imports.push(n);

        // Referee swap (plan §3c): on real-map scenarios, forward the spatial/
        // world arms to the real engine (real BSP + SV_Trace/SV_LinkEntity/
        // sv_world). Recording above is unchanged (digest semantics preserved);
        // everything NOT in the REAL set falls through to the trusted mock.
        if m.real.is_some() && is_real_set(n) {
            // G_LOCATE_GAME_DATA feeds BOTH sides: the mock records the module's
            // array pointers (the referee snapshots through them) AND the real
            // SV_LocateGameData records them so SV_LinkEntity/SV_Trace reach the
            // module's entities. Capture here, then route.
            if n == G_LOCATE_GAME_DATA {
                m.locate = Some(LocateData {
                    g_ents: unsafe { word(args, 1) } as *mut c_void,
                    num_g_entities: unsafe { word(args, 2) } as c_int,
                    sizeof_g_entity_t: unsafe { word(args, 3) } as c_int,
                    clients: unsafe { word(args, 4) } as *mut c_void,
                    sizeof_g_client: unsafe { word(args, 5) } as c_int,
                });
            }
            let real = m.real.as_mut().unwrap();
            return real.dispatch(args);
        }

        match n {
            G_PRINT => {
                let s = unsafe { c_str(word(args, 1)) };
                eprint!("[G_PRINT] {s}");
                m.frame_texts.push((n, 0, s.clone()));
                m.prints.push(s);
                0
            }
            G_ERROR => {
                // A real Com_Error must fail the test loudly (SEAM-D12: this
                // panic unwinds back through the C-unwind trampoline frame).
                let s = unsafe { c_str(word(args, 1)) };
                m.g_error = Some(s.clone());
                panic!("module raised G_ERROR: {s}");
            }
            G_MILLISECONDS => {
                let t = m.millis;
                m.millis += 1;
                t as isize
            }
            // ---- cvar family ------------------------------------------------
            G_CVAR_REGISTER => {
                // ( vmCvar_t *vmCvar, const char *varName, const char *default, int flags )
                let cvar_ptr = unsafe { word(args, 1) } as *mut vmCvar_t;
                let name = unsafe { c_str(word(args, 2)) };
                let default = unsafe { c_str(word(args, 3)) };
                let value = m.cvars.entry(name.clone()).or_insert(default).clone();
                let handle = m.next_handle;
                m.next_handle += 1;
                m.handle_names.insert(handle, name);
                unsafe { write_vmcvar(cvar_ptr, handle, &value) };
                0
            }
            G_CVAR_UPDATE => {
                // ( vmCvar_t *vmCvar ) — recover the name via the stored handle.
                let cvar_ptr = unsafe { word(args, 1) } as *mut vmCvar_t;
                if !cvar_ptr.is_null() {
                    let handle = unsafe { (*cvar_ptr).handle };
                    if let Some(name) = m.handle_names.get(&handle).cloned() {
                        if let Some(value) = m.cvars.get(&name).cloned() {
                            unsafe { write_vmcvar(cvar_ptr, handle, &value) };
                        }
                    }
                }
                0
            }
            G_CVAR_SET => {
                // ( const char *var_name, const char *value )
                let name = unsafe { c_str(word(args, 1)) };
                let value = unsafe { c_str(word(args, 2)) };
                m.cvars.insert(name, value);
                0
            }
            G_CVAR_VARIABLE_INTEGER_VALUE => {
                let name = unsafe { c_str(word(args, 1)) };
                m.cvars
                    .get(&name)
                    .and_then(|v| v.trim().parse::<i32>().ok())
                    .unwrap_or(0) as isize
            }
            G_CVAR_VARIABLE_STRING_BUFFER => {
                let name = unsafe { c_str(word(args, 1)) };
                let value = m.cvars.get(&name).cloned().unwrap_or_default();
                unsafe { write_c_buffer(word(args, 2), word(args, 3), &value) };
                0
            }
            // ---- command args ----------------------------------------------
            // `int trap_Argc( void )` / `void trap_Argv( int n, char *buffer,
            // int bufferSize )` — the module's tokenized view of the current
            // command. Source: oracle/codemp/game/g_public.h (trap_Argc/Argv).
            G_ARGC => m.cmd_args.len() as isize,
            G_ARGV => {
                // `int n` — the LP64 oracle's varargs promote int with garbage
                // upper bits (2026-07-17 finding: 0x1_0000_0000 for n=0), so
                // decode at C int width like the real engine does.
                let idx = unsafe { word(args, 1) } as c_int as usize;
                let s = m
                    .cmd_args
                    .get(idx)
                    .map(|c| c.to_str().unwrap())
                    .unwrap_or("");
                unsafe { write_c_buffer(word(args, 2), word(args, 3), s) };
                0
            }
            // ---- filesystem: one committed fixture, everything else missing
            G_FS_FOPEN_FILE => {
                // ( const char *qpath, fileHandle_t *f, fsMode_t mode ) -> int len
                // The humanoid animation.cfg is REQUIRED once tape clients
                // actually join and animate (2026-07-17: the oracle SIGSEGVs in
                // BG_SetAnim on the unparsed table; spectators never animated).
                // Serve the committed synthetic fixture; every other path stays
                // missing (handle 0, length -1) so optional loads skip.
                let qpath = unsafe { c_str(word(args, 1)) };
                let handle_out = unsafe { word(args, 2) } as *mut c_int;
                if qpath == "models/players/_humanoid/animation.cfg" {
                    if !handle_out.is_null() {
                        unsafe { *handle_out = ANIMCFG_HANDLE };
                    }
                    ANIMCFG.len() as isize
                } else {
                    if !handle_out.is_null() {
                        unsafe { *handle_out = 0 };
                    }
                    -1
                }
            }
            G_FS_READ => {
                // ( void *buffer, int len, fileHandle_t f ) — whole-file reads
                // of the one served fixture; anything else is a no-op.
                let buf = unsafe { word(args, 1) } as *mut u8;
                let len = unsafe { word(args, 2) } as c_int;
                let handle = unsafe { word(args, 3) } as c_int;
                if handle == ANIMCFG_HANDLE && !buf.is_null() && len > 0 {
                    let n = (len as usize).min(ANIMCFG.len());
                    unsafe { core::ptr::copy_nonoverlapping(ANIMCFG.as_ptr(), buf, n) };
                }
                0
            }
            G_FS_WRITE | G_FS_FCLOSE_FILE => 0,
            G_FS_GETFILELIST => 0,
            // ---- server info / config strings ------------------------------
            G_GET_SERVERINFO => {
                unsafe {
                    write_c_buffer(
                        word(args, 1),
                        word(args, 2),
                        "\\g_gametype\\0\\sv_maxclients\\16\\mapname\\smoke",
                    )
                };
                0
            }
            G_GET_USERINFO => {
                // `void trap_GetUserinfo( int num, char *buffer, int bufferSize )`.
                // A realistic client userinfo drives ClientUserinfoChanged's key
                // reads (name/model/color/sex/handicap/cg_predictItems/team_model
                // /snaps/rate) faithfully; unknown clients get an empty string.
                // Source: oracle/codemp/game/g_client.c:1912,2269 (trap_GetUserinfo).
                let num = unsafe { word(args, 1) } as c_int;
                let s = m.userinfos.get(&num).cloned().unwrap_or_default();
                unsafe { write_c_buffer(word(args, 2), word(args, 3), &s) };
                0
            }
            G_SET_USERINFO => {
                // `void trap_SetUserinfo( int num, const char *buffer )`.
                // Source: oracle/codemp/game/g_public.h (trap_SetUserinfo).
                let num = unsafe { word(args, 1) } as c_int;
                let s = unsafe { c_str(word(args, 2)) };
                m.userinfos.insert(num, s);
                0
            }
            G_SET_CONFIGSTRING => {
                let num = unsafe { word(args, 1) } as c_int;
                let s = unsafe { c_str(word(args, 2)) };
                m.frame_texts.push((n, num, s.clone()));
                m.configstrings.insert(num, s);
                0
            }
            // ---- usercmd delivery (referee replay input) -------------------
            // `void trap_GetUsercmd( int clientNum, usercmd_t *cmd )` — fill the
            // out-param from the per-client cmd the referee injected for this
            // frame (all-zero for unset clients — a valid idle cmd).
            // Source: oracle/codemp/game/g_public.h:219.
            G_GET_USERCMD => {
                let num = unsafe { word(args, 1) } as c_int;
                let out = unsafe { word(args, 2) } as *mut usercmd_t;
                if !out.is_null() {
                    let cmd = m.usercmds.get(&num).copied().unwrap_or_default();
                    unsafe { *out = cmd };
                }
                0
            }
            // ---- broadcast commands (captured by DATA for the digest) -------
            // `void trap_SendServerCommand( int clientNum, const char *text )`
            // and `void trap_SendConsoleCommand( int when, const char *text )`.
            // The pointed-to string is hashed (not the pointer) so the two runs'
            // differing heap addresses never perturb the syscall-stream digest.
            G_SEND_SERVER_COMMAND | G_SEND_CONSOLE_COMMAND => {
                let arg1 = unsafe { word(args, 1) } as c_int;
                let s = unsafe { c_str(word(args, 2)) };
                m.frame_texts.push((n, arg1, s));
                0
            }
            // ---- collision traces (DETERMINISM-CRITICAL) -------------------
            // `trap_Trace`/`trap_TraceCapsule`/`trap_G2Trace` are OUT-PARAM
            // syscalls: the engine writes the `trace_t` result. The old
            // permissive default returned 0 but left `*results` UNINITIALIZED —
            // pmove's ground/step traces then read the module's stack garbage,
            // making `origin.z` (and everything downstream) nondeterministic
            // ACROSS PROCESSES (the referee's oracle-vs-oracle self-test caught
            // exactly this). Serve a deterministic empty-space result: moved
            // fully to `end`, hit nothing. Same layout for all three (results at
            // word 1, end vec3 at word 5). Source: g_public.h trap_Trace.
            G_TRACE | G_TRACECAPSULE | G_G2TRACE => {
                let results = unsafe { word(args, 1) } as *mut trace_t;
                let end = unsafe { word(args, 5) } as *const vec3_t;
                if !results.is_null() {
                    let endpos = if end.is_null() {
                        [0.0, 0.0, 0.0]
                    } else {
                        unsafe { *end }
                    };
                    let mut tr: trace_t = unsafe { core::mem::zeroed() };
                    tr.fraction = 1.0; // 1.0 = didn't hit anything
                    tr.entityNum = ENTITYNUM_NONE as c_short;
                    tr.endpos = endpos;
                    unsafe { *results = tr };
                }
                0
            }
            G_GET_CONFIGSTRING => {
                let num = unsafe { word(args, 1) } as c_int;
                let s = m.configstrings.get(&num).cloned().unwrap_or_default();
                unsafe { write_c_buffer(word(args, 2), word(args, 3), &s) };
                0
            }
            G_GET_ENTITY_TOKEN => {
                // qboolean ( char *buffer, int bufferSize )
                if m.token_idx < m.tokens.len() {
                    let tok = m.tokens[m.token_idx].clone();
                    m.token_idx += 1;
                    unsafe { write_c_buffer(word(args, 1), word(args, 2), tok.to_str().unwrap()) };
                    1 // qtrue
                } else {
                    0 // qfalse — end of entity string
                }
            }
            G_LOCATE_GAME_DATA => {
                m.locate = Some(LocateData {
                    g_ents: unsafe { word(args, 1) } as *mut c_void,
                    num_g_entities: unsafe { word(args, 2) } as c_int,
                    sizeof_g_entity_t: unsafe { word(args, 3) } as c_int,
                    clients: unsafe { word(args, 4) } as *mut c_void,
                    sizeof_g_client: unsafe { word(args, 5) } as c_int,
                });
                0
            }
            // ---- permissive default: log + return 0 ------------------------
            other => {
                *m.logged_only.entry(other).or_insert(0) += 1;
                0
            }
        }
    })
}

/// Write a `vmCvar_t` back into the module's memory (`q_shared.h:1818-1830`):
/// `handle`, `modificationCount`, `value = atof(str)`, `integer = atoi(str)`,
/// and the NUL-terminated `string`.
unsafe fn write_vmcvar(ptr: *mut vmCvar_t, handle: c_int, value: &str) {
    if ptr.is_null() {
        return;
    }
    let integer = value.trim().parse::<i32>().unwrap_or(0);
    let float = value.trim().parse::<f32>().unwrap_or(0.0);
    (*ptr).handle = handle;
    (*ptr).modificationCount = 1;
    (*ptr).value = float;
    (*ptr).integer = integer;
    let bytes = value.as_bytes();
    let n = bytes.len().min((*ptr).string.len() - 1);
    for (k, &b) in bytes.iter().take(n).enumerate() {
        (*ptr).string[k] = b as c_char;
    }
    (*ptr).string[n] = 0;
}

// ---------------------------------------------------------------------------
// Artifact location helpers
// ---------------------------------------------------------------------------

/// Platform cdylib filename for a `[lib] name = "<base>"` crate:
/// `"jampgame"` → `libjampgame.dylib` / `libjampgame.so` / `jampgame.dll`.
pub fn dylib_filename(base: &str) -> String {
    #[cfg(target_os = "macos")]
    {
        format!("lib{base}.dylib")
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        format!("lib{base}.so")
    }
    #[cfg(windows)]
    {
        format!("{base}.dll")
    }
}

/// Locate a cargo-built cdylib next to the test binary. Integration tests run
/// from `target/<profile>/deps/`; cargo places the cdylib in both `deps/` and its
/// parent `target/<profile>/`. `cargo build --workspace` (CI) or `cargo test -p
/// jampgame` (which builds the package's cdylib target) produces it beforehand.
pub fn locate_cargo_cdylib(filename: &str) -> PathBuf {
    let exe = std::env::current_exe().expect("current_exe");
    let deps = exe.parent().expect("deps dir");
    for dir in [deps, deps.parent().expect("target/<profile> dir")] {
        let candidate = dir.join(filename);
        if candidate.exists() {
            return candidate;
        }
    }
    panic!(
        "built cdylib `{filename}` not found next to test binary ({}). \
         Run `cargo build --workspace` (or `cargo test -p jampgame`, which builds \
         the cdylib target) first — the test never spawns cargo.",
        deps.display()
    );
}

// ---------------------------------------------------------------------------
// The single lifecycle drive (with all structural assertions)
// ---------------------------------------------------------------------------

/// `GameWorld` / Raven's `level` island (MAX_GENTITIES entities + clients +
/// shared buffers) is many megabytes and is built by value inside `vmMain`'s
/// GAME_INIT bootstrap. The default 2 MiB test-thread stack overflows on it; the
/// real engine drives `vmMain` on its large main-thread stack. Run the whole
/// single-shot lifecycle on a thread with a generous stack to match. (The oracle
/// DLL's GAME_INIT allocates just as big.)
pub fn run_on_engine_thread(dylib: PathBuf) {
    let handle = std::thread::Builder::new()
        .name("abi-smoke-engine".into())
        .stack_size(64 * 1024 * 1024)
        .spawn(move || run_lifecycle(dylib))
        .expect("spawn engine thread");
    handle.join().expect("engine lifecycle thread panicked");
}

fn run_lifecycle(dylib: PathBuf) {
    let dir = dylib.parent().unwrap().to_path_buf();
    let file = dylib
        .file_name()
        .expect("dylib filename")
        .to_str()
        .expect("utf8 dylib filename")
        .to_string();
    eprintln!("[smoke] loading {}", dylib.display());

    // Load through the real ported loader. A single FsPath step whose
    // base/gamedir join to the artifact directory, plus explicit naming, drives
    // the loader straight at our file — exercising its FS probe + the
    // `dllEntry(syscall)` handshake (win_main.cpp:858-887). The syscall handed
    // over is the real C-variadic inbound trampoline.
    // `name` borrows `file` (any lifetime); `suffix` must be `&'static str`
    // (`ModuleNaming.suffix: Option<&'static str>`) so it comes from the
    // platform-fixed literal, not a slice of the local filename.
    let name: &str = split_dylib_stem(&file);
    let policy = ModuleSearchPolicy {
        naming: ModuleNaming {
            suffix: Some(platform_dylib_suffix()),
        },
        direct_first: false,
        steps: vec![SearchStep::FsPath {
            base: dir,
            gamedir: String::new(),
        }],
    };

    // Arm the engine slot BEFORE any module syscall can fire (the first one is
    // inside GAME_INIT). dllEntry (called by the loader) only stores the pointer.
    arm_game_slot(std::ptr::null_mut(), mock_syscall);

    let syscall: RawSyscall = game_syscall_trampoline as *const c_void;
    let module = sys_load_dll(&policy, name, syscall)
        .expect("sys_load_dll resolved dllEntry+vmMain and completed the handshake");
    let vm_main: RawVmMain = module.entry();

    // ---- GAME_INIT -------------------------------------------------------
    // ( int levelTime, int randomSeed, int restart ). qfalse restart.
    let init_ret = call_vm(vm_main, MpGameExport::GAME_INIT, &[600, 42, 0]);
    eprintln!("[smoke] GAME_INIT returned {init_ret}");

    // ---- structural assertions on INIT side effects ----------------------
    MOCK.with(|m| {
        let m = m.borrow();

        assert!(
            m.g_error.is_none(),
            "GAME_INIT raised G_ERROR: {:?}",
            m.g_error
        );

        let locate = m
            .locate
            .expect("GAME_INIT must call G_LOCATE_GAME_DATA (g_public.h:145)");
        assert!(
            !locate.g_ents.is_null(),
            "LOCATE_GAME_DATA gentities base is null"
        );
        assert!(
            !locate.clients.is_null(),
            "LOCATE_GAME_DATA clients base is null"
        );
        assert!(
            locate.num_g_entities >= 1,
            "LOCATE_GAME_DATA numGEntities = {} (< 1: no worldspawn slot)",
            locate.num_g_entities
        );
        assert!(
            locate.sizeof_g_entity_t > 0,
            "LOCATE_GAME_DATA sizeofGEntity_t = {} (must be > 0)",
            locate.sizeof_g_entity_t
        );
        assert!(
            locate.sizeof_g_client > 0,
            "LOCATE_GAME_DATA sizeofGameClient = {} (must be > 0)",
            locate.sizeof_g_client
        );

        assert!(
            !m.configstrings.is_empty(),
            "GAME_INIT set no configstrings"
        );

        // The worldspawn token stream should have been fully consumed.
        assert_eq!(
            m.token_idx,
            m.tokens.len(),
            "GAME_INIT did not consume the entity token stream"
        );
    });

    // ---- warm-up frames before the client connects -----------------------
    // The engine runs a few game frames before wiring up a client (g_main.c:2949
    // "we run 3 game frames before calling Connect"). +50ms steps.
    let mut t = 600;
    let run_frame = |t: AbiWord| {
        let ret = call_vm(vm_main, MpGameExport::GAME_RUN_FRAME, &[t]);
        assert_eq!(ret, 0, "GAME_RUN_FRAME(t={t}) returned {ret}, expected 0");
        MOCK.with(|m| {
            assert!(
                m.borrow().g_error.is_none(),
                "GAME_RUN_FRAME raised G_ERROR"
            );
        });
    };
    for _ in 0..3 {
        t += 50;
        run_frame(t);
    }
    eprintln!("[smoke] 3 warm-up frames survived (level time now {t})");

    // ---- GAME_CLIENT_CONNECT(0, firstTime=qtrue, isBot=qfalse) ------------
    // Contract (g_client.c ClientConnect): returns NULL (0) to admit the client,
    // or a pointer to a rejection string on failure. Source: g_client.c:2258.
    let connect_ret = call_vm(
        vm_main,
        MpGameExport::GAME_CLIENT_CONNECT,
        &[0, 1 /*qtrue*/, 0 /*qfalse*/],
    );
    if connect_ret != 0 {
        let reason = unsafe { c_str(connect_ret) };
        panic!("GAME_CLIENT_CONNECT(0) rejected the client: {reason:?}");
    }
    MOCK.with(|m| {
        let m = m.borrow();
        assert!(m.g_error.is_none(), "GAME_CLIENT_CONNECT raised G_ERROR");
        // ClientConnect → ClientUserinfoChanged reads the client's userinfo
        // (g_client.c:2269,2347). Prove the substantive G_GET_USERINFO path ran.
        assert!(
            *m.counts.get(&G_GET_USERINFO).unwrap_or(&0) > 0,
            "GAME_CLIENT_CONNECT never read client userinfo (G_GET_USERINFO)"
        );
    });
    eprintln!("[smoke] GAME_CLIENT_CONNECT(0) admitted (returned NULL)");

    // GAME_CLIENT_USERINFO_CHANGED is NOT driven here: the engine only issues it
    // on an actual userinfo change; the connect/begin flow calls it internally
    // (ClientConnect g_client.c:2347, ClientBegin g_client.c:2437). We drive only
    // what the engine drives.

    // ---- GAME_CLIENT_BEGIN(0) --------------------------------------------
    // vmMain calls ClientBegin(arg0, qtrue) — spawns the client into the level
    // (ClientSpawn → WP_InitForcePowers etc.). Source: g_main.c:534, g_client.c:2393.
    let begin_ret = call_vm(vm_main, MpGameExport::GAME_CLIENT_BEGIN, &[0]);
    assert_eq!(begin_ret, 0, "GAME_CLIENT_BEGIN returned {begin_ret}");
    MOCK.with(|m| {
        assert!(
            m.borrow().g_error.is_none(),
            "GAME_CLIENT_BEGIN raised G_ERROR"
        );
    });
    eprintln!("[smoke] GAME_CLIENT_BEGIN(0) spawned the client");

    // ---- GAME_RUN_FRAME x10 with the client connected --------------------
    for _ in 0..10 {
        t += 50;
        run_frame(t);
    }
    eprintln!("[smoke] 10 connected frames survived (level time now {t})");

    // ---- GAME_CLIENT_COMMAND(0, "say hello") -----------------------------
    // vmMain calls ClientCommand(arg0), which reads the command via trap_Argc/
    // trap_Argv (g_main.c:537, g_cmds.c ClientCommand). The mock serves the
    // command tokens through G_ARGC/G_ARGV below.
    MOCK.with(|m| m.borrow_mut().set_cmd(&["say", "hello"]));
    let cmd_ret = call_vm(vm_main, MpGameExport::GAME_CLIENT_COMMAND, &[0]);
    assert_eq!(cmd_ret, 0, "GAME_CLIENT_COMMAND returned {cmd_ret}");
    MOCK.with(|m| {
        let mut m = m.borrow_mut();
        assert!(m.g_error.is_none(), "GAME_CLIENT_COMMAND raised G_ERROR");
        m.clear_cmd();
    });
    eprintln!("[smoke] GAME_CLIENT_COMMAND(0, \"say hello\") survived");

    // A couple more frames with the client still in.
    for _ in 0..2 {
        t += 50;
        run_frame(t);
    }

    // ---- GAME_CLIENT_DISCONNECT(0) ---------------------------------------
    // Source: g_main.c:531, g_client.c:3816.
    let disc_ret = call_vm(vm_main, MpGameExport::GAME_CLIENT_DISCONNECT, &[0]);
    assert_eq!(disc_ret, 0, "GAME_CLIENT_DISCONNECT returned {disc_ret}");
    MOCK.with(|m| {
        assert!(
            m.borrow().g_error.is_none(),
            "GAME_CLIENT_DISCONNECT raised G_ERROR"
        );
    });
    eprintln!("[smoke] GAME_CLIENT_DISCONNECT(0) clean");

    // A frame or two after the disconnect.
    for _ in 0..2 {
        t += 50;
        run_frame(t);
    }
    eprintln!("[smoke] post-disconnect frames survived (level time now {t})");

    // ---- GAME_SHUTDOWN (restart qfalse) ----------------------------------
    let sd_ret = call_vm(vm_main, MpGameExport::GAME_SHUTDOWN, &[0]);
    assert_eq!(sd_ret, 0, "GAME_SHUTDOWN returned {sd_ret}, expected 0");
    MOCK.with(|m| {
        assert!(m.borrow().g_error.is_none(), "GAME_SHUTDOWN raised G_ERROR");
    });
    eprintln!("[smoke] GAME_SHUTDOWN clean");

    // ---- coverage report -------------------------------------------------
    MOCK.with(|m| {
        let m = m.borrow();
        eprintln!("\n===== mock syscall coverage =====");
        eprintln!("implemented syscalls (name: count):");
        for (&num, &count) in &m.counts {
            if !m.logged_only.contains_key(&num) {
                eprintln!("  {:<32} {count}", import_name(num));
            }
        }
        eprintln!("logged-only (permissive default 0) syscalls (name: count):");
        for (&num, &count) in &m.logged_only {
            eprintln!("  {:<32} {count}", import_name(num));
        }
        eprintln!(
            "totals: {} distinct imports, {} prints, {} configstrings set",
            m.counts.len(),
            m.prints.len(),
            m.configstrings.len()
        );
        eprintln!("=================================\n");
    });
}

/// The stem of a platform cdylib filename — the part the loader recombines with
/// `suffix` via `format!("{name}{suffix}")`: `libjampgame.dylib` → `libjampgame`,
/// `jampgame.dll` → `jampgame`.
fn split_dylib_stem(file: &str) -> &str {
    let dot = file.find('.').expect("dylib name has an extension");
    &file[..dot]
}

/// Platform dylib suffix as a `&'static str` (required by `ModuleNaming.suffix`).
fn platform_dylib_suffix() -> &'static str {
    #[cfg(target_os = "macos")]
    {
        ".dylib"
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        ".so"
    }
    #[cfg(windows)]
    {
        ".dll"
    }
}

/// Invoke `vmMain(command, arg0..arg11)`, zero-filling the unused words.
fn call_vm(vm_main: RawVmMain, command: MpGameExport, args: &[AbiWord]) -> AbiWord {
    let mut a = [0 as AbiWord; 12];
    a[..args.len()].copy_from_slice(args);
    vm_main(
        command as c_int,
        a[0],
        a[1],
        a[2],
        a[3],
        a[4],
        a[5],
        a[6],
        a[7],
        a[8],
        a[9],
        a[10],
        a[11],
    )
}

/// Best-effort import name for the coverage report (the handled set plus the
/// syscalls GAME_INIT/frame/shutdown are known to raise); anything else prints
/// as its raw wire number.
fn import_name(n: isize) -> String {
    macro_rules! named {
        ($($v:ident),* $(,)?) => {
            $(if n == MpGameImport::$v as isize { return stringify!($v).to_string(); })*
        };
    }
    named!(
        G_PRINT,
        G_ERROR,
        G_MILLISECONDS,
        G_CVAR_REGISTER,
        G_CVAR_UPDATE,
        G_CVAR_SET,
        G_CVAR_VARIABLE_INTEGER_VALUE,
        G_CVAR_VARIABLE_STRING_BUFFER,
        G_ARGC,
        G_ARGV,
        G_FS_FOPEN_FILE,
        G_FS_READ,
        G_FS_WRITE,
        G_FS_FCLOSE_FILE,
        G_FS_GETFILELIST,
        G_GET_SERVERINFO,
        G_GET_USERINFO,
        G_SET_CONFIGSTRING,
        G_GET_CONFIGSTRING,
        G_GET_ENTITY_TOKEN,
        G_LOCATE_GAME_DATA,
        G_SET_SHARED_BUFFER,
        G_G2_CLEANENTATTACHMENTS,
        G_ICARUS_INIT,
        G_ICARUS_SHUTDOWN,
        G_NAV_LOAD,
        G_NAV_SETPATHSCALCULATED,
        G_SET_SERVER_CULL,
        G_G2_HAVEWEGHOULMODELS,
        G_ROFF_CLEAN,
        G_SEND_SERVER_COMMAND,
        G_SEND_CONSOLE_COMMAND,
        G_LINKENTITY,
        G_UNLINKENTITY,
        G_GET_USERCMD,
        G_ROFF_UPDATE_ENTITIES,
        G_ICARUS_FREEENT,
        G_ICARUS_INITENT,
        G_NAV_SAVE,
        G_NAV_CALCULATEPATHS,
        G_NAV_CLEARALLFAILEDEDGES,
        G_NAV_CHECKBLOCKEDEDGES,
        G_NAV_CLEARCHECKEDNODES,
        G_G2_INITGHOUL2MODEL,
    );
    format!("syscall#{n}")
}

// ===========================================================================
// Referee differential-driver surface (used by `tests/referee.rs`).
//
// The same MOCK, the same trampoline, the same loader — the referee simply
// drives it with a scenario, snapshots the module's playerState/entityState via
// the LOCATE_GAME_DATA pointers, and diffs. Everything here is a deterministic
// pure function of (call sequence, injected inputs). The one call-count-coupled
// syscall is `G_MILLISECONDS` (monotonic counter); it feeds profiling paths, not
// snapshot state — see the referee module doc for the determinism audit.
// ===========================================================================

/// FNV-1a over a byte slice — a stable, portable rolling hash for the
/// syscall-stream digest (no `DefaultHasher` random seed, no address content).
fn fnv1a(seed: u64, bytes: &[u8]) -> u64 {
    let mut h = seed;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;

/// Reset the mock to a fresh, empty engine (for the second referee run). The
/// caller re-applies the scenario (map tokens, userinfos) afterwards.
pub fn referee_reset() {
    MOCK.with(|m| *m.borrow_mut() = MockEngine::new());
}

/// Install the BSP entity token stream for a scenario's map variant (resets the
/// read cursor). Tokens are served verbatim by `G_GET_ENTITY_TOKEN`.
pub fn referee_set_map(tokens: &[&str]) {
    MOCK.with(|m| {
        let mut m = m.borrow_mut();
        m.tokens = tokens.iter().map(|s| CString::new(*s).unwrap()).collect();
        m.token_idx = 0;
    });
}

/// Referee swap (plan §3c): boot the real engine island for a real-map scenario
/// and arm it in the mock. Call BEFORE `referee_arm` and INSTEAD OF
/// `referee_set_map` — the entity token stream comes from the real BSP's entity
/// string (routed `G_GET_ENTITY_TOKEN`), not synthetic tokens. `map_bsp` is
/// `"maps/<map>.bsp"`; `basepath` is the assets dir (its `base/` holds the
/// pk3s). Runs on the caller's (engine) thread; must follow `referee_reset`.
pub fn referee_install_real_world(map_bsp: &str, basepath: &str) {
    let real = RealWorld::new(map_bsp, basepath);
    MOCK.with(|m| m.borrow_mut().real = Some(real));
}

/// Override an engine cvar value (served by the cvar-family syscalls). The
/// referee sets `g_synchronousClients=1` so `G_RunClient` simulates each client
/// from its latched usercmd every frame.
pub fn referee_set_cvar(name: &str, value: &str) {
    MOCK.with(|m| {
        m.borrow_mut()
            .cvars
            .insert(name.to_string(), value.to_string());
    });
}

/// Seed a client's userinfo string (served by `G_GET_USERINFO`).
pub fn referee_set_userinfo(num: c_int, s: &str) {
    MOCK.with(|m| {
        m.borrow_mut().userinfos.insert(num, s.to_string());
    });
}

/// Inject the replay usercmd for a client for the upcoming frame's
/// `GAME_CLIENT_THINK` (`G_GET_USERCMD` returns it).
/// Install the tokenized command view served by `G_ARGC`/`G_ARGV` for the
/// duration of one `GAME_CLIENT_COMMAND` dispatch (call `referee_clear_cmd`
/// after the vm call returns).
pub fn referee_set_cmd(tokens: &[&str]) {
    MOCK.with(|m| m.borrow_mut().set_cmd(tokens));
}

/// Clear the command view installed by `referee_set_cmd`.
pub fn referee_clear_cmd() {
    MOCK.with(|m| m.borrow_mut().clear_cmd());
}

pub fn referee_set_usercmd(num: c_int, cmd: usercmd_t) {
    MOCK.with(|m| {
        m.borrow_mut().usercmds.insert(num, cmd);
    });
}

/// Mark a frame boundary: clear the per-frame syscall accumulators.
pub fn referee_begin_frame() {
    MOCK.with(|m| {
        let mut m = m.borrow_mut();
        m.frame_imports.clear();
        m.frame_texts.clear();
    });
}

/// Rolling hash of the syscalls issued since the last `referee_begin_frame`:
/// the import-number sequence plus every captured string payload (by DATA). No
/// pointer value ever enters the hash, so the two runs' differing heap layouts
/// cannot perturb it.
pub fn referee_frame_syscall_digest() -> u64 {
    MOCK.with(|m| {
        let m = m.borrow();
        let mut h = FNV_OFFSET;
        for &n in &m.frame_imports {
            h = fnv1a(h, &(n as i64).to_le_bytes());
        }
        for (imp, arg, text) in &m.frame_texts {
            h = fnv1a(h, &(*imp as i64).to_le_bytes());
            h = fnv1a(h, &(*arg as i64).to_le_bytes());
            h = fnv1a(h, text.as_bytes());
            h = fnv1a(h, &[0]);
        }
        h
    })
}

/// Decoded syscall stream for the current frame (for the divergence report):
/// the import-number sequence and the string payloads.
pub fn referee_frame_syscalls() -> (Vec<isize>, Vec<(isize, c_int, String)>) {
    MOCK.with(|m| {
        let m = m.borrow();
        (m.frame_imports.clone(), m.frame_texts.clone())
    })
}

/// The captured `G_LOCATE_GAME_DATA` payload (`None` until `GAME_INIT`).
pub fn referee_locate() -> Option<LocateData> {
    MOCK.with(|m| m.borrow().locate)
}

/// The `G_ERROR` message, if the module raised one (a hard failure).
pub fn referee_error() -> Option<String> {
    MOCK.with(|m| m.borrow().g_error.clone())
}

/// Human-readable import name (re-exported for the divergence report).
pub fn referee_import_name(n: isize) -> String {
    import_name(n)
}

/// Arm the engine slot with the mock syscall (call before the first `vmMain`).
pub fn referee_arm() {
    arm_game_slot(std::ptr::null_mut(), mock_syscall);
}

/// Load a module dylib through the real ported loader and hand back the owning
/// handle (keep it alive for the whole run — dropping it unloads the module).
pub fn referee_load(dylib: &Path) -> LoadedModule {
    let dir = dylib.parent().unwrap().to_path_buf();
    let file = dylib.file_name().unwrap().to_str().unwrap().to_string();
    let name: &str = split_dylib_stem(&file);
    let policy = ModuleSearchPolicy {
        naming: ModuleNaming {
            suffix: Some(platform_dylib_suffix()),
        },
        direct_first: false,
        steps: vec![SearchStep::FsPath {
            base: dir,
            gamedir: String::new(),
        }],
    };
    let syscall: RawSyscall = game_syscall_trampoline as *const c_void;
    sys_load_dll(&policy, name, syscall).expect("sys_load_dll resolved dllEntry+vmMain")
}

/// Invoke `vmMain(command, args...)` (pub wrapper over the shared `call_vm`).
pub fn referee_vm_call(vm_main: RawVmMain, command: MpGameExport, args: &[AbiWord]) -> AbiWord {
    call_vm(vm_main, command, args)
}

/// Run `f` on a 64 MiB-stack "engine" thread (GAME_INIT builds a multi-MiB
/// `GameWorld` by value — the default test stack overflows, as in the smoke
/// drive). Panics propagate as a join failure = test failure.
pub fn run_on_engine_thread_fn(f: impl FnOnce() + Send + 'static) {
    std::thread::Builder::new()
        .name("referee-engine".into())
        .stack_size(64 * 1024 * 1024)
        .spawn(f)
        .expect("spawn engine thread")
        .join()
        .expect("referee engine thread panicked");
}
