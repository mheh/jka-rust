//! FAITHFUL port of `oracle/codemp/game/g_saga.c`, the Siege gametype game-side module.
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;

use crate::bg_channel::GameCallbacksImpl;
use crate::client::gclient::gclient_t;
use crate::client::player_team_state::playerTeamStateState_t;
use crate::client::spectator_state::spectatorState_t;
use crate::client::CON_CONNECTED;
use crate::g_client::{ClientBegin, ClientSpawn, ClientUserinfoChanged};
use crate::g_combat::AddScore;
use crate::g_exphysics::G_RunExPhys;
use crate::g_items::RegisterItem;
use crate::g_main::{Com_Printf, G_Error, LogExit};
use crate::g_utils::{
    G_EffectIndex, G_Find, G_IconIndex, G_PlayEffectID, G_SetOrigin, G_Sound, G_TempEntity,
    G_UseTargets2, GlobalUse,
};
use crate::q_shared;
use crate::q_shared::Info_SetValueForKey;
use mp_bg::bg_misc::{BG_FindItemForHoldable, BG_FindItemForWeapon};
use mp_bg::bg_saga::{
    BG_PrecacheSabersForSiegeTeam, BG_SiegeFindClassIndexByName, BG_SiegeFindThemeForTeam,
    BG_SiegeGetPairedValue, BG_SiegeGetValueGroup, BG_SiegeLoadClasses, BG_SiegeLoadTeams,
    BG_SiegeSetTeamTheme,
};
use mp_bg::public::entity_event::entity_event_t;
use mp_bg::public::holdable::HI_NUM_HOLDABLE;
use mp_bg::weapons::weapon_t::WP_NUM_WEAPONS;
use mp_qshared::shared::surface_flags::{CONTENTS_SOLID, CONTENTS_TERRAIN};
use native_string::latin1_to_string;
use native_string::strncpyz_string;
use native_string::{atoi_bytes, buf_to_string, strcat_string, Q_stricmp, Q_strncpyzBytes};

// Raven `qboolean` is `c_int`. Keep the source spelling at call sites.
// Source: `oracle/codemp/game/q_shared.h`

/// Raven `SIEGEITEM_STARTOFFRADAR`, a spawnflag. A siege item with this flag starts off the team radar.
///
/// Raven: file-local `#define` in `g_saga.c`, not a `bg_saga.h` header const.
/// Source: `oracle/codemp/game/g_saga.c:15`
const SIEGEITEM_STARTOFFRADAR: c_int = 8;

/// Raven `SIEGE_ITEM_RESPAWN_TIME`.
///
/// Raven: file-local `#define` in `g_saga.c`, not a `bg_saga.h` header const.
/// Source: `oracle/codemp/game/g_saga.c:1336`
const SIEGE_ITEM_RESPAWN_TIME: c_int = 20000;

/// Raven `G_SiegeRegisterWeaponsAndHoldables`.
///
/// Raven: go through all classes on a team and register their weapons and items for precaching.
///
/// Source: `oracle/codemp/game/g_saga.c:51-87`
pub fn G_SiegeRegisterWeaponsAndHoldables(ctx: &mut GameContext, team: c_int) {
    unsafe {
        let stm = BG_SiegeFindThemeForTeam(team, &ctx.world.bg_state);

        if !stm.is_null() {
            let mut i = 0;
            while i < (*stm).numClasses {
                let scl = (*stm).classes[i as usize];

                if !scl.is_null() {
                    let mut j = 0;
                    while j < WP_NUM_WEAPONS {
                        if (*scl).weapons & (1 << j) != 0 {
                            // we use this weapon so register it.
                            RegisterItem(ctx, BG_FindItemForWeapon(j));
                        }
                        j += 1;
                    }
                    j = 0;
                    while j < HI_NUM_HOLDABLE {
                        if (*scl).invenItems & (1 << j) != 0 {
                            // we use this item so register it.
                            RegisterItem(ctx, BG_FindItemForHoldable(j));
                        }
                        j += 1;
                    }
                }
                i += 1;
            }
        }
    }
}

/// Raven `SiegeSetCompleteData`.
///
/// Raven: tell clients that this team won and print it on their scoreboard for intermission or whatever.
///
/// Source: `oracle/codemp/game/g_saga.c:91-94`
pub fn SiegeSetCompleteData(ctx: &mut GameContext, team: c_int) {
    trap::SetConfigstring(ctx.engine, CS_SIEGE_WINTEAM, &format!("{}", team));
}

/// Raven `InitSiegeMode`.
///
/// Source: `oracle/codemp/game/g_saga.c:96-373`
pub fn InitSiegeMode(ctx: &mut GameContext) {
    unsafe {
        let mut mapname: vmCvar_t = vmCvar_t::zeroed();
        let mut teamIcon: [c_char; 128] = [0; 128];
        let mut goalreq: [c_char; 64] = [0; 64];
        let mut teams: [c_char; 2048] = [0; 2048];
        let mut objecStr: [c_char; 8192] = [0; 8192];
        let teamIcon_len = teamIcon.len();
        let goalreq_len = goalreq.len();
        let teams_len = teams.len();
        let mut len: c_int = 0;
        let mut objectiveNumTeam1: c_int = 0;
        let mut objectiveNumTeam2: c_int = 0;
        let mut f: fileHandle_t = 0;

        'body: {
            if ctx.world.cvars.g_gametype.integer != GT_SIEGE {
                break 'body;
            }

            // reset
            SiegeSetCompleteData(ctx, 0);

            // get pers data in case it existed from last level
            if ctx.world.cvars.g_siegeTeamSwitch.integer != 0 {
                trap::SiegePersGet(
                    ctx.engine,
                    mp_abi::game::syscalls::G_SIEGEPERSGET::GSiegepersgetArgs::new(
                        &mut ctx.world.globals.g_siegePersistant as *mut siegePers_t,
                    ),
                );
                if ctx.world.globals.g_siegePersistant.beatingTime != 0 {
                    trap::SetConfigstring(
                        ctx.engine,
                        CS_SIEGE_TIMEOVERRIDE,
                        &format!("{}", ctx.world.globals.g_siegePersistant.lastTime),
                    );
                } else {
                    trap::SetConfigstring(ctx.engine, CS_SIEGE_TIMEOVERRIDE, "0");
                }
            } else {
                // hmm, ok, nothing.
                trap::SetConfigstring(ctx.engine, CS_SIEGE_TIMEOVERRIDE, "0");
            }

            ctx.world.globals.imperial_goals_completed = 0;
            ctx.world.globals.rebel_goals_completed = 0;

            trap::Cvar_Register(
                ctx.engine,
                Some(&mut mapname),
                "mapname",
                "",
                CVAR_SERVERINFO | CVAR_ROM,
            );

            let levelname_s = format!(
                "maps/{}.siege\0",
                latin1_to_string(cstr_from_chars(&mapname.string).to_bytes())
            );

            if levelname_s.is_empty() || levelname_s.as_bytes()[0] == 0 {
                break 'body;
            }

            let levelname_c = levelname_s.trim_end_matches('\0');
            len = trap::FS_FOpenFile(ctx.engine, levelname_c, &mut f, FS_READ);

            if f == 0 || len >= MAX_SIEGE_INFO_SIZE {
                break 'body;
            }

            trap::FS_Read(
                ctx.engine,
                &mut ctx.world.bg_state.siege_info[..len as usize],
                f,
            );

            trap::FS_FCloseFile(ctx.engine, f);

            ctx.world.bg_state.siege_valid = 1;

            // See if players should be specs or ingame preround
            if let Some(val) = BG_SiegeGetPairedValue(
                &buf_to_string(&ctx.world.bg_state.siege_info),
                "preround_state",
            ) {
                Q_strncpyzBytes(&mut teams, val.as_bytes(), teams_len);
                if teams[0] != 0 {
                    ctx.world.globals.g_preroundState = atoi(teams.as_ptr());
                }
            }

            if let Some(val) =
                BG_SiegeGetValueGroup(&buf_to_string(&ctx.world.bg_state.siege_info), "Teams")
            {
                Q_strncpyzBytes(&mut teams, val.as_bytes(), teams_len);
                if ctx.world.cvars.g_siegeTeam1.string[0] != 0
                    && Q_stricmp(
                        &cstr_to_str(ctx.world.cvars.g_siegeTeam1.string.as_ptr()),
                        "none",
                    ) != 0
                {
                    // check for override
                    ctx.world.globals.team1 = strncpyz_string(
                        cstr_from_chars(&ctx.world.cvars.g_siegeTeam1.string).to_bytes(),
                        512,
                    );
                } else {
                    // otherwise use level default
                    if let Some(val) = BG_SiegeGetPairedValue(&cstr_to_str(teams.as_ptr()), "team1")
                    {
                        ctx.world.globals.team1 = val;
                    }
                }

                if ctx.world.cvars.g_siegeTeam2.string[0] != 0
                    && Q_stricmp(
                        &cstr_to_str(ctx.world.cvars.g_siegeTeam2.string.as_ptr()),
                        "none",
                    ) != 0
                {
                    ctx.world.globals.team2 = strncpyz_string(
                        cstr_from_chars(&ctx.world.cvars.g_siegeTeam2.string).to_bytes(),
                        512,
                    );
                } else {
                    if let Some(val) = BG_SiegeGetPairedValue(&cstr_to_str(teams.as_ptr()), "team2")
                    {
                        ctx.world.globals.team2 = val;
                    }
                }
            } else {
                G_Error(ctx, "Siege teams not defined");
            }

            if let Some(val) = BG_SiegeGetValueGroup(
                &buf_to_string(&ctx.world.bg_state.siege_info),
                &ctx.world.globals.team2.clone(),
            ) {
                ctx.world.globals.gParseObjectives = val;
                let go = ctx.world.globals.gParseObjectives.clone();
                if let Some(val) = BG_SiegeGetPairedValue(&go, "TeamIcon") {
                    trap::Cvar_Set(ctx.engine, "team2_icon", &val);
                }

                if let Some(val) = BG_SiegeGetPairedValue(&go, "RequiredObjectives") {
                    Q_strncpyzBytes(&mut goalreq, val.as_bytes(), goalreq_len);
                    ctx.world.globals.rebel_goals_required = atoi(goalreq.as_ptr());
                }
                if let Some(val) = BG_SiegeGetPairedValue(&go, "Timed") {
                    Q_strncpyzBytes(&mut goalreq, val.as_bytes(), goalreq_len);
                    ctx.world.globals.rebel_time_limit = atoi(goalreq.as_ptr()) * 1000;
                    if ctx.world.cvars.g_siegeTeamSwitch.integer != 0
                        && ctx.world.globals.g_siegePersistant.beatingTime != 0
                    {
                        ctx.world.globals.gRebelCountdown =
                            ctx.world.level.time + ctx.world.globals.g_siegePersistant.lastTime;
                    } else {
                        ctx.world.globals.gRebelCountdown =
                            ctx.world.level.time + ctx.world.globals.rebel_time_limit;
                    }
                }
                if let Some(val) = BG_SiegeGetPairedValue(&go, "attackers") {
                    Q_strncpyzBytes(&mut goalreq, val.as_bytes(), goalreq_len);
                    ctx.world.globals.rebel_attackers = atoi(goalreq.as_ptr());
                }
            }

            if let Some(val) = BG_SiegeGetValueGroup(
                &buf_to_string(&ctx.world.bg_state.siege_info),
                &ctx.world.globals.team1.clone(),
            ) {
                ctx.world.globals.gParseObjectives = val;
                let go = ctx.world.globals.gParseObjectives.clone();
                if let Some(val) = BG_SiegeGetPairedValue(&go, "TeamIcon") {
                    trap::Cvar_Set(ctx.engine, "team1_icon", &val);
                }

                if let Some(val) = BG_SiegeGetPairedValue(&go, "RequiredObjectives") {
                    Q_strncpyzBytes(&mut goalreq, val.as_bytes(), goalreq_len);
                    ctx.world.globals.imperial_goals_required = atoi(goalreq.as_ptr());
                }
                if let Some(val) = BG_SiegeGetPairedValue(&go, "Timed") {
                    Q_strncpyzBytes(&mut goalreq, val.as_bytes(), goalreq_len);
                    if ctx.world.globals.rebel_time_limit != 0 {
                        Com_Printf("Tried to set imperial time limit, but there's already a rebel time limit!\nOnly one team can have a time limit.\n");
                    } else {
                        ctx.world.globals.imperial_time_limit = atoi(goalreq.as_ptr()) * 1000;
                        if ctx.world.cvars.g_siegeTeamSwitch.integer != 0
                            && ctx.world.globals.g_siegePersistant.beatingTime != 0
                        {
                            ctx.world.globals.gImperialCountdown =
                                ctx.world.level.time + ctx.world.globals.g_siegePersistant.lastTime;
                        } else {
                            ctx.world.globals.gImperialCountdown =
                                ctx.world.level.time + ctx.world.globals.imperial_time_limit;
                        }
                    }
                }
                if let Some(val) = BG_SiegeGetPairedValue(&go, "attackers") {
                    Q_strncpyzBytes(&mut goalreq, val.as_bytes(), goalreq_len);
                    ctx.world.globals.imperial_attackers = atoi(goalreq.as_ptr());
                }
            }

            // Load the player class types
            let mut callbacks = GameCallbacksImpl {
                // SEAM-BG-REENTRY (DEC-28, sanctioned): GameCallbacksImpl.world is a `*mut GameWorld` field.
                // A raw store is required for this bg-seam re-entry.
                world: ctx.world_raw(),
                engine: ctx.engine,
            };
            BG_SiegeLoadClasses(
                core::ptr::null_mut(),
                &mut ctx.world.bg_state,
                &crate::bg_channel::GameBgTraps::new(ctx.engine),
                &mut callbacks,
            );

            if ctx.world.bg_state.bgNumSiegeClasses == 0 {
                G_Error(ctx, "Couldn't find any player classes for Siege");
            }

            // Now load the teams since we have class data.
            BG_SiegeLoadTeams(
                &mut ctx.world.bg_state,
                &crate::bg_channel::GameBgTraps::new(ctx.engine),
            );

            if ctx.world.bg_state.bgNumSiegeTeams == 0 {
                G_Error(ctx, "Couldn't find any player teams for Siege");
            }

            // Get and set the team themes for each team.
            if let Some(val) = BG_SiegeGetValueGroup(
                &buf_to_string(&ctx.world.bg_state.siege_info),
                &ctx.world.globals.team1.clone(),
            ) {
                ctx.world.globals.gParseObjectives = val;
                let go = ctx.world.globals.gParseObjectives.clone();
                if let Some(val) = BG_SiegeGetPairedValue(&go, "UseTeam") {
                    Q_strncpyzBytes(&mut goalreq, val.as_bytes(), goalreq_len);
                    BG_SiegeSetTeamTheme(
                        SIEGETEAM_TEAM1,
                        goalreq.as_mut_ptr(),
                        &mut ctx.world.bg_state,
                    );
                }

                // Now count up the objectives for this team.
                let mut i: c_int = 1;
                write_cstr_field(&mut objecStr, &format!("Objective{}", i));
                while BG_SiegeGetValueGroup(&go, &cstr_to_str(objecStr.as_ptr())).is_some() {
                    objectiveNumTeam1 += 1;
                    i += 1;
                    write_cstr_field(&mut objecStr, &format!("Objective{}", i));
                }
            }
            if let Some(val) = BG_SiegeGetValueGroup(
                &buf_to_string(&ctx.world.bg_state.siege_info),
                &ctx.world.globals.team2.clone(),
            ) {
                ctx.world.globals.gParseObjectives = val;
                let go = ctx.world.globals.gParseObjectives.clone();
                if let Some(val) = BG_SiegeGetPairedValue(&go, "UseTeam") {
                    Q_strncpyzBytes(&mut goalreq, val.as_bytes(), goalreq_len);
                    BG_SiegeSetTeamTheme(
                        SIEGETEAM_TEAM2,
                        goalreq.as_mut_ptr(),
                        &mut ctx.world.bg_state,
                    );
                }

                let mut i: c_int = 1;
                write_cstr_field(&mut objecStr, &format!("Objective{}", i));
                while BG_SiegeGetValueGroup(&go, &cstr_to_str(objecStr.as_ptr())).is_some() {
                    objectiveNumTeam2 += 1;
                    i += 1;
                    write_cstr_field(&mut objecStr, &format!("Objective{}", i));
                }
            }

            // Set the configstring to show status of all current objectives
            let mut cfg = String::from("t1");
            while objectiveNumTeam1 > 0 {
                cfg.push_str("-0");
                objectiveNumTeam1 -= 1;
            }
            cfg.push_str("|t2");
            while objectiveNumTeam2 > 0 {
                cfg.push_str("-0");
                objectiveNumTeam2 -= 1;
            }
            ctx.world.globals.gObjectiveCfgStr = strncpyz_string(cfg.as_bytes(), 1024);

            trap::SetConfigstring(ctx.engine, CS_SIEGE_OBJECTIVES, &cfg);

            // precache saber data for classes that use sabers on both teams
            let mut callbacks = GameCallbacksImpl {
                // SEAM-BG-REENTRY (DEC-28, sanctioned): GameCallbacksImpl.world is a `*mut GameWorld` field aliasing bg_state.
                // A raw store is required for this bg-seam re-entry.
                world: ctx.world_raw(),
                engine: ctx.engine,
            };
            BG_PrecacheSabersForSiegeTeam(
                SIEGETEAM_TEAM1,
                &mut ctx.world.bg_state,
                &crate::bg_channel::GameBgTraps::new(ctx.engine),
                &mut callbacks,
            );
            BG_PrecacheSabersForSiegeTeam(
                SIEGETEAM_TEAM2,
                &mut ctx.world.bg_state,
                &crate::bg_channel::GameBgTraps::new(ctx.engine),
                &mut callbacks,
            );

            G_SiegeRegisterWeaponsAndHoldables(ctx, SIEGETEAM_TEAM1);
            G_SiegeRegisterWeaponsAndHoldables(ctx, SIEGETEAM_TEAM2);

            return;
        }

        // failure:
        ctx.world.bg_state.siege_valid = 0;
    }
}

/// Raven `G_SiegeSetObjectiveComplete`.
///
/// Source: `oracle/codemp/game/g_saga.c:375-427`
pub fn G_SiegeSetObjectiveComplete(
    ctx: &mut GameContext,
    team: c_int,
    objective: c_int,
    failIt: qboolean,
) {
    let needle: &[u8] = if team == SIEGETEAM_TEAM1 {
        b"t1"
    } else if team == SIEGETEAM_TEAM2 {
        b"t2"
    } else {
        b""
    };

    // Operate on the config string's own bytes. They are ASCII throughout, so byte-level edits stay valid UTF-8.
    // Byte positions match Raven's pointer walk.
    let mut buf = ctx.world.globals.gObjectiveCfgStr.clone().into_bytes();
    let buf_len = buf.len();

    let mut start: Option<usize> = None;
    if !needle.is_empty() {
        let mut i = 0usize;
        while i + needle.len() <= buf_len {
            if &buf[i..i + needle.len()] == needle {
                start = Some(i);
                break;
            }
            i += 1;
        }
    }

    let Some(mut p) = start else {
        // Raven: assert(0); return;
        return;
    };

    let mut onObjective: c_int = 0;

    // Parse from the beginning of this team's objectives until we get to the desired objective number.
    while p < buf_len && buf[p] != 0 && buf[p] != b'|' {
        if buf[p] == b'-' {
            onObjective += 1;
        }

        if onObjective == objective {
            // this is the one we want
            // Move to the next char, the status of this objective
            p += 1;
            if p < buf_len {
                buf[p] = if failIt != 0 { b'0' } else { b'1' };
            }
            break;
        }

        p += 1;
    }

    ctx.world.globals.gObjectiveCfgStr = latin1_to_string(&buf);

    // Now re-update the configstring.
    let cfg = ctx.world.globals.gObjectiveCfgStr.clone();
    trap::SetConfigstring(ctx.engine, CS_SIEGE_OBJECTIVES, &cfg);
}

/// Raven `G_SiegeGetCompletionStatus`.
///
/// Raven: returns qtrue if objective complete currently, otherwise qfalse.
///
/// Source: `oracle/codemp/game/g_saga.c:430-480`
pub fn G_SiegeGetCompletionStatus(
    ctx: &mut GameContext,
    team: c_int,
    objective: c_int,
) -> qboolean {
    let needle: &[u8] = if team == SIEGETEAM_TEAM1 {
        b"t1"
    } else if team == SIEGETEAM_TEAM2 {
        b"t2"
    } else {
        b""
    };

    let buf = ctx.world.globals.gObjectiveCfgStr.as_bytes();
    let buf_len = buf.len();

    let mut start: Option<usize> = None;
    if !needle.is_empty() {
        let mut i = 0usize;
        while i + needle.len() <= buf_len {
            if &buf[i..i + needle.len()] == needle {
                start = Some(i);
                break;
            }
            i += 1;
        }
    }

    let Some(mut p) = start else {
        // Raven: assert(0); return qfalse;
        return qfalse;
    };

    let mut onObjective: c_int = 0;

    while p < buf_len && buf[p] != 0 && buf[p] != b'|' {
        if buf[p] == b'-' {
            onObjective += 1;
        }

        if onObjective == objective {
            p += 1;

            // return qtrue if it's '1', qfalse if it's anything else
            if p < buf_len && buf[p] == b'1' {
                return qtrue;
            } else {
                return qfalse;
            }
        }

        p += 1;
    }

    qfalse
}

/// Raven `UseSiegeTarget`.
///
/// Raven: actually use the player which triggered the object which triggered the siege objective to trigger the target.
///
/// Source: `oracle/codemp/game/g_saga.c:482-526`
pub fn UseSiegeTarget(
    ctx: &mut GameContext,
    other: Option<EntityId>,
    en: Option<EntityId>,
    target: &str,
) {
    // Raven: "looks like we don't have access to a player, so just use the activating entity"
    // When `en` has no client, all three uses below (self-use test, GlobalUse activator/owner, inuse guard) target `other` instead.
    let ent: Option<EntityId> = match en {
        Some(en_id) if !ctx.world.entity(en_id).client.is_null() => Some(en_id),
        _ => other,
    };

    if en.is_none() {
        return;
    }
    // Raven guards only `if (!target)` for a null pointer.
    // An empty name is still searched, so this file has no empty check here.

    let mut t: Option<EntityId> = None;
    loop {
        let t_raw = G_Find(ctx, t, EntFindField::Targetname, target);
        t = ctx.entity_id_of(t_raw);
        let Some(t_id) = t else {
            break;
        };
        if t == ent {
            crate::g_main::G_Printf(ctx, "WARNING: Entity used itself.\n");
        } else if !ctx.world.entity(t_id).use_.is_none() {
            GlobalUse(ctx, t, ent, ent);
        }
        // Raven derefs `ent` unconditionally here.
        // When `ent` is null, with no client and no `other`, that is a null deref and undefined behavior.
        // This case is unreachable in practice, so the `Some` guard preserves the defined path (porting-rules §19).
        if let Some(ent_id) = ent {
            if ctx.world.entity(ent_id).inuse == 0 {
                crate::g_main::G_Printf(ctx, "entity was removed while using targets\n");
                return;
            }
        }
    }
}

/// Raven `SiegeBroadcast_OBJECTIVECOMPLETE`.
///
/// Source: `oracle/codemp/game/g_saga.c:528-540`
pub fn SiegeBroadcast_OBJECTIVECOMPLETE(
    ctx: &mut GameContext,
    team: c_int,
    client: c_int,
    objective: c_int,
) {
    let nomatter: vec3_t = [0.0, 0.0, 0.0];

    let te = G_TempEntity(
        ctx,
        nomatter,
        entity_event_t::EV_SIEGE_OBJECTIVECOMPLETE as c_int,
    );
    let te = ctx.world.entity_mut(te);
    te.r.svFlags |= SVF_BROADCAST;
    te.s.eventParm = team;
    te.s.weapon = client;
    te.s.trickedentindex = objective;
}

/// Raven `SiegeBroadcast_ROUNDOVER`.
///
/// Source: `oracle/codemp/game/g_saga.c:542-553`
pub fn SiegeBroadcast_ROUNDOVER(ctx: &mut GameContext, winningteam: c_int, winningclient: c_int) {
    let nomatter: vec3_t = [0.0, 0.0, 0.0];

    let te = G_TempEntity(ctx, nomatter, entity_event_t::EV_SIEGE_ROUNDOVER as c_int);
    let te = ctx.world.entity_mut(te);
    te.r.svFlags |= SVF_BROADCAST;
    te.s.eventParm = winningteam;
    te.s.weapon = winningclient;
}

/// Raven `BroadcastObjectiveCompletion`.
///
/// Source: `oracle/codemp/game/g_saga.c:555-564`
pub fn BroadcastObjectiveCompletion(
    ctx: &mut GameContext,
    team: c_int,
    objective: c_int,
    r#final: c_int,
    client: c_int,
) {
    if client != ENTITYNUM_NONE {
        let cid = EntityId::from_num(client).unwrap();
        // FLAG: gclient_t has no accessor.
        // The client-pointer deref stays raw.
        // We read the pointer through the safe entity borrow, then deref it in a tight unsafe block.
        let cl = ctx.world.entity(cid).client;
        if !cl.is_null() && unsafe { (*cl).sess.sessionTeam } == team {
            let client_origin = unsafe { (*cl).ps.origin };
            // guy who completed this objective gets points, providing he's on the opposing team
            AddScore(ctx, cid, client_origin, SIEGE_POINTS_OBJECTIVECOMPLETED);
        }
    }

    SiegeBroadcast_OBJECTIVECOMPLETE(ctx, team, client, objective);
    // G_Printf("Broadcast goal completion team %i objective %i final %i\n", team, objective, final);
}

/// Raven `AddSiegeWinningTeamPoints`.
///
/// Source: `oracle/codemp/game/g_saga.c:566-589`
pub fn AddSiegeWinningTeamPoints(ctx: &mut GameContext, team: c_int, winner: c_int) {
    let mut i: c_int = 0;

    while i < (MAX_CLIENTS) as i32 {
        let id = EntityId(i as u32);
        // FLAG: the gclient_t deref stays raw.
        // `i < MAX_CLIENTS` guarantees a real client slot.
        let cl = ctx.world.entity(id).client;

        if !cl.is_null() && unsafe { (*cl).sess.sessionTeam } == team {
            let origin = unsafe { (*cl).ps.origin };
            if i == winner {
                AddScore(
                    ctx,
                    id,
                    origin,
                    SIEGE_POINTS_TEAMWONROUND + SIEGE_POINTS_FINALOBJECTIVECOMPLETED,
                );
            } else {
                AddScore(ctx, id, origin, SIEGE_POINTS_TEAMWONROUND);
            }
        }

        i += 1;
    }
}

/// Raven `SiegeClearSwitchData`.
///
/// Source: `oracle/codemp/game/g_saga.c:591-595`
pub fn SiegeClearSwitchData(ctx: &mut GameContext) {
    ctx.world.globals.g_siegePersistant = Default::default();
    trap::SiegePersSet(
        ctx.engine,
        mp_abi::game::syscalls::G_SIEGEPERSSET::GSiegeperssetArgs::new(
            &ctx.world.globals.g_siegePersistant as *const siegePers_t,
        ),
    );
}

/// Raven `SiegeDoTeamAssign`.
///
/// Raven: yeah, this is great...
///
/// Source: `oracle/codemp/game/g_saga.c:597-630`
pub fn SiegeDoTeamAssign(ctx: &mut GameContext) {
    let mut i: c_int = 0;

    // yeah, this is great...
    while i < (MAX_CLIENTS) as i32 {
        let id = EntityId(i as u32);
        // FLAG: the gclient_t deref stays raw.
        // `i < MAX_CLIENTS` guarantees a real client slot.
        let cl = ctx.world.entity(id).client;

        if ctx.world.entity(id).inuse != 0
            && !cl.is_null()
            && unsafe { (*cl).pers.connected } == CON_CONNECTED
        {
            // a connected client, switch his frickin teams around
            unsafe {
                if (*cl).sess.siegeDesiredTeam == SIEGETEAM_TEAM1 {
                    (*cl).sess.siegeDesiredTeam = SIEGETEAM_TEAM2;
                } else if (*cl).sess.siegeDesiredTeam == SIEGETEAM_TEAM2 {
                    (*cl).sess.siegeDesiredTeam = SIEGETEAM_TEAM1;
                }
            }

            let sess_team = unsafe { (*cl).sess.sessionTeam };
            if sess_team == SIEGETEAM_TEAM1 {
                SetTeamQuick(ctx, id, SIEGETEAM_TEAM2, qfalse);
            } else if sess_team == SIEGETEAM_TEAM2 {
                SetTeamQuick(ctx, id, SIEGETEAM_TEAM1, qfalse);
            }
        }
        i += 1;
    }
}

/// Raven `SiegeTeamSwitch`.
///
/// Source: `oracle/codemp/game/g_saga.c:632-652`
pub fn SiegeTeamSwitch(ctx: &mut GameContext, winTeam: c_int, winTime: c_int) {
    trap::SiegePersGet(
        ctx.engine,
        mp_abi::game::syscalls::G_SIEGEPERSGET::GSiegepersgetArgs::new(
            &mut ctx.world.globals.g_siegePersistant as *mut siegePers_t,
        ),
    );
    if ctx.world.globals.g_siegePersistant.beatingTime != 0 {
        // was already in "switched" mode, change back.
        // announce the winning team.
        // either the first team won again, or the second team beat the time set by the initial team.
        // In any case the winTeam here is the overall winning team.
        SiegeSetCompleteData(ctx, winTeam);
        SiegeClearSwitchData(ctx);
    } else {
        // go into "beat their time" mode
        ctx.world.globals.g_siegePersistant.beatingTime = qtrue;
        ctx.world.globals.g_siegePersistant.lastTeam = winTeam;
        ctx.world.globals.g_siegePersistant.lastTime = winTime;

        trap::SiegePersSet(
            ctx.engine,
            mp_abi::game::syscalls::G_SIEGEPERSSET::GSiegeperssetArgs::new(
                &ctx.world.globals.g_siegePersistant as *const siegePers_t,
            ),
        );
    }
}

/// Raven `SiegeRoundComplete`.
///
/// Source: `oracle/codemp/game/g_saga.c:654-742`
pub fn SiegeRoundComplete(ctx: &mut GameContext, winningteam: c_int, winningclient: c_int) {
    unsafe {
        let nomatter: vec3_t = [0.0, 0.0, 0.0];
        let mut teamstr: [c_char; 1024] = [0; 1024];
        let teamstr_len = teamstr.len();
        let mut originalWinningClient = winningclient;
        let mut winningclient = winningclient;

        if winningclient != ENTITYNUM_NONE {
            let cid = EntityId::from_num(winningclient).unwrap();
            // FLAG: gclient_t has no accessor.
            // The client-pointer deref stays raw.
            // This fn's outer unsafe block covers it.
            let cl = ctx.world.entity(cid).client;
            if !cl.is_null() && (*cl).sess.sessionTeam != winningteam {
                // this person just won the round for the other team..
                winningclient = ENTITYNUM_NONE;
            }
        }

        SiegeBroadcast_ROUNDOVER(ctx, winningteam, winningclient);

        AddSiegeWinningTeamPoints(ctx, winningteam, winningclient);

        // Instead of exiting like this, fire off a target, and let it handle things.
        // Can be a script or whatever the designer wants.
        if winningteam == SIEGETEAM_TEAM1 {
            write_cstr_field(&mut teamstr, &ctx.world.globals.team1);
        } else {
            write_cstr_field(&mut teamstr, &ctx.world.globals.team2);
        }

        trap::SetConfigstring(
            ctx.engine,
            CS_SIEGE_STATE,
            &format!("3|{}", ctx.world.level.time),
        ); // ended
        ctx.world.globals.gSiegeRoundBegun = qfalse;
        ctx.world.globals.gSiegeRoundEnded = qtrue;
        ctx.world.globals.gSiegeRoundWinningTeam = winningteam;

        if let Some(val) = BG_SiegeGetValueGroup(
            &buf_to_string(&ctx.world.bg_state.siege_info),
            &cstr_to_str(teamstr.as_ptr()),
        ) {
            ctx.world.globals.gParseObjectives = val;
            if let Some(val) = BG_SiegeGetPairedValue(
                &ctx.world.globals.gParseObjectives.clone(),
                "roundover_target",
            ) {
                Q_strncpyzBytes(&mut teamstr, val.as_bytes(), teamstr_len);
            } else {
                // didn't find the name of the thing to target upon win, just logexit now then.
                LogExit(ctx, "Objectives completed");
                return;
            }

            if originalWinningClient == ENTITYNUM_NONE {
                // oh well, just find something active and use it then.
                let mut i: c_int = 0;

                while i < (MAX_CLIENTS) as i32 {
                    let id = EntityId(i as u32);

                    if ctx.world.entity(id).inuse != 0 {
                        // sure, you'll do.
                        originalWinningClient = ctx.world.entity(id).s.number;
                        break;
                    }

                    i += 1;
                }
            }
            let teamstr_s = cstr_to_str(teamstr.as_ptr());
            G_UseTargets2(
                ctx,
                Some(EntityId(originalWinningClient as u32)),
                Some(EntityId(originalWinningClient as u32)),
                Some(&teamstr_s),
            );
        }

        if ctx.world.cvars.g_siegeTeamSwitch.integer != 0
            && (ctx.world.globals.imperial_time_limit != 0
                || ctx.world.globals.rebel_time_limit != 0)
        {
            // handle stupid team switching crap
            let mut time: c_int = 0;
            if ctx.world.globals.imperial_time_limit != 0 {
                time = ctx.world.globals.imperial_time_limit
                    - (ctx.world.globals.gImperialCountdown - ctx.world.level.time);
            } else if ctx.world.globals.rebel_time_limit != 0 {
                time = ctx.world.globals.rebel_time_limit
                    - (ctx.world.globals.gRebelCountdown - ctx.world.level.time);
            }

            if time < 1 {
                time = 1;
            }
            SiegeTeamSwitch(ctx, winningteam, time);
        } else {
            // assure it's clear for next round
            SiegeClearSwitchData(ctx);
        }
    }
}

/// Raven `G_ValidateSiegeClassForTeam`.
///
/// Source: `oracle/codemp/game/g_saga.c:744-784`
pub fn G_ValidateSiegeClassForTeam(ctx: &mut GameContext, ent: EntityId, team: c_int) {
    unsafe {
        let mut newClassIndex: c_int = -1;
        // FLAG: gclient_t has no accessor.
        // The client-pointer deref stays raw.
        // This fn's outer unsafe block covers it.
        let cl = ctx.world.entity(ent).client;
        if (*cl).siegeClass == -1 {
            // uh.. sure.
            return;
        }

        let scl = &mut (&mut ctx.world.bg_state.bgSiegeClasses)[(*cl).siegeClass as usize]
            as *mut siegeClass_t;

        let stm = BG_SiegeFindThemeForTeam(team, &ctx.world.bg_state);
        if !stm.is_null() {
            let mut i = 0;

            while i < (*stm).numClasses {
                // go through the team and see its valid classes, can we find one that matches our current player class?
                if !(*stm).classes[i as usize].is_null() {
                    if (*scl)
                        .name
                        .eq_ignore_ascii_case(&(*(*stm).classes[i as usize]).name)
                    {
                        // the class we're using is already ok for this team.
                        return;
                    }
                    if (*(*stm).classes[i as usize]).playerClass == (*scl).playerClass
                        || newClassIndex == -1
                    {
                        newClassIndex = i;
                    }
                }
                i += 1;
            }

            if newClassIndex != -1 {
                // ok, let's find it in the global class array
                (*cl).siegeClass = BG_SiegeFindClassIndexByName(
                    &(*(*stm).classes[newClassIndex as usize]).name,
                    &ctx.world.bg_state,
                );
                (*cl).sess.siegeClass = strncpyz_string(
                    (*(*stm).classes[newClassIndex as usize]).name.as_bytes(),
                    64,
                );
            }
        }
    }
}

/// Raven `SetTeamQuick`.
///
/// Raven: bypass most of the normal checks in SetTeam.
///
/// Source: `oracle/codemp/game/g_saga.c:787-834`
pub fn SetTeamQuick(ctx: &mut GameContext, ent: EntityId, team: c_int, doBegin: qboolean) {
    unsafe {
        let ent_num = ctx.world.entity(ent).s.number;
        let mut userinfo = trap::GetUserinfo(ctx.engine, ent_num, MAX_INFO_STRING);

        if ctx.world.cvars.g_gametype.integer == GT_SIEGE {
            G_ValidateSiegeClassForTeam(ctx, ent, team);
        }

        // FLAG: gclient_t has no accessor.
        // The client-pointer deref stays raw.
        // This fn's outer unsafe block covers it.
        let cl = ctx.world.entity(ent).client;
        (*cl).sess.sessionTeam = team;

        if team == TEAM_SPECTATOR {
            (*cl).sess.spectatorState = spectatorState_t::SPECTATOR_FREE;
            Info_SetValueForKey(&mut userinfo, "team", "s");
        } else {
            (*cl).sess.spectatorState = spectatorState_t::SPECTATOR_NOT;
            if team == TEAM_RED {
                Info_SetValueForKey(&mut userinfo, "team", "r");
            } else if team == TEAM_BLUE {
                Info_SetValueForKey(&mut userinfo, "team", "b");
            } else {
                Info_SetValueForKey(&mut userinfo, "team", "?");
            }
        }

        trap::SetUserinfo(ctx.engine, ent_num, &userinfo);

        (*cl).sess.spectatorClient = 0;

        (*cl).pers.teamState.state = playerTeamStateState_t::TEAM_BEGIN;

        ClientUserinfoChanged(ctx, ent_num);

        if doBegin != 0 {
            ClientBegin(ctx, ent_num, qfalse);
        }
    }
}

/// Raven `SiegeRespawn`. This respawns a siege player, honoring a pending team switch.
///
/// Source: `oracle/codemp/game/g_saga.c:836-851`
pub fn SiegeRespawn(ctx: &mut GameContext, ent: EntityId) {
    unsafe {
        // FLAG: gclient_t has no accessor.
        // The client-pointer deref stays raw.
        // This fn's outer unsafe block covers it.
        let client = ctx.world.entity(ent).client;

        if (*client).sess.sessionTeam != (*client).sess.siegeDesiredTeam {
            let desired = (*client).sess.siegeDesiredTeam;
            SetTeamQuick(ctx, ent, desired, qtrue);
        } else {
            ClientSpawn(ctx, ent);
            // add a teleportation effect
            let origin = (*client).ps.origin;
            let tent = G_TempEntity(ctx, origin, entity_event_t::EV_PLAYER_TELEPORT_IN as c_int);
            let client_num = ctx.world.entity(ent).s.clientNum;
            let tent = tent;
            ctx.world.entity_mut(tent).s.clientNum = client_num;
        }
    }
}

/// Raven `SiegeBeginRound`.
///
/// Raven: entNum is just used as something to fire targets from.
///
/// Source: `oracle/codemp/game/g_saga.c:853-903`
pub fn SiegeBeginRound(ctx: &mut GameContext, entNum: c_int) {
    unsafe {
        // entNum is just used as something to fire targets from.
        let mut targname: [c_char; 1024] = [0; 1024];
        let targname_len = targname.len();

        if ctx.world.globals.g_preroundState == 0 {
            // if players are not ingame on round start then respawn them now
            let mut i: c_int = 0;
            let mut spawnEnt: qboolean = qfalse;

            // respawn everyone now
            while i < (MAX_CLIENTS) as i32 {
                let id = EntityId(i as u32);
                // FLAG: the gclient_t deref stays raw.
                // `i < MAX_CLIENTS` guarantees a real client slot, and this fn's outer unsafe block covers it.
                let cl = ctx.world.entity(id).client;

                if ctx.world.entity(id).inuse != 0 && !cl.is_null() {
                    if (*cl).sess.sessionTeam != TEAM_SPECTATOR
                        && (*cl).ps.pm_flags & PMF_FOLLOW == 0
                    {
                        // not a spec, just respawn them
                        spawnEnt = qtrue;
                    } else if (*cl).sess.sessionTeam == TEAM_SPECTATOR
                        && ((*cl).sess.siegeDesiredTeam == TEAM_RED
                            || (*cl).sess.siegeDesiredTeam == TEAM_BLUE)
                    {
                        // spectator but has a desired team
                        spawnEnt = qtrue;
                    }
                }

                if spawnEnt != 0 {
                    SiegeRespawn(ctx, EntityId(i as u32));
                    spawnEnt = qfalse;
                }
                i += 1;
            }
        }

        // Now check if there's something to fire off at the round start, if so do it.
        if let Some(val) = BG_SiegeGetPairedValue(
            &buf_to_string(&ctx.world.bg_state.siege_info),
            "roundbegin_target",
        ) {
            Q_strncpyzBytes(&mut targname, val.as_bytes(), targname_len);
            if targname[0] != 0 {
                let targname_s = cstr_to_str(targname.as_ptr());
                G_UseTargets2(
                    ctx,
                    Some(EntityId(entNum as u32)),
                    Some(EntityId(entNum as u32)),
                    Some(&targname_s),
                );
            }
        }

        trap::SetConfigstring(
            ctx.engine,
            CS_SIEGE_STATE,
            &format!("0|{}", ctx.world.level.time),
        ); // we're ready to g0g0g0
    }
}

/// Raven `SiegeCheckTimers`.
///
/// Source: `oracle/codemp/game/g_saga.c:905-1013`
pub fn SiegeCheckTimers(ctx: &mut GameContext) {
    unsafe {
        let mut i: c_int = 0;
        let mut numTeam1: c_int = 0;
        let mut numTeam2: c_int = 0;

        if ctx.world.cvars.g_gametype.integer != GT_SIEGE {
            return;
        }

        if ctx.world.level.intermissiontime != 0 {
            return;
        }

        if ctx.world.globals.gSiegeRoundEnded != 0 {
            return;
        }

        if ctx.world.globals.gSiegeRoundBegun == 0 {
            // check if anyone is active on this team - if not, keep the timer set up.
            i = 0;

            while i < (MAX_CLIENTS) as i32 {
                let id = EntityId(i as u32);
                // FLAG: the gclient_t deref stays raw.
                // This fn's outer unsafe block covers it.
                let cl = ctx.world.entity(id).client;

                if !cl.is_null()
                    && ctx.world.entity(id).inuse != 0
                    && (*cl).pers.connected == CON_CONNECTED
                    && (*cl).sess.siegeDesiredTeam == SIEGETEAM_TEAM1
                {
                    numTeam1 += 1;
                }
                i += 1;
            }

            i = 0;

            while i < (MAX_CLIENTS) as i32 {
                let id = EntityId(i as u32);
                // FLAG: the gclient_t deref stays raw.
                // This fn's outer unsafe block covers it.
                let cl = ctx.world.entity(id).client;

                if !cl.is_null()
                    && ctx.world.entity(id).inuse != 0
                    && (*cl).pers.connected == CON_CONNECTED
                    && (*cl).sess.siegeDesiredTeam == SIEGETEAM_TEAM2
                {
                    numTeam2 += 1;
                }
                i += 1;
            }

            if ctx.world.cvars.g_siegeTeamSwitch.integer != 0
                && ctx.world.globals.g_siegePersistant.beatingTime != 0
            {
                ctx.world.globals.gImperialCountdown =
                    ctx.world.level.time + ctx.world.globals.g_siegePersistant.lastTime;
                ctx.world.globals.gRebelCountdown =
                    ctx.world.level.time + ctx.world.globals.g_siegePersistant.lastTime;
            } else {
                ctx.world.globals.gImperialCountdown =
                    ctx.world.level.time + ctx.world.globals.imperial_time_limit;
                ctx.world.globals.gRebelCountdown =
                    ctx.world.level.time + ctx.world.globals.rebel_time_limit;
            }
        }

        if ctx.world.globals.imperial_time_limit != 0 {
            // team1
            if ctx.world.globals.gImperialCountdown < ctx.world.level.time {
                SiegeRoundComplete(ctx, SIEGETEAM_TEAM2, ENTITYNUM_NONE);
                ctx.world.globals.imperial_time_limit = 0;
                return;
            }
        }

        if ctx.world.globals.rebel_time_limit != 0 {
            // team2
            if ctx.world.globals.gRebelCountdown < ctx.world.level.time {
                SiegeRoundComplete(ctx, SIEGETEAM_TEAM1, ENTITYNUM_NONE);
                ctx.world.globals.rebel_time_limit = 0;
                return;
            }
        }

        if ctx.world.globals.gSiegeRoundBegun == 0 {
            if numTeam1 == 0 || numTeam2 == 0 {
                // don't have people on both teams yet.
                ctx.world.globals.gSiegeBeginTime = ctx.world.level.time + SIEGE_ROUND_BEGIN_TIME;
                trap::SetConfigstring(ctx.engine, CS_SIEGE_STATE, "1"); // "waiting for players on both teams"
            } else if ctx.world.globals.gSiegeBeginTime < ctx.world.level.time {
                // mark the round as having begun
                ctx.world.globals.gSiegeRoundBegun = qtrue;
                SiegeBeginRound(ctx, i); // perform any round start tasks
            } else if ctx.world.globals.gSiegeBeginTime
                > (ctx.world.level.time + SIEGE_ROUND_BEGIN_TIME)
            {
                ctx.world.globals.gSiegeBeginTime = ctx.world.level.time + SIEGE_ROUND_BEGIN_TIME;
            } else {
                trap::SetConfigstring(
                    ctx.engine,
                    CS_SIEGE_STATE,
                    &format!(
                        "2|{}",
                        ctx.world.globals.gSiegeBeginTime - SIEGE_ROUND_BEGIN_TIME
                    ),
                ); // getting ready to begin
            }
        }
    }
}

/// Raven `SiegeObjectiveCompleted`.
///
/// Source: `oracle/codemp/game/g_saga.c:1015-1058`
pub fn SiegeObjectiveCompleted(
    ctx: &mut GameContext,
    team: c_int,
    objective: c_int,
    r#final: c_int,
    client: c_int,
) {
    let goals_completed: c_int;
    let goals_required: c_int;

    if ctx.world.globals.gSiegeRoundEnded != 0 {
        return;
    }

    // Update the configstring status
    G_SiegeSetObjectiveComplete(ctx, team, objective, qfalse);

    if r#final != -1 {
        if team == SIEGETEAM_TEAM1 {
            ctx.world.globals.imperial_goals_completed += 1;
        } else {
            ctx.world.globals.rebel_goals_completed += 1;
        }
    }

    if team == SIEGETEAM_TEAM1 {
        goals_completed = ctx.world.globals.imperial_goals_completed;
        goals_required = ctx.world.globals.imperial_goals_required;
    } else {
        goals_completed = ctx.world.globals.rebel_goals_completed;
        goals_required = ctx.world.globals.rebel_goals_required;
    }

    if r#final == 1 || goals_completed >= goals_required {
        SiegeRoundComplete(ctx, team, client);
    } else {
        BroadcastObjectiveCompletion(ctx, team, objective, r#final, client);
    }
}

/// Raven `siegeTriggerUse`.
///
/// Source: `oracle/codemp/game/g_saga.c:1060-1128`
pub fn siegeTriggerUse(
    ctx: &mut GameContext,
    ent: EntityId,
    other: Option<EntityId>,
    activator: Option<EntityId>,
) {
    unsafe {
        let mut teamstr = String::new();
        let mut objectivestr = String::new();
        let mut desiredobjective: [c_char; MAX_SIEGE_INFO_SIZE as usize] =
            [0; MAX_SIEGE_INFO_SIZE as usize];
        let desiredobjective_len = desiredobjective.len();
        let mut clUser: c_int = ENTITYNUM_NONE;
        let mut r#final: c_int = 0;

        if ctx.world.bg_state.siege_valid == 0 {
            return;
        }

        if ctx.world.entity(ent).s.eFlags & EF_RADAROBJECT == 0 {
            // toggle radar on and exit if it is not showing up already
            ctx.world.entity_mut(ent).s.eFlags |= EF_RADAROBJECT;
            return;
        }

        if let Some(aid) = activator {
            // activator will hopefully be the person who triggered this event
            if !ctx.world.entity(aid).client.is_null() {
                clUser = ctx.world.entity(aid).s.number;
            }
        }

        if ctx.world.entity(ent).side == SIEGETEAM_TEAM1 {
            teamstr = strncpyz_string(ctx.world.globals.team1.as_bytes(), 64);
        } else {
            teamstr = strncpyz_string(ctx.world.globals.team2.as_bytes(), 64);
        }

        if let Some(objectives) =
            BG_SiegeGetValueGroup(&buf_to_string(&ctx.world.bg_state.siege_info), &teamstr)
        {
            ctx.world.globals.gParseObjectives = objectives;
            let obj_num = ctx.world.entity(ent).objective;
            objectivestr = strncpyz_string(format!("Objective{}", obj_num).as_bytes(), 64);

            if let Some(desired) =
                BG_SiegeGetValueGroup(&ctx.world.globals.gParseObjectives.clone(), &objectivestr)
            {
                Q_strncpyzBytes(
                    &mut desiredobjective,
                    desired.as_bytes(),
                    desiredobjective_len,
                );
                if let Some(val) =
                    BG_SiegeGetPairedValue(&cstr_to_str(desiredobjective.as_ptr()), "final")
                {
                    teamstr = strncpyz_string(val.as_bytes(), 64);
                    r#final = atoi_bytes(teamstr.as_bytes());
                }

                if let Some(val) =
                    BG_SiegeGetPairedValue(&cstr_to_str(desiredobjective.as_ptr()), "target")
                {
                    teamstr = strncpyz_string(val.as_bytes(), 64);
                    // Raven NUL-terminates at the first carriage-return/newline.
                    if let Some(pos) = teamstr
                        .as_bytes()
                        .iter()
                        .position(|&b| b == b'\r' || b == b'\n')
                    {
                        teamstr.truncate(pos);
                    }
                    UseSiegeTarget(ctx, other, activator, &teamstr);
                }

                let ent_target = ctx.world.entity(ent).target.clone();
                if ent_target.as_deref().is_some_and(|s| !s.is_empty()) {
                    // use this too
                    UseSiegeTarget(ctx, other, activator, ent_target.as_deref().unwrap());
                }

                let side = ctx.world.entity(ent).side;
                let obj = ctx.world.entity(ent).objective;
                SiegeObjectiveCompleted(ctx, side, obj, r#final, clUser);
            }
        }
    }
}

/// Raven `SP_info_siege_objective`.
///
/// Source: `oracle/codemp/game/g_saga.c:1137-1179`
pub fn SP_info_siege_objective(ctx: &mut GameContext, ent: EntityId) {
    unsafe {
        let mut s: String = String::new();

        if ctx.world.bg_state.siege_valid == 0 || ctx.world.cvars.g_gametype.integer != GT_SIEGE {
            crate::g_utils::G_FreeEntity(ctx, Some(ent));
            return;
        }

        ctx.world.entity_mut(ent).use_ = Some(EntUse::siegeTriggerUse).into();
        // G_Spawn* out-params: copy the field out, parse into the local, copy back (the out-pointer can't alias `ctx` across the `ctx`-taking call).
        let mut objective = ctx.world.entity(ent).objective;
        G_SpawnInt(
            ctx,
            b"objective\0".as_ptr() as *const c_char,
            b"0\0".as_ptr() as *const c_char,
            &mut objective,
        );
        ctx.world.entity_mut(ent).objective = objective;
        let mut side = ctx.world.entity(ent).side;
        G_SpawnInt(
            ctx,
            b"side\0".as_ptr() as *const c_char,
            b"0\0".as_ptr() as *const c_char,
            &mut side,
        );
        ctx.world.entity_mut(ent).side = side;

        if objective == 0 || side == 0 {
            // j00 fux0red something up
            crate::g_utils::G_FreeEntity(ctx, Some(ent));
            crate::g_main::G_Printf(
                ctx,
                "ERROR: info_siege_objective without an objective or side value\n",
            );
            return;
        }

        // Set it up to be drawn on radar
        if ctx.world.entity(ent).spawnflags & SIEGEITEM_STARTOFFRADAR == 0 {
            ctx.world.entity_mut(ent).s.eFlags |= EF_RADAROBJECT;
        }

        // All clients want to know where it is at all times for radar
        ctx.world.entity_mut(ent).r.svFlags |= SVF_BROADCAST;

        s = G_SpawnString(ctx, "icon", "").1;

        if !s.is_empty() {
            // We have an icon, so index it now.  We are reusing the genericenemyindex variable rather than adding a new one to the entity state.
            let idx = G_IconIndex(ctx, &s);
            ctx.world.entity_mut(ent).s.genericenemyindex = idx;
        }

        ctx.world.entity_mut(ent).s.brokenLimbs = side;
        ctx.world.entity_mut(ent).s.frame = objective;
        trap::LinkEntity(
            ctx.engine,
            mp_abi::game::syscalls::G_LINKENTITY::GLinkentityArgs::new(
                (ctx.world.entity_mut(ent) as *mut gentity_t).cast(),
            ),
        );
    }
}

/// Raven `SiegeIconUse`.
///
/// Raven: toggle it on and off.
///
/// Source: `oracle/codemp/game/g_saga.c:1182-1194`
pub fn SiegeIconUse(ent: &mut gentity_t, other: Option<EntityId>, activator: Option<EntityId>) {
    // `other`/`activator` are unused handler params kept as Option<EntityId>.
    // toggle it on and off
    if ent.s.eFlags & EF_RADAROBJECT != 0 {
        ent.s.eFlags &= !EF_RADAROBJECT;
        ent.r.svFlags &= !SVF_BROADCAST;
    } else {
        ent.s.eFlags |= EF_RADAROBJECT;
        ent.r.svFlags |= SVF_BROADCAST;
    }
}

/// Raven `SP_info_siege_radaricon`.
///
/// Source: `oracle/codemp/game/g_saga.c:1203-1234`
pub fn SP_info_siege_radaricon(ctx: &mut GameContext, ent: EntityId) {
    unsafe {
        let mut s: String = String::new();
        let mut i: c_int = 0;

        if ctx.world.bg_state.siege_valid == 0 || ctx.world.cvars.g_gametype.integer != GT_SIEGE {
            crate::g_utils::G_FreeEntity(ctx, Some(ent));
            return;
        }

        G_SpawnInt(
            ctx,
            b"startoff\0".as_ptr() as *const c_char,
            b"0\0".as_ptr() as *const c_char,
            &mut i,
        );

        if i == 0 {
            // start on then
            ctx.world.entity_mut(ent).s.eFlags |= EF_RADAROBJECT;
            ctx.world.entity_mut(ent).r.svFlags |= SVF_BROADCAST;
        }

        s = G_SpawnString(ctx, "icon", "").1;
        if s.is_empty() {
            // that's the whole point of the entity
            crate::g_main::Com_Error(
                (ERR_DROP) as i32,
                cstr("misc_siege_radaricon without an icon").as_ptr(),
            );
            return;
        }

        ctx.world.entity_mut(ent).use_ = Some(EntUse::SiegeIconUse).into();

        let idx = G_IconIndex(ctx, &s);
        ctx.world.entity_mut(ent).s.genericenemyindex = idx;

        trap::LinkEntity(
            ctx.engine,
            mp_abi::game::syscalls::G_LINKENTITY::GLinkentityArgs::new(
                (ctx.world.entity_mut(ent) as *mut gentity_t).cast(),
            ),
        );
    }
}

/// Raven `decompTriggerUse`.
///
/// Source: `oracle/codemp/game/g_saga.c:1236-1291`
pub fn decompTriggerUse(
    ctx: &mut GameContext,
    ent: EntityId,
    other: Option<EntityId>,
    activator: Option<EntityId>,
) {
    unsafe {
        let mut r#final: c_int = 0;
        let mut teamstr: [c_char; 1024] = [0; 1024];
        let mut objectivestr: [c_char; 64] = [0; 64];
        let mut desiredobjective: [c_char; MAX_SIEGE_INFO_SIZE as usize] =
            [0; MAX_SIEGE_INFO_SIZE as usize];
        let teamstr_len = teamstr.len();
        let desiredobjective_len = desiredobjective.len();

        if ctx.world.globals.gSiegeRoundEnded != 0 {
            return;
        }

        let side = ctx.world.entity(ent).side;
        let objective = ctx.world.entity(ent).objective;

        if G_SiegeGetCompletionStatus(ctx, side, objective) == 0 {
            // if it's not complete then there's nothing to do here
            return;
        }

        // Update the configstring status
        G_SiegeSetObjectiveComplete(ctx, side, objective, qtrue);

        // Find out if this objective counts toward the final objective count
        if side == SIEGETEAM_TEAM1 {
            write_cstr_field(&mut teamstr, &ctx.world.globals.team1);
        } else {
            write_cstr_field(&mut teamstr, &ctx.world.globals.team2);
        }

        if let Some(objectives) = BG_SiegeGetValueGroup(
            &buf_to_string(&ctx.world.bg_state.siege_info),
            &cstr_to_str(teamstr.as_ptr()),
        ) {
            ctx.world.globals.gParseObjectives = objectives;
            write_cstr_field(&mut objectivestr, &format!("Objective{}", objective));

            if let Some(desired) = BG_SiegeGetValueGroup(
                &ctx.world.globals.gParseObjectives.clone(),
                &cstr_to_str(objectivestr.as_ptr()),
            ) {
                Q_strncpyzBytes(
                    &mut desiredobjective,
                    desired.as_bytes(),
                    desiredobjective_len,
                );
                if let Some(val) =
                    BG_SiegeGetPairedValue(&cstr_to_str(desiredobjective.as_ptr()), "final")
                {
                    Q_strncpyzBytes(&mut teamstr, val.as_bytes(), teamstr_len);
                    r#final = atoi(teamstr.as_ptr());
                }
            }
        }

        // Subtract the goal num if applicable
        if r#final != -1 {
            if side == SIEGETEAM_TEAM1 {
                ctx.world.globals.imperial_goals_completed -= 1;
            } else {
                ctx.world.globals.rebel_goals_completed -= 1;
            }
        }
    }
}

/// Raven `SP_info_siege_decomplete`.
///
/// Source: `oracle/codemp/game/g_saga.c:1297-1315`
pub fn SP_info_siege_decomplete(ctx: &mut GameContext, ent: EntityId) {
    unsafe {
        if ctx.world.bg_state.siege_valid == 0 || ctx.world.cvars.g_gametype.integer != GT_SIEGE {
            crate::g_utils::G_FreeEntity(ctx, Some(ent));
            return;
        }

        ctx.world.entity_mut(ent).use_ = Some(EntUse::decompTriggerUse).into();
        let mut objective = ctx.world.entity(ent).objective;
        G_SpawnInt(
            ctx,
            b"objective\0".as_ptr() as *const c_char,
            b"0\0".as_ptr() as *const c_char,
            &mut objective,
        );
        ctx.world.entity_mut(ent).objective = objective;
        let mut side = ctx.world.entity(ent).side;
        G_SpawnInt(
            ctx,
            b"side\0".as_ptr() as *const c_char,
            b"0\0".as_ptr() as *const c_char,
            &mut side,
        );
        ctx.world.entity_mut(ent).side = side;

        if objective == 0 || side == 0 {
            // j00 fux0red something up
            crate::g_utils::G_FreeEntity(ctx, Some(ent));
            crate::g_main::G_Printf(
                ctx,
                "ERROR: info_siege_objective_decomplete without an objective or side value\n",
            );
            return;
        }
    }
}

/// Raven `siegeEndUse`. It does a `LogExit` for siege when used.
///
/// Source: `oracle/codemp/game/g_saga.c:1317-1320`
pub fn siegeEndUse(
    ctx: &mut GameContext,
    ent: EntityId,
    other: Option<EntityId>,
    activator: Option<EntityId>,
) {
    // `ent`/`other`/`activator` are unused handler params (body is a bare LogExit).
    LogExit(ctx, "Round ended");
}

/// Raven `SP_target_siege_end`.
///
/// Source: `oracle/codemp/game/g_saga.c:1325-1334`
pub fn SP_target_siege_end(ctx: &mut GameContext, ent: EntityId) {
    if ctx.world.bg_state.siege_valid == 0 || ctx.world.cvars.g_gametype.integer != GT_SIEGE {
        crate::g_utils::G_FreeEntity(ctx, Some(ent));
        return;
    }

    ctx.world.entity_mut(ent).use_ = Some(EntUse::siegeEndUse).into();
}

/// Raven `SiegeItemRemoveOwner`.
///
/// Source: `oracle/codemp/game/g_saga.c:1338-1349`
pub fn SiegeItemRemoveOwner(ctx: &mut GameContext, ent: EntityId, carrier: Option<EntityId>) {
    // This is a two-entity write.
    // The two `&mut` entity borrows Raven passed as raw pointers become `EntityId`s reached one at a time through the accessor.
    // The carrier's `gclient_t` write stays a raw deref.
    ctx.world.entity_mut(ent).genericValue2 = 0; // Remove picked-up flag
    ctx.world.entity_mut(ent).genericValue8 = ENTITYNUM_NONE; // Mark entity carrying us as none

    if let Some(carrier) = carrier {
        // FLAG: gclient_t has no accessor.
        // The client-pointer deref stays raw.
        let cl = ctx.world.entity(carrier).client;
        unsafe {
            (*cl).holdingObjectiveItem = 0; // The carrier is no longer carrying us
        }
        ctx.world.entity_mut(carrier).r.svFlags &= !SVF_BROADCAST;
    }
}

/// Raven `SiegeItemRespawnEffect`. It plays the objective-item respawn effect and fires its respawn target.
///
/// Source: `oracle/codemp/game/g_saga.c:1351-1370`
pub fn SiegeItemRespawnEffect(ctx: &mut GameContext, ent: EntityId, newOrg: vec3_t) {
    unsafe {
        // `target5` is an owned `String`, where `""` means absent.
        // We fire it through the `Option<&str>` target seam.
        let target5 = ctx.world.entity(ent).target5.clone();
        if !target5.is_empty() {
            G_UseTargets2(ctx, Some(ent), Some(ent), Some(&target5));
        }

        if ctx.world.entity(ent).genericValue10 == 0 {
            // no respawn effect
            return;
        }

        let upAng: vec3_t = [0.0, 0.0, 1.0];

        // Play it once on the current origin, and once on the origin we're respawning to.
        let gv10 = ctx.world.entity(ent).genericValue10;
        let current_origin = ctx.world.entity(ent).r.currentOrigin;
        G_PlayEffectID(gv10, current_origin, upAng);
        G_PlayEffectID(gv10, newOrg, upAng);
    }
}

/// Raven `SiegeItemRespawnOnOriginalSpot`.
///
/// Source: `oracle/codemp/game/g_saga.c:1372-1380`
pub fn SiegeItemRespawnOnOriginalSpot(
    ctx: &mut GameContext,
    ent: EntityId,
    carrier: Option<EntityId>,
) {
    let pos1 = ctx.world.entity(ent).pos1;
    SiegeItemRespawnEffect(ctx, ent, pos1);
    G_SetOrigin(ctx.world.entity_mut(ent), pos1);
    SiegeItemRemoveOwner(ctx, ent, carrier);

    // Stop the item from flashing on the radar
    ctx.world.entity_mut(ent).s.time2 = 0;
}

/// Raven `SiegeItemThink`.
///
/// Source: `oracle/codemp/game/g_saga.c:1382-1475`
pub fn SiegeItemThink(ctx: &mut GameContext, ent: EntityId) {
    unsafe {
        let mut carrier_id: Option<EntityId> = None;

        if ctx.world.entity(ent).genericValue12 != 0 {
            // recharge health
            let health = ctx.world.entity(ent).health;
            let max_health = ctx.world.entity(ent).maxHealth;
            if health > 0
                && health < max_health
                && ctx.world.entity(ent).genericValue14 < ctx.world.level.time
            {
                let gv13 = ctx.world.entity(ent).genericValue13;
                ctx.world.entity_mut(ent).genericValue14 = ctx.world.level.time + gv13;
                let gv12 = ctx.world.entity(ent).genericValue12;
                ctx.world.entity_mut(ent).health += gv12;
                if ctx.world.entity(ent).health > max_health {
                    ctx.world.entity_mut(ent).health = max_health;
                }
            }
        }

        if ctx.world.entity(ent).genericValue8 != ENTITYNUM_NONE {
            // Just keep sticking it on top of the owner. We need it in the same PVS as him so it will render bolted onto him properly.
            let carrier = EntityId(ctx.world.entity(ent).genericValue8 as u32);
            carrier_id = Some(carrier);

            // FLAG: the gclient_t deref stays raw.
            // This fn's outer unsafe block covers it.
            let ccl = ctx.world.entity(carrier).client;
            if ctx.world.entity(carrier).inuse != 0 && !ccl.is_null() {
                let new_origin = (*ccl).ps.origin;
                crate::q_math::_VectorCopy(
                    new_origin,
                    &mut ctx.world.entity_mut(ent).r.currentOrigin,
                );
                trap::LinkEntity(
                    ctx.engine,
                    mp_abi::game::syscalls::G_LINKENTITY::GLinkentityArgs::new(
                        (ctx.world.entity_mut(ent) as *mut gentity_t).cast(),
                    ),
                );
            }
        } else if ctx.world.entity(ent).genericValue1 != 0 {
            // this means we want to run physics on the object
            let radius = ctx.world.entity(ent).radius;
            let mass = ctx.world.entity(ent).mass;
            let random = ctx.world.entity(ent).random;
            G_RunExPhys(
                ctx,
                ent,
                radius,
                mass,
                random,
                false,
                core::ptr::null_mut(),
                0,
            );
        }

        // Bolt us to whoever is carrying us if a client
        let gv8 = ctx.world.entity(ent).genericValue8;
        if gv8 < (MAX_CLIENTS) as i32 {
            ctx.world.entity_mut(ent).s.boltToPlayer = gv8 + 1;
        } else {
            ctx.world.entity_mut(ent).s.boltToPlayer = 0;
        }

        if let Some(carrier) = carrier_id {
            // FLAG: the gclient_t deref stays raw.
            // This fn's outer unsafe block covers it.
            let ccl = ctx.world.entity(carrier).client;

            // This checking can be a bit iffy on the death stuff,
            // but in theory we should always get a think in before the default minimum respawn time is exceeded.
            if ctx.world.entity(carrier).inuse == 0
                || ccl.is_null()
                || ((*ccl).sess.sessionTeam != SIEGETEAM_TEAM1
                    && (*ccl).sess.sessionTeam != SIEGETEAM_TEAM2)
                || ((*ccl).ps.pm_flags & PMF_FOLLOW != 0)
            {
                // respawn on the original spot
                // Raven passes NULL here (g_saga.c:1435).
                SiegeItemRespawnOnOriginalSpot(ctx, ent, None);
            } else if ctx.world.entity(carrier).health < 1 {
                // The carrier died so pop out where he is (unless in nodrop).
                let target6 = ctx.world.entity(ent).target6.clone();
                if !target6.is_empty() {
                    G_UseTargets2(ctx, Some(ent), Some(ent), Some(&target6));
                }

                let carrier_origin = (*ccl).ps.origin;
                let carrier_num = ctx.world.entity(carrier).s.number;
                let contents = trap::PointContents(
                    ctx.engine,
                    mp_abi::game::syscalls::G_POINT_CONTENTS::GPointContentsArgs::new(
                        &carrier_origin as *const vec3_t,
                        carrier_num,
                    ),
                );
                if contents & CONTENTS_NODROP != 0 {
                    // In nodrop land, go back to the original spot.
                    SiegeItemRespawnOnOriginalSpot(ctx, ent, Some(carrier));
                } else {
                    G_SetOrigin(ctx.world.entity_mut(ent), carrier_origin);
                    let v0 = ctx.world.bg_state.rng.Q_irand(-80, 80) as f32;
                    ctx.world.entity_mut(ent).epVelocity[0] = v0;
                    let v1 = ctx.world.bg_state.rng.Q_irand(-80, 80) as f32;
                    ctx.world.entity_mut(ent).epVelocity[1] = v1;
                    let v2 = ctx.world.bg_state.rng.Q_irand(40, 80) as f32;
                    ctx.world.entity_mut(ent).epVelocity[2] = v2;

                    // We're in a nonstandard place, so if we go this long without being touched,
                    // assume we may not be reachable and respawn on the original spot.
                    let respawn_at = ctx.world.level.time + SIEGE_ITEM_RESPAWN_TIME;
                    ctx.world.entity_mut(ent).genericValue9 = respawn_at;

                    SiegeItemRemoveOwner(ctx, ent, Some(carrier));
                }
            }
        }

        let gv9 = ctx.world.entity(ent).genericValue9;
        if gv9 != 0 && gv9 < ctx.world.level.time {
            // time to respawn on the original spot then
            let pos1 = ctx.world.entity(ent).pos1;
            SiegeItemRespawnEffect(ctx, ent, pos1);
            G_SetOrigin(ctx.world.entity_mut(ent), pos1);
            ctx.world.entity_mut(ent).genericValue9 = 0;

            // stop flashing on radar
            ctx.world.entity_mut(ent).s.time2 = 0;
        }

        let next = ctx.world.level.time + FRAMETIME / 2;
        ctx.world.entity_mut(ent).nextthink = next;
    }
}

/// Raven `SiegeItemTouch`.
///
/// Source: `oracle/codemp/game/g_saga.c:1477-1545`
pub fn SiegeItemTouch(
    ctx: &mut GameContext,
    self_: EntityId,
    other: Option<EntityId>,
    trace: *mut trace_t,
) {
    unsafe {
        // `other`, the toucher, can be an NPC, so we read its `client` pointer through the accessor.
        // We deref it raw only once the guard proves it is a real player slot.
        let other_bad = match other {
            None => true,
            Some(o) => {
                let ocl = ctx.world.entity(o).client;
                ctx.world.entity(o).inuse == 0
                    || ocl.is_null()
                    || ctx.world.entity(o).s.eType == entityType_t::ET_NPC as c_int
            }
        };
        if other_bad {
            if !trace.is_null() && (*trace).startsolid != 0 {
                // let me out! (ideally this should not happen, but such is life)
                let mut escapePos = ctx.world.entity(self_).r.currentOrigin;
                escapePos[2] += 1.0;

                // I hope you weren't stuck in the ceiling.
                G_SetOrigin(ctx.world.entity_mut(self_), escapePos);
            }
            return;
        }
        let other = other.unwrap();

        if ctx.world.entity(other).health < 1 {
            // dead people can't pick us up.
            return;
        }

        // FLAG: the gclient_t deref stays raw.
        // This fn's outer unsafe block covers it.
        let ocl = ctx.world.entity(other).client;

        if (*ocl).holdingObjectiveItem != 0 {
            // this guy's already carrying a siege item
            return;
        }

        if (*ocl).ps.pm_type == pmtype_t::PM_SPECTATOR as c_int {
            // spectators don't pick stuff up
            return;
        }

        if ctx.world.entity(self_).genericValue2 != 0 {
            // Am I already picked up?
            return;
        }

        if ctx.world.entity(self_).genericValue6 == (*ocl).sess.sessionTeam {
            // Set to not be touchable by players on this team.
            return;
        }

        if ctx.world.globals.gSiegeRoundBegun == 0 {
            // can't pick it up if round hasn't started yet
            return;
        }

        if ctx.world.entity(self_).noise_index != 0 {
            // play the pickup noise.
            let noise_index = ctx.world.entity(self_).noise_index;
            G_Sound(ctx, Some(other), CHAN_AUTO, noise_index);
        }

        ctx.world.entity_mut(self_).genericValue2 = 1; // Mark it as picked up.

        let other_num = ctx.world.entity(other).s.number;
        (*ocl).holdingObjectiveItem = other_num;
        ctx.world.entity_mut(other).r.svFlags |= SVF_BROADCAST; // broadcast player while he carries this
        ctx.world.entity_mut(self_).genericValue8 = other_num; // Keep the index so we know who is "carrying" us

        ctx.world.entity_mut(self_).genericValue9 = 0; // So it doesn't think it has to respawn.

        let target2 = ctx.world.entity(self_).target2.clone();
        if target2.as_deref().is_some_and(|s| !s.is_empty())
            && (ctx.world.entity(self_).genericValue4 == 0
                || ctx.world.entity(self_).genericValue5 == 0)
        {
            // fire the target for pickup, if it's set to fire every time, or set to only fire the first time and the first time has not yet occured.
            G_UseTargets2(ctx, Some(self_), Some(self_), target2.as_deref());
            ctx.world.entity_mut(self_).genericValue5 = 1; // mark it as having been picked up
        }

        // time2 set to -1 will blink the item on the radar indefinately
        ctx.world.entity_mut(self_).s.time2 = -1;
    }
}

/// Raven `SiegeItemPain`.
///
/// Raven: Time 2 is used to pulse the radar icon to show its under attack.
///
/// Source: `oracle/codemp/game/g_saga.c:1547-1551`
pub fn SiegeItemPain(
    ctx: &mut GameContext,
    self_: EntityId,
    attacker: Option<EntityId>,
    damage: c_int,
) {
    // `attacker` is an unused handler param kept as Option<EntityId>.
    let t = ctx.world.level.time;
    ctx.world.entity_mut(self_).s.time2 = t;
}

/// Raven `SiegeItemDie`.
///
/// Source: `oracle/codemp/game/g_saga.c:1553-1574`
pub fn SiegeItemDie(
    ctx: &mut GameContext,
    self_: EntityId,
    inflictor: Option<EntityId>,
    attacker: Option<EntityId>,
    damage: c_int,
    meansOfDeath: c_int,
) {
    // `inflictor`/`attacker` are unused handler params kept as Option<EntityId>.
    unsafe {
        ctx.world.entity_mut(self_).takedamage = qfalse; // don't die more than once

        if ctx.world.entity(self_).genericValue3 != 0 {
            // An indexed effect to play on death
            let upAng: vec3_t = [0.0, 0.0, 1.0];
            let gv3 = ctx.world.entity(self_).genericValue3;
            let current_origin = ctx.world.entity(self_).r.currentOrigin;
            G_PlayEffectID(gv3, current_origin, upAng);
        }

        ctx.world.entity_mut(self_).neverFree = qfalse;
        ctx.world.entity_mut(self_).think = Some(EntThink::G_FreeEntity).into();
        let t = ctx.world.level.time;
        ctx.world.entity_mut(self_).nextthink = t;

        // Fire off the death target if we've got one.
        let target4 = ctx.world.entity(self_).target4.clone();
        if !target4.is_empty() {
            G_UseTargets2(ctx, Some(self_), Some(self_), Some(&target4));
        }
    }
}

/// Raven `SiegeItemUse`.
///
/// Raven: once used, become active.
///
/// Source: `oracle/codemp/game/g_saga.c:1576-1623`
pub fn SiegeItemUse(
    ctx: &mut GameContext,
    ent: EntityId,
    other: Option<EntityId>,
    activator: Option<EntityId>,
) {
    // `other`/`activator` are unused handler params kept as Option<EntityId>.
    // once used, become active
    unsafe {
        if ctx.world.entity(ent).spawnflags & SIEGEITEM_STARTOFFRADAR != 0 {
            // start showing on radar
            ctx.world.entity_mut(ent).s.eFlags |= EF_RADAROBJECT;

            if ctx.world.entity(ent).s.eFlags & EF_NODRAW != 0 {
                // we've nothing else to do here
                return;
            }
        } else {
            // make sure it's showing up
            ctx.world.entity_mut(ent).s.eFlags |= EF_RADAROBJECT;
        }

        if ctx.world.entity(ent).genericValue11 != 0 || ctx.world.entity(ent).takedamage == 0 {
            // We want to be able to walk into it to pick it up then.
            ctx.world.entity_mut(ent).r.contents = CONTENTS_TRIGGER;
            ctx.world.entity_mut(ent).clipmask = CONTENTS_SOLID | CONTENTS_TERRAIN;
            if ctx.world.entity(ent).genericValue11 != 0 {
                ctx.world.entity_mut(ent).touch = Some(EntTouch::SiegeItemTouch).into();
            }
        } else {
            // Make it solid.
            ctx.world.entity_mut(ent).r.contents = MASK_PLAYERSOLID;
            ctx.world.entity_mut(ent).clipmask = MASK_PLAYERSOLID;
        }

        ctx.world.entity_mut(ent).think = Some(EntThink::SiegeItemThink).into();
        let next = ctx.world.level.time + FRAMETIME / 2;
        ctx.world.entity_mut(ent).nextthink = next;

        // take off nodraw
        ctx.world.entity_mut(ent).s.eFlags &= !EF_NODRAW;

        // `None`/`""` ≡ Raven's `!ent->paintarget || !ent->paintarget[0]` guard.
        let paintarget = ctx.world.entity(ent).paintarget.clone();
        if let Some(paintarget) = paintarget.as_deref().filter(|s| !s.is_empty()) {
            // want to be on this guy's origin now then
            let targ = G_Find(ctx, None, EntFindField::Targetname, paintarget);
            let targ = ctx.entity_id_of(targ);

            if let Some(targ) = targ {
                if ctx.world.entity(targ).inuse != 0 {
                    let org = ctx.world.entity(targ).r.currentOrigin;
                    G_SetOrigin(ctx.world.entity_mut(ent), org);
                    trap::LinkEntity(
                        ctx.engine,
                        mp_abi::game::syscalls::G_LINKENTITY::GLinkentityArgs::new(
                            (ctx.world.entity_mut(ent) as *mut gentity_t).cast(),
                        ),
                    );
                }
            }
        }
    }
}

/// Raven `SP_misc_siege_item`.
///
/// Source: `oracle/codemp/game/g_saga.c:1676-1835`
pub fn SP_misc_siege_item(ctx: &mut GameContext, ent: EntityId) {
    unsafe {
        let mut canpickup: c_int = 0;
        let mut noradar: c_int = 0;
        let mut s: String = String::new();

        if ctx.world.bg_state.siege_valid == 0 || ctx.world.cvars.g_gametype.integer != GT_SIEGE {
            crate::g_utils::G_FreeEntity(ctx, Some(ent));
            return;
        }

        // `None`/`""` ≡ Raven's `!ent->model || !ent->model[0]` guard.
        if ctx
            .world
            .entity(ent)
            .model
            .as_deref()
            .unwrap_or("")
            .is_empty()
        {
            G_Error(ctx, "You must specify a model for misc_siege_item types.");
        }

        // G_Spawn* out-params can't alias `ctx` across the `ctx`-taking call,
        // so each entity field is copied out into a local, parsed, and copied back.
        G_SpawnInt(
            ctx,
            b"canpickup\0".as_ptr() as *const c_char,
            b"1\0".as_ptr() as *const c_char,
            &mut canpickup,
        );
        let mut genericValue1 = ctx.world.entity(ent).genericValue1;
        G_SpawnInt(
            ctx,
            b"usephysics\0".as_ptr() as *const c_char,
            b"1\0".as_ptr() as *const c_char,
            &mut genericValue1,
        );
        ctx.world.entity_mut(ent).genericValue1 = genericValue1;

        if genericValue1 != 0 {
            // if we're using physics we want lerporigin smoothing
            ctx.world.entity_mut(ent).s.eFlags |= EF_CLIENTSMOOTH;
        }

        G_SpawnInt(
            ctx,
            b"noradar\0".as_ptr() as *const c_char,
            b"0\0".as_ptr() as *const c_char,
            &mut noradar,
        );
        // Want it to always show up as a goal object on radar
        if noradar == 0 && ctx.world.entity(ent).spawnflags & SIEGEITEM_STARTOFFRADAR == 0 {
            ctx.world.entity_mut(ent).s.eFlags |= EF_RADAROBJECT;
        }

        // All clients want to know where it is at all times for radar
        ctx.world.entity_mut(ent).r.svFlags |= SVF_BROADCAST;

        let mut genericValue4 = ctx.world.entity(ent).genericValue4;
        G_SpawnInt(
            ctx,
            b"pickuponlyonce\0".as_ptr() as *const c_char,
            b"1\0".as_ptr() as *const c_char,
            &mut genericValue4,
        );
        ctx.world.entity_mut(ent).genericValue4 = genericValue4;

        let mut genericValue6 = ctx.world.entity(ent).genericValue6;
        G_SpawnInt(
            ctx,
            b"teamnotouch\0".as_ptr() as *const c_char,
            b"0\0".as_ptr() as *const c_char,
            &mut genericValue6,
        );
        ctx.world.entity_mut(ent).genericValue6 = genericValue6;
        let mut genericValue7 = ctx.world.entity(ent).genericValue7;
        G_SpawnInt(
            ctx,
            b"teamnocomplete\0".as_ptr() as *const c_char,
            b"0\0".as_ptr() as *const c_char,
            &mut genericValue7,
        );
        ctx.world.entity_mut(ent).genericValue7 = genericValue7;

        // Get default physics values.
        let mut mass = ctx.world.entity(ent).mass;
        G_SpawnFloat(
            ctx,
            b"mass\0".as_ptr() as *const c_char,
            b"0.09\0".as_ptr() as *const c_char,
            &mut mass,
        );
        ctx.world.entity_mut(ent).mass = mass;
        let mut radius = ctx.world.entity(ent).radius;
        G_SpawnFloat(
            ctx,
            b"gravity\0".as_ptr() as *const c_char,
            b"3.0\0".as_ptr() as *const c_char,
            &mut radius,
        );
        ctx.world.entity_mut(ent).radius = radius;
        let mut random = ctx.world.entity(ent).random;
        G_SpawnFloat(
            ctx,
            b"bounce\0".as_ptr() as *const c_char,
            b"1.3\0".as_ptr() as *const c_char,
            &mut random,
        );
        ctx.world.entity_mut(ent).random = random;

        s = G_SpawnString(ctx, "pickupsound", "").1;

        if !s.is_empty() {
            // We have a pickup sound, so index it now.
            ctx.world.entity_mut(ent).noise_index = G_SoundIndex(ctx, &s);
        }

        s = G_SpawnString(ctx, "deathfx", "").1;

        if !s.is_empty() {
            // We have a death effect, so index it now.
            ctx.world.entity_mut(ent).genericValue3 = G_EffectIndex(ctx, &s);
        }

        s = G_SpawnString(ctx, "respawnfx", "").1;

        if !s.is_empty() {
            // We have a respawn effect, so index it now.
            ctx.world.entity_mut(ent).genericValue10 = G_EffectIndex(ctx, &s);
        }

        s = G_SpawnString(ctx, "icon", "").1;

        if !s.is_empty() {
            // We have an icon, so index it now.  We are reusing the genericenemyindex variable rather than adding a new one to the entity state.
            let idx = G_IconIndex(ctx, &s);
            ctx.world.entity_mut(ent).s.genericenemyindex = idx;
        }

        let model = ctx.world.entity(ent).model.clone().unwrap_or_default();
        ctx.world.entity_mut(ent).s.modelindex = G_ModelIndex(ctx, &model);

        // Is the model a ghoul2 model?
        // Raven indexes `model[strlen(model) - 4]`, which underflows for names shorter than 4 chars.
        // The `>= 4` guard defines that case as leaving `modelGhoul2` unset.
        if model.len() >= 4 && model[model.len() - 4..].eq_ignore_ascii_case(".glm") {
            // apparently so.
            ctx.world.entity_mut(ent).s.modelGhoul2 = 1;
        }

        ctx.world.entity_mut(ent).s.eType = entityType_t::ET_GENERAL as c_int;

        // Set the mins/maxs with default values.
        let mut mins = ctx.world.entity(ent).r.mins;
        G_SpawnVector(
            ctx,
            b"mins\0".as_ptr() as *const c_char,
            b"-16 -16 -24\0".as_ptr() as *const c_char,
            mins.as_mut_ptr(),
        );
        ctx.world.entity_mut(ent).r.mins = mins;
        let mut maxs = ctx.world.entity(ent).r.maxs;
        G_SpawnVector(
            ctx,
            b"maxs\0".as_ptr() as *const c_char,
            b"16 16 32\0".as_ptr() as *const c_char,
            maxs.as_mut_ptr(),
        );
        ctx.world.entity_mut(ent).r.maxs = maxs;

        let s_origin = ctx.world.entity(ent).s.origin;
        crate::q_math::_VectorCopy(s_origin, &mut ctx.world.entity_mut(ent).pos1); // store off the initial origin for respawning
        G_SetOrigin(ctx.world.entity_mut(ent), s_origin);

        let s_angles = ctx.world.entity(ent).s.angles;
        crate::q_math::_VectorCopy(s_angles, &mut ctx.world.entity_mut(ent).r.currentAngles);
        crate::q_math::_VectorCopy(s_angles, &mut ctx.world.entity_mut(ent).s.apos.trBase);

        let mut genericValue15 = ctx.world.entity(ent).genericValue15;
        G_SpawnInt(
            ctx,
            b"forcelimit\0".as_ptr() as *const c_char,
            b"0\0".as_ptr() as *const c_char,
            &mut genericValue15,
        );
        ctx.world.entity_mut(ent).genericValue15 = genericValue15;

        if ctx.world.entity(ent).health > 0 {
            // If it has health, it can be killed.
            let mut t: c_int = 0;

            ctx.world.entity_mut(ent).pain = Some(EntPain::SiegeItemPain).into();
            ctx.world.entity_mut(ent).die = Some(EntDie::SiegeItemDie).into();
            ctx.world.entity_mut(ent).takedamage = qtrue;

            G_SpawnInt(
                ctx,
                b"showhealth\0".as_ptr() as *const c_char,
                b"0\0".as_ptr() as *const c_char,
                &mut t,
            );
            if t != 0 {
                // a non-0 maxhealth value will mean we want to show the health on the hud
                let health = ctx.world.entity(ent).health;
                ctx.world.entity_mut(ent).maxHealth = health;
                G_ScaleNetHealth(ctx.world.entity_mut(ent));

                let mut genericValue12 = ctx.world.entity(ent).genericValue12;
                G_SpawnInt(
                    ctx,
                    b"health_chargeamt\0".as_ptr() as *const c_char,
                    b"0\0".as_ptr() as *const c_char,
                    &mut genericValue12,
                );
                ctx.world.entity_mut(ent).genericValue12 = genericValue12;
                let mut genericValue13 = ctx.world.entity(ent).genericValue13;
                G_SpawnInt(
                    ctx,
                    b"health_chargerate\0".as_ptr() as *const c_char,
                    b"0\0".as_ptr() as *const c_char,
                    &mut genericValue13,
                );
                ctx.world.entity_mut(ent).genericValue13 = genericValue13;
            }
        } else {
            // Otherwise no.
            ctx.world.entity_mut(ent).takedamage = qfalse;
        }

        let targetname = ctx.world.entity(ent).targetname_str();
        if ctx.world.entity(ent).spawnflags & SIEGEITEM_STARTOFFRADAR != 0 {
            ctx.world.entity_mut(ent).use_ = Some(EntUse::SiegeItemUse).into();
        } else if targetname.as_deref().is_some_and(|s| !s.is_empty()) {
            ctx.world.entity_mut(ent).s.eFlags |= EF_NODRAW; // kind of hacky, but whatever
            ctx.world.entity_mut(ent).genericValue11 = canpickup;
            ctx.world.entity_mut(ent).use_ = Some(EntUse::SiegeItemUse).into();
            ctx.world.entity_mut(ent).s.eFlags &= !EF_RADAROBJECT;
        }

        if targetname.as_deref().map_or(true, |s| s.is_empty())
            || (ctx.world.entity(ent).spawnflags & SIEGEITEM_STARTOFFRADAR != 0)
        {
            if canpickup != 0 || ctx.world.entity(ent).takedamage == 0 {
                // We want to be able to walk into it to pick it up then.
                ctx.world.entity_mut(ent).r.contents = CONTENTS_TRIGGER;
                ctx.world.entity_mut(ent).clipmask = CONTENTS_SOLID | CONTENTS_TERRAIN;
                if canpickup != 0 {
                    ctx.world.entity_mut(ent).touch = Some(EntTouch::SiegeItemTouch).into();
                }
            } else {
                // Make it solid.
                ctx.world.entity_mut(ent).r.contents = MASK_PLAYERSOLID;
                ctx.world.entity_mut(ent).clipmask = MASK_PLAYERSOLID;
            }

            ctx.world.entity_mut(ent).think = Some(EntThink::SiegeItemThink).into();
            let next = ctx.world.level.time + FRAMETIME / 2;
            ctx.world.entity_mut(ent).nextthink = next;
        }

        ctx.world.entity_mut(ent).genericValue8 = ENTITYNUM_NONE; // initialize the carrier to none

        ctx.world.entity_mut(ent).neverFree = qtrue; // never free us unless we specifically request it.

        trap::LinkEntity(
            ctx.engine,
            mp_abi::game::syscalls::G_LINKENTITY::GLinkentityArgs::new(
                (ctx.world.entity_mut(ent) as *mut gentity_t).cast(),
            ),
        );
    }
}

/// Raven `G_SiegeClientExData`.
///
/// Raven: sends extra data about other client's in this client's PVS. used for support guy etc.
///
/// Source: `oracle/codemp/game/g_saga.c:1845-1886`
pub fn G_SiegeClientExData(ctx: &mut GameContext, msgTarg: EntityId) {
    unsafe {
        let mut count: c_int = 0;
        let mut i: c_int = 0;
        let mut str_buf = String::new();

        while i < ctx.world.level.num_entities && count < MAX_EXDATA_ENTS_TO_SEND {
            let id = EntityId(i as u32);
            // FLAG: this loop walks arbitrary entities, which may be NPCs.
            // We read the client pointers through the accessor,
            // and deref them raw only after the null and `ET_PLAYER` guard proves a real client slot.
            // This fn's outer unsafe block covers it.
            let ent_cl = ctx.world.entity(id).client;
            let msgtarg_cl = ctx.world.entity(msgTarg).client;
            let msgtarg_num = ctx.world.entity(msgTarg).s.number;
            let ent_num = ctx.world.entity(id).s.number;

            if ctx.world.entity(id).inuse != 0
                && !ent_cl.is_null()
                && msgtarg_num != ent_num
                && ctx.world.entity(id).s.eType == entityType_t::ET_PLAYER as c_int
                && (*msgtarg_cl).sess.sessionTeam == (*ent_cl).sess.sessionTeam
                && trap::InPVS(
                    ctx.engine,
                    mp_abi::game::syscalls::G_IN_PVS::GInPvsArgs::new(
                        &(*msgtarg_cl).ps.origin as *const vec3_t,
                        &(*ent_cl).ps.origin as *const vec3_t,
                    ),
                ) != 0
            {
                // another client in the same pvs, send his jive
                if count != 0 {
                    // append a seperating space if we are not the first in the list
                    strcat_string(&mut str_buf, MAX_STRING_CHARS, " ");
                } else {
                    // otherwise create the prepended chunk
                    str_buf = strncpyz_string(b"sxd ", MAX_STRING_CHARS);
                }

                // append the stats
                let cl = ent_cl;
                let scratch = format!(
                    "{}|{}|{}|{}",
                    ent_num,
                    (*cl).ps.stats[statIndex_t::STAT_HEALTH as usize],
                    (*cl).ps.stats[statIndex_t::STAT_MAX_HEALTH as usize],
                    (*cl).ps.ammo[weaponData[(*cl).ps.weapon as usize].ammoIndex as usize],
                );
                strcat_string(&mut str_buf, MAX_STRING_CHARS, &scratch);
                count += 1;
            }
            i += 1;
        }

        if count == 0 {
            // nothing to send
            return;
        }

        // send the string to him
        let msgtarg_num = ctx.world.entity(msgTarg).s.number;
        trap::SendServerCommand(ctx.engine, msgtarg_num, &str_buf);
    }
}
