//! `Server` (the `Engine.sv` island host) + `ServerGame` (the game dispatcher's
//! reborrowed host state) + `sv_game_system_calls` (the MP game dispatcher).

use core::ffi::c_int;

use mp_qshared::shared::wpobject_t;

use crate::server::bot_debugpoly_t::bot_debugpoly_t;
use crate::server::server_static_t::serverStatic_t;
use crate::server::server_t::server_t;
use crate::server::world_sector_s::{worldSector_t, AREA_NODES};

/// Raven `MAX_WPARRAY_SIZE` — bot waypoint pointer-table capacity
/// (`gWPArray[MAX_WPARRAY_SIZE]`).
///
/// Type definition source: `oracle/oracle/codemp/game/q_shared.h:993`
pub const MAX_WPARRAY_SIZE: usize = 4096;

/// Raven `sv_bot.cpp` file-scope globals: `debugpolygons`/`bot_maxdebugpolys`
/// (the bot-debug-polygon pool) + `gWPArray`/`gWPNum` (the bot waypoint
/// table), grouped under Raven names (WorldSectors precedent: a Rust-side
/// grouping colocated with its sole owner).
///
/// Source: `oracle/oracle/codemp/server/sv_bot.cpp:16-23`
#[allow(non_snake_case)]
pub struct Bot {
    /// Raven `debugpolygons` — heap array of `bot_debugpoly_t`, allocated by
    /// `SV_BotInitBotLib` (`Z_Malloc(sizeof(bot_debugpoly_t) *
    /// bot_maxdebugpolys, ...)`); null until then.
    ///
    /// Source: `oracle/oracle/codemp/server/sv_bot.cpp:16,689`
    pub debugpolygons: *mut bot_debugpoly_t,
    /// Raven `bot_maxdebugpolys`.
    ///
    /// Source: `oracle/oracle/codemp/server/sv_bot.cpp:17`
    pub bot_maxdebugpolys: c_int,
    /// Raven `gWPArray[MAX_WPARRAY_SIZE]` — bot waypoint pointer table.
    ///
    /// Source: `oracle/oracle/codemp/server/sv_bot.cpp:23`
    pub gWPArray: [*mut wpobject_t; MAX_WPARRAY_SIZE],
    /// Raven `gWPNum`.
    ///
    /// Source: `oracle/oracle/codemp/server/sv_bot.cpp:22`
    pub gWPNum: c_int,
}

/// Raven `sv_worldSectors[AREA_NODES]` + `sv_numworldSectors` — the master
/// table's `Server.world_sectors` row, grouped under Raven names (Savegame
/// precedent: a Rust-side grouping colocated with its sole owner).
///
/// Source: `oracle/oracle/codemp/server/sv_world.cpp:58-59`
#[allow(non_snake_case)]
pub struct WorldSectors {
    pub sv_worldSectors: [worldSector_t; AREA_NODES],
    pub sv_numworldSectors: c_int,
}

/// Raven `MAX_MASTER_SERVERS` — master-server slot count (`#ifndef _XBOX`).
///
/// Type definition source: `oracle/oracle/codemp/server/server.h:236`
pub const MAX_MASTER_SERVERS: usize = 5;

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
/// Source: `oracle/oracle/codemp/server/sv_main.cpp:10-11`;
/// `oracle/oracle/codemp/server/server.h:46-54` (state/`SS_DEAD`).
pub struct Server {
    /// Raven `sv` (`server_t`) — embeds `svEntities`/`configstrings`/`models`
    /// and holds the `SharedGameData` registration.
    ///
    /// Source: `oracle/oracle/codemp/server/sv_main.cpp:11`
    pub sv: server_t,
    /// Raven `svs` (`serverStatic_t`, persists across maps) — challenges, heap
    /// `clients[]`, snapshot ring.
    ///
    /// Source: `oracle/oracle/codemp/server/sv_main.cpp:10`
    pub svs: serverStatic_t,
    /// Raven `sv_worldSectors[AREA_NODES]` + `sv_numworldSectors` — the
    /// Chain-A disjoint field.
    ///
    /// Source: `oracle/oracle/codemp/server/sv_world.cpp:58-59`
    pub world_sectors: WorldSectors,
    /// Raven `debugpolygons`/`bot_maxdebugpolys`/`gWPArray`/`gWPNum` —
    /// `sv_bot.cpp`'s file-scope bot state.
    ///
    /// Source: `oracle/oracle/codemp/server/sv_bot.cpp:16-23`
    pub bot: Bot,
    /// Raven `g_lastResolveTime[MAX_MASTER_SERVERS]` — master-server
    /// DNS-resolve throttle timestamps (`#ifndef _XBOX`, MP only).
    ///
    /// Source: `oracle/oracle/codemp/server/sv_main.cpp:192`
    pub master_heartbeat: [c_int; MAX_MASTER_SERVERS],
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
/// Source: `oracle/oracle/codemp/server/sv_game.cpp:458`
pub fn sv_game_system_calls(engine: &mut ServerGame, args: &[isize]) -> isize {
    use mp_abi::game::imports::MpGameImport;
    let _ = engine;

    // Slice-0 minimal dispatch: only the GAME_INIT-era traps the minimal
    // module emits. The full exhaustive TryFrom<i32>-decoded match (SEAM-D3)
    // and the ERR_DROP bad-number fallback (sv_game.cpp:1654) land with it:
    //TODO: Port SV_GameSystemCalls exhaustive dispatch
    // Source: oracle/oracle/codemp/server/sv_game.cpp:458-1654
    let trap = args[0] as i32;
    if trap == MpGameImport::G_PRINT as i32 {
        // `case G_PRINT: Com_Printf( "%s", VMA(1) );` (sv_game.cpp:503-505;
        // VMA is a native-DLL identity cast, vm.cpp:648-649). Slice-0 sink is
        // the com_printf minimal console write; routing through
        // `&mut Common` lands with the ServerGame reborrow wiring.
        let msg = unsafe { core::ffi::CStr::from_ptr(args[1] as *const core::ffi::c_char) };
        print!("{}", msg.to_string_lossy());
        use std::io::Write as _;
        let _ = std::io::stdout().flush();
        return 0;
    }
    todo!("Port SV_GameSystemCalls trap {trap} — oracle/oracle/codemp/server/sv_game.cpp:458")
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
/// Source: `oracle/oracle/codemp/qcommon/vm.cpp:377` (`currentVM->systemCall( args )`).
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

    // Null-ctx fallback (Slice-0): sv_init_game_progs has not yet wired the
    // `&mut Engine.sv` injection, so no `&mut ServerGame` exists to dispatch
    // against — G_PRINT needs no host state and is handled inline; anything
    // else fails loudly rather than reading fake state.
    //TODO: Port SV_InitGameProgs ctx injection (&mut Engine.sv)
    // Source: docs/architecture/engine-seam.md § Engine-side dispatchers;
    // oracle/oracle/codemp/server/sv_game.cpp:1734-1753
    use mp_abi::game::imports::MpGameImport;
    let trap = frame[0] as i32;
    if trap == MpGameImport::G_PRINT as i32 {
        // `case G_PRINT: Com_Printf( "%s", VMA(1) );` (sv_game.cpp:503-505;
        // VMA is a native-DLL identity cast, vm.cpp:648-649). Slice-0 sink is
        // the com_printf minimal console write; routing through
        // `&mut Common` lands with the ctx injection wiring.
        let msg = unsafe { core::ffi::CStr::from_ptr(frame[1] as *const core::ffi::c_char) };
        print!("{}", msg.to_string_lossy());
        use std::io::Write as _;
        let _ = std::io::stdout().flush();
        return 0;
    }
    todo!(
        "Port SV_InitGameProgs ctx injection (&mut Engine.sv) — null ctx, trap {trap} needs host state — oracle/oracle/codemp/server/sv_game.cpp:1734-1753"
    )
}
