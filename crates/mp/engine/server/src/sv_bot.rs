//! `sv_bot.cpp` — the server's botlib interface glue: debug-polygon pool
//! management, botlib shutdown, the bot console-message / snapshot-entity
//! read paths, and bot client-slot allocate/free bookkeeping.
//!
//! Source: `oracle/codemp/server/sv_bot.cpp`

#![allow(non_snake_case)]

use core::ffi::{c_char, c_int};

use mp_engine_qcommon::collision_world::CollisionWorld;
use mp_engine_qcommon::cm_load::RenderModels;
use mp_engine_qcommon::common::common::{com_printf, Common};
use mp_engine_qcommon::common::error::com_error;
use mp_engine_qcommon::cvar_fns::Cvar_VariableIntegerValue;
use mp_engine_qcommon::qcommon::net_limits::{MAX_RELIABLE_COMMANDS, PACKET_MASK};
use mp_engine_qcommon::z_memman_pc::{Z_Free, Z_Malloc};
use mp_host_interface::engine_host::EngineHost;
use mp_qshared::common::mp::botlib::botlib_import_s::botlib_import_t;
use mp_qshared::common::mp::botlib::botlib_misc::BOTLIB_API_VERSION;
use mp_qshared::common::mp::game::g_public::SVF_BOT;
use mp_qshared::common::mp::qcommon::netadrtype_t::netadrtype_t;
use mp_qshared::common::mp::qcommon::tags::memtag_t;
use mp_qshared::shared::error_parm::errorParm_t;
use mp_qshared::shared::{qfalse, qtrue};

use crate::server::bot_debugpoly_t::bot_debugpoly_t;
use crate::server::client_state_t::clientState_t;
use crate::sv_game::SV_GentityNum;
use crate::Server;
use mp_engine_botlib::be_interface_fns::GetBotLibAPI;
use mp_engine_botlib::BotLib;
use mp_qshared::shared::q_string::Q_strncpyz;

/// Raven `SV_BotLibSetup`.
///
/// Source: `oracle/codemp/server/sv_bot.cpp:595-608`
pub fn SV_BotLibSetup(common: &mut Common, sv: &mut Server, bot: &mut BotLib) -> c_int {
    if common.bot_enable == 0 {
        return 0;
    }

    if sv.botlib_export.is_null() {
        // `S_COLOR_RED` (`^1`).
        com_printf(common, "^1Error: SV_BotLibSetup without SV_BotInitBotLib\n");
        return -1;
    }

    unsafe { ((*sv.botlib_export).BotLibSetup.unwrap())(bot) }
}

/// Raven `SV_BotInitBotLib` — (re)build the botlib debug-polygon pool and hand
/// the `botlib_import_t` engine-service table to the bot library's
/// `GetBotLibAPI` entry point, storing the returned export on `Server`
/// (`sv.botlib_export`).
///
/// The `debugpolygons` pool is (re)allocated from `bot_maxdebugpolys` exactly
/// as Raven does. The `botlib_import` fn-pointer fields are the C-ABI
/// `BotImport_*` engine callbacks (`botlib.h:157-193`); each reaches threaded
/// engine state (`Com_Printf`/`SV_Trace`/`Z_Malloc`/…) that a bare
/// `extern "C" fn` cannot carry, so populating them awaits the botlib import
/// slot — the state-capture dual of the game `GAME_SLOT`. The table is
/// zero-initialised (every field `None`) until that seam lands.
///
/// Source: `oracle/codemp/server/sv_bot.cpp:684-724`
pub fn SV_BotInitBotLib(
    common: &mut Common,
    cm: &mut CollisionWorld,
    sv: &mut Server,
    bot: &mut BotLib,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
) {
    if !sv.bot.debugpolygons.is_null() {
        Z_Free(common, sv.bot.debugpolygons as *mut ());
    }
    sv.bot.bot_maxdebugpolys =
        Cvar_VariableIntegerValue(common, c"bot_maxdebugpolys".as_ptr());
    sv.bot.debugpolygons = Z_Malloc(
        common,
        cm,
        rm,
        host,
        core::mem::size_of::<bot_debugpoly_t>() as c_int * sv.bot.bot_maxdebugpolys,
        memtag_t::TAG_BOTLIB,
        qtrue,
        0,
    ) as *mut bot_debugpoly_t;

    // SAFETY: `botlib_import_t` is a `#[repr(C)]` table of `Option<fn>` plus POD
    // fields; an all-zero value is every field `None`, the valid initial state
    // Raven overwrites field-by-field (deferred to the botlib import slot here).
    let mut botlib_import: botlib_import_t = unsafe { core::mem::zeroed() };

    sv.botlib_export = GetBotLibAPI(bot, BOTLIB_API_VERSION, &mut botlib_import);
    // bk001129 - somehow we end up with a zero import.
    debug_assert!(!sv.botlib_export.is_null());
}

/// Raven `BotImport_DebugPolygonDelete`.
///
/// Source: `oracle/codemp/server/sv_bot.cpp:514-518`
pub fn BotImport_DebugPolygonDelete(sv: &mut Server, id: c_int) {
    if sv.bot.debugpolygons.is_null() {
        return;
    }
    unsafe {
        (*sv.bot.debugpolygons.offset(id as isize)).inuse = qfalse;
    }
}

/// Raven `SV_BotLibShutdown`.
///
/// Source: `oracle/codemp/server/sv_bot.cpp:619-626`
pub fn SV_BotLibShutdown(sv: &mut Server, bot: &mut BotLib) -> c_int {
    if sv.botlib_export.is_null() {
        return -1;
    }

    unsafe { ((*sv.botlib_export).BotLibShutdown.unwrap())(bot) }
}

/// Raven `SV_BotGetConsoleMessage`.
///
/// Source: `oracle/codemp/server/sv_bot.cpp:736-757`
pub fn SV_BotGetConsoleMessage(
    sv: &mut Server,
    client: c_int,
    buf: *mut c_char,
    size: c_int,
) -> c_int {
    unsafe {
        let cl = sv.svs.clients.offset(client as isize);
        (*cl).lastPacketTime = sv.svs.time;

        if (*cl).reliableAcknowledge == (*cl).reliableSequence {
            return qfalse;
        }

        (*cl).reliableAcknowledge += 1;
        let index = ((*cl).reliableAcknowledge & (MAX_RELIABLE_COMMANDS as c_int - 1)) as usize;

        if (*cl).reliableCommands[index][0] == 0 {
            return qfalse;
        }

        Q_strncpyz(buf, (*cl).reliableCommands[index].as_ptr(), size);
        qtrue
    }
}

/// Raven `SV_BotGetSnapshotEntity`.
///
/// Source: `oracle/codemp/server/sv_bot.cpp:786-796`
pub fn SV_BotGetSnapshotEntity(sv: &mut Server, client: c_int, sequence: c_int) -> c_int {
    unsafe {
        let cl = sv.svs.clients.offset(client as isize);
        let frame = &(*cl).frames[(*cl).netchan.outgoingSequence as usize & PACKET_MASK];
        if sequence < 0 || sequence >= frame.num_entities {
            return -1;
        }
        (*sv.svs
            .snapshotEntities
            .offset(((frame.first_entity + sequence) % sv.svs.numSnapshotEntities) as isize))
        .number
    }
}

/// Raven `SV_BotAllocateClient`.
///
/// Source: `oracle/codemp/server/sv_bot.cpp:178-201`
pub fn SV_BotAllocateClient(common: &mut Common, sv: &mut Server) -> c_int {
    unsafe {
        // find a client slot
        let mut i: c_int = 0;
        let mut cl = sv.svs.clients;
        while i < (*common.sv_maxclients).integer {
            if (*cl).state == clientState_t::CS_FREE {
                break;
            }
            i += 1;
            cl = cl.offset(1);
        }

        if i == (*common.sv_maxclients).integer {
            return -1;
        }

        (*cl).gentity = SV_GentityNum(sv, i);
        (*(*cl).gentity).s.number = i;
        (*cl).state = clientState_t::CS_ACTIVE;
        (*cl).lastPacketTime = sv.svs.time;
        (*cl).netchan.remoteAddress.r#type = netadrtype_t::NA_BOT;
        (*cl).rate = 16384;

        i
    }
}

/// Raven `SV_BotFreeClient`.
///
/// Source: `oracle/codemp/server/sv_bot.cpp:208-221`
pub fn SV_BotFreeClient(common: &mut Common, sv: &mut Server, clientNum: c_int) {
    unsafe {
        if clientNum < 0 || clientNum >= (*common.sv_maxclients).integer {
            com_error(
                errorParm_t::ERR_DROP,
                format!("SV_BotFreeClient: bad clientNum: {}", clientNum),
            );
        }
        let cl = sv.svs.clients.offset(clientNum as isize);
        (*cl).state = clientState_t::CS_FREE;
        (*cl).name[0] = 0;
        if !(*cl).gentity.is_null() {
            (*(*cl).gentity).r.svFlags &= !SVF_BOT;
        }
    }
}
