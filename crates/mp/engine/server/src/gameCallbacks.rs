//! `gameCallbacks.cpp` — NAV/nav-mesh callbacks the engine exposes to the game
//! VM via `VM_Call`. Each fn here forwards to the loaded game module and
//! returns its result, translating the entity pointer to `s.number` where the
//! callee expects an entity index.
//!
//! Source: `oracle/codemp/server/NPCNav/gameCallbacks.cpp`

use std::os::raw::c_int;

use mp_abi::game::exports::MpGameExport;
use mp_qshared::common::mp::qcommon::shared_entity_t::sharedEntity_t;
use mp_qshared::shared::qboolean;
use native_math::vector::vec3_t;

use mp_engine_qcommon::common::common::Common;

use crate::server_host::Server;

/// Raven `GNavCallback_NAV_ClearPathToPoint`.
///
/// Source: `oracle/codemp/server/NPCNav/gameCallbacks.cpp:6-9`
pub fn GNavCallback_NAV_ClearPathToPoint(
    common: &mut Common,
    sv: &mut Server,
    self_: *mut sharedEntity_t,
    pmins: vec3_t,
    pmaxs: vec3_t,
    point: vec3_t,
    clipmask: c_int,
    okToHitEntNum: c_int,
) -> qboolean {
    // PORT-NOTE(vm_call_vec3): `pmins`/`pmaxs`/`point` are `vec3_t` (by-value
    // arrays in the Raven signature); the established `VM_Call(.., &[isize])`
    // call convention (see sv_init.rs/sv_ccmds.rs) only carries scalar args,
    // so vec3_t args are passed by pointer here pending VM_Call's real body.
    qboolean::from(mp_engine_qcommon::vm::VM_Call(
        common,
        sv.gvm,
        MpGameExport::GAME_NAV_CLEARPATHTOPOINT as c_int,
        &[
            unsafe { (*self_).s.number as isize },
            pmins.as_ptr() as isize,
            pmaxs.as_ptr() as isize,
            point.as_ptr() as isize,
            clipmask as isize,
            okToHitEntNum as isize,
        ],
    ) as i32)
}

/// Raven `GNavCallback_NPC_ClearLOS`.
///
/// Source: `oracle/codemp/server/NPCNav/gameCallbacks.cpp:11-14`
pub fn GNavCallback_NPC_ClearLOS(
    common: &mut Common,
    sv: &mut Server,
    ent: *mut sharedEntity_t,
    end: vec3_t,
) -> qboolean {
    qboolean::from(mp_engine_qcommon::vm::VM_Call(
        common,
        sv.gvm,
        MpGameExport::GAME_NAV_CLEARLOS as c_int,
        &[unsafe { (*ent).s.number as isize }, end.as_ptr() as isize],
    ) as i32)
}

/// Raven `GNavCallback_NAVNEW_ClearPathBetweenPoints`.
///
/// Source: `oracle/codemp/server/NPCNav/gameCallbacks.cpp:16-19`
pub fn GNavCallback_NAVNEW_ClearPathBetweenPoints(
    common: &mut Common,
    sv: &mut Server,
    start: vec3_t,
    end: vec3_t,
    mins: vec3_t,
    maxs: vec3_t,
    ignore: c_int,
    clipmask: c_int,
) -> c_int {
    mp_engine_qcommon::vm::VM_Call(
        common,
        sv.gvm,
        MpGameExport::GAME_NAV_CLEARPATHBETWEENPOINTS as c_int,
        &[
            start.as_ptr() as isize,
            end.as_ptr() as isize,
            mins.as_ptr() as isize,
            maxs.as_ptr() as isize,
            ignore as isize,
            clipmask as isize,
        ],
    )
}

/// Raven `GNavCallback_NAV_CheckNodeFailedForEnt`.
///
/// Source: `oracle/codemp/server/NPCNav/gameCallbacks.cpp:21-24`
pub fn GNavCallback_NAV_CheckNodeFailedForEnt(
    common: &mut Common,
    sv: &mut Server,
    ent: *mut sharedEntity_t,
    nodeNum: c_int,
) -> qboolean {
    qboolean::from(mp_engine_qcommon::vm::VM_Call(
        common,
        sv.gvm,
        MpGameExport::GAME_NAV_CHECKNODEFAILEDFORENT as c_int,
        &[unsafe { (*ent).s.number as isize }, nodeNum as isize],
    ) as i32)
}

/// Raven `GNavCallback_G_EntIsUnlockedDoor`.
///
/// Source: `oracle/codemp/server/NPCNav/gameCallbacks.cpp:26-29`
pub fn GNavCallback_G_EntIsUnlockedDoor(
    common: &mut Common,
    sv: &mut Server,
    entityNum: c_int,
) -> qboolean {
    qboolean::from(mp_engine_qcommon::vm::VM_Call(
        common,
        sv.gvm,
        MpGameExport::GAME_NAV_ENTISUNLOCKEDDOOR as c_int,
        &[entityNum as isize],
    ) as i32)
}

/// Raven `GNavCallback_G_EntIsDoor`.
///
/// Source: `oracle/codemp/server/NPCNav/gameCallbacks.cpp:31-34`
pub fn GNavCallback_G_EntIsDoor(
    common: &mut Common,
    sv: &mut Server,
    entityNum: c_int,
) -> qboolean {
    qboolean::from(mp_engine_qcommon::vm::VM_Call(
        common,
        sv.gvm,
        MpGameExport::GAME_NAV_ENTISDOOR as c_int,
        &[entityNum as isize],
    ) as i32)
}

/// Raven `GNavCallback_G_EntIsBreakable`.
///
/// Source: `oracle/codemp/server/NPCNav/gameCallbacks.cpp:36-39`
pub fn GNavCallback_G_EntIsBreakable(
    common: &mut Common,
    sv: &mut Server,
    entityNum: c_int,
) -> qboolean {
    qboolean::from(mp_engine_qcommon::vm::VM_Call(
        common,
        sv.gvm,
        MpGameExport::GAME_NAV_ENTISBREAKABLE as c_int,
        &[entityNum as isize],
    ) as i32)
}

/// Raven `GNavCallback_G_EntIsRemovableUsable`.
///
/// Source: `oracle/codemp/server/NPCNav/gameCallbacks.cpp:41-44`
pub fn GNavCallback_G_EntIsRemovableUsable(
    common: &mut Common,
    sv: &mut Server,
    entNum: c_int,
) -> qboolean {
    qboolean::from(mp_engine_qcommon::vm::VM_Call(
        common,
        sv.gvm,
        MpGameExport::GAME_NAV_ENTISREMOVABLEUSABLE as c_int,
        &[entNum as isize],
    ) as i32)
}

/// Raven `GNavCallback_CP_FindCombatPointWaypoints`.
///
/// Source: `oracle/codemp/server/NPCNav/gameCallbacks.cpp:46-49`
pub fn GNavCallback_CP_FindCombatPointWaypoints(common: &mut Common, sv: &mut Server) {
    mp_engine_qcommon::vm::VM_Call(
        common,
        sv.gvm,
        MpGameExport::GAME_NAV_FINDCOMBATPOINTWAYPOINTS as c_int,
        &[],
    );
}
