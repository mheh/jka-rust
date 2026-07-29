//! Port of `oracle/codemp/cgame/cg_saga.c` — siege/saga round state, objectives and briefing parsing. Functions land via the C5
//! transcription waves.

#![allow(non_snake_case)]

use core::ffi::{c_char, c_int};
use core::ptr::null_mut;

use mp_bg::bg_saga::{
    BG_PrecacheSabersForSiegeTeam, BG_SiegeFindThemeForTeam, BG_SiegeGetPairedValue,
    BG_SiegeGetValueGroup, BG_SiegeLoadClasses, BG_SiegeLoadTeams, BG_SiegeSetTeamTheme,
};
use mp_bg::public::entity_flags::EF_DOUBLE_AMMO;
use mp_bg::public::gametype::GT_SIEGE;
use mp_bg::public::team::TEAM_SPECTATOR;
use mp_bg::saga::siege_team_t::{MAX_SIEGE_INFO_SIZE, SIEGETEAM_TEAM1, SIEGETEAM_TEAM2};
use mp_bg::weapons::ammo_data::ammoData;
use mp_bg::weapons::weapon_data::weaponData;
use mp_qshared::common::mp::playerstate::PERS_TEAM;
use mp_qshared::shared::sound_channel::CHAN_ANNOUNCER;
use mp_qshared::shared::{fileHandle_t, FS_READ, MAX_CLIENTS_I32, MAX_QPATH};
use native_string::{atoi, buf_to_string, Q_stricmp, Q_strncpyz};

use crate::bg_channel::{CgBgTraps, CgGameCallbacks};
use crate::cg_draw::{CG_DrawSiegeMessage, CG_DrawSiegeMessageNonMenu};
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

/// Raven `CG_InitSiegeMode` — loads the map's `.siege` file into `siege_info`, resolves both team names and their
/// timers/icons/shaders into the menu cvars, loads the class + team tables, then precaches every forced model, skin,
/// saber and objective asset the two themes use.
///
/// Raven's `goto failure` tail is the labeled block below: every bail lands on `siege_valid = 0`, while the success
/// path returns. The `CG_Error` arms long-jump in Raven, so each one gets an explicit `return` here.
/// Source: `oracle/codemp/cgame/cg_saga.c:143-431`
pub fn CG_InitSiegeMode(ctx: &mut CgContext) {
    let mut f: fileHandle_t = 0;

    'failure: {
        if ctx.world.cgs.gametype != GT_SIEGE {
            break 'failure;
        }

        // Raven's `Com_sprintf` into `levelname[MAX_QPATH]` truncates at 63 chars.
        let mut levelname: String = buf_to_string(&ctx.world.cgs.mapname.map(|c| c as u8))
            .chars()
            .take(MAX_QPATH - 1)
            .collect();

        // walk back from the end to the '.' that starts the ".bsp"; Raven's extra
        // `levelname[i]` non-NUL test can't fire on a NUL-terminated copy
        let i = {
            let bytes = levelname.as_bytes();
            let mut i = bytes.len() as isize - 1;
            while i > 0 && bytes[i as usize] != b'.' {
                i -= 1;
            }
            i
        };

        // An empty mapname leaves Raven at i == -1, which slips past `if (!i)` and writes
        // `levelname[-1] = '\0'` (§F19 UB); the port takes the failure path for it too.
        if i <= 0 {
            break 'failure;
        }

        levelname.truncate(i as usize); //kill the ".bsp"

        let levelname: String = format!("{}.siege", levelname)
            .chars()
            .take(MAX_QPATH - 1)
            .collect();

        if levelname.is_empty() {
            break 'failure;
        }

        let len = trap::FS_FOpenFile(ctx.engine, &levelname, &mut f, FS_READ);

        // PORT-NOTE: the over-long-file bail leaks `f` - Raven never closes it here.
        if f == 0 || len >= MAX_SIEGE_INFO_SIZE {
            break 'failure;
        }

        // PORT-NOTE: unlike ui_main.c's copy of this read, cgame never writes the terminating
        // NUL after `len` bytes, so a shorter `.siege` file leaves the last map's tail behind.
        trap::FS_Read(
            ctx.engine,
            &mut ctx.world.bg_state.siege_info[..len as usize],
            f,
        );

        trap::FS_FCloseFile(ctx.engine, f);

        ctx.world.bg_state.siege_valid = 1;

        let siege_info = buf_to_string(&ctx.world.bg_state.siege_info);

        if let Some(teams) = BG_SiegeGetValueGroup(&siege_info, "Teams") {
            let buf = trap::Cvar_VariableStringBuffer(ctx.engine, "cg_siegeTeam1", 1024);
            if !buf.is_empty() && Q_stricmp(&buf, "none") != 0 {
                ctx.world.saga.team1 = buf;
            } else if let Some(team1) = BG_SiegeGetPairedValue(&teams, "team1") {
                // a miss leaves team1 holding whatever the last map put there (Raven's static)
                ctx.world.saga.team1 = team1;
            }

            if ctx.world.saga.team1.starts_with('@') {
                //it's a damn stringed reference.
                let b = trap::SP_GetStringTextString(ctx.engine, &ctx.world.saga.team1[1..], 256)
                    .unwrap_or_else(|| format!("??{}", &ctx.world.saga.team1[1..]));
                trap::Cvar_Set(ctx.engine, "cg_siegeTeam1Name", &b);
            } else {
                trap::Cvar_Set(ctx.engine, "cg_siegeTeam1Name", &ctx.world.saga.team1);
            }

            let buf = trap::Cvar_VariableStringBuffer(ctx.engine, "cg_siegeTeam2", 1024);
            if !buf.is_empty() && Q_stricmp(&buf, "none") != 0 {
                ctx.world.saga.team2 = buf;
            } else if let Some(team2) = BG_SiegeGetPairedValue(&teams, "team2") {
                ctx.world.saga.team2 = team2;
            }

            if ctx.world.saga.team2.starts_with('@') {
                //it's a damn stringed reference.
                let b = trap::SP_GetStringTextString(ctx.engine, &ctx.world.saga.team2[1..], 256)
                    .unwrap_or_else(|| format!("??{}", &ctx.world.saga.team2[1..]));
                trap::Cvar_Set(ctx.engine, "cg_siegeTeam2Name", &b);
            } else {
                trap::Cvar_Set(ctx.engine, "cg_siegeTeam2Name", &ctx.world.saga.team2);
            }
        } else {
            CG_Error(ctx, "Siege teams not defined");
            return;
        }

        let team1 = ctx.world.saga.team1.clone();
        let team2 = ctx.world.saga.team2.clone();

        if let Some(teamInfo) = BG_SiegeGetValueGroup(&siege_info, &team1) {
            if let Some(teamIcon) = BG_SiegeGetPairedValue(&teamInfo, "TeamIcon") {
                trap::Cvar_Set(ctx.engine, "team1_icon", &teamIcon);
            }

            if let Some(btime) = BG_SiegeGetPairedValue(&teamInfo, "Timed") {
                let team1Timed = atoi(&btime) * 1000;
                ctx.world.saga.team1Timed = team1Timed;
                CG_SetSiegeTimerCvar(ctx, team1Timed);
            } else {
                ctx.world.saga.team1Timed = 0;
            }
        } else {
            CG_Error(ctx, &format!("No team entry for '{}'\n", team1));
            return;
        }

        if let Some(teamInfo) = BG_SiegeGetPairedValue(&siege_info, "mapgraphic") {
            trap::Cvar_Set(ctx.engine, "siege_mapgraphic", &teamInfo);
        } else {
            trap::Cvar_Set(ctx.engine, "siege_mapgraphic", "gfx/mplevels/siege1_hoth");
        }

        if let Some(teamInfo) = BG_SiegeGetPairedValue(&siege_info, "missionname") {
            trap::Cvar_Set(ctx.engine, "siege_missionname", &teamInfo);
        } else {
            trap::Cvar_Set(ctx.engine, "siege_missionname", " ");
        }

        if let Some(teamInfo) = BG_SiegeGetValueGroup(&siege_info, &team2) {
            if let Some(teamIcon) = BG_SiegeGetPairedValue(&teamInfo, "TeamIcon") {
                trap::Cvar_Set(ctx.engine, "team2_icon", &teamIcon);
            }

            if let Some(btime) = BG_SiegeGetPairedValue(&teamInfo, "Timed") {
                let team2Timed = atoi(&btime) * 1000;
                ctx.world.saga.team2Timed = team2Timed;
                CG_SetSiegeTimerCvar(ctx, team2Timed);
            } else {
                ctx.world.saga.team2Timed = 0;
            }
        } else {
            CG_Error(ctx, &format!("No team entry for '{}'\n", team2));
            return;
        }

        //Load the player class types
        {
            let traps = CgBgTraps::new(ctx.engine, ctx.world_raw());
            let mut callbacks = CgGameCallbacks::new(ctx.engine, ctx.world_raw());
            BG_SiegeLoadClasses(null_mut(), &mut ctx.world.bg_state, &traps, &mut callbacks);
        }

        if ctx.world.bg_state.bgNumSiegeClasses == 0 {
            //We didn't find any?!
            CG_Error(ctx, "Couldn't find any player classes for Siege");
            return;
        }

        //Now load the teams since we have class data.
        {
            let traps = CgBgTraps::new(ctx.engine, ctx.world_raw());
            BG_SiegeLoadTeams(&mut ctx.world.bg_state, &traps);
        }

        if ctx.world.bg_state.bgNumSiegeTeams == 0 {
            //React same as with classes.
            CG_Error(ctx, "Couldn't find any player teams for Siege");
            return;
        }

        //Get and set the team themes for each team. This will control which classes can be
        //used on each team.
        if let Some(teamInfo) = BG_SiegeGetValueGroup(&siege_info, &team1) {
            if let Some(btime) = BG_SiegeGetPairedValue(&teamInfo, "UseTeam") {
                let mut buf: [c_char; 1024] = [0; 1024];
                Q_strncpyz(&mut buf, &btime, 1024);
                BG_SiegeSetTeamTheme(SIEGETEAM_TEAM1, buf.as_mut_ptr(), &mut ctx.world.bg_state);
            }
            if let Some(btime) = BG_SiegeGetPairedValue(&teamInfo, "FriendlyShader") {
                let shader = trap::R_RegisterShaderNoMip(ctx.engine, &btime);
                ctx.world.saga.cgSiegeTeam1PlShader = shader;
            } else {
                ctx.world.saga.cgSiegeTeam1PlShader = 0;
            }
        }
        if let Some(teamInfo) = BG_SiegeGetValueGroup(&siege_info, &team2) {
            if let Some(btime) = BG_SiegeGetPairedValue(&teamInfo, "UseTeam") {
                let mut buf: [c_char; 1024] = [0; 1024];
                Q_strncpyz(&mut buf, &btime, 1024);
                BG_SiegeSetTeamTheme(SIEGETEAM_TEAM2, buf.as_mut_ptr(), &mut ctx.world.bg_state);
            }
            if let Some(btime) = BG_SiegeGetPairedValue(&teamInfo, "FriendlyShader") {
                let shader = trap::R_RegisterShaderNoMip(ctx.engine, &btime);
                ctx.world.saga.cgSiegeTeam2PlShader = shader;
            } else {
                ctx.world.saga.cgSiegeTeam2PlShader = 0;
            }
        }

        //Now go through the classes used by the loaded teams and try to precache
        //any forced models or forced skins.
        let mut i = SIEGETEAM_TEAM1;

        while i <= SIEGETEAM_TEAM2 {
            let sTeam = BG_SiegeFindThemeForTeam(i, &ctx.world.bg_state);

            if sTeam.is_null() {
                i += 1;
                continue;
            }

            // SAFETY: `sTeam` and `(*sTeam).classes[0..numClasses)` are non-null pointers into
            // `bg_state.bgSiegeTeams`'s owned storage - the same walk `CG_PrecachePlayersForSiegeTeam`
            // above does, and nothing here mutates that storage while the borrow is live.
            unsafe {
                //Get custom team shaders while we're at it.
                if i == SIEGETEAM_TEAM1 {
                    ctx.world.saga.cgSiegeTeam1PlShader = (*sTeam).friendlyShader;
                } else if i == SIEGETEAM_TEAM2 {
                    ctx.world.saga.cgSiegeTeam2PlShader = (*sTeam).friendlyShader;
                }

                let mut j = 0;
                while j < (*sTeam).numClasses {
                    let cl = &*(*sTeam).classes[j as usize];

                    if !cl.forcedModel.is_empty() {
                        //This class has a forced model, so precache it.
                        trap::R_RegisterModel(
                            ctx.engine,
                            &format!("models/players/{}/model.glm", cl.forcedModel),
                        );

                        if !cl.forcedSkin.is_empty() {
                            //also has a forced skin, precache it.
                            let useSkinName = if cl.forcedSkin.contains('|') {
                                //three part skin
                                format!("models/players/{}/|{}", cl.forcedModel, cl.forcedSkin)
                            } else {
                                format!(
                                    "models/players/{}/model_{}.skin",
                                    cl.forcedModel, cl.forcedSkin
                                )
                            };

                            trap::R_RegisterSkin(ctx.engine, &useSkinName);
                        }
                    }

                    j += 1;
                }
            }
            i += 1;
        }

        //precache saber data for classes that use sabers on both teams
        {
            let traps = CgBgTraps::new(ctx.engine, ctx.world_raw());
            let mut callbacks = CgGameCallbacks::new(ctx.engine, ctx.world_raw());
            BG_PrecacheSabersForSiegeTeam(
                SIEGETEAM_TEAM1,
                &mut ctx.world.bg_state,
                &traps,
                &mut callbacks,
            );
            BG_PrecacheSabersForSiegeTeam(
                SIEGETEAM_TEAM2,
                &mut ctx.world.bg_state,
                &traps,
                &mut callbacks,
            );
        }

        CG_PrecachePlayersForSiegeTeam(ctx, SIEGETEAM_TEAM1);
        CG_PrecachePlayersForSiegeTeam(ctx, SIEGETEAM_TEAM2);

        // PORT-NOTE: Raven precaches both teams' players a second time right here - kept as written.
        CG_PrecachePlayersForSiegeTeam(ctx, SIEGETEAM_TEAM1);
        CG_PrecachePlayersForSiegeTeam(ctx, SIEGETEAM_TEAM2);

        CG_PrecacheSiegeObjectiveAssetsForTeam(ctx, SIEGETEAM_TEAM1);
        CG_PrecacheSiegeObjectiveAssetsForTeam(ctx, SIEGETEAM_TEAM2);

        return;
    }

    // failure:
    ctx.world.bg_state.siege_valid = 0;
}

/// Raven `CG_ParseSiegeObjectiveStatus` — decodes the `CS_SIEGE_OBJECTIVES` configstring
/// (`t1-0-1|t2-0-0` style) into the per-objective completion cvars, mirroring each objective's
/// description/graphic/map data alongside, then refreshes the briefing menu for the local team.
/// Source: `oracle/codemp/cgame/cg_saga.c:458-566`
pub fn CG_ParseSiegeObjectiveStatus(ctx: &mut CgContext, str: &str) {
    if str.is_empty() {
        return;
    }

    let bytes = str.as_bytes();
    // A trailing '-' pushes Raven's index one past the terminator and the next loop test then
    // reads out of bounds (§F19 UB); reads at or past the end answer 0 here, ending the walk.
    let at = |idx: usize| -> u8 {
        if idx < bytes.len() {
            bytes[idx]
        } else {
            0
        }
    };

    let mut i: usize = 0;
    let mut team = SIEGETEAM_TEAM1;
    let mut objectiveNum: c_int = 0;

    while at(i) != 0 {
        if at(i) == b'|' {
            //switch over to team2, this is the next section
            team = SIEGETEAM_TEAM2;
            objectiveNum = 0;
        } else if at(i) == b'-' {
            objectiveNum += 1;
            i += 1;

            let cvarName = format!("team{}_objective{}", team, objectiveNum);
            if at(i) == b'1' {
                //it's completed
                trap::Cvar_Set(ctx.engine, &cvarName, "1");
            } else {
                //otherwise assume it is not
                trap::Cvar_Set(ctx.engine, &cvarName, "0");
            }

            let s = CG_SiegeObjectiveBuffer(ctx.world, team, objectiveNum);
            if let Some(s) = s.filter(|s| !s.is_empty()) {
                //now set the description and graphic cvars to by read by the menu
                let cvarName = format!("team{}_objective{}_longdesc", team, objectiveNum);
                if let Some(buffer) = BG_SiegeGetPairedValue(&s, "objdesc") {
                    trap::Cvar_Set(ctx.engine, &cvarName, &buffer);
                } else {
                    trap::Cvar_Set(ctx.engine, &cvarName, "UNSPECIFIED");
                }

                let cvarName = format!("team{}_objective{}_gfx", team, objectiveNum);
                if let Some(buffer) = BG_SiegeGetPairedValue(&s, "objgfx") {
                    trap::Cvar_Set(ctx.engine, &cvarName, &buffer);
                } else {
                    trap::Cvar_Set(ctx.engine, &cvarName, "UNSPECIFIED");
                }

                let cvarName = format!("team{}_objective{}_mapicon", team, objectiveNum);
                if let Some(buffer) = BG_SiegeGetPairedValue(&s, "mapicon") {
                    trap::Cvar_Set(ctx.engine, &cvarName, &buffer);
                } else {
                    trap::Cvar_Set(ctx.engine, &cvarName, "UNSPECIFIED");
                }

                let cvarName = format!("team{}_objective{}_litmapicon", team, objectiveNum);
                if let Some(buffer) = BG_SiegeGetPairedValue(&s, "litmapicon") {
                    trap::Cvar_Set(ctx.engine, &cvarName, &buffer);
                } else {
                    trap::Cvar_Set(ctx.engine, &cvarName, "UNSPECIFIED");
                }

                let cvarName = format!("team{}_objective{}_donemapicon", team, objectiveNum);
                if let Some(buffer) = BG_SiegeGetPairedValue(&s, "donemapicon") {
                    trap::Cvar_Set(ctx.engine, &cvarName, &buffer);
                } else {
                    trap::Cvar_Set(ctx.engine, &cvarName, "UNSPECIFIED");
                }

                let cvarName = format!("team{}_objective{}_mappos", team, objectiveNum);
                if let Some(buffer) = BG_SiegeGetPairedValue(&s, "mappos") {
                    trap::Cvar_Set(ctx.engine, &cvarName, &buffer);
                } else {
                    trap::Cvar_Set(ctx.engine, &cvarName, "0 0 32 32");
                }
            }
        }
        i += 1;
    }

    let myTeam = ctx.world.cg.predictedPlayerState.persistant[PERS_TEAM as usize];
    if myTeam != TEAM_SPECTATOR {
        //update menu cvars
        CG_SiegeBriefingDisplay(ctx, myTeam, 1);
    }
}

/// Raven `CG_SiegeObjectiveCompleted` — pops the objective-completed message and plays the team's
/// callout sound for whichever side the local player is on.
///
/// `ent` (Raven's `centity_t *ent` parameter) is never read in the oracle body; kept only to match the call surface.
/// Source: `oracle/codemp/cgame/cg_saga.c:878-982`
#[allow(unused_variables)]
pub fn CG_SiegeObjectiveCompleted(
    ctx: &mut CgContext,
    centNum: usize,
    won: c_int,
    objectivenum: c_int,
) {
    if ctx.world.bg_state.siege_valid == 0 {
        CG_Error(ctx, "Siege data does not exist on client!\n");
        return;
    }

    // this should always be true, if it isn't though use the predicted ps as a fallback
    //
    // Raven's follow-up `if (!ps) { assert(0); return; }` null-guard can never fire (one of these two branches
    // always resolves a valid playerState_t), so the port reads myTeam off whichever one it resolved to.
    let myTeam = match ctx.world.cg.snap_ref() {
        Some(snap) => snap.ps.persistant[PERS_TEAM as usize],
        None => ctx.world.cg.predictedPlayerState.persistant[PERS_TEAM as usize],
    };

    if myTeam == TEAM_SPECTATOR {
        return;
    }

    // PORT-NOTE: Raven copies the team name via `Com_sprintf(teamstr, sizeof(teamstr), team1)` - team1/team2 stand in
    // AS the format string with no variadic args, a plain copy for these names, truncated to `teamstr[64]`'s 63 chars.
    let teamstr: String = if won == SIEGETEAM_TEAM1 {
        ctx.world.saga.team1.chars().take(63).collect()
    } else {
        ctx.world.saga.team2.chars().take(63).collect()
    };

    let siege_info = buf_to_string(&ctx.world.bg_state.siege_info);
    let Some(cgParseObjectives) = BG_SiegeGetValueGroup(&siege_info, &teamstr) else {
        return;
    };

    let objstr = format!("Objective{}", objectivenum);

    let Some(foundobjective) = BG_SiegeGetValueGroup(&cgParseObjectives, &objstr) else {
        return;
    };

    let appstring = if myTeam == SIEGETEAM_TEAM1 {
        BG_SiegeGetPairedValue(&foundobjective, "message_team1")
    } else {
        BG_SiegeGetPairedValue(&foundobjective, "message_team2")
    };

    if let Some(appstring) = &appstring {
        CG_DrawSiegeMessageNonMenu(ctx, appstring);
    }

    let teamstr = if myTeam == SIEGETEAM_TEAM1 {
        "sound_team1"
    } else {
        "sound_team2"
    };

    let soundstr = BG_SiegeGetPairedValue(&foundobjective, teamstr);
    // Raven's commented-out `else` (falling back to DEFAULT_WIN/LOSE_OBJECTIVE) never compiled; a miss here
    // leaves `soundstr` unset and no sound plays, matching the shipped build.

    if let Some(soundstr) = soundstr {
        if !soundstr.is_empty() {
            let sfx = trap::S_RegisterSound(ctx.engine, &soundstr);
            trap::S_StartLocalSound(ctx.engine, sfx, CHAN_ANNOUNCER);
        }
    }
}
