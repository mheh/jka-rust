#![allow(non_snake_case)]

//! The CGAME arms of the four `G_Create*NPC` vehicle constructors.
//!
//! Raven compiles `game/{Animal,Speeder,Fighter,Walker}NPC.c` into cgame too;
//! outside QAGAME only these create fns survive the preprocessor. The server
//! arms live in `mp_game` over its 128-slot `g_vehiclePool`; these client arms
//! allocate out of `CgWorld::vehicle_pool` instead of Raven's per-cent
//! `BG_Alloc` (DEC-47.3) - same null test ("only allocate a new one if we
//! really have to"), same zero + `m_pVehicleInfo` re-init on every call.

use core::ptr::write_bytes;

use mp_bg::bg_vehicleLoad::BG_VehicleGetIndex;
use mp_bg::cstr_util::cstr;

use crate::bg_channel::{CgBgTraps, CgGameCallbacks};
use crate::local::vehicle_id::VehicleId;
use crate::world::cg_context::CgContext;

/// Raven `G_CreateAnimalNPC` — create/allocate a new Animal Vehicle
/// (initializing it as well), CGAME arm.
///
/// Raven: this is a BG function too in MP so don't un-bg-compatibilify it.
/// Source: `oracle/codemp/game/AnimalNPC.c:904-925`
pub fn G_CreateAnimalNPC(ctx: &mut CgContext, centNum: usize, strAnimalType: &str) {
    create_vehicle_object(ctx, centNum, strAnimalType);
}

/// Raven `G_CreateSpeederNPC` — create/allocate a new Speeder Vehicle
/// (initializing it as well), CGAME arm.
///
/// Source: `oracle/codemp/game/SpeederNPC.c:1092-1113`
pub fn G_CreateSpeederNPC(ctx: &mut CgContext, centNum: usize, strType: &str) {
    create_vehicle_object(ctx, centNum, strType);
}

/// Raven `G_CreateFighterNPC` — create/allocate a new Fighter Vehicle
/// (initializing it as well), CGAME arm.
///
/// Source: `oracle/codemp/game/FighterNPC.c:1994-2014`
pub fn G_CreateFighterNPC(ctx: &mut CgContext, centNum: usize, strType: &str) {
    create_vehicle_object(ctx, centNum, strType);
}

/// Raven `G_CreateWalkerNPC` — create/allocate a new Walker Vehicle
/// (initializing it as well), CGAME arm.
///
/// Source: `oracle/codemp/game/WalkerNPC.c:594-617`
pub fn G_CreateWalkerNPC(ctx: &mut CgContext, centNum: usize, strAnimalType: &str) {
    create_vehicle_object(ctx, centNum, strAnimalType);
}

/// The one body all four Raven fns share client-side: claim the cent's pool
/// row if it isn't claimed yet, zero it, point `m_pVehicleInfo` at the
/// `g_vehicleInfo` slot for `strType`.
fn create_vehicle_object(ctx: &mut CgContext, centNum: usize, strType: &str) {
    //only allocate a new one if we really have to
    if ctx.world.entity(centNum).m_pVehicle.is_none() {
        ctx.world.entity_mut(centNum).m_pVehicle = VehicleId::new(centNum as u32);
    }

    let strType_c = cstr(strType);
    let traps = CgBgTraps::new(ctx.engine, ctx.world_raw());
    let mut callbacks = CgGameCallbacks::new(ctx.engine, ctx.world_raw());
    let veh_index = BG_VehicleGetIndex(
        strType_c.as_ptr(),
        &mut ctx.world.bg_state,
        &traps,
        &mut callbacks,
    ) as usize;

    let row = &mut ctx.world.vehicle_pool[centNum];
    // SAFETY: Vehicle_t is ZeroValid (Raven's `memset(*pVeh, 0, sizeof(Vehicle_t))`)
    unsafe { write_bytes(row, 0, 1) };
    row.m_pVehicleInfo = &raw mut ctx.world.bg_state.g_vehicleInfo[veh_index];
}
