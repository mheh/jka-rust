//! `veh_process` — the shared vehicle move/orient dispatch.
//!
//! The bg half of the `vehicleType_t`-keyed dispatch (`veh_dispatch` in
//! `mp_game`): the two slots Raven compiles into BOTH game and cgame
//! (`ProcessMoveCommands`/`ProcessOrientCommands`). Both hosts reach these — the
//! cgame vehicle `Pmove` calls them during prediction, the game-tier base
//! `Update` calls them through a `mp_game`'s `veh_dispatch` adapter that builds a
//! `pm`-null `PmoveContext`. Game-only dispatch slots (`Board`/`Eject`/`Update`/
//! `AnimateVehicle`/…) stay in `mp_game`'s `veh_dispatch`.
#![allow(non_snake_case)]

use crate::prelude::*;
use crate::vehicles::{animal_npc, fighter_npc, speeder_npc, walker_npc};

/// Read the vehicle's static type (the dispatch key).
#[inline]
unsafe fn veh_type(pVeh: *mut Vehicle_t) -> vehicleType_t {
    (*(*pVeh).m_pVehicleInfo).r#type
}

/// `ProcessMoveCommands` — per-class only.
/// Source: `oracle/codemp/game/{Fighter,Speeder,Walker,Animal}NPC.c`.
pub fn process_move_commands(pmc: &mut PmoveContext, pVeh: *mut Vehicle_t) {
    match unsafe { veh_type(pVeh) } {
        vehicleType_t::VH_SPEEDER => speeder_npc::ProcessMoveCommands(pmc, pVeh),
        vehicleType_t::VH_WALKER => walker_npc::ProcessMoveCommands(pmc, pVeh),
        vehicleType_t::VH_ANIMAL => animal_npc::ProcessMoveCommands(pmc, pVeh),
        vehicleType_t::VH_FIGHTER => fighter_npc::ProcessMoveCommands(pmc, pVeh),
        _ => {}
    }
}

/// `ProcessOrientCommands` — per-class only.
/// Source: `oracle/codemp/game/{Fighter,Speeder,Walker,Animal}NPC.c`.
pub fn process_orient_commands(pmc: &mut PmoveContext, pVeh: *mut Vehicle_t) {
    match unsafe { veh_type(pVeh) } {
        vehicleType_t::VH_SPEEDER => speeder_npc::ProcessOrientCommands(pmc, pVeh),
        vehicleType_t::VH_WALKER => walker_npc::ProcessOrientCommands(pmc, pVeh),
        vehicleType_t::VH_ANIMAL => animal_npc::ProcessOrientCommands(pmc, pVeh),
        vehicleType_t::VH_FIGHTER => fighter_npc::ProcessOrientCommands(pmc, pVeh),
        _ => {}
    }
}
