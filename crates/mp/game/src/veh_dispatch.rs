//! `veh_dispatch`: vehicle "virtual" dispatch.
//!
//! This retires Raven's `vehicleInfo_t` function-pointer table (the 25
//! `AnimateVehicle`/`Board`/`Eject`/… slots filled once at `.veh` load by
//! `G_Set*VehicleFunctions`) in favour of `vehicleType_t`-keyed dispatch
//! (porting-rules §C8/§F17: a closed hierarchy of `VH_WALKER`/`VH_FIGHTER`/
//! `VH_SPEEDER`/`VH_ANIMAL` plus a generic base).
//! Each Raven `pVeh->m_pVehicleInfo->SLOT(...)` call becomes `veh_dispatch::<slot>(...)`,
//! which matches on the vehicle's `type` field and calls the per-class override or the
//! generic base impl.
//! This is exactly the choice Raven's setters baked into the fn-ptr slots.
//!
//! The override map is the QAGAME (server) column of the Raven setters:
//! - generic base: `oracle/codemp/game/g_vehicles.c:3290` (`G_SetSharedVehicleFunctions`)
//! - Fighter overrides Board+Eject: `FighterNPC.c:1948`
//! - Walker overrides Board+RegisterAssets: `WalkerNPC.c:547`
//! - Animal overrides DeathUpdate: `AnimalNPC.c:857`
//! - Speeder overrides no game-tier slot reachable here: `SpeederNPC.c:1044`
//!
//! Tier note: the slot impls are game-tier (they take `GameContext`), so this
//! dispatch is game-tier too and lives in `mp_game` rather than `mp_bg`.
//! The earlier plan's "mp_bg vehicles subsystem" predates the settled decision
//! that the bg/game boundary is the `BgTraps`/`GameCallbacks` traits, not a
//! crate wall, and the pmove/vehicle bodies all live in `mp_game`.
//! Game-tier callers invoke these directly.
//! The one bg-tier caller (`bg_pmove` boarding) reaches `board` through
//! [`crate::bg_channel::GameCallbacks::board_vehicle`].
#![allow(non_snake_case)]

use crate::bg_channel::{GameBgTraps, GameCallbacksImpl, PmoveContext};
use crate::prelude::*;
use mp_bg::vehicles::veh_process;

/// Read the vehicle's static type (the dispatch key). Safe as long as `pVeh`
/// and its `m_pVehicleInfo` are valid, which every call site already assumes.
#[inline]
unsafe fn veh_type(pVeh: *mut Vehicle_t) -> vehicleType_t {
    (*(*pVeh).m_pVehicleInfo).r#type
}

/// `Board`: Fighter/Walker override, else generic base.
/// Source: `oracle/codemp/game/g_vehicles.c:630` (generic),
/// `FighterNPC.c:212`, `WalkerNPC.c:186`.
pub fn board(ctx: &mut GameContext, pVeh: *mut Vehicle_t, pEnt: *mut bgEntity_t) -> qboolean {
    match unsafe { veh_type(pVeh) } {
        vehicleType_t::VH_FIGHTER => crate::FighterNPC::Board(ctx, pVeh, pEnt),
        vehicleType_t::VH_WALKER => {
            if crate::WalkerNPC::Board(ctx, pVeh, pEnt) {
                qtrue
            } else {
                qfalse
            }
        }
        _ => crate::g_vehicles::Board(ctx, pVeh, pEnt),
    }
}

/// `Eject`: Fighter override, else generic base.
/// Source: `oracle/codemp/game/g_vehicles.c:1019` (generic), `FighterNPC.c:224`.
pub fn eject(
    ctx: &mut GameContext,
    pVeh: *mut Vehicle_t,
    pEnt: *mut bgEntity_t,
    forceEject: qboolean,
) -> qboolean {
    match unsafe { veh_type(pVeh) } {
        vehicleType_t::VH_FIGHTER => crate::FighterNPC::Eject(ctx, pVeh, pEnt, forceEject),
        _ => crate::g_vehicles::Eject(ctx, pVeh, pEnt, forceEject),
    }
}

/// `EjectAll`: generic base only (no game-tier override).
/// Source: `oracle/codemp/game/g_vehicles.c:1377`.
pub fn eject_all(ctx: &mut GameContext, pVeh: *mut Vehicle_t) -> qboolean {
    crate::g_vehicles::EjectAll(ctx, pVeh)
}

/// `DeathUpdate`: Animal override, else generic base.
/// Source: `oracle/codemp/game/g_vehicles.c:1485` (generic), `AnimalNPC.c:...`.
pub fn death_update(ctx: &mut GameContext, pVeh: *mut Vehicle_t) {
    match unsafe { veh_type(pVeh) } {
        vehicleType_t::VH_ANIMAL => crate::AnimalNPC::DeathUpdate(ctx, pVeh),
        _ => crate::g_vehicles::DeathUpdate(ctx, pVeh),
    }
}

/// `RegisterAssets`: Walker override, else generic base (empty).
/// Source: `oracle/codemp/game/g_vehicles.c:1619` (generic), `WalkerNPC.c:...`.
pub fn register_assets(ctx: &mut GameContext, pVeh: *mut Vehicle_t) {
    match unsafe { veh_type(pVeh) } {
        vehicleType_t::VH_WALKER => crate::WalkerNPC::RegisterAssets(ctx, pVeh),
        _ => unsafe { crate::g_vehicles::RegisterAssets(pVeh) },
    }
}

/// `Initialize`: generic base only.
/// Source: `oracle/codemp/game/g_vehicles.c:...` (`Initialize`).
pub fn initialize(ctx: &mut GameContext, pVeh: *mut Vehicle_t) -> qboolean {
    crate::g_vehicles::Initialize(ctx, pVeh)
}

/// `StartDeathDelay`: generic base only.
/// Source: `oracle/codemp/game/g_vehicles.c:1451`.
pub fn start_death_delay(ctx: &mut GameContext, pVeh: *mut Vehicle_t, iDelayTimeOverride: c_int) {
    crate::g_vehicles::StartDeathDelay(ctx, pVeh, iDelayTimeOverride)
}

/// `Inhabited`: generic base only (`type`-independent).
/// Source: `oracle/codemp/game/g_vehicles.c:...` (`Inhabited`).
pub fn inhabited(ctx: &mut GameContext, pVeh: *mut Vehicle_t) -> qboolean {
    crate::g_vehicles::Inhabited(ctx, pVeh)
}

/// `ValidateBoard`: generic base only.
/// Source: `oracle/codemp/game/g_vehicles.c:...` (`ValidateBoard`).
pub fn validate_board(
    ctx: &mut GameContext,
    pVeh: *mut Vehicle_t,
    pEnt: *mut bgEntity_t,
) -> qboolean {
    crate::g_vehicles::ValidateBoard(ctx, pVeh, pEnt)
}

/// `SetPilot`: generic base only.
/// Source: `oracle/codemp/game/g_vehicles.c:3280`.
pub fn set_pilot(ctx: &mut GameContext, pVeh: *mut Vehicle_t, pPilot: *mut bgEntity_t) {
    crate::g_vehicles::SetPilot(ctx, pVeh, pPilot)
}

/// `Ghost`: generic base only.
/// Source: `oracle/codemp/game/g_vehicles.c:...` (`Ghost`).
pub fn ghost(ctx: &mut GameContext, pVeh: *mut Vehicle_t, pEnt: *mut bgEntity_t) {
    crate::g_vehicles::Ghost(ctx, pVeh, pEnt)
}

/// `UnGhost`: generic base only.
/// Source: `oracle/codemp/game/g_vehicles.c:1718` (`UnGhost`).
pub fn un_ghost(ctx: &mut GameContext, pVeh: *mut Vehicle_t, pEnt: *mut bgEntity_t) {
    crate::g_vehicles::UnGhost(ctx, pVeh, pEnt)
}

/// `AnimateVehicle`: per-class only.
/// The generic base leaves the slot null, so Raven's callers `if`-guard it.
/// The `_` arm is that skip.
/// Source: `oracle/codemp/game/{Fighter,Speeder,Walker,Animal}NPC.c` (`AnimateVehicle`).
pub fn animate_vehicle(ctx: &mut GameContext, pVeh: *mut Vehicle_t) {
    match unsafe { veh_type(pVeh) } {
        vehicleType_t::VH_SPEEDER => crate::SpeederNPC::AnimateVehicle(ctx, pVeh),
        vehicleType_t::VH_WALKER => crate::WalkerNPC::AnimateVehicle(ctx, pVeh),
        vehicleType_t::VH_ANIMAL => crate::AnimalNPC::AnimateVehicle(ctx, pVeh),
        vehicleType_t::VH_FIGHTER => crate::FighterNPC::AnimateVehicle(ctx, pVeh),
        _ => {}
    }
}

/// `AnimateRiders`: per-class only.
/// Walker leaves it null.
/// Source: `oracle/codemp/game/{Fighter,Speeder,Animal}NPC.c` (`AnimateRiders`).
pub fn animate_riders(ctx: &mut GameContext, pVeh: *mut Vehicle_t) {
    match unsafe { veh_type(pVeh) } {
        vehicleType_t::VH_SPEEDER => crate::SpeederNPC::AnimateRiders(ctx, pVeh),
        vehicleType_t::VH_ANIMAL => crate::AnimalNPC::AnimateRiders(ctx, pVeh),
        vehicleType_t::VH_FIGHTER => crate::FighterNPC::AnimateRiders(ctx, pVeh),
        _ => {}
    }
}

/// `ProcessMoveCommands`: per-class only.
/// The dispatch and bodies moved to `mp_bg::vehicles::veh_process`, shared with cgame
/// prediction (DEC-32, one canonical home per fn).
/// This game-tier adapter builds a `pm`-null `PmoveContext` and forwards.
/// Source: `oracle/codemp/game/{Fighter,Speeder,Walker,Animal}NPC.c`.
pub fn process_move_commands(ctx: &mut GameContext, pVeh: *mut Vehicle_t) {
    let traps = GameBgTraps::new(ctx.engine);
    let mut callbacks = GameCallbacksImpl {
        world: ctx.world_raw(),
        engine: ctx.engine,
    };
    // Raven reaches these bodies with the TU-static `pm` still pointing at the last Pmove,
    // whose baseEnt/entSize are the g_entities arena.
    // The bodies' PM_BGEntForNum rider lookups depend on exactly those two fields, so the
    // shim carries them.
    // A bare pm-null context resolves every rider to the vehicle itself.
    // SAFETY: zeroed pmove_t is the g_active.rs pmove-setup precedent.
    let mut shim_pm: pmove_t = unsafe { core::mem::zeroed() };
    shim_pm.baseEnt = ctx.world.g_entities.as_mut_ptr() as *mut _;
    shim_pm.entSize = core::mem::size_of::<gentity_t>() as c_int;
    let mut pmc = PmoveContext::new(&mut ctx.world.bg_state, &traps, &mut callbacks);
    pmc.pm = &raw mut shim_pm;
    veh_process::process_move_commands(&mut pmc, pVeh);
}

/// `ProcessOrientCommands`: per-class only.
/// Moved to `mp_bg::vehicles::veh_process`.
/// Game-tier adapter, see [`process_move_commands`], including the baseEnt shim.
/// Source: `oracle/codemp/game/{Fighter,Speeder,Walker,Animal}NPC.c`.
pub fn process_orient_commands(ctx: &mut GameContext, pVeh: *mut Vehicle_t) {
    let traps = GameBgTraps::new(ctx.engine);
    let mut callbacks = GameCallbacksImpl {
        world: ctx.world_raw(),
        engine: ctx.engine,
    };
    // same rider-lookup shim as `process_move_commands`
    // SAFETY: zeroed pmove_t is the g_active.rs pmove-setup precedent.
    let mut shim_pm: pmove_t = unsafe { core::mem::zeroed() };
    shim_pm.baseEnt = ctx.world.g_entities.as_mut_ptr() as *mut _;
    shim_pm.entSize = core::mem::size_of::<gentity_t>() as c_int;
    let mut pmc = PmoveContext::new(&mut ctx.world.bg_state, &traps, &mut callbacks);
    pmc.pm = &raw mut shim_pm;
    veh_process::process_orient_commands(&mut pmc, pVeh);
}

/// `Update`: per-class where the class setter overrides the base slot.
///
/// Oracle wiring: Fighter/Speeder/Animal each assign `pVehInfo->Update` to their own `Update`.
/// Walker's assignment is commented out, so Walker (and any other type) keeps the base
/// `Update` set by the shared setter.
/// Each per-class override first runs its own body (e.g. `FighterNPC::Update` → `BG_FighterUpdate`)
/// then chains the generic base.
/// Source: `oracle/codemp/game/{Fighter,Speeder,Animal,Walker}NPC.c`
/// (per-class `Update` wiring), `g_vehicles.c:3306` (base).
pub fn update(ctx: &mut GameContext, pVeh: *mut Vehicle_t, pUcmd: *const usercmd_t) -> qboolean {
    match unsafe { veh_type(pVeh) } {
        vehicleType_t::VH_SPEEDER => crate::SpeederNPC::Update(ctx, pVeh, pUcmd),
        vehicleType_t::VH_ANIMAL => crate::AnimalNPC::Update(ctx, pVeh, pUcmd),
        vehicleType_t::VH_FIGHTER => crate::FighterNPC::Update(ctx, pVeh, pUcmd),
        _ => crate::g_vehicles::Update(ctx, pVeh, pUcmd),
    }
}

/// `Animate`: generic base only.
/// This is the `vehicleInfo_t.Animate` slot, distinct from the per-class `AnimateVehicle`
/// slot dispatched by [`animate_vehicle`].
/// Source: `oracle/codemp/game/g_vehicles.c:3298`.
pub fn animate(ctx: &mut GameContext, pVeh: *mut Vehicle_t) {
    crate::g_vehicles::Animate(ctx, pVeh)
}

/// `UpdateRider`: generic base only.
/// Source: `oracle/codemp/game/g_vehicles.c:3307`.
pub fn update_rider(
    ctx: &mut GameContext,
    pVeh: *mut Vehicle_t,
    pRider: *mut bgEntity_t,
    pUcmd: *mut usercmd_t,
) -> qboolean {
    crate::g_vehicles::UpdateRider(ctx, pVeh, pRider, pUcmd)
}

/// `AttachRiders`: generic base only.
/// Source: `oracle/codemp/game/g_vehicles.c:3310`.
pub fn attach_riders(ctx: &mut GameContext, pVeh: *mut Vehicle_t) {
    crate::g_vehicles::AttachRiders(ctx, pVeh)
}
