//! `sv_bot.cpp` — the server's botlib interface glue: debug-polygon pool
//! management, botlib shutdown, and the bot console-message / snapshot-entity
//! read paths.
//!
//! Source: `oracle/codemp/server/sv_bot.cpp`

#![allow(non_snake_case)]

use core::ffi::{c_char, c_int};

use mp_engine_qcommon::qcommon::net_limits::{MAX_RELIABLE_COMMANDS, PACKET_MASK};
use mp_qshared::shared::{qfalse, qtrue};

use crate::Server;
use mp_qshared::shared::q_string::{Q_strncpyz};

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
pub fn SV_BotLibShutdown(sv: &mut Server) -> c_int {
    if sv.botlib_export.is_null() {
        return -1;
    }

    unsafe { ((*sv.botlib_export).BotLibShutdown.unwrap())() }
}

/// Raven `SV_BotGetConsoleMessage`.
///
/// Source: `oracle/codemp/server/sv_bot.cpp:736-757`
pub fn SV_BotGetConsoleMessage(sv: &mut Server, client: c_int, buf: *mut c_char, size: c_int) -> c_int {
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
        (*sv
            .svs
            .snapshotEntities
            .offset(((frame.first_entity + sequence) % sv.svs.numSnapshotEntities) as isize))
        .number
    }
}
