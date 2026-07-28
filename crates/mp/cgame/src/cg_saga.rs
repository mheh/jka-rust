//! Port of `oracle/codemp/cgame/cg_saga.c` — siege/saga round state, objectives and briefing parsing. Functions land via the C5
//! transcription waves.

#![allow(non_snake_case)]

use core::ffi::c_int;

use mp_bg::bg_saga::{BG_SiegeFindThemeForTeam, BG_SiegeGetPairedValue, BG_SiegeGetValueGroup};
use mp_bg::public::entity_flags::EF_DOUBLE_AMMO;
use mp_bg::public::team::TEAM_SPECTATOR;
use mp_bg::saga::siege_team_t::{SIEGETEAM_TEAM1, SIEGETEAM_TEAM2};
use mp_bg::weapons::ammo_data::ammoData;
use mp_bg::weapons::weapon_data::weaponData;
use mp_qshared::common::mp::playerstate::PERS_TEAM;
use mp_qshared::shared::sound_channel::CHAN_ANNOUNCER;
use mp_qshared::shared::{MAX_CLIENTS_I32, MAX_QPATH};
use native_string::{atoi, buf_to_string, Q_strncpyz};

use crate::cg_draw::CG_DrawSiegeMessage;
use crate::cg_main::{CG_Argv, CG_Error};
use crate::cg_players::CG_LoadCISounds;
use crate::local::client_info_t::clientInfo_t;
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

/// Raven `CG_PrecacheSiegeObjectiveAssetsForTeam` — walks up to 32 `ObjectiveN` groups for `myTeam` out of the loaded
/// `.siege` file, registering each one's team-callout sounds and HUD/map-icon shaders.
/// Source: `oracle/codemp/cgame/cg_saga.c:35-99`
pub fn CG_PrecacheSiegeObjectiveAssetsForTeam(ctx: &mut CgContext, myTeam: c_int) {
    if ctx.world.bg_state.siege_valid == 0 {
        CG_Error(ctx, "Siege data does not exist on client!\n");
        return;
    }

    let teamstr = if myTeam == SIEGETEAM_TEAM1 {
        &ctx.world.saga.team1
    } else {
        &ctx.world.saga.team2
    };

    let siege_info = buf_to_string(&ctx.world.bg_state.siege_info);
    let Some(cgParseObjectives) = BG_SiegeGetValueGroup(&siege_info, teamstr) else {
        return;
    };

    let mut i = 1;
    while i < 32 {
        // eh, just try 32 I guess
        let objstr = format!("Objective{}", i);

        let Some(foundobjective) = BG_SiegeGetValueGroup(&cgParseObjectives, &objstr) else {
            // no more
            break;
        };

        if let Some(str) = BG_SiegeGetPairedValue(&foundobjective, "sound_team1") {
            trap::S_RegisterSound(ctx.engine, &str);
        }
        if let Some(str) = BG_SiegeGetPairedValue(&foundobjective, "sound_team2") {
            trap::S_RegisterSound(ctx.engine, &str);
        }
        if let Some(str) = BG_SiegeGetPairedValue(&foundobjective, "objgfx") {
            trap::R_RegisterShaderNoMip(ctx.engine, &str);
        }
        if let Some(str) = BG_SiegeGetPairedValue(&foundobjective, "mapicon") {
            trap::R_RegisterShaderNoMip(ctx.engine, &str);
        }
        if let Some(str) = BG_SiegeGetPairedValue(&foundobjective, "litmapicon") {
            trap::R_RegisterShaderNoMip(ctx.engine, &str);
        }
        if let Some(str) = BG_SiegeGetPairedValue(&foundobjective, "donemapicon") {
            trap::R_RegisterShaderNoMip(ctx.engine, &str);
        }

        i += 1;
    }
}

/// Raven `CG_PrecachePlayersForSiegeTeam` — for each class on `team`'s siege theme that forces a specific model,
/// register that model/skin and precache its sound set under a scratch `clientInfo_t`.
/// Source: `oracle/codemp/cgame/cg_saga.c:101-141`
pub fn CG_PrecachePlayersForSiegeTeam(ctx: &mut CgContext, team: c_int) {
    let stm = BG_SiegeFindThemeForTeam(team, &ctx.world.bg_state);

    if stm.is_null() {
        // invalid team/no theme for team?
        return;
    }

    let mut i = 0;
    // SAFETY: `stm` and `(*stm).classes[0..numClasses)` are non-null pointers into `bg_state.bgSiegeTeams`'s owned
    // storage - `BG_SiegeLoadClasses` panics before incrementing `numClasses` on any null slot (bg_saga.rs), so
    // every class in range is a valid `siegeClass_t`. Matches the established siegeTeam_t/siegeClass_t raw-pointer
    // walk (see g_saga.rs::G_SiegeRegisterWeaponsAndHoldables).
    unsafe {
        while i < (*stm).numClasses {
            // one explicit reference per class instead of autoref-ing through
            // the raw pointer at every field read
            let scl = &*(*stm).classes[i as usize];

            if !scl.forcedModel.is_empty() {
                // SAFETY: `clientInfo_t` is `#[repr(C)]` scalars, arrays, `qhandle_t`s and opaque ghoul2 `*mut
                // c_void` tokens; its two enum members (`team_t`, `gender_t`) both have a 0 discriminant, so
                // all-zero is a legal value - the same fill Raven gets from `memset(&fake, 0, sizeof(fake))`.
                let mut fake: clientInfo_t = core::mem::zeroed();
                Q_strncpyz(&mut fake.modelName, &scl.forcedModel, MAX_QPATH);

                trap::R_RegisterModel(
                    ctx.engine,
                    &format!("models/players/{}/model.glm", scl.forcedModel),
                );
                if !scl.forcedSkin.is_empty() {
                    trap::R_RegisterSkin(
                        ctx.engine,
                        &format!(
                            "models/players/{}/model_{}.skin",
                            scl.forcedModel, scl.forcedSkin
                        ),
                    );
                    Q_strncpyz(&mut fake.skinName, &scl.forcedSkin, MAX_QPATH);
                } else {
                    Q_strncpyz(&mut fake.skinName, "default", MAX_QPATH);
                }

                // precache the sounds for the model...
                CG_LoadCISounds(ctx, &mut fake, true);
            }

            i += 1;
        }
    }
}

/// Raven `CG_SiegeRoundOver` — plays the round-over message/sound announcing whether the local player's team won or
/// lost this siege round.
///
/// `ent` (Raven's `centity_t *ent` parameter) is never read in the oracle body; kept only to match the call surface.
/// Source: `oracle/codemp/cgame/cg_saga.c:568-665`
#[allow(unused_variables)]
pub fn CG_SiegeRoundOver(ctx: &mut CgContext, centNum: usize, won: c_int) {
    if ctx.world.bg_state.siege_valid == 0 {
        CG_Error(ctx, "ERROR: Siege data does not exist on client!\n");
        return;
    }

    // this should always be true, if it isn't though use the predicted ps as a fallback
    //
    // Raven's follow-up `if (!ps) { assert(0); return; }` null-guard can never fire (one of these two branches
    // always resolves a valid playerState_t; taking its address is never null in C either), so the port just
    // reads myTeam off whichever one it resolved to and drops the dead branch.
    let myTeam = match ctx.world.cg.snap_ref() {
        Some(snap) => snap.ps.persistant[PERS_TEAM as usize],
        None => ctx.world.cg.predictedPlayerState.persistant[PERS_TEAM as usize],
    };

    if myTeam == TEAM_SPECTATOR {
        return;
    }

    let teamstr = if myTeam == SIEGETEAM_TEAM1 {
        &ctx.world.saga.team1
    } else {
        &ctx.world.saga.team2
    };

    let siege_info = buf_to_string(&ctx.world.bg_state.siege_info);
    let Some(cgParseObjectives) = BG_SiegeGetValueGroup(&siege_info, teamstr) else {
        return;
    };

    let appstring = if won == myTeam {
        BG_SiegeGetPairedValue(&cgParseObjectives, "wonround")
    } else {
        BG_SiegeGetPairedValue(&cgParseObjectives, "lostround")
    };

    if let Some(appstring) = &appstring {
        CG_DrawSiegeMessage(ctx, appstring, 0);
    }

    let soundKey = if myTeam == won {
        "roundover_sound_wewon"
    } else {
        "roundover_sound_welost"
    };

    let soundstr = BG_SiegeGetPairedValue(&cgParseObjectives, soundKey);
    // Raven's commented-out `else` (falling back to DEFAULT_WIN_ROUND/DEFAULT_LOSE_ROUND) never compiled; a miss
    // here leaves `soundstr` unset and no sound plays, matching the shipped build.

    if let Some(soundstr) = soundstr {
        if !soundstr.is_empty() {
            let sfx = trap::S_RegisterSound(ctx.engine, &soundstr);
            trap::S_StartLocalSound(ctx.engine, sfx, CHAN_ANNOUNCER);
        }
    }
}

/// Raven `CG_SiegeBriefingDisplay` — walks up to 16 objective slots for `team`, mirroring each one's briefing text/
/// gfx/map data into the `siege_*`/`team*_objective*` menu cvars, then (unless `dontshow`) pops the team's full
/// briefing text via [`CG_DrawSiegeMessage`].
/// Source: `oracle/codemp/cgame/cg_saga.c:720-876`
pub fn CG_SiegeBriefingDisplay(ctx: &mut CgContext, team: c_int, dontshow: c_int) {
    if ctx.world.bg_state.siege_valid == 0 {
        return;
    }

    if team == TEAM_SPECTATOR {
        return;
    }

    let teamstr = if team == SIEGETEAM_TEAM1 {
        &ctx.world.saga.team1
    } else {
        &ctx.world.saga.team2
    };

    // This shouldn't be happening. But just fall back to team 2 anyway.
    let useTeam = if team != SIEGETEAM_TEAM1 && team != SIEGETEAM_TEAM2 {
        SIEGETEAM_TEAM2
    } else {
        team
    };

    trap::Cvar_Set(ctx.engine, "siege_primobj_inuse", "0");

    let mut i = 1;
    while i < 16 {
        // do up to 16 objectives I suppose
        // Get the value for this objective on this team. Now set the cvar for the menu to display.
        let primary = CG_SiegeGetObjectiveFinal(ctx.world, useTeam, i) > 0;

        let properValue = trap::Cvar_VariableStringBuffer(
            ctx.engine,
            &format!("team{}_objective{}", useTeam, i),
            1024,
        );
        if primary {
            trap::Cvar_Set(ctx.engine, "siege_primobj", &properValue);
        } else {
            trap::Cvar_Set(ctx.engine, &format!("siege_objective{}", i), &properValue);
        }

        // Now set the long desc cvar for the menu to display.
        let properValue = trap::Cvar_VariableStringBuffer(
            ctx.engine,
            &format!("team{}_objective{}_longdesc", useTeam, i),
            1024,
        );
        if primary {
            trap::Cvar_Set(ctx.engine, "siege_primobj_longdesc", &properValue);
        } else {
            trap::Cvar_Set(
                ctx.engine,
                &format!("siege_objective{}_longdesc", i),
                &properValue,
            );
        }

        // Now set the gfx cvar for the menu to display.
        let properValue = trap::Cvar_VariableStringBuffer(
            ctx.engine,
            &format!("team{}_objective{}_gfx", useTeam, i),
            1024,
        );
        if primary {
            trap::Cvar_Set(ctx.engine, "siege_primobj_gfx", &properValue);
        } else {
            trap::Cvar_Set(
                ctx.engine,
                &format!("siege_objective{}_gfx", i),
                &properValue,
            );
        }

        // Now set the mapicon cvar for the menu to display.
        let properValue = trap::Cvar_VariableStringBuffer(
            ctx.engine,
            &format!("team{}_objective{}_mapicon", useTeam, i),
            1024,
        );
        if primary {
            trap::Cvar_Set(ctx.engine, "siege_primobj_mapicon", &properValue);
        } else {
            trap::Cvar_Set(
                ctx.engine,
                &format!("siege_objective{}_mapicon", i),
                &properValue,
            );
        }

        // Now set the mappos cvar for the menu to display.
        let properValue = trap::Cvar_VariableStringBuffer(
            ctx.engine,
            &format!("team{}_objective{}_mappos", useTeam, i),
            1024,
        );
        if primary {
            trap::Cvar_Set(ctx.engine, "siege_primobj_mappos", &properValue);
        } else {
            trap::Cvar_Set(
                ctx.engine,
                &format!("siege_objective{}_mappos", i),
                &properValue,
            );
        }

        // Now set the description cvar for the objective.
        let objectiveDesc = CG_SiegeGetObjectiveDescription(ctx.world, useTeam, i);

        if !objectiveDesc.is_empty() {
            // found a valid objective description
            if primary {
                trap::Cvar_Set(ctx.engine, "siege_primobj_desc", &objectiveDesc);
                // this one is marked not in use because it gets primobj
                trap::Cvar_Set(ctx.engine, &format!("siege_objective{}_inuse", i), "0");
                trap::Cvar_Set(ctx.engine, "siege_primobj_inuse", "1");
                trap::Cvar_Set(
                    ctx.engine,
                    &format!("team{}_objective{}_inuse", useTeam, i),
                    "1",
                );
            } else {
                trap::Cvar_Set(
                    ctx.engine,
                    &format!("siege_objective{}_desc", i),
                    &objectiveDesc,
                );
                trap::Cvar_Set(ctx.engine, &format!("siege_objective{}_inuse", i), "2");
                trap::Cvar_Set(
                    ctx.engine,
                    &format!("team{}_objective{}_inuse", useTeam, i),
                    "2",
                );
            }
        } else {
            // didn't find one, so set the "inuse" cvar to 0 for the objective and mark it non-complete.
            trap::Cvar_Set(ctx.engine, &format!("siege_objective{}_inuse", i), "0");
            trap::Cvar_Set(ctx.engine, &format!("siege_objective{}", i), "0");
            trap::Cvar_Set(
                ctx.engine,
                &format!("team{}_objective{}_inuse", useTeam, i),
                "0",
            );
            trap::Cvar_Set(ctx.engine, &format!("team{}_objective{}", useTeam, i), "0");

            trap::Cvar_Set(ctx.engine, &format!("siege_objective{}_mappos", i), "");
            trap::Cvar_Set(
                ctx.engine,
                &format!("team{}_objective{}_mappos", useTeam, i),
                "",
            );
            trap::Cvar_Set(ctx.engine, &format!("siege_objective{}_gfx", i), "");
            trap::Cvar_Set(
                ctx.engine,
                &format!("team{}_objective{}_gfx", useTeam, i),
                "",
            );
            trap::Cvar_Set(ctx.engine, &format!("siege_objective{}_mapicon", i), "");
            trap::Cvar_Set(
                ctx.engine,
                &format!("team{}_objective{}_mapicon", useTeam, i),
                "",
            );
        }

        i += 1;
    }

    if dontshow != 0 {
        return;
    }

    let siege_info = buf_to_string(&ctx.world.bg_state.siege_info);
    if let Some(cgParseObjectives) = BG_SiegeGetValueGroup(&siege_info, teamstr) {
        if let Some(briefing) = BG_SiegeGetPairedValue(&cgParseObjectives, "briefing") {
            CG_DrawSiegeMessage(ctx, &briefing, 1);
        }
    }
}

/// Raven `CG_ParseSiegeExtendedData` — the `sxd` server command handler; each console arg is one client's
/// `clNum|health|maxhealth|ammo` string, decoded via [`CG_ParseSiegeExtendedDataEntry`].
/// Source: `oracle/codemp/cgame/cg_saga.c:1063-1079`
pub fn CG_ParseSiegeExtendedData(ctx: &mut CgContext) {
    let numEntries = trap::Argc(ctx.engine);

    if numEntries < 1 {
        debug_assert!(false, "Bad numEntries for sxd");
        return;
    }

    let mut i = 0;
    while i < numEntries {
        let argv = CG_Argv(ctx, i + 1);
        CG_ParseSiegeExtendedDataEntry(ctx.world, &argv);
        i += 1;
    }
}
