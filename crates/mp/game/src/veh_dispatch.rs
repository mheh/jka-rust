//! `veh_dispatch` — fork-7 vehicle "virtual" dispatch.
//!
//! Fork 7 (blessed 2026-07-03) retires Raven's `vehicleInfo_t` function-pointer
//! table (the 25 `AnimateVehicle`/`Board`/`Eject`/… slots filled once at `.veh`
//! load by `G_Set*VehicleFunctions`) in favour of `vehicleType_t`-keyed dispatch
//! (porting-rules §C8/§F17 — a closed hierarchy: `VH_WALKER`/`VH_FIGHTER`/
//! `VH_SPEEDER`/`VH_ANIMAL` + a generic base). Each Raven `pVeh->m_pVehicleInfo->
//! SLOT(...)` call becomes `veh_dispatch::<slot>(...)`, which matches on the
//! vehicle's `type` field and calls the per-class override or the generic base
//! impl — exactly the choice Raven's setters baked into the fn-ptr slots.
//!
//! The override map is the QAGAME (server) column of the Raven setters:
//! - generic base: `oracle/oracle/codemp/game/g_vehicles.c:3290` (`G_SetSharedVehicleFunctions`)
//! - Fighter overrides Board+Eject: `FighterNPC.c:1948`
//! - Walker overrides Board+RegisterAssets: `WalkerNPC.c:547`
//! - Animal overrides DeathUpdate: `AnimalNPC.c:857`
//! - Speeder overrides no game-tier slot reachable here: `SpeederNPC.c:1044`
//!
//! Tier note: the slot impls are game-tier (they take `GameContext`), so this
//! dispatch is game-tier too and lives in `mp_game` rather than `mp_bg` (the
//! ruling's "mp_bg vehicles subsystem" predates ruling-19, under which the
//! bg/game boundary is the `BgTraps`/`GameCallbacks` traits, not a crate wall,
//! and the pmove/vehicle bodies all live in `mp_game`). Game-tier callers invoke
//! these directly; the one bg-tier caller (`bg_pmove` boarding) reaches `board`
//! through [`crate::bg_channel::GameCallbacks::board_vehicle`].
#![allow(non_snake_case)]

use crate::prelude::*;

/// Read the vehicle's static type (the dispatch key). Safe as long as `pVeh`
/// and its `m_pVehicleInfo` are valid, which every call site already assumes.
#[inline]
unsafe fn veh_type(pVeh: *mut Vehicle_t) -> vehicleType_t {
    (*(*pVeh).m_pVehicleInfo).r#type
}

/// `Board` — Fighter/Walker override, else generic base.
/// Source: `oracle/oracle/codemp/game/g_vehicles.c:630` (generic),
/// `FighterNPC.c:212`, `WalkerNPC.c:186`.
pub fn board(ctx: GameContext<'_>, pVeh: *mut Vehicle_t, pEnt: *mut bgEntity_t) -> qboolean {
    match unsafe { veh_type(pVeh) } {
        VH_FIGHTER => crate::FighterNPC::Board(ctx, pVeh, pEnt),
        VH_WALKER => {
            if crate::WalkerNPC::Board(ctx, pVeh, pEnt) {
                qtrue
            } else {
                qfalse
            }
        }
        _ => crate::g_vehicles::Board(ctx, pVeh, pEnt),
    }
}

/// `Eject` — Fighter override, else generic base.
/// Source: `oracle/oracle/codemp/game/g_vehicles.c:1019` (generic), `FighterNPC.c:224`.
pub fn eject(
    ctx: GameContext<'_>,
    pVeh: *mut Vehicle_t,
    pEnt: *mut bgEntity_t,
    forceEject: qboolean,
) -> qboolean {
    match unsafe { veh_type(pVeh) } {
        VH_FIGHTER => crate::FighterNPC::Eject(ctx, pVeh, pEnt, forceEject),
        _ => crate::g_vehicles::Eject(ctx, pVeh, pEnt, forceEject),
    }
}

/// `EjectAll` — generic base only (no game-tier override).
/// Source: `oracle/oracle/codemp/game/g_vehicles.c:1377`.
pub fn eject_all(ctx: GameContext<'_>, pVeh: *mut Vehicle_t) -> qboolean {
    crate::g_vehicles::EjectAll(ctx, pVeh)
}

/// `DeathUpdate` — Animal override, else generic base.
/// Source: `oracle/oracle/codemp/game/g_vehicles.c:1485` (generic), `AnimalNPC.c:...`.
pub fn death_update(ctx: GameContext<'_>, pVeh: *mut Vehicle_t) {
    match unsafe { veh_type(pVeh) } {
        VH_ANIMAL => crate::AnimalNPC::DeathUpdate(ctx, pVeh),
        _ => crate::g_vehicles::DeathUpdate(ctx, pVeh),
    }
}

/// `RegisterAssets` — Walker override, else generic base (empty).
/// Source: `oracle/oracle/codemp/game/g_vehicles.c:1619` (generic), `WalkerNPC.c:...`.
pub fn register_assets(ctx: GameContext<'_>, pVeh: *mut Vehicle_t) {
    match unsafe { veh_type(pVeh) } {
        VH_WALKER => crate::WalkerNPC::RegisterAssets(ctx, pVeh),
        _ => unsafe { crate::g_vehicles::RegisterAssets(pVeh) },
    }
}

/// `Initialize` — generic base only.
/// Source: `oracle/oracle/codemp/game/g_vehicles.c:...` (`Initialize`).
pub fn initialize(ctx: GameContext<'_>, pVeh: *mut Vehicle_t) -> qboolean {
    crate::g_vehicles::Initialize(ctx, pVeh)
}

/// `StartDeathDelay` — generic base only.
/// Source: `oracle/oracle/codemp/game/g_vehicles.c:1451`.
pub fn start_death_delay(ctx: GameContext<'_>, pVeh: *mut Vehicle_t, iDelayTimeOverride: c_int) {
    crate::g_vehicles::StartDeathDelay(ctx, pVeh, iDelayTimeOverride)
}

/// `Inhabited` — generic base only (`type`-independent).
/// Source: `oracle/oracle/codemp/game/g_vehicles.c:...` (`Inhabited`).
pub fn inhabited(pVeh: *mut Vehicle_t) -> qboolean {
    crate::g_vehicles::Inhabited(pVeh)
}

/// `ValidateBoard` — generic base only.
/// Source: `oracle/oracle/codemp/game/g_vehicles.c:...` (`ValidateBoard`).
pub fn validate_board(pVeh: *mut Vehicle_t, pEnt: *mut bgEntity_t) -> qboolean {
    crate::g_vehicles::ValidateBoard(pVeh, pEnt)
}

/// `SetPilot` — generic base only.
/// Source: `oracle/oracle/codemp/game/g_vehicles.c:3280`.
pub fn set_pilot(pVeh: *mut Vehicle_t, pPilot: *mut bgEntity_t) {
    crate::g_vehicles::SetPilot(pVeh, pPilot)
}

/// `Ghost` — generic base only.
/// Source: `oracle/oracle/codemp/game/g_vehicles.c:...` (`Ghost`).
pub fn ghost(pVeh: *mut Vehicle_t, pEnt: *mut bgEntity_t) {
    crate::g_vehicles::Ghost(pVeh, pEnt)
}

/// `UnGhost` — generic base only.
/// Source: `oracle/oracle/codemp/game/g_vehicles.c:1718` (`UnGhost`).
pub fn un_ghost(pVeh: *mut Vehicle_t, pEnt: *mut bgEntity_t) {
    crate::g_vehicles::UnGhost(pVeh, pEnt)
}

/// `AnimateVehicle` — per-class only (the generic base leaves the slot null, so
/// Raven's callers `if`-guard it; the `_` arm is that skip). Fighter's override
/// is not yet ported.
/// Source: `oracle/oracle/codemp/game/{Speeder,Walker,Animal}NPC.c` (`AnimateVehicle`).
pub fn animate_vehicle(pVeh: *mut Vehicle_t) {
    match unsafe { veh_type(pVeh) } {
        VH_SPEEDER => crate::SpeederNPC::AnimateVehicle(pVeh),
        VH_WALKER => crate::WalkerNPC::AnimateVehicle(pVeh),
        VH_ANIMAL => crate::AnimalNPC::AnimateVehicle(pVeh),
        //TODO: Port FighterNPC::AnimateVehicle
        // Source: oracle/oracle/codemp/game/FighterNPC.c:1951
        VH_FIGHTER => todo!("Port FighterNPC::AnimateVehicle — FighterNPC.c:1951"),
        _ => {}
    }
}

/// `AnimateRiders` — per-class only (Walker leaves it null). Fighter's override
/// is not yet ported.
/// Source: `oracle/oracle/codemp/game/{Speeder,Animal}NPC.c` (`AnimateRiders`).
pub fn animate_riders(pVeh: *mut Vehicle_t) {
    match unsafe { veh_type(pVeh) } {
        VH_SPEEDER => crate::SpeederNPC::AnimateRiders(pVeh),
        VH_ANIMAL => crate::AnimalNPC::AnimateRiders(pVeh),
        //TODO: Port FighterNPC::AnimateRiders
        // Source: oracle/oracle/codemp/game/FighterNPC.c:1952
        VH_FIGHTER => todo!("Port FighterNPC::AnimateRiders — FighterNPC.c:1952"),
        _ => {}
    }
}

/// `ProcessMoveCommands` — per-class only. Fighter's override is not yet ported.
/// Source: `oracle/oracle/codemp/game/{Speeder,Walker,Animal}NPC.c`.
pub fn process_move_commands(pVeh: *mut Vehicle_t) {
    match unsafe { veh_type(pVeh) } {
        VH_SPEEDER => crate::SpeederNPC::ProcessMoveCommands(pVeh),
        VH_WALKER => crate::WalkerNPC::ProcessMoveCommands(pVeh),
        VH_ANIMAL => crate::AnimalNPC::ProcessMoveCommands(pVeh),
        //TODO: Port FighterNPC::ProcessMoveCommands
        // Source: oracle/oracle/codemp/game/FighterNPC.c:1970
        VH_FIGHTER => todo!("Port FighterNPC::ProcessMoveCommands — FighterNPC.c:1970"),
        _ => {}
    }
}

/// `ProcessOrientCommands` — per-class only. Fighter's override is not yet ported.
/// Source: `oracle/oracle/codemp/game/{Speeder,Walker,Animal}NPC.c`.
pub fn process_orient_commands(pVeh: *mut Vehicle_t) {
    match unsafe { veh_type(pVeh) } {
        VH_SPEEDER => crate::SpeederNPC::ProcessOrientCommands(pVeh),
        VH_WALKER => crate::WalkerNPC::ProcessOrientCommands(pVeh),
        VH_ANIMAL => crate::AnimalNPC::ProcessOrientCommands(pVeh),
        //TODO: Port FighterNPC::ProcessOrientCommands
        // Source: oracle/oracle/codemp/game/FighterNPC.c:1971
        VH_FIGHTER => todo!("Port FighterNPC::ProcessOrientCommands — FighterNPC.c:1971"),
        _ => {}
    }
}
