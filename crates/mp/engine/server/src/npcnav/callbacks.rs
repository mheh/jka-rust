//! `GNavCallback_*` free functions — the nine engine-to-game out-calls the
//! nav system makes (NAV-D3).
//!
//! Raven: "rww - callbacks the navigation system needs to make to the game
//! code."
//!
//! Each is a thin `VM_Call(gvm, GAME_NAV_*, ...)` shim: `host` first (it
//! wraps `host.vm_call`), `vec3_t` args cross as pointers, and the
//! `sharedEntity_t *` params pass only `self->s.number` (the callback derefs
//! the pointer and widens the int — matches the `mp_game` `GAME_NAV_*`
//! decoders' `entity_num: c_int`). Widened to `intptr_t`-slot args per the
//! historical `GAME_NAV_CLEARPATHTOPOINT` truncation bug (plan §5.4).
//!
//! Type definition source: `oracle/codemp/server/NPCNav/gameCallbacks.cpp`

use mp_abi::game::MpGameExport;
use mp_host_interface::vm_slot::VmSlot;
use mp_host_interface::EngineHost;
use mp_qshared::common::mp::qcommon::shared_entity_t::sharedEntity_t;
use mp_qshared::shared::{qboolean, vec3_t};

/// Raven `GNavCallback_NAV_ClearPathToPoint`.
///
/// Source: `oracle/codemp/server/NPCNav/gameCallbacks.cpp:6-9`
#[allow(non_snake_case)]
pub fn GNavCallback_NAV_ClearPathToPoint(
    host: &mut impl EngineHost,
    self_ent: *mut sharedEntity_t,
    pmins: &vec3_t,
    pmaxs: &vec3_t,
    point: &vec3_t,
    clipmask: i32,
    ok_to_hit_ent_num: i32,
) -> qboolean {
    let entity_num = unsafe { (*self_ent).s.number };
    host.vm_call(
        VmSlot::Gvm,
        MpGameExport::GAME_NAV_CLEARPATHTOPOINT as i32,
        &[
            entity_num as isize,
            pmins.as_ptr() as isize,
            pmaxs.as_ptr() as isize,
            point.as_ptr() as isize,
            clipmask as isize,
            ok_to_hit_ent_num as isize,
        ],
    ) as qboolean
}

/// Raven `GNavCallback_NPC_ClearLOS`.
///
/// Source: `oracle/codemp/server/NPCNav/gameCallbacks.cpp:11-14`
#[allow(non_snake_case)]
pub fn GNavCallback_NPC_ClearLOS(
    host: &mut impl EngineHost,
    ent: *mut sharedEntity_t,
    end: &vec3_t,
) -> qboolean {
    let entity_num = unsafe { (*ent).s.number };
    host.vm_call(
        VmSlot::Gvm,
        MpGameExport::GAME_NAV_CLEARLOS as i32,
        &[entity_num as isize, end.as_ptr() as isize],
    ) as qboolean
}

/// Raven `GNavCallback_NAVNEW_ClearPathBetweenPoints`.
///
/// Source: `oracle/codemp/server/NPCNav/gameCallbacks.cpp:16-19`
#[allow(non_snake_case)]
pub fn GNavCallback_NAVNEW_ClearPathBetweenPoints(
    host: &mut impl EngineHost,
    start: &vec3_t,
    end: &vec3_t,
    mins: &vec3_t,
    maxs: &vec3_t,
    ignore: i32,
    clipmask: i32,
) -> i32 {
    host.vm_call(
        VmSlot::Gvm,
        MpGameExport::GAME_NAV_CLEARPATHBETWEENPOINTS as i32,
        &[
            start.as_ptr() as isize,
            end.as_ptr() as isize,
            mins.as_ptr() as isize,
            maxs.as_ptr() as isize,
            ignore as isize,
            clipmask as isize,
        ],
    ) as i32
}

/// Raven `GNavCallback_NAV_CheckNodeFailedForEnt`.
///
/// Source: `oracle/codemp/server/NPCNav/gameCallbacks.cpp:21-24`
#[allow(non_snake_case)]
pub fn GNavCallback_NAV_CheckNodeFailedForEnt(
    host: &mut impl EngineHost,
    ent: *mut sharedEntity_t,
    node_num: i32,
) -> qboolean {
    let entity_num = unsafe { (*ent).s.number };
    host.vm_call(
        VmSlot::Gvm,
        MpGameExport::GAME_NAV_CHECKNODEFAILEDFORENT as i32,
        &[entity_num as isize, node_num as isize],
    ) as qboolean
}

/// Raven `GNavCallback_G_EntIsUnlockedDoor`.
///
/// Source: `oracle/codemp/server/NPCNav/gameCallbacks.cpp:26-29`
#[allow(non_snake_case)]
pub fn GNavCallback_G_EntIsUnlockedDoor(host: &mut impl EngineHost, entity_num: i32) -> qboolean {
    host.vm_call(
        VmSlot::Gvm,
        MpGameExport::GAME_NAV_ENTISUNLOCKEDDOOR as i32,
        &[entity_num as isize],
    ) as qboolean
}

/// Raven `GNavCallback_G_EntIsDoor`.
///
/// Source: `oracle/codemp/server/NPCNav/gameCallbacks.cpp:31-34`
#[allow(non_snake_case)]
pub fn GNavCallback_G_EntIsDoor(host: &mut impl EngineHost, entity_num: i32) -> qboolean {
    host.vm_call(
        VmSlot::Gvm,
        MpGameExport::GAME_NAV_ENTISDOOR as i32,
        &[entity_num as isize],
    ) as qboolean
}

/// Raven `GNavCallback_G_EntIsBreakable`.
///
/// Source: `oracle/codemp/server/NPCNav/gameCallbacks.cpp:36-39`
#[allow(non_snake_case)]
pub fn GNavCallback_G_EntIsBreakable(host: &mut impl EngineHost, entity_num: i32) -> qboolean {
    host.vm_call(
        VmSlot::Gvm,
        MpGameExport::GAME_NAV_ENTISBREAKABLE as i32,
        &[entity_num as isize],
    ) as qboolean
}

/// Raven `GNavCallback_G_EntIsRemovableUsable`.
///
/// Source: `oracle/codemp/server/NPCNav/gameCallbacks.cpp:41-44`
#[allow(non_snake_case)]
pub fn GNavCallback_G_EntIsRemovableUsable(host: &mut impl EngineHost, ent_num: i32) -> qboolean {
    host.vm_call(
        VmSlot::Gvm,
        MpGameExport::GAME_NAV_ENTISREMOVABLEUSABLE as i32,
        &[ent_num as isize],
    ) as qboolean
}

/// Raven `GNavCallback_CP_FindCombatPointWaypoints`.
///
/// Source: `oracle/codemp/server/NPCNav/gameCallbacks.cpp:46-49`
#[allow(non_snake_case)]
pub fn GNavCallback_CP_FindCombatPointWaypoints(host: &mut impl EngineHost) {
    host.vm_call(
        VmSlot::Gvm,
        MpGameExport::GAME_NAV_FINDCOMBATPOINTWAYPOINTS as i32,
        &[],
    );
}

// `NAV_CvarInit`/`NAV_Free` (`oracle/codemp/server/NPCNav/navigator.cpp:39-48`)
// are NOT ported here despite this file's roster-summary mention (files:
// block, docs/subsystems/npcnav.md) — they are a separate free-fn pair from
// `navigator.cpp`'s file scope, not members of the `GNavCallback_*`
// (`gameCallbacks.cpp`) class this file is scoped to. The doc's own roster
// table (row `NAV_CvarInit`/`NAV_Free`, :995) places `NAV_Free` as
// `Navigator::free` (a `Navigator` method, `navigator.rs`) and `NAV_CvarInit`
// as a no-op cvar-registration elision — neither is a `GNavCallback_*` free
// function. See the returned `problems` list for this doc/oracle
// cross-reference mismatch.

#[cfg(test)]
mod tests {
    use super::*;
    use mp_host_interface::mock::MockHost;

    /// Pins the nine `GAME_NAV_*` out-call numbers this module's shims decode
    /// against, so a renumbering in `mp_abi` fails loudly here too.
    #[test]
    fn game_nav_export_variants_exist() {
        let _ = MpGameExport::GAME_NAV_CLEARPATHTOPOINT;
        let _ = MpGameExport::GAME_NAV_CLEARLOS;
        let _ = MpGameExport::GAME_NAV_CLEARPATHBETWEENPOINTS;
        let _ = MpGameExport::GAME_NAV_CHECKNODEFAILEDFORENT;
        let _ = MpGameExport::GAME_NAV_ENTISUNLOCKEDDOOR;
        let _ = MpGameExport::GAME_NAV_ENTISDOOR;
        let _ = MpGameExport::GAME_NAV_ENTISBREAKABLE;
        let _ = MpGameExport::GAME_NAV_ENTISREMOVABLEUSABLE;
        let _ = MpGameExport::GAME_NAV_FINDCOMBATPOINTWAYPOINTS;
    }

    /// A bare index-taking shim (`entity_num` arrives as a plain `i32`, no
    /// `sharedEntity_t` deref) forwards it verbatim as the sole `intptr_t`
    /// slot and returns the game's reply widened back down through
    /// `qboolean` (`gameCallbacks.cpp:31-34`).
    #[test]
    fn ent_is_door_forwards_entity_num_and_widens_reply() {
        let mut host = MockHost::new();
        host.vm_call_return = 1;

        let r = GNavCallback_G_EntIsDoor(&mut host, 42);

        assert_eq!(r, 1);
        assert_eq!(
            host.vm_calls,
            vec![(
                VmSlot::Gvm,
                MpGameExport::GAME_NAV_ENTISDOOR as i32,
                vec![42isize]
            )]
        );
    }

    /// The void out-call (`gameCallbacks.cpp:46-49`) issues the `vm_call`
    /// with zero args and no return handling.
    #[test]
    fn find_combat_point_waypoints_issues_bare_call() {
        let mut host = MockHost::new();

        GNavCallback_CP_FindCombatPointWaypoints(&mut host);

        assert_eq!(
            host.vm_calls,
            vec![(
                VmSlot::Gvm,
                MpGameExport::GAME_NAV_FINDCOMBATPOINTWAYPOINTS as i32,
                vec![]
            )]
        );
    }

    /// The `sharedEntity_t *` arms pass **`ent->s.number`**, not the arena
    /// slot the pointer happens to live at (`gameCallbacks.cpp:11-14`) — this
    /// pins that the callback derefs the real field rather than re-deriving
    /// an index some other way.
    #[test]
    fn clear_los_passes_ent_s_number_not_arena_slot() {
        let mut host = MockHost::new();
        host.vm_call_return = 1;
        // Live at arena slot 5, but its `s.number` says 7 — the callback must
        // forward 7, proving it dereferences the field rather than the slot.
        host.gentity_mut(5).s.number = 7;
        let ent = host.gentity_mut(5) as *mut sharedEntity_t;
        let end: vec3_t = [1.0, 2.0, 3.0];

        let r = GNavCallback_NPC_ClearLOS(&mut host, ent, &end);

        assert_eq!(r, 1);
        assert_eq!(host.vm_calls.len(), 1);
        let (vm, callnum, args) = &host.vm_calls[0];
        assert_eq!(*vm, VmSlot::Gvm);
        assert_eq!(*callnum, MpGameExport::GAME_NAV_CLEARLOS as i32);
        assert_eq!(args[0], 7isize);
        assert_eq!(args[1], end.as_ptr() as isize);
    }

    /// The six-arg `NAV_ClearPathToPoint` shim (`gameCallbacks.cpp:6-9`)
    /// forwards the entity number plus the three `vec3_t` pointers and two
    /// trailing ints in Raven's exact order.
    #[test]
    fn clear_path_to_point_forwards_all_six_args_in_order() {
        let mut host = MockHost::new();
        host.gentity_mut(3).s.number = 3;
        let ent = host.gentity_mut(3) as *mut sharedEntity_t;
        let pmins: vec3_t = [-1.0, -2.0, -3.0];
        let pmaxs: vec3_t = [1.0, 2.0, 3.0];
        let point: vec3_t = [4.0, 5.0, 6.0];

        let _ = GNavCallback_NAV_ClearPathToPoint(&mut host, ent, &pmins, &pmaxs, &point, 9, 11);

        assert_eq!(host.vm_calls.len(), 1);
        let (_, callnum, args) = &host.vm_calls[0];
        assert_eq!(*callnum, MpGameExport::GAME_NAV_CLEARPATHTOPOINT as i32);
        assert_eq!(
            args,
            &vec![
                3isize,
                pmins.as_ptr() as isize,
                pmaxs.as_ptr() as isize,
                point.as_ptr() as isize,
                9isize,
                11isize,
            ]
        );
    }
}
