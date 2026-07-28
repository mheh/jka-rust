//! Port of `oracle/codemp/cgame/cg_saga.c` — siege/saga round state, objectives and briefing parsing. Functions land via the C5
//! transcription waves.

#![allow(non_snake_case)]

use core::ffi::c_int;

use mp_bg::bg_saga::{BG_SiegeGetPairedValue, BG_SiegeGetValueGroup};
use mp_bg::public::entity_flags::EF_DOUBLE_AMMO;
use mp_bg::saga::siege_team_t::SIEGETEAM_TEAM1;
use mp_bg::weapons::ammo_data::ammoData;
use mp_bg::weapons::weapon_data::weaponData;
use mp_qshared::shared::MAX_CLIENTS_I32;
use native_string::{atoi, buf_to_string};

use crate::trap;
use crate::world::cg_context::CgContext;
use crate::world::cg_world::CgWorld;

/// Raven `CG_SiegeObjectiveBuffer` — looks up one team's `ObjectiveN` value-group text out of the loaded `.siege` file.
///
/// Raven backs the return value with a reused `static char buf[8192]`; we hand back an owned `String` instead of a pointer
/// into scratch storage the next call would stomp.
/// Source: `oracle/codemp/cgame/cg_saga.c:433-456`
pub fn CG_SiegeObjectiveBuffer(world: &CgWorld, team: c_int, objective: c_int) -> Option<String> {
    // PORT-NOTE: Raven copies the team name via `Com_sprintf(teamstr, sizeof(teamstr), team1)` - team1/team2 stand in AS
    // the format string with no variadic args, which is a plain copy for the team names these ever hold.
    let teamstr = if team == SIEGETEAM_TEAM1 {
        &world.saga.team1
    } else {
        &world.saga.team2
    };

    let siege_info = buf_to_string(&world.bg_state.siege_info);
    // found the team group
    let team_group = BG_SiegeGetValueGroup(&siege_info, teamstr)?;
    // found the objective group
    BG_SiegeGetValueGroup(&team_group, &format!("Objective{}", objective))
}

/// Raven `CG_SiegeGetObjectiveDescription` — resolves an objective's `goalname` display text.
///
/// The out-param `buffer` becomes the return value (§C7); Raven pre-zeroes `buffer[0]` so a lookup miss reads as an empty
/// string, which the early `String::new()` returns preserve.
/// Source: `oracle/codemp/cgame/cg_saga.c:667-691`
pub fn CG_SiegeGetObjectiveDescription(world: &CgWorld, team: c_int, objective: c_int) -> String {
    let teamstr = if team == SIEGETEAM_TEAM1 {
        &world.saga.team1
    } else {
        &world.saga.team2
    };

    let siege_info = buf_to_string(&world.bg_state.siege_info);
    // found the team group
    let Some(team_group) = BG_SiegeGetValueGroup(&siege_info, teamstr) else {
        return String::new();
    };
    // found the objective group
    let Some(objective_group) =
        BG_SiegeGetValueGroup(&team_group, &format!("Objective{}", objective))
    else {
        return String::new();
    };

    // parse the name right into the buffer
    BG_SiegeGetPairedValue(&objective_group, "goalname").unwrap_or_default()
}

/// Raven `CG_SiegeGetObjectiveFinal` — reads an objective's `final` paired value (whether completing it ends the round).
///
/// Source: `oracle/codemp/cgame/cg_saga.c:693-718`
pub fn CG_SiegeGetObjectiveFinal(world: &CgWorld, team: c_int, objective: c_int) -> c_int {
    let teamstr = if team == SIEGETEAM_TEAM1 {
        &world.saga.team1
    } else {
        &world.saga.team2
    };

    let siege_info = buf_to_string(&world.bg_state.siege_info);
    // found the team group
    let Some(team_group) = BG_SiegeGetValueGroup(&siege_info, teamstr) else {
        return 0;
    };
    // found the objective group
    let Some(objective_group) =
        BG_SiegeGetValueGroup(&team_group, &format!("Objective{}", objective))
    else {
        return 0;
    };

    // parse the name right into the buffer
    // Raven's `finalStr` is an uninitialized stack buffer; `BG_SiegeGetPairedValue`
    // returns 0 without touching it when "final" is missing, so `atoi(finalStr)`
    // reads garbage stack memory (UB). The port answers 0 deterministically via
    // the empty-string default instead (§F19).
    let final_str = BG_SiegeGetPairedValue(&objective_group, "final").unwrap_or_default();
    atoi(&final_str)
}

/// Raven `CG_ParseSiegeExtendedDataEntry` — decodes one `clNum|health|maxhealth|ammo` console string into
/// `cg_siegeExtendedData`, the per-client siege HUD cache.
///
/// Raven's hand-rolled `while`/`switch` walk is a plain `|`-delimited 4-field split; behavior preserved (fields past the
/// 4th are ignored, a short string just leaves the trailing fields at their `1`/`1`/`1` initializers).
/// Source: `oracle/codemp/cgame/cg_saga.c:987-1060`
pub fn CG_ParseSiegeExtendedDataEntry(world: &mut CgWorld, conStr: &str) {
    if conStr.is_empty() {
        return;
    }

    let mut clNum: c_int = -1;
    let mut health: c_int = 1;
    let mut maxhealth: c_int = 1;
    let mut ammo: c_int = 1;

    for (argParses, field) in conStr.split('|').take(4).enumerate() {
        match argParses {
            0 => clNum = atoi(field),
            1 => health = atoi(field),
            2 => maxhealth = atoi(field),
            3 => ammo = atoi(field),
            _ => {}
        }
    }

    if clNum < 0 || clNum >= MAX_CLIENTS_I32 {
        return;
    }
    let clNum = clNum as usize;

    world.saga.cg_siegeExtendedData[clNum].health = health;
    world.saga.cg_siegeExtendedData[clNum].maxhealth = maxhealth;
    world.saga.cg_siegeExtendedData[clNum].ammo = ammo;

    let cent_weapon = world.entities[clNum].currentState.weapon;
    let cent_eFlags = world.entities[clNum].currentState.eFlags;

    // `cent_weapon` comes off the network snapshot unclamped; an out-of-range
    // value reads past `weaponData`/`ammoData` in Raven (UB) where the port's
    // fixed-size array indexing panics instead (§F19).
    let mut maxAmmo = ammoData[weaponData[cent_weapon as usize].ammoIndex as usize].max;
    if cent_eFlags & EF_DOUBLE_AMMO != 0 {
        maxAmmo = (maxAmmo as f32 * 2.0) as i32;
    }

    if ammo >= 0 && ammo <= maxAmmo {
        // keep the weapon so if it changes before our next ext data update we'll know that the ammo is not applicable
        world.saga.cg_siegeExtendedData[clNum].weapon = cent_weapon;
    } else {
        // not valid? Oh well, just invalidate the weapon too then so we don't display ammo
        world.saga.cg_siegeExtendedData[clNum].weapon = -1;
    }

    world.saga.cg_siegeExtendedData[clNum].lastUpdated = world.cg.time;
}

/// Raven `CG_SetSiegeTimerCvar` — formats a round-clock `mins:tens_of_secs secs` string into `ui_siegeTimer`.
///
/// Source: `oracle/codemp/cgame/cg_saga.c:1081-1094`
pub fn CG_SetSiegeTimerCvar(ctx: &mut CgContext, msec: c_int) {
    let mut seconds = msec / 1000;
    let mins = seconds / 60;
    seconds -= mins * 60;
    let tens = seconds / 10;
    seconds -= tens * 10;

    trap::Cvar_Set(
        ctx.engine,
        "ui_siegeTimer",
        &format!("{}:{}{}", mins, tens, seconds),
    );
}
