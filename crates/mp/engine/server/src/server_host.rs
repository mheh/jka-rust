//! `Server` (the `Engine.sv` island host) + `ServerGame` (the game dispatcher's
//! reborrowed host state) + `sv_game_system_calls` (the MP game dispatcher).

use core::ffi::{c_char, c_int};
use mp_abi::game::imports::MpGameImport;
use std::io::Write;

use mp_engine_qcommon::vm::game_syscall_trampoline_words;
use mp_qshared::shared::error_parm::errorParm_t;

use mp_engine_botlib::be_interface::botlib_export_s::botlib_export_t;
use mp_qshared::common::mp::qcommon::netadr_t::netadr_t;
use mp_qshared::common::mp::qcommon::shared_entity_t::sharedEntity_t;
use mp_qshared::common::mp::qcommon::siege_pers::siegePers_t;
use mp_qshared::shared::limits::MAX_WPARRAY_SIZE as MAX_WPARRAY_SIZE_I32;
use mp_qshared::shared::wpobject_t;

use mp_engine_qcommon::cm_load::RenderModels as CmRenderModelsSlot;
use mp_engine_qcommon::cm_load::RmManager as CmRmManagerSlot;
use mp_engine_qcommon::cmd_pc::Server as CmdServerSlot;
use mp_engine_qcommon::common::opaque_slots::Ghoul2System as CmdGhoul2Slot;
use mp_engine_qcommon::vm::vm_s::vm_t;

use mp_engine_ghoul2::ghoul2_system::Ghoul2System;
use mp_engine_renderer::tr_model::render_models::RenderModels as RealRenderModels;
use mp_engine_rmg::rm_manager::RmManager as RealRmManager;

use crate::server::bot_debugpoly_t::bot_debugpoly_t;
use crate::server::server_static_t::serverStatic_t;
use crate::server::server_t::server_t;
use crate::server::world_sector_s::{worldSector_t, AREA_NODES};

/// Raven `MAX_WPARRAY_SIZE` — bot waypoint pointer-table capacity
/// (`gWPArray[MAX_WPARRAY_SIZE]`). `usize`-typed dual of the canonical
/// `mp_qshared::shared::limits::MAX_WPARRAY_SIZE` (`c_int`), for array sizing.
///
/// Type definition source: `oracle/codemp/game/q_shared.h:993`
pub const MAX_WPARRAY_SIZE: usize = MAX_WPARRAY_SIZE_I32 as usize;

/// Raven `sv_bot.cpp` file-scope globals: `debugpolygons`/`bot_maxdebugpolys`
/// (the bot-debug-polygon pool) + `gWPArray`/`gWPNum` (the bot waypoint
/// table), grouped under Raven names (WorldSectors precedent: a Rust-side
/// grouping colocated with its sole owner).
///
/// Source: `oracle/codemp/server/sv_bot.cpp:16-23`
#[allow(non_snake_case)]
pub struct Bot {
    /// Raven `debugpolygons` — heap array of `bot_debugpoly_t`, allocated by
    /// `SV_BotInitBotLib` (`Z_Malloc(sizeof(bot_debugpoly_t) *
    /// bot_maxdebugpolys, ...)`); null until then.
    ///
    /// Source: `oracle/codemp/server/sv_bot.cpp:16,689`
    pub debugpolygons: *mut bot_debugpoly_t,
    /// Raven `bot_maxdebugpolys`.
    ///
    /// Source: `oracle/codemp/server/sv_bot.cpp:17`
    pub bot_maxdebugpolys: c_int,
    /// Raven `gWPArray[MAX_WPARRAY_SIZE]` — bot waypoint pointer table.
    ///
    /// Source: `oracle/codemp/server/sv_bot.cpp:23`
    pub gWPArray: [*mut wpobject_t; MAX_WPARRAY_SIZE],
    /// Raven `gWPNum`.
    ///
    /// Source: `oracle/codemp/server/sv_bot.cpp:22`
    pub gWPNum: c_int,
}

/// Raven `sv_worldSectors[AREA_NODES]` + `sv_numworldSectors` — the master
/// table's `Server.world_sectors` row, grouped under Raven names (Savegame
/// precedent: a Rust-side grouping colocated with its sole owner).
///
/// Source: `oracle/codemp/server/sv_world.cpp:58-59`
#[allow(non_snake_case)]
pub struct WorldSectors {
    pub sv_worldSectors: [worldSector_t; AREA_NODES],
    pub sv_numworldSectors: c_int,
}

/// Raven `MAX_MASTER_SERVERS` — master-server slot count (`#ifndef _XBOX`).
///
/// Type definition source: `oracle/codemp/server/server.h:236`
pub const MAX_MASTER_SERVERS: usize = 5;

/// Raven `NEW_RESOLVE_DURATION` — master-address re-resolve interval, 24 hours
/// in milliseconds (`#ifndef _XBOX`).
///
/// Source: `oracle/codemp/server/sv_main.cpp:191`
pub const NEW_RESOLVE_DURATION: c_int = 86400000;

/// Raven `HEARTBEAT_MSEC` — interval between master-server heartbeats.
///
/// Source: `oracle/codemp/server/sv_main.cpp:220`
pub const HEARTBEAT_MSEC: c_int = 300 * 1000;

/// Raven `HEARTBEAT_GAME` — game identifier string sent in heartbeats.
///
/// Source: `oracle/codemp/server/sv_main.cpp:221`
pub const HEARTBEAT_GAME: &str = "QuakeArena-1";

/// Raven `SV_OUTPUTBUF_LENGTH` — `SVC_RemoteCommand` reply buffer size.
///
/// Source: `oracle/codemp/server/sv_main.cpp:494`
pub const SV_OUTPUTBUF_LENGTH: usize = mp_engine_qcommon::qcommon::net_limits::MAX_MSGLEN - 16;

/// The server-island state owned by `Engine.sv: Server` — always present, NOT
/// an `Option` (LIFE-Q7 resolution, round-6): liveness is `sv.state == SS_DEAD`
/// (`serverState_t`, `SS_DEAD` = "no map loaded", `codemp/server/server.h:46-54`),
/// the direct dual of Raven's loader-zero-filled `sv`/`svs` statics. Reuses the
/// existing ported `server_t`/`serverStatic_t` types as fields
/// (`server/server_t.rs`, `server/server_static_t.rs`); `sv.state` (embedded in
/// `server_t`) IS the liveness flag — no field here is ever `Option`. MP only:
/// the master table's SP `savegame` row (`qbLoadTransition`/
/// `eSavedGameJustLoaded`, `code/server/sv_ccmds.cpp:22`) has no MP counterpart
/// and is deliberately not a field of this (MP) `Server`.
///
/// Source: `oracle/codemp/server/sv_main.cpp:10-11`;
/// `oracle/codemp/server/server.h:46-54` (state/`SS_DEAD`).
pub struct Server {
    /// Raven `sv` (`server_t`) — embeds `svEntities`/`configstrings`/`models`
    /// and holds the `SharedGameData` registration.
    ///
    /// Source: `oracle/codemp/server/sv_main.cpp:11`
    pub sv: server_t,
    /// Raven `svs` (`serverStatic_t`, persists across maps) — challenges, heap
    /// `clients[]`, snapshot ring.
    ///
    /// Source: `oracle/codemp/server/sv_main.cpp:10`
    pub svs: serverStatic_t,
    /// Raven `sv_worldSectors[AREA_NODES]` + `sv_numworldSectors` — the
    /// Chain-A disjoint field.
    ///
    /// Source: `oracle/codemp/server/sv_world.cpp:58-59`
    pub world_sectors: WorldSectors,
    /// Raven `debugpolygons`/`bot_maxdebugpolys`/`gWPArray`/`gWPNum` —
    /// `sv_bot.cpp`'s file-scope bot state.
    ///
    /// Source: `oracle/codemp/server/sv_bot.cpp:16-23`
    pub bot: Bot,
    /// Raven `g_lastResolveTime[MAX_MASTER_SERVERS]` — master-server
    /// DNS-resolve throttle timestamps (`#ifndef _XBOX`, MP only).
    ///
    /// Source: `oracle/codemp/server/sv_main.cpp:192`
    pub master_heartbeat: [c_int; MAX_MASTER_SERVERS],
    /// Raven `SV_MasterHeartbeat::adr[MAX_MASTER_SERVERS]` (`static netadr_t`) —
    /// the resolved master-server addresses cached across heartbeats so DNS is
    /// only re-hit when a name changes or the re-resolve interval elapses.
    ///
    /// Source: `oracle/codemp/server/sv_main.cpp:213`
    pub master_adr: [netadr_t; MAX_MASTER_SERVERS],
    /// Raven `gLocalModifier` (`sv_game.cpp` file-scope static) — the
    /// `ConvertedEntity` shifted-pointer scratch buffer.
    ///
    /// Source: `oracle/codemp/server/sv_game.cpp:420`
    pub g_local_modifier: sharedEntity_t,
    /// Raven `g_svCullDist` (`sv_snapshot.cpp` file-scope global) — per-entity
    /// snapshot cull-distance override, `-1.0f` (disabled) unless set by the
    /// `G_SET_SNAPSHOT_CALLBACK`-family trap.
    ///
    /// Source: `oracle/codemp/server/sv_snapshot.cpp:300`
    pub g_svCullDist: f32,
    /// Raven `gvm` — the game virtual machine.
    ///
    /// Source: `oracle/codemp/server/server.h:234`
    pub gvm: *mut vm_t,
    /// Raven `sv_siegePersData` (`sv_game.cpp` file-scope static) — siege
    /// persistent-data mirror for `G_GET_SIEGE_PERS_DATA`/`G_SET_SIEGE_PERS_DATA`.
    ///
    /// Source: `oracle/codemp/server/sv_game.cpp:454`
    pub sv_siegePersData: siegePers_t,
    /// Raven `botlib_export` (`extern botlib_export_t *`, defined by the botlib
    /// interface `be_interface.cpp`, referenced from `sv_bot.cpp`). Null until
    /// `SV_BotInitBotLib` assigns it.
    // Mirrored here per the packet's `sv` threading; real owner is `Engine.bot` (ruling 43).
    /// Source: `oracle/codemp/server/sv_bot.cpp:19`
    pub botlib_export: *mut botlib_export_t,
    /// Raven `SV_ExpandNewlines::string` (`static char string[1024]`) — the
    /// newline-expansion return buffer; a function-scope static whose pointer is
    /// returned to the caller, so it lives as cross-call `Server` state (fork-3
    /// kind 3).
    ///
    /// Source: `oracle/codemp/server/sv_main.cpp:60`
    pub sv_expand_newlines_string: [c_char; 1024],
}

/// Wrap a live `&mut Server` into qcommon's type-erased command slot for
/// passing INTO qcommon's registration/dispatch seam
/// (`Cmd_AddCommand`/`Cmd_ExecuteString`/`Cbuf_ExecuteText`). qcommon never
/// dereferences the slot — it only threads it back to our command handlers,
/// where `server_from_slot` casts it back. Opaque-slot ruling (user,
/// 2026-07-12, option A).
pub fn server_slot(sv: &mut Server) -> CmdServerSlot {
    CmdServerSlot::from_raw(sv as *mut Server as *mut ())
}

/// Cast a qcommon command slot back into the live `&mut Server`, inside a
/// registered-command or hook handler body.
///
/// SAFETY: every slot reaching a handler was constructed by `server_slot` from
/// a live, unique `&mut Server` that outlives the entire dispatch call (the
/// engine is single-threaded and holds no other borrow of that `Server` across
/// the seam), so the erased pointer is non-null, well-aligned, and uniquely
/// borrowable for the returned reference's lifetime.
pub unsafe fn server_from_slot(slot: &mut CmdServerSlot) -> &mut Server {
    &mut *(slot.as_raw() as *mut Server)
}

/// Wrap a live `&mut Ghoul2System` into qcommon's type-erased command slot for
/// passing INTO qcommon's registration/dispatch seam (`Cbuf_ExecuteText`, the
/// `CmdFunction` receiver chain). qcommon never dereferences the slot — it only
/// threads it back to our command handlers, where `ghoul2_from_slot` casts it
/// back. Opaque-slot ruling (user, 2026-07-12, option A).
pub fn ghoul2_slot(g2: &mut Ghoul2System) -> CmdGhoul2Slot {
    CmdGhoul2Slot::from_raw(g2 as *mut Ghoul2System as *mut ())
}

/// Cast a qcommon command slot back into the live `&mut Ghoul2System`, inside a
/// registered-command or hook handler body.
///
/// SAFETY: every slot reaching a handler was constructed by `ghoul2_slot` from
/// a live, unique `&mut Ghoul2System` that outlives the entire dispatch call
/// (the engine is single-threaded and holds no other borrow of that
/// `Ghoul2System` across the seam), so the erased pointer is non-null,
/// well-aligned, and uniquely borrowable for the returned reference's lifetime.
pub unsafe fn ghoul2_from_slot(slot: &mut CmdGhoul2Slot) -> &mut Ghoul2System {
    &mut *(slot.as_raw() as *mut Ghoul2System)
}

/// Wrap a live `&mut RmManager` (the real `mp_engine_rmg` state, owned by
/// `Engine.rmg`) into qcommon's type-erased `cm_load::RmManager` slot for
/// passing INTO the cm_load/server threading. qcommon never dereferences the
/// slot — it only threads it back to server handlers, where `rmg_from_slot`
/// casts it back. Opaque-slot ruling (user, 2026-07-12, option A).
pub fn rmg_slot(rmg: &mut RealRmManager) -> CmRmManagerSlot {
    CmRmManagerSlot::from_raw(rmg as *mut RealRmManager as *mut ())
}

/// Cast a qcommon `cm_load::RmManager` slot back into the live real
/// `mp_engine_rmg::RmManager`, inside a server handler body.
///
/// SAFETY: every slot reaching a server handler was constructed by `rmg_slot`
/// from a live, unique `&mut RmManager` that outlives the entire dispatch call
/// (the engine is single-threaded and holds no other borrow of that `RmManager`
/// across the seam), so the erased pointer is non-null, well-aligned, and
/// uniquely borrowable for the returned reference's lifetime.
pub unsafe fn rmg_from_slot(slot: &mut CmRmManagerSlot) -> &mut RealRmManager {
    &mut *(slot.as_raw() as *mut RealRmManager)
}

/// Wrap a live `&mut RenderModels` (the real `mp_renderer` model registry, owned
/// by `Engine.render_models`, `tr-model.md`) into qcommon's type-erased
/// `cm_load::RenderModels` slot for passing INTO the cm_load/server threading.
/// qcommon never dereferences the slot — it only threads it back to server
/// handlers, where `rm_from_slot` casts it back. Opaque-slot ruling (user,
/// 2026-07-12, option A).
pub fn rm_slot(rm: &mut RealRenderModels) -> CmRenderModelsSlot {
    CmRenderModelsSlot::from_raw(rm as *mut RealRenderModels as *mut ())
}

/// Cast a qcommon `cm_load::RenderModels` slot back into the live real
/// `mp_renderer::tr_model::render_models::RenderModels`, inside a server handler
/// body (the model-registry entry points in `sv_renderer.rs`).
///
/// SAFETY: every slot reaching a server handler was constructed by `rm_slot`
/// from a live, unique `&mut RenderModels` that outlives the entire dispatch
/// call (the engine is single-threaded and holds no other borrow of that
/// `RenderModels` across the seam), so the erased pointer is non-null,
/// well-aligned, and uniquely borrowable for the returned reference's lifetime.
pub unsafe fn rm_from_slot(slot: &mut CmRenderModelsSlot) -> &mut RealRenderModels {
    &mut *(slot.as_raw() as *mut RealRenderModels)
}

/// engine-seam's name for the game dispatcher's `&mut ServerGame` argument — the
/// server-island reborrow (`&mut Engine.sv`'s `Server`) carrying its
/// `SharedGameData` registration. `ServerGame` and `Server` denote the same
/// value (engine-seam.md § Engine-side dispatchers).
///
/// Concrete shape (STATE-Q7 residual, user ruling 2026-07-05): a plain alias,
/// not a wrapper — `ServerGame` IS `Server` reborrowed, nothing more. Residual
/// CLOSED.
///
/// Source: `docs/architecture/engine-seam.md` § Engine-side dispatchers;
/// `docs/architecture/state-ownership.md` § Seam definition (`ServerGame`
/// amendment, 2026-07-05).
pub type ServerGame = Server;

/// The MP game outbound dispatcher — our `SV_GameSystemCalls` equivalent
/// (SEAM-D3). A hand-written exhaustive `match` over `MpGameImport`; `args[0]` =
/// syscall number decoded via `TryFrom<i32>`; return is the C `intptr_t` word.
/// An unknown trap number reproduces Raven's `Com_Error(ERR_DROP, "Bad game
/// system trap: %i")` faithfully (`sv_game.cpp:1654`).
///
/// Source: `oracle/codemp/server/sv_game.cpp:458`
pub fn sv_game_system_calls(engine: &mut ServerGame, args: &[isize]) -> isize {
    let _ = engine;

    // Minimal ctx-less dispatch: only the traps this SEAM-D3 shim decodes
    // without the full receiver set (`G_PRINT`). The exhaustive receiver-rich
    // dispatcher is `sv_game.rs::SV_GameSystemCalls`; wiring it as the slot
    // target awaits the full engine-state capture. Any other trap falls through
    // to Raven's own default (`Com_Error(ERR_DROP, "Bad game system trap: %i")`,
    // sv_game.cpp:1654).
    let trap = args[0] as i32;
    if trap == MpGameImport::G_PRINT as i32 {
        // `case G_PRINT: Com_Printf( "%s", VMA(1) );` (sv_game.cpp:503-505;
        // VMA is a native-DLL identity cast, vm.cpp:648-649).
        let msg = unsafe { core::ffi::CStr::from_ptr(args[1] as *const core::ffi::c_char) };
        print!("{}", msg.to_string_lossy());
        let _ = std::io::stdout().flush();
        return 0;
    }
    mp_engine_qcommon::common::com_error(
        errorParm_t::ERR_DROP,
        format!("Bad game system trap: {trap}"),
    )
}

/// The injected `SlotSyscall` target (LOAD-D8 injection): unpacks the
/// trampoline's 16-word frame and enters the typed dispatcher above — the
/// inbound dual of `CEngine::raw_syscall_words`'s frame.
///
/// `ServerGame`'s concrete shape is now pinned (`type ServerGame = Server`,
/// STATE-Q7 residual CLOSED, user ruling 2026-07-05), so a non-null `ctx` is
/// reborrowed as `&mut ServerGame` and enters the typed dispatcher — the real
/// path, ready for when `sv_init_game_progs`
/// (`crates/mp/engine/core/src/sv_init_game_progs.rs`) stops injecting
/// `core::ptr::null_mut()`. On the null path NO reference is created and no
/// stand-in `Server` is fabricated (a silent fake, porting-rules #14): the
/// Slice-0 G_PRINT case — which needs no host state — is handled inline, and
/// every other trap panics loudly.
///
/// Source: `oracle/codemp/qcommon/vm.cpp:377` (`currentVM->systemCall( args )`).
pub extern "C-unwind" fn game_system_calls_shim(
    ctx: *mut core::ffi::c_void,
    args: *const isize,
) -> isize {
    // SAFETY: the trampoline shim always forwards its full 16-word frame.
    let frame = unsafe { core::slice::from_raw_parts(args, 16) };

    if !ctx.is_null() {
        // SAFETY: the LOAD-D8 injection passes the `&mut Engine.sv` reborrow
        // (`ServerGame` = `Server`, engine-seam § Engine-side dispatchers) as
        // `ctx`; module dispatch is single-threaded, so the exclusive reborrow
        // holds for the duration of this call.
        let server_game = unsafe { &mut *(ctx as *mut ServerGame) };
        return sv_game_system_calls(server_game, frame);
    }

    // Null-ctx path: a slot armed without a `&mut ServerGame` (the core-crate
    // Slice-0 loader arms `ctx = null`). G_PRINT needs no host state; every
    // other trap falls through to Raven's own default rather than reading fake
    // state (`Com_Error(ERR_DROP, "Bad game system trap: %i")`,
    // sv_game.cpp:1654).
    let trap = frame[0] as i32;
    if trap == MpGameImport::G_PRINT as i32 {
        // `case G_PRINT: Com_Printf( "%s", VMA(1) );` (sv_game.cpp:503-505;
        // VMA is a native-DLL identity cast, vm.cpp:648-649).
        let msg = unsafe { core::ffi::CStr::from_ptr(frame[1] as *const core::ffi::c_char) };
        print!("{}", msg.to_string_lossy());
        let _ = std::io::stdout().flush();
        return 0;
    }
    mp_engine_qcommon::common::com_error(
        errorParm_t::ERR_DROP,
        format!("Bad game system trap: {trap}"),
    )
}

/// The `int (*)(int*)` C-ABI adapter handed to `VM_Create` as `systemCalls`
/// (`vm.cpp:471-472`, stored `vm->systemCall`). On the SEAM-D11 native path the
/// module reaches the engine through `game_syscall_trampoline` → the armed
/// `GAME_SLOT`, so `vm->systemCall` (the legacy `VM_DllSyscall` target,
/// `vm.cpp:363-380`) is vestigial; this adapter widens the legacy contiguous
/// int arg block to the trampoline's `isize` words and forwards to the same
/// armed slot for parity if ever invoked.
pub extern "C" fn sv_game_system_call(args: *mut c_int) -> c_int {
    // SAFETY: the legacy `VM_DllSyscall` convention passes a contiguous 16-int
    // arg block (`args[i] = va_arg(...)`, vm.cpp:366); widen it to the
    // trampoline frame and dispatch through the armed game slot.
    unsafe {
        let mut frame = [0isize; 16];
        for (i, w) in frame.iter_mut().enumerate() {
            *w = *args.add(i) as isize;
        }
        game_syscall_trampoline_words(frame.as_ptr()) as c_int
    }
}
