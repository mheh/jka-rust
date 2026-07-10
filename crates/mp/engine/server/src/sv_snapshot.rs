//! `sv_snapshot.cpp` — server snapshot building/sending.

use mp_engine_qcommon::collision_world::CollisionWorld;
use mp_engine_qcommon::common::common::Common;
use mp_engine_renderer::RenderModels;
use mp_host_interface::engine_host::EngineHost;
use mp_qshared::common::mp::qcommon::shared_entity_t::sharedEntity_t;

use crate::server::snapshot_entity_numbers_t::{snapshotEntityNumbers_t, MAX_SNAPSHOT_ENTITIES};
use crate::server::sv_entity_s::svEntity_t;
use crate::sv_net_chan::{SV_Netchan_TransmitNextFragment, SV_RateMsec};
use crate::Server;

/// Raven `SV_AddEntToSnapshot`.
///
/// Source: `oracle/codemp/server/sv_snapshot.cpp:279-293`
pub fn SV_AddEntToSnapshot(
    sv: &mut Server,
    svEnt: *mut svEntity_t,
    gEnt: *mut sharedEntity_t,
    eNums: *mut snapshotEntityNumbers_t,
) {
    unsafe {
        // if we have already added this entity to this snapshot, don't add again
        if (*svEnt).snapshotCounter == sv.sv.snapshotCounter {
            return;
        }
        (*svEnt).snapshotCounter = sv.sv.snapshotCounter;

        // if we are full, silently discard entities
        if (*eNums).numSnapshotEntities as usize == MAX_SNAPSHOT_ENTITIES {
            return;
        }

        (*eNums).snapshotEntities[(*eNums).numSnapshotEntities as usize] = (*gEnt).s.number;
        (*eNums).numSnapshotEntities += 1;
    }
}

/// Raven `SV_SendClientMessages`.
///
/// Source: `oracle/codemp/server/sv_snapshot.cpp:806-832`
pub fn SV_SendClientMessages(
    common: &mut Common,
    cm: &mut CollisionWorld,
    sv: &mut Server,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
) {
    // send a message to each connected client
    let max_clients = mp_engine_qcommon::cvar::sv_maxclients(common).integer;
    for i in 0..max_clients {
        let c = unsafe { sv.svs.clients.offset(i as isize) };
        unsafe {
            if (*c).state == 0 {
                continue; // not connected
            }

            if sv.svs.time < (*c).nextSnapshotTime {
                continue; // not time yet
            }

            // send additional message fragments if the last message
            // was too large to send at once
            if (*c).netchan.unsentFragments != 0 {
                (*c).nextSnapshotTime = sv.svs.time
                    + SV_RateMsec(
                        common,
                        cm,
                        sv,
                        rm,
                        host,
                        c,
                        (*c).netchan.unsentLength - (*c).netchan.unsentFragmentStart,
                    );
                SV_Netchan_TransmitNextFragment(common, cm, rm, host, &mut (*c).netchan);
                continue;
            }

            // generate and send a new message
            SV_SendClientSnapshot(common, cm, sv, rm, host, c);
        }
    }
}
