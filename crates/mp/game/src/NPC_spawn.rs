//! FAITHFUL port of `oracle/codemp/game/NPC_spawn.c`.
//!
//! Functions reach file-scope game state (`level`, `g_entities`, cvars) and engine traps through the threaded
//! `GameContext`/`GameWorld` handle.
//! Entity-pointer parameters are `EntityId`/`Option<EntityId>` handles (§B5), not raw `gentity_t*`.
//! Ctx-free leaf helpers take `&mut gentity_t` or `&gentity_t` directly.
//!
//! Every world reach is a checked `ctx.world.…` borrow.
//! One `world_raw()` use survives as irreducible: the raw `*mut GameWorld` field of `GameCallbacksImpl` fed to
//! `BG_ParseAnimationFile`.
//!
//! Per-body `gentity_t` derefs go through `ctx.world.entity()`/`entity_mut()` accessors at point of use.
//! Pool clients (`ent.client`) and `gNPC_t` (`ent.NPC` / `globals.NPCInfo`) have no accessor, so those derefs
//! stay raw in tight `unsafe` blocks through a copied pointer value.
//! `NPC_Spawn_Do` and `NPC_SpawnType` still re-derive a raw pointer at the top of the function body: each
//! cross-copies a fresh `G_Spawn` entity with the spawner, so two live entities exist at once.
//! This file is referee-blind. Parity rests on the compile and the golden suite.
#![allow(non_snake_case, unused, clippy::all)]

use crate::g_utils::G_ModelIndex;
use crate::prelude::*;
// Dedupe the `SVF_NOCLIENT` glob ambiguity: both `g_items::*` and `g_public_consts::*` define it.
// The canonical home is `g_public_consts`, per house convention.
use crate::ent_fn_enums::dispatch_die;
use crate::g_ICARUScb::G_DebugPrint;
use crate::g_ICARUScb::Q3_SetParm;
use crate::g_public_consts::SVF_NOCLIENT;
use crate::q_shared;
use crate::NPC_stats::TeamTable;
use mp_qshared::common::mp::gentity::BSET_FIRST;
use native_string::atoi_bytes;
use native_string::latin1_to_string;
use native_string::strncpyz_string;
use native_string::Q_stricmp;
use native_string::Q_strncmp;

/// Raven `NPC_spawn.c` NPC spawnflag bit.
/// Source: `oracle/codemp/game/NPC_spawn.c:57`
pub const NSF_DROP_TO_FLOOR: c_int = 16;

/// Raven `q_shared.h` world-bounds constant.
/// Source: `oracle/codemp/game/q_shared.h:19`
pub const MIN_WORLD_COORD: f32 = -64.0 * 1024.0;

/// Raven `NPC_ShySpawn` local constants.
/// Source: `oracle/codemp/game/NPC_spawn.c:1810-1812`
pub const SHY_THINK_TIME: c_int = 1000;
pub const SHY_SPAWN_DISTANCE: c_int = 128;
pub const SHY_SPAWN_DISTANCE_SQR: c_int = SHY_SPAWN_DISTANCE * SHY_SPAWN_DISTANCE;

/// Raven `g_local.h` temp-ent removal delay.
/// Source: `oracle/codemp/game/g_local.h:48`
pub const START_TIME_REMOVE_ENTS: c_int = FRAMETIME * 3;

/// Raven `b_local.h` sound-flag bits (`SFB_*`).
/// Source: `oracle/codemp/game/b_local.h:139-141`
pub const SFB_RIFLEMAN: c_int = 2;
pub const SFB_PHASER: c_int = 4;
/// Source: `oracle/codemp/game/b_local.h:147-148`
pub const SFB_CINEMATIC: c_int = 32;
pub const SFB_NOTSOLID: c_int = 64;
/// Source: `oracle/codemp/game/b_local.h:149`
pub const SFB_STARTINSOLID: c_int = 128;

/// Raven `WP_SetSaberModel`.
///
/// Raven: rwwFIXMEFIXME: Do something here, need to let the client know.
/// Source: `oracle/codemp/game/NPC_spawn.c:90-94`
pub fn WP_SetSaberModel(client: Option<&mut gclient_t>, npcClass: class_t) -> c_int {
    // This is a ctx-free leaf function.
    // The body ignores `client`, because every caller passes `None`.
    let _ = client;
    1
}

/// Raven `NPC_PainFunc`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:103-189`
pub fn NPC_PainFunc(ent: &gentity_t) -> Option<crate::ent_fn_enums::EntPain> {
    // Raven returns the selected pain function pointer.
    // Here, fn-pointer fields are the `Option<EntPain>` fn-ID enum directly, with no `*mut c_void` encoding.
    let pain = unsafe {
        if (*((*ent).client)).ps.weapon == WP_SABER {
            crate::ent_fn_enums::EntPain::NPC_Jedi_Pain
        } else {
            match (*((*ent).client)).NPC_class {
                CLASS_STORMTROOPER | CLASS_SWAMPTROOPER => {
                    crate::ent_fn_enums::EntPain::NPC_ST_Pain
                }
                CLASS_SEEKER => crate::ent_fn_enums::EntPain::NPC_Seeker_Pain,
                CLASS_REMOTE => crate::ent_fn_enums::EntPain::NPC_Remote_Pain,
                CLASS_MINEMONSTER => crate::ent_fn_enums::EntPain::NPC_MineMonster_Pain,
                CLASS_HOWLER => crate::ent_fn_enums::EntPain::NPC_Howler_Pain,
                CLASS_GONK | CLASS_R2D2 | CLASS_R5D2 | CLASS_MOUSE | CLASS_PROTOCOL
                | CLASS_INTERROGATOR => crate::ent_fn_enums::EntPain::NPC_Droid_Pain,
                CLASS_PROBE => crate::ent_fn_enums::EntPain::NPC_Probe_Pain,
                CLASS_SENTRY => crate::ent_fn_enums::EntPain::NPC_Sentry_Pain,
                CLASS_MARK1 => crate::ent_fn_enums::EntPain::NPC_Mark1_Pain,
                CLASS_MARK2 => crate::ent_fn_enums::EntPain::NPC_Mark2_Pain,
                CLASS_ATST => crate::ent_fn_enums::EntPain::NPC_ATST_Pain,
                CLASS_GALAKMECH => crate::ent_fn_enums::EntPain::NPC_GM_Pain,
                CLASS_RANCOR => crate::ent_fn_enums::EntPain::NPC_Rancor_Pain,
                CLASS_WAMPA => crate::ent_fn_enums::EntPain::NPC_Wampa_Pain,
                _ => crate::ent_fn_enums::EntPain::NPC_Pain,
            }
        }
    };
    Some(pain)
}

/// Raven `NPC_TouchFunc`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:199-206`
pub fn NPC_TouchFunc(_ent: &gentity_t) -> Option<crate::ent_fn_enums::EntTouch> {
    // Raven always returns `NPC_Touch`. Here it returns `Option<EntTouch>` directly.
    Some(crate::ent_fn_enums::EntTouch::NPC_Touch)
}

/// Raven `NPC_SetMiscDefaultData`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:215-501`
pub fn NPC_SetMiscDefaultData(ctx: &mut GameContext, ent: EntityId) {
    // Pool client, gNPC_t, and vehicle derefs stay raw, through copied pointer values in tight `unsafe` blocks.
    // Entity fields go through the world accessor.
    let client = ctx.world.entity(ent).client;
    let npc = ctx.world.entity(ent).NPC;
    unsafe {
        if ctx.world.entity(ent).spawnflags & SFB_CINEMATIC != 0 {
            (*npc).behaviorState = BS_CINEMATIC;
        }
        if (*client).NPC_class == CLASS_BOBAFETT {
            crate::NPC_AI_Jedi::Boba_Precache(ctx);
            (*client).ps.fd.forcePowersKnown |= 1 << FP_LEVITATION;
            (*client).ps.fd.forcePowerLevel[FP_LEVITATION as usize] = FORCE_LEVEL_3;
            (*client).ps.fd.forcePower = 100;
            (*npc).scriptFlags |= SCF_ALT_FIRE | SCF_NO_GROUPS;
        }
        if ctx.world.entity(ent).s.NPC_class == CLASS_VEHICLE as c_int
            && ctx.world.entity(ent).m_pVehicle != core::ptr::null_mut()
        {
            ctx.world.entity_mut(ent).s.g2radius = 255;
            let veh = ctx.world.entity(ent).m_pVehicle;
            if (*(*veh).m_pVehicleInfo).r#type == VH_WALKER {
                ctx.world.entity_mut(ent).mass = 2000.0;
                ctx.world.entity_mut(ent).flags |= FL_SHIELDED | FL_NO_KNOCKBACK;
                ctx.world.entity_mut(ent).pain =
                    Some(crate::ent_fn_enums::EntPain::NPC_ATST_Pain).into();
            }
            let ghoul2 = ctx.world.entity(ent).ghoul2;
            trap::G2API_SetSurfaceOnOff(ctx.engine, ghoul2, "head_hatchcover", 0);
        }
        if {
            let p = ctx.world.entity(ent).NPC_type.as_deref();
            p.is_some_and(|p| Q_stricmp("wampa", p) == 0)
        } {
            crate::NPC_AI_Wampa::Wampa_SetBolts(ctx, Some(ent));
            ctx.world.entity_mut(ent).s.g2radius = 80;
            ctx.world.entity_mut(ent).mass = 300.0;
            ctx.world.entity_mut(ent).flags |= FL_NO_KNOCKBACK;
            ctx.world.entity_mut(ent).pain =
                Some(crate::ent_fn_enums::EntPain::NPC_Wampa_Pain).into();
        }
        if (*client).NPC_class == CLASS_RANCOR {
            crate::NPC_AI_Rancor::Rancor_SetBolts(ctx, Some(ent));
            ctx.world.entity_mut(ent).s.g2radius = 255;
            ctx.world.entity_mut(ent).mass = 1000.0;
            ctx.world.entity_mut(ent).flags |= FL_NO_KNOCKBACK;
            ctx.world.entity_mut(ent).pain =
                Some(crate::ent_fn_enums::EntPain::NPC_Rancor_Pain).into();
            ctx.world.entity_mut(ent).health *= 4;
        }
        if {
            let p = ctx.world.entity(ent).NPC_type.as_deref();
            p.is_some_and(|p| Q_stricmp("Yoda", p) == 0)
        } {
            (*npc).scriptFlags |= SCF_NO_FORCE;
        }
        if {
            let p = ctx.world.entity(ent).NPC_type.as_deref();
            p.is_some_and(|p| Q_stricmp("emperor", p) == 0)
        } {
            (*npc).scriptFlags |= SCF_DONT_FIRE;
        }
        if (*client).ps.weapon == WP_SABER {
            crate::w_saber::WP_SaberInitBladeData(ctx, ent);
            (*client).ps.saberHolstered = 2;
            crate::NPC_AI_Jedi::Jedi_ClearTimers(ctx, ent);
        }
        if (*client).ps.fd.forcePowersKnown != 0 {
            crate::w_force::WP_InitForcePowers(ctx, Some(ent));
            crate::w_force::WP_SpawnInitForcePowers(ctx, ent);
        }
        if (*client).NPC_class == CLASS_SEEKER {
            (*npc).defaultBehavior = BS_DEFAULT;
            (*client).ps.gravity = 0;
            (*npc).aiFlags |= NPCAI_CUSTOM_GRAVITY;
            (*client).ps.eFlags2 |= EF2_FLYING;
            ctx.world.entity_mut(ent).count = 30;
        }
        match (*client).playerTeam {
            NPCTEAM_PLAYER => {
                if (*client).NPC_class == CLASS_JEDI || (*client).NPC_class == CLASS_LUKE {
                    (*client).enemyTeam = NPCTEAM_ENEMY;
                    if ctx.world.entity(ent).spawnflags & JSF_AMBUSH != 0 {
                        (*npc).scriptFlags |= SCF_IGNORE_ALERTS;
                        (*client).noclip = qtrue;
                    }
                } else {
                    match (*client).ps.weapon {
                        WP_THERMAL | WP_BLASTER => {
                            crate::NPC_AI_Stormtrooper::ST_ClearTimers(ctx, ent);
                            if (*npc).rank >= RANK_LT || (*client).ps.weapon == WP_THERMAL {
                                // officers, grenade-throwers use alt-fire
                            }
                        }
                        _ => {}
                    }
                }
                if (*client).NPC_class == CLASS_KYLE
                    || (*client).NPC_class == CLASS_VEHICLE
                    || ctx.world.entity(ent).spawnflags & SFB_CINEMATIC != 0
                {
                    (*npc).defaultBehavior = BS_CINEMATIC;
                }
            }
            NPCTEAM_NEUTRAL => {
                if {
                    let p = ctx.world.entity(ent).NPC_type.as_deref();
                    p.is_some_and(|p| Q_stricmp(p, "gonk") == 0)
                } {
                    ctx.world.entity_mut(ent).r.svFlags |= SVF_PLAYER_USABLE;
                }
            }
            NPCTEAM_ENEMY => {
                (*npc).defaultBehavior = BS_DEFAULT;
                if (*client).NPC_class == CLASS_SHADOWTROOPER {
                    crate::NPC_AI_Jedi::Jedi_Cloak(ctx, Some(ent));
                }
                if (*client).NPC_class == CLASS_TAVION
                    || (*client).NPC_class == CLASS_REBORN
                    || (*client).NPC_class == CLASS_DESANN
                    || (*client).NPC_class == CLASS_SHADOWTROOPER
                {
                    (*client).enemyTeam = NPCTEAM_PLAYER;
                    if ctx.world.entity(ent).spawnflags & JSF_AMBUSH != 0 {
                        (*npc).scriptFlags |= SCF_IGNORE_ALERTS;
                        (*client).noclip = qtrue;
                    }
                } else if (*client).NPC_class == CLASS_PROBE
                    || (*client).NPC_class == CLASS_REMOTE
                    || (*client).NPC_class == CLASS_INTERROGATOR
                    || (*client).NPC_class == CLASS_SENTRY
                {
                    (*npc).defaultBehavior = BS_DEFAULT;
                    (*client).ps.gravity = 0;
                    (*npc).aiFlags |= NPCAI_CUSTOM_GRAVITY;
                    (*client).ps.eFlags2 |= EF2_FLYING;
                } else {
                    // Raven's `default:` case falls into `case WP_BLASTER:`, so `ST_ClearTimers` runs for
                    // `WP_BLASTER` and for any weapon outside the explicit no-op set.
                    // Source: oracle/codemp/game/NPC_spawn.c:412-458
                    match (*client).ps.weapon {
                        WP_BRYAR_PISTOL | WP_DISRUPTOR | WP_BOWCASTER | WP_REPEATER | WP_DEMP2
                        | WP_FLECHETTE | WP_ROCKET_LAUNCHER | WP_THERMAL | WP_STUN_BATON => {}
                        _ => {
                            crate::NPC_AI_Stormtrooper::ST_ClearTimers(ctx, ent);
                        }
                    }
                    if {
                        let p = ctx.world.entity(ent).NPC_type.as_deref();
                        p.is_some_and(|p| Q_stricmp(p, "galak_mech") == 0)
                    } {
                        crate::NPC_AI_GalakMech::NPC_GalakMech_Init(ctx, ent);
                    }
                }
            }
            _ => {}
        }

        if (*client).NPC_class == CLASS_SEEKER && ctx.world.entity(ent).activator.is_some() {
            // assume my teams are already set correctly
        } else if ctx.world.cvars.g_gametype.integer == GT_SIEGE
            && ctx.world.entity(ent).s.NPC_class != CLASS_VEHICLE as c_int
        {
            if (*client).enemyTeam == NPCTEAM_PLAYER {
                (*client).sess.sessionTeam = SIEGETEAM_TEAM1;
            } else if (*client).enemyTeam == NPCTEAM_ENEMY {
                (*client).sess.sessionTeam = SIEGETEAM_TEAM2;
            } else {
                (*client).sess.sessionTeam = TEAM_FREE;
            }
        }

        if (*client).NPC_class == CLASS_ATST || (*client).NPC_class == CLASS_MARK1 {
            ctx.world.entity_mut(ent).flags |= FL_SHIELDED | FL_NO_KNOCKBACK;
        }
    }
}

/// Raven `NPC_WeaponsForTeam`.
///
/// Raven: not sure how to handle this, should I pass in class instead of team
/// and go from there? - dmv
/// Source: `oracle/codemp/game/NPC_spawn.c:509-749`
pub fn NPC_WeaponsForTeam(team: team_t, spawnflags: c_int, NPC_type: &str) -> c_int {
    // This is a faithful transcription of the C string-compare cascade.
    // `NPC_type` is the NPC species name.
    // The caller maps Raven's NULL pointer to an empty string.
    let name = NPC_type;

    let stricmp = |a: &str, b: &str| a.eq_ignore_ascii_case(b);
    // `Q_strncmp` is case-sensitive, unlike `Q_stricmp`. It compares prefixes exactly.
    let strncmp = |a: &str, b: &str, n: usize| {
        let a_pre: String = a.chars().take(n).collect();
        let b_pre: String = b.chars().take(n).collect();
        a_pre == b_pre && a.chars().count() >= n && b.chars().count() >= n
    };

    match team {
        NPCTEAM_ENEMY => {
            if stricmp(name, "tavion")
                || strncmp(name, "reborn", 6)
                || stricmp(name, "desann")
                || strncmp(name, "shadowtrooper", 13)
            {
                return 1 << WP_SABER;
            }
            if strncmp(name, "stofficer", 9) {
                return 1 << WP_FLECHETTE;
            }
            if stricmp(name, "stcommander") {
                return 1 << WP_REPEATER;
            }
            if stricmp(name, "swamptrooper") {
                return 1 << WP_FLECHETTE;
            }
            if stricmp(name, "swamptrooper2") {
                return 1 << WP_REPEATER;
            }
            if stricmp(name, "rockettrooper") {
                return 1 << WP_ROCKET_LAUNCHER;
            }
            if strncmp(name, "shadowtrooper", 13) {
                return 1 << WP_SABER;
            }
            if stricmp(name, "imperial") {
                return 1 << WP_BLASTER;
            }
            if strncmp(name, "impworker", 9) {
                return 1 << WP_BLASTER;
            }
            if stricmp(name, "stormpilot") {
                return 1 << WP_BLASTER;
            }
            if stricmp(name, "galak") {
                return 1 << WP_BLASTER;
            }
            if stricmp(name, "galak_mech") {
                return 1 << WP_REPEATER;
            }
            if strncmp(name, "ugnaught", 8) {
                return WP_NONE;
            }
            if stricmp(name, "granshooter") {
                return 1 << WP_BLASTER;
            }
            if stricmp(name, "granboxer") {
                return 1 << WP_STUN_BATON;
            }
            if strncmp(name, "gran", 4) {
                return (1 << WP_THERMAL) | (1 << WP_STUN_BATON);
            }
            if stricmp(name, "rodian") {
                return 1 << WP_DISRUPTOR;
            }
            if stricmp(name, "rodian2") {
                return 1 << WP_BLASTER;
            }
            if stricmp(name, "interrogator")
                || stricmp(name, "sentry")
                || strncmp(name, "protocol", 8)
            {
                return WP_NONE;
            }
            if strncmp(name, "weequay", 7) {
                return 1 << WP_BOWCASTER;
            }
            if stricmp(name, "impofficer") {
                return 1 << WP_BLASTER;
            }
            if stricmp(name, "impcommander") {
                return 1 << WP_BLASTER;
            }
            if stricmp(name, "probe") || stricmp(name, "seeker") {
                return 0;
            }
            if stricmp(name, "remote") {
                return 0;
            }
            if stricmp(name, "trandoshan") {
                return 1 << WP_REPEATER;
            }
            if stricmp(name, "atst") {
                return 0;
            }
            if stricmp(name, "mark1") {
                return 0;
            }
            if stricmp(name, "mark2") {
                return 0;
            }
            if stricmp(name, "minemonster") {
                return 1 << WP_STUN_BATON;
            }
            if stricmp(name, "howler") {
                return 1 << WP_STUN_BATON;
            }
            // Stormtroopers, etc.
            1 << WP_BLASTER
        }
        NPCTEAM_PLAYER => {
            if spawnflags & SFB_RIFLEMAN != 0 {
                return 1 << WP_REPEATER;
            }
            if spawnflags & SFB_PHASER != 0 {
                return 1 << WP_BLASTER;
            }
            if strncmp(name, "jedi", 4) || stricmp(name, "luke") {
                return 1 << WP_SABER;
            }
            if strncmp(name, "prisoner", 8) {
                return WP_NONE;
            }
            if strncmp(name, "bespincop", 9) {
                return 1 << WP_BLASTER;
            }
            if stricmp(name, "MonMothma") {
                return WP_NONE;
            }
            // rebel
            1 << WP_BLASTER
        }
        NPCTEAM_NEUTRAL => {
            if stricmp(name, "mark1") {
                return WP_NONE;
            }
            if stricmp(name, "mark2") {
                return WP_NONE;
            }
            if strncmp(name, "ugnaught", 8) {
                return WP_NONE;
            }
            if stricmp(name, "bartender") {
                return WP_NONE;
            }
            if stricmp(name, "morgankatarn") {
                return WP_NONE;
            }
            WP_NONE
        }
        _ => WP_NONE,
    }
}

/// Raven `NPC_SetWeapons`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:759-797`
pub fn NPC_SetWeapons(ctx: &mut GameContext, ent: EntityId) {
    // Pool client and gNPC_t derefs stay raw, through copied pointer values.
    let client = ctx.world.entity(ent).client;
    let npc = ctx.world.entity(ent).NPC;
    let mut bestWeap: c_int = WP_NONE;
    let player_team = unsafe { (*client).playerTeam };
    let spawnflags = ctx.world.entity(ent).spawnflags;
    let npc_type = ctx.world.entity(ent).NPC_type.clone();
    let weapons = NPC_WeaponsForTeam(player_team, spawnflags, npc_type.as_deref().unwrap_or(""));

    unsafe {
        (*client).ps.stats[STAT_WEAPONS as usize] = 0;
        let mut curWeap = WP_SABER;
        while curWeap < WP_NUM_WEAPONS {
            if weapons & (1 << curWeap) != 0 {
                (*client).ps.stats[STAT_WEAPONS as usize] |= 1 << curWeap;
                let ammo_index = weaponData[curWeap as usize].ammoIndex;
                (*client).ps.ammo[ammo_index as usize] = 100;
                (*npc).currentAmmo = 100;

                if bestWeap == WP_SABER {
                    curWeap += 1;
                    continue;
                }

                if curWeap == WP_STUN_BATON {
                    if bestWeap == WP_NONE {
                        bestWeap = curWeap;
                    }
                } else if curWeap > bestWeap || bestWeap == WP_STUN_BATON {
                    bestWeap = curWeap;
                }
            }
            curWeap += 1;
        }

        (*client).ps.weapon = bestWeap;
    }
}

/// Raven `NPC_SpawnEffect`.
///
/// Raven: NOTE: Make sure any effects called here have their models, tga's
/// and sounds precached in CG_RegisterNPCEffects in cg_player.cpp.
/// Source: `oracle/codemp/game/NPC_spawn.c:808-810`
pub fn NPC_SpawnEffect(ent: &gentity_t) {
    // The body is empty in the oracle. This is an effect hook that Raven never filled in.
    let _ = ent;
}

/// Raven `NPC_SetFX_SpawnStates`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:817-823`
pub fn NPC_SetFX_SpawnStates(ctx: &mut GameContext, ent: EntityId) {
    // Pool client and gNPC_t derefs stay raw, through copied pointer values.
    let npc = ctx.world.entity(ent).NPC;
    let client = ctx.world.entity(ent).client;
    unsafe {
        if (*npc).aiFlags & NPCAI_CUSTOM_GRAVITY == 0 {
            (*client).ps.gravity = ctx.world.cvars.g_gravity.value as c_int;
        }
    }
}

/// Raven `NPC_SpotWouldTelefrag`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:831-859`
pub fn NPC_SpotWouldTelefrag(ctx: &mut GameContext, npc: EntityId) -> qboolean {
    let npc_origin = ctx.world.entity(npc).r.currentOrigin;
    let npc_mins = ctx.world.entity(npc).r.mins;
    let npc_maxs = ctx.world.entity(npc).r.maxs;
    let npc_number = ctx.world.entity(npc).s.number;
    let npc_ownernum = ctx.world.entity(npc).r.ownerNum;
    let mut mins: vec3_t = [0.0; 3];
    let mut maxs: vec3_t = [0.0; 3];
    crate::q_math::_VectorAdd(npc_origin, npc_mins, &mut mins);
    crate::q_math::_VectorAdd(npc_origin, npc_maxs, &mut maxs);
    let mut touch: [c_int; MAX_GENTITIES as usize] = [0; MAX_GENTITIES as usize];
    let num = unsafe {
        trap::EntitiesInBox(
            ctx.engine,
            mp_abi::game::syscalls::G_ENTITIES_IN_BOX::GEntitiesInBoxArgs::new(
                &mins as *const vec3_t,
                &maxs as *const vec3_t,
                touch.as_mut_ptr(),
                (MAX_GENTITIES) as i32,
            ),
        )
    };

    for i in 0..num {
        let hit = ctx.world.entity(EntityId(touch[i as usize] as u32));
        if hit.inuse != 0
            && !hit.client.is_null()
            && hit.s.number != npc_number
            && hit.r.contents & MASK_NPCSOLID != 0
            && hit.s.number != npc_ownernum
            && hit.r.ownerNum != npc_number
        {
            return qtrue;
        }
    }

    qfalse
}

/// Raven `NPC_Begin`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:862-1274`
pub fn NPC_Begin(ctx: &mut GameContext, ent: EntityId) {
    // Pool client and gNPC_t derefs stay raw, through copied pointer values.
    // Entity fields go through the world accessor.
    let client = ctx.world.entity(ent).client;
    let npc = ctx.world.entity(ent).NPC;
    unsafe {
        let mut spawn_origin: vec3_t = [0.0; 3];
        let mut spawn_angles: vec3_t = [0.0; 3];
        let mut ucmd: usercmd_t = core::mem::zeroed();
        let spawn_point: *mut gentity_t = core::ptr::null_mut();

        if ctx.world.entity(ent).spawnflags & SFB_NOTSOLID == 0 {
            if NPC_SpotWouldTelefrag(ctx, ent) != 0 {
                if ctx.world.entity(ent).wait < (0) as f32 {
                    let t3 = ctx.world.entity(ent).target3.clone();
                    let tn = ctx.world.entity(ent).targetname_str().unwrap_or_default();
                    G_DebugPrint(
                        ctx,
                        WL_DEBUG as i32,
                        &format!(
                            "NPC {} could not spawn, firing target3 ({}) and removing self\n",
                            tn, t3
                        ),
                    );
                    // `target3` is an owned `String`, where an empty value stands for absent.
                    // An empty value fires nothing, matching Raven's NULL-pointer skip.
                    let t3s = (!t3.is_empty()).then_some(t3);
                    G_UseTargets2(ctx, Some(ent), Some(ent), t3s.as_deref());
                    ctx.world.entity_mut(ent).think = Some(EntThink::G_FreeEntity).into();
                    let nt = ctx.world.level.time + 100;
                    ctx.world.entity_mut(ent).nextthink = nt;
                } else {
                    let tn = ctx.world.entity(ent).targetname_str().unwrap_or_default();
                    let wait = ctx.world.entity(ent).wait;
                    G_DebugPrint(
                        ctx,
                        WL_DEBUG as i32,
                        &format!(
                            "NPC {} could not spawn, waiting {:.2} secs to try again\n",
                            tn,
                            wait as f32 / 1000.0f32
                        ),
                    );
                    ctx.world.entity_mut(ent).think = Some(EntThink::NPC_Begin).into();
                    let nt = ((ctx.world.level.time as f32) + ctx.world.entity(ent).wait) as i32;
                    ctx.world.entity_mut(ent).nextthink = nt;
                }
                return;
            }
        }
        NPC_SpawnEffect(ctx.world.entity(ent));

        crate::q_math::_VectorCopy((*client).ps.origin, &mut spawn_origin);
        crate::q_math::_VectorCopy(ctx.world.entity(ent).s.angles, &mut spawn_angles);
        spawn_angles[YAW as usize] = (*npc).desiredYaw;

        (*client).ps.persistant[PERS_SPAWN_COUNT as usize] += 1;
        (*client).airOutTime = ctx.world.level.time + 12000;
        (*client).ps.clientNum = ctx.world.entity(ent).s.number;

        if ctx.world.entity(ent).health != 0 {
            (*client).pers.maxHealth = ctx.world.entity(ent).health;
            (*client).ps.stats[STAT_MAX_HEALTH as usize] = ctx.world.entity(ent).health;
        } else if (*npc).stats.health != 0 {
            if (*client).NPC_class != CLASS_REBORN
                && (*client).NPC_class != CLASS_SHADOWTROOPER
                && (*client).NPC_class != CLASS_JEDI
            {
                (*npc).stats.health += (*npc).stats.health / 4 * ctx.world.cvars.g_spskill.integer;
            }
            (*client).pers.maxHealth = (*npc).stats.health;
            (*client).ps.stats[STAT_MAX_HEALTH as usize] = (*npc).stats.health;
        } else {
            (*client).pers.maxHealth = 100;
            (*client).ps.stats[STAT_MAX_HEALTH as usize] = 100;
        }

        if {
            let p = ctx.world.entity(ent).NPC_type.as_deref();
            p.is_some_and(|p| Q_stricmp("rodian", p) == 0)
        } {
            match ctx.world.cvars.g_spskill.integer {
                0 => (*npc).stats.aim = (1.0) as i32,
                1 => (*npc).stats.aim = (ctx.world.bg_state.rng.Q_irand(2, 3) as f32) as i32,
                2 => (*npc).stats.aim = (ctx.world.bg_state.rng.Q_irand(3, 4) as f32) as i32,
                _ => {}
            }
        } else {
            if (*client).NPC_class == CLASS_STORMTROOPER
                || (*client).NPC_class == CLASS_SWAMPTROOPER
                || (*client).NPC_class == CLASS_IMPWORKER
                || {
                    let p = ctx.world.entity(ent).NPC_type.as_deref();
                    p.is_some_and(|p| Q_stricmp("rodian2", p) == 0)
                }
            {
                match ctx.world.cvars.g_spskill.integer {
                    0 => {
                        (*npc).stats.yawSpeed *= 0.75f32;
                        if (*client).NPC_class == CLASS_IMPWORKER {
                            (*npc).stats.aim -= ctx.world.bg_state.rng.Q_irand(3, 6);
                        }
                    }
                    1 => {
                        if (*client).NPC_class == CLASS_IMPWORKER {
                            (*npc).stats.aim -= ctx.world.bg_state.rng.Q_irand(2, 4);
                        }
                    }
                    2 => {
                        (*npc).stats.yawSpeed *= 1.5f32;
                        if (*client).NPC_class == CLASS_IMPWORKER {
                            (*npc).stats.aim -= ctx.world.bg_state.rng.Q_irand(0, 2);
                        }
                    }
                    _ => {}
                }
            } else if (*client).NPC_class == CLASS_REBORN
                || (*client).NPC_class == CLASS_SHADOWTROOPER
            {
                match ctx.world.cvars.g_spskill.integer {
                    1 => (*npc).stats.yawSpeed *= 1.25f32,
                    2 => (*npc).stats.yawSpeed *= 1.5f32,
                    _ => {}
                }
            }
        }

        ctx.world.entity_mut(ent).s.groundEntityNum = ENTITYNUM_NONE;
        ctx.world.entity_mut(ent).mass = 10.0;
        ctx.world.entity_mut(ent).takedamage = qtrue;
        ctx.world.entity_mut(ent).inuse = qtrue;
        ctx.ent_set(ent, PrefixSet::ClassnameStatic(c"NPC"));
        if ctx.world.entity(ent).spawnflags & SFB_NOTSOLID == 0 {
            ctx.world.entity_mut(ent).r.contents = CONTENTS_BODY;
            ctx.world.entity_mut(ent).clipmask = MASK_NPCSOLID;
        } else {
            ctx.world.entity_mut(ent).r.contents = 0;
            ctx.world.entity_mut(ent).clipmask = MASK_NPCSOLID & !CONTENTS_BODY;
        }

        ctx.world.entity_mut(ent).die = Some(EntDie::player_die).into();
        ctx.world.entity_mut(ent).waterlevel = 0;
        ctx.world.entity_mut(ent).watertype = 0;
        (*client).ps.rocketLockIndex = ENTITYNUM_NONE;
        (*client).ps.rocketLockTime = (0) as f32;

        if (*client).NPC_class != CLASS_R2D2
            && (*client).NPC_class != CLASS_R5D2
            && (*client).NPC_class != CLASS_MOUSE
            && (*client).NPC_class != CLASS_GONK
            && (*client).NPC_class != CLASS_PROTOCOL
        {
            ctx.world.entity_mut(ent).flags &= !FL_NOTARGET;
        }
        ctx.world.entity_mut(ent).s.eFlags &= !EF_NODRAW;

        NPC_SetFX_SpawnStates(ctx, ent);

        if (*client).ps.weapon == WP_NONE {
            NPC_SetWeapons(ctx, ent);
        }
        (*npc).currentAmmo =
            (*client).ps.ammo[weaponData[(*client).ps.weapon as usize].ammoIndex as usize];
        (*client).ps.weaponstate = (WEAPON_IDLE) as i32;
        ChangeWeapon(ctx, Some(ent), (*client).ps.weapon);

        crate::q_math::_VectorCopy(spawn_origin, &mut (*client).ps.origin);

        (*client).ps.pm_flags |= PMF_RESPAWNED;

        ctx.world.entity_mut(ent).s.eType = ET_NPC as c_int;

        crate::q_math::_VectorCopy(spawn_origin, &mut ctx.world.entity_mut(ent).s.origin);

        SetClientViewAngle(ctx.world.entity_mut(ent), spawn_angles);
        (*client).renderInfo.lookTarget = ENTITYNUM_NONE;

        if ctx.world.entity(ent).spawnflags & 64 == 0 {
            crate::g_utils::G_KillBox(ctx, ent);
            let e_ptr = ctx.world.entity_mut(ent) as *mut gentity_t;
            trap::LinkEntity(
                ctx.engine,
                mp_abi::game::syscalls::G_LINKENTITY::GLinkentityArgs::new(e_ptr.cast()),
            );
        }

        (*client).ps.pm_flags |= PMF_TIME_KNOCKBACK;
        (*client).ps.pm_time = 100;

        (*client).respawnTime = ctx.world.level.time;
        (*client).inactivityTime =
            ctx.world.level.time + (ctx.world.cvars.g_inactivity.value as c_int) * 1000;
        (*client).latched_buttons = 0;
        if ctx.world.entity(ent).s.m_iVehicleNum != 0 {
            // I'm an NPC in a vehicle (or a vehicle), I already have owner set
        } else if (*client).NPC_class == CLASS_SEEKER && ctx.world.entity(ent).activator.is_some() {
            let act = ctx.world.entity(ent).activator.unwrap();
            let num = ctx.world.entity(act).s.number;
            ctx.world.entity_mut(ent).s.owner = num;
            ctx.world.entity_mut(ent).r.ownerNum = num;
        } else {
            ctx.world.entity_mut(ent).s.owner = ENTITYNUM_NONE;
        }

        if (*client).NPC_class != CLASS_VEHICLE {
            NPC_SetAnim(
                ctx,
                ent,
                SETANIM_BOTH,
                (BOTH_STAND1) as i32,
                SETANIM_FLAG_NORMAL,
            );
        }

        if !spawn_point.is_null() {
            crate::g_utils::G_UseTargets(ctx, ctx.entity_id_of(spawn_point), Some(ent));
        }

        let e_ptr = ctx.world.entity_mut(ent) as *mut gentity_t;
        trap::ICARUS_InitEnt(
            ctx.engine,
            mp_abi::game::syscalls::G_ICARUS_INITENT::GIcarusInitentArgs::new(e_ptr.cast()),
        );

        SetNPCGlobals(ctx, ent);

        ctx.world.entity_mut(ent).enemy = None;
        // `gNPC_t` (`NPCInfo`) has no accessor. The deref stays raw.
        let npc_info = ctx.world.globals.NPCInfo;
        (*npc_info).timeOfDeath = 0;
        (*npc_info).shotTime = 0;
        crate::NPC_goal::NPC_ClearGoal(ctx);
        NPC_ChangeWeapon((*client).ps.weapon);

        ctx.world.entity_mut(ent).pain = Some(crate::ent_fn_enums::EntPain::NPC_Pain).into();
        // The pain and touch fn-ID enums come straight from the selector functions.
        let pain_opt = NPC_PainFunc(ctx.world.entity(ent));
        ctx.world.entity_mut(ent).pain = pain_opt.into();
        let touch_opt = NPC_TouchFunc(ctx.world.entity(ent));
        ctx.world.entity_mut(ent).touch = touch_opt.into();

        (*client).ps.ping = (*npc).stats.reactions * 50;

        if ctx.world.entity(ent).s.NPC_class != CLASS_VEHICLE as c_int
            || ctx.world.cvars.g_gametype.integer != GT_SIEGE
        {
            (*client).ps.persistant[PERS_TEAM as usize] = (*client).playerTeam;
        }

        ctx.world.entity_mut(ent).use_ = Some(EntUse::NPC_Use).into();
        ctx.world.entity_mut(ent).think = Some(EntThink::NPC_Think).into();
        let nt = ctx.world.level.time + FRAMETIME + ctx.world.bg_state.rng.Q_irand(0, 100);
        ctx.world.entity_mut(ent).nextthink = nt;

        NPC_SetMiscDefaultData(ctx, ent);
        if ctx.world.entity(ent).health <= 0 {
            ctx.world.entity_mut(ent).health = (*client).pers.maxHealth;
            (*client).ps.stats[STAT_HEALTH as usize] = ctx.world.entity(ent).health;
        } else {
            (*client).ps.stats[STAT_HEALTH as usize] = ctx.world.entity(ent).health;
        }

        if ctx.world.entity(ent).s.shouldtarget != 0 {
            let h = ctx.world.entity(ent).health;
            ctx.world.entity_mut(ent).maxHealth = h;
            crate::g_utils::G_ScaleNetHealth(ctx.world.entity_mut(ent));
        }

        ChangeWeapon(ctx, Some(ent), (*client).ps.weapon);

        if ctx.world.entity(ent).spawnflags & SFB_STARTINSOLID == 0 {
            crate::g_utils::G_CheckInSolid(ctx, ent, qtrue);
        }
        (*npc).lastClearOrigin = [0.0; 3];

        if crate::NPC_utils::G_ActivateBehavior(ctx, Some(ent), (BSET_SPAWN) as i32) != 0 {
            let num = ctx.world.entity(ent).s.number;
            trap::ICARUS_MaintainTaskManager(ctx.engine, mp_abi::game::syscalls::G_ICARUS_MAINTAINTASKMANAGER::GIcarusMaintaintaskmanagerArgs::new(num));
        }

        crate::q_math::_VectorCopy(
            ctx.world.entity(ent).r.currentOrigin,
            &mut (*client).renderInfo.eyePoint,
        );

        ucmd = core::mem::zeroed();
        // Raven's `VectorCopy` is a macro. Here it copies the `int angles[3]`.
        ucmd.angles = (*client).pers.cmd.angles;

        (*client).ps.groundEntityNum = ENTITYNUM_NONE;

        // Raven gates a commented-out `G_MatchPlayerWeapon` call behind `NPCAI_MATCHPLAYERWEAPON`.
        // The guard has no effect, so it is dropped here.

        let num = ctx.world.entity(ent).s.number;
        ClientThink(ctx, num, &mut ucmd as *mut usercmd_t);

        let e_ptr = ctx.world.entity_mut(ent) as *mut gentity_t;
        trap::LinkEntity(
            ctx.engine,
            mp_abi::game::syscalls::G_LINKENTITY::GLinkentityArgs::new(e_ptr.cast()),
        );

        if (*client).playerTeam == NPCTEAM_ENEMY {
            if ctx.world.entity(ent).spawnflags & SFB_CINEMATIC == 0
                && (*npc).behaviorState != BS_CINEMATIC
            {
                // Raven's `g_entities[0].client` stats bump here is commented out in the oracle, so this block
                // stays a no-op.
            }
        }
        ctx.world.entity_mut(ent).waypoint = WAYPOINT_NONE;
        (*npc).homeWp = WAYPOINT_NONE;

        // FLAG: this is a two-entity copy.
        // A fresh droid NPC (`droid_ent`, a raw `*mut gentity_t` from `NPC_SpawnType`) is cross-copied with the
        // parent `ent`.
        // Parent fields are read into locals first, so the `ctx` borrow ends before each raw droid write.
        // `droid_ent`, `veh`, and the pool clients stay raw.
        // The copy-out/copy-in pilot shape does not apply here, so this stays open work.
        let veh = ctx.world.entity(ent).m_pVehicle;
        if !veh.is_null() {
            if (*veh).m_iDroidUnitTag != -1 {
                let model2 = ctx.world.entity(ent).model2.clone();
                // Raven prefers `ent->model2`, and falls back to the vehicle's `droidNPC`.
                // `None` stands for the C null pointer that skips the spawn.
                // Raven's guard (`NPC_spawn.c:1213-1214`) is `droidNPC && droidNPC[0]`, non-null and non-empty.
                // This site checks only null, so a non-null empty `droidNPC` string enters this block that Raven skips.
                // This divergence is tracked as issue #47.
                let droid_npc_type = if !model2.is_empty() {
                    Some(model2)
                } else if !(*(*veh).m_pVehicleInfo).droidNPC.is_null() {
                    Some(cstr_to_str(
                        (*(*veh).m_pVehicleInfo).droidNPC as *const c_char,
                    ))
                } else {
                    None
                };

                if let Some(mut droid_npc_type_s) = droid_npc_type {
                    if Q_stricmp("random", &droid_npc_type_s) == 0
                        || Q_stricmp("default", &droid_npc_type_s) == 0
                    {
                        droid_npc_type_s = if ctx.world.bg_state.rng.Q_irand(0, 1) != 0 {
                            "r2d2".to_string()
                        } else {
                            "r5d2".to_string()
                        };
                    }
                    let droid_ent = NPC_SpawnType(ctx, Some(ent), &droid_npc_type_s, None, qfalse);
                    if !droid_ent.is_null() {
                        if !(*droid_ent).client.is_null() {
                            let ent_number = ctx.world.entity(ent).s.number;
                            let ent_allied = ctx.world.entity(ent).alliedTeam;
                            let ent_teamnodmg = ctx.world.entity(ent).teamnodmg;
                            let ent_origin = ctx.world.entity(ent).r.currentOrigin;
                            let ent_angles = ctx.world.entity(ent).r.currentAngles;
                            let client_sess = (*client).sess.sessionTeam;
                            let client_pers_team = (*client).ps.persistant[PERS_TEAM as usize];
                            (*((*droid_ent).client)).ps.m_iVehicleNum = ent_number;
                            (*droid_ent).s.m_iVehicleNum = ent_number;
                            (*droid_ent).s.owner = ent_number;
                            (*droid_ent).r.ownerNum = ent_number;
                            // `Vehicle_t.m_pDroidUnit` is `mp_bg`'s own `bgEntity_t`.
                            // This crate's own `bgEntity_t` name is the prelude's `gentity_t` alias, so the
                            // overlay cast targets the bg type fully qualified.
                            (*veh).m_pDroidUnit =
                                droid_ent as *mut mp_bg::public::bg_entity::bgEntity_t;
                            (*droid_ent).alliedTeam = ent_allied;
                            (*droid_ent).teamnodmg = ent_teamnodmg;
                            (*((*droid_ent).client)).sess.sessionTeam = client_sess;
                            (*((*droid_ent).client)).ps.persistant[PERS_TEAM as usize] =
                                client_pers_team;
                            crate::q_math::_VectorCopy(ent_origin, &mut (*droid_ent).s.origin);
                            crate::q_math::_VectorCopy(
                                ent_origin,
                                &mut (*((*droid_ent).client)).ps.origin,
                            );
                            let d_origin = (*droid_ent).s.origin;
                            crate::g_utils::G_SetOrigin(&mut *droid_ent, d_origin);
                            trap::LinkEntity(
                                ctx.engine,
                                mp_abi::game::syscalls::G_LINKENTITY::GLinkentityArgs::new(
                                    droid_ent.cast(),
                                ),
                            );
                            crate::q_math::_VectorCopy(ent_angles, &mut (*droid_ent).s.angles);
                            let d_angles = (*droid_ent).s.angles;
                            crate::g_utils::G_SetAngles(&mut *droid_ent, d_angles);
                            if !(*droid_ent).NPC.is_null() {
                                (*((*droid_ent).NPC)).desiredYaw =
                                    (*droid_ent).s.angles[YAW as usize];
                                (*((*droid_ent).NPC)).desiredPitch =
                                    (*droid_ent).s.angles[PITCH as usize];
                            }
                            (*droid_ent).flags |= FL_UNDYING;
                        } else {
                            crate::g_utils::G_FreeEntity(ctx, ctx.entity_id_of(droid_ent));
                        }
                    }
                }
            }
        }
    }
}

/// Raven `New_NPC_t`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:1278-1297`
pub fn New_NPC_t(ctx: &mut GameContext, entNum: c_int) -> *mut gNPC_t {
    unsafe {
        if (&ctx.world.globals.gNPCPtrs)[entNum as usize].is_null() {
            // `gNPC_t` holds a `*mut AIGroupInfo_t` field with 8-byte alignment.
            // Pad to an 8-byte boundary first, with `BG_AllocPad8`, so every `(*ptr).field` access downstream
            // is safely dereferenceable.
            mp_bg::bg_misc::BG_AllocPad8(&mut ctx.world.bg_state);
            (&mut ctx.world.globals.gNPCPtrs)[entNum as usize] = BG_Alloc(
                core::mem::size_of::<gNPC_t>() as c_int,
                &mut ctx.world.bg_state,
            ) as *mut gNPC_t;
        }

        let ptr = (&ctx.world.globals.gNPCPtrs)[entNum as usize];

        if !ptr.is_null() {
            // This zeroes byte-wise, like C's `memset`.
            // `ptr` is `BG_Alloc` pool storage, aligned to only 4 bytes, so it is not guaranteed 8-aligned for
            // `gNPC_t`'s pointer field.
            core::ptr::write_bytes(ptr as *mut u8, 0, core::mem::size_of::<gNPC_t>());
        }

        ptr
    }
}

/// Raven `NPC_DefaultScriptFlags`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:1356-1364`
pub fn NPC_DefaultScriptFlags(ent: &gentity_t) {
    // This ctx-free leaf takes `&gentity_t`.
    // The caller's `ent.is_null()` guard is vacuous behind a reference, so it is dropped, and the `NPC` guard
    // stays.
    let npc = ent.NPC;
    if npc.is_null() {
        return;
    }
    // The `gNPC_t` deref stays raw: a copied pointer value in a tight `unsafe` block.
    unsafe {
        (*npc).scriptFlags = SCF_CHASE_ENEMIES | SCF_LOOK_FOR_ENEMIES;
    }
}

/// Raven `NPC_Spawn_Do`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:1377-1763`
pub fn NPC_Spawn_Do(ctx: &mut GameContext, ent: EntityId) -> *mut gentity_t {
    unsafe {
        // The `EntityId` parameter re-derives to a raw pointer, and the body stays verbatim raw-pointer code,
        // as open work.
        // The return stays a raw `*mut gentity_t`, because return-type conversion is a later pass.
        let ent: *mut gentity_t = ctx.entity_mut(ent);
        let mut newent: *mut gentity_t = core::ptr::null_mut();
        let mut save_org: vec3_t = [0.0; 3];

        if (*ent).spawnflags & NSF_DROP_TO_FLOOR != 0 {
            let mut tr: trace_t = core::mem::zeroed();
            let mut bottom: vec3_t = [0.0; 3];

            crate::q_math::_VectorCopy((*ent).r.currentOrigin, &mut save_org);
            crate::q_math::_VectorCopy((*ent).r.currentOrigin, &mut bottom);
            bottom[2] = MIN_WORLD_COORD as f32;
            trap::Trace(
                ctx.engine,
                mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
                    &mut tr as *mut trace_t,
                    &(*ent).r.currentOrigin as *const vec3_t,
                    &(*ent).r.mins as *const vec3_t,
                    &(*ent).r.maxs as *const vec3_t,
                    &bottom as *const vec3_t,
                    (*ent).s.number,
                    MASK_NPCSOLID,
                ),
            );
            if tr.allsolid == 0 && tr.startsolid == 0 && tr.fraction < 1.0 {
                crate::g_utils::G_SetOrigin(&mut *(ent), tr.endpos);
            }
        }

        if (*ent).count != -1 {
            (*ent).count -= 1;
            if (*ent).count <= 0 {
                (*ent).use_ = FnId::NONE;
            }
        }

        let __teid16 = crate::g_utils::G_Spawn(ctx);
        newent = ctx.entity_mut(__teid16);

        if newent.is_null() {
            crate::g_main::Com_Printf(&format!(
                "{}ERROR: NPC G_Spawn failed\n",
                latin1_to_string(S_COLOR_RED.to_bytes())
            ));
            return core::ptr::null_mut();
        }

        // Raven aliased the one `fullName` pool allocation into both entities.
        (*newent).alias_from(&*ent, PrefixSlot::FullName);

        (*newent).NPC = New_NPC_t(ctx, (*newent).s.number);
        if (*newent).NPC.is_null() {
            crate::g_main::Com_Printf(&format!(
                "{}ERROR: NPC G_Alloc NPC failed\n",
                latin1_to_string(S_COLOR_RED.to_bytes())
            ));
            // Raven: `goto finish;`.
            // The `return NULL;` right after is unreachable, because the goto always wins.
            // This preserves control flow, not shape (§C10).
            if (*ent).spawnflags & NSF_DROP_TO_FLOOR != 0 {
                crate::g_utils::G_SetOrigin(&mut *(ent), save_org);
            }
            return newent;
        }

        crate::g_utils::G_CreateFakeClient(ctx, (*newent).s.number, &mut (*newent).client);

        (*((*newent).NPC)).tempGoal = Some(crate::g_utils::G_Spawn(ctx));

        if (*((*newent).NPC)).tempGoal.is_none() {
            // The oracle nulls `NPC` and does `goto finish`.
            // The finish path returns the non-null `newent`, and the `return NULL` after the goto is
            // unreachable.
            // Source: oracle/codemp/game/NPC_spawn.c:1442-1447,1756-1762
            (*newent).NPC = core::ptr::null_mut();
            if (*ent).spawnflags & NSF_DROP_TO_FLOOR != 0 {
                crate::g_utils::G_SetOrigin(&mut *(ent), save_org);
            }
            return newent;
        }
        let temp_goal = ent_id::resolve(
            ctx.world.g_entities.as_mut_ptr(),
            (*((*newent).NPC)).tempGoal,
        );
        ctx.ent_set(
            ctx.entity_id_of(temp_goal).unwrap(),
            PrefixSet::ClassnameStatic(c"NPC_goal"),
        );
        (*temp_goal).parent = Some(ent_id(ctx.world.g_entities.as_mut_ptr(), newent));
        (*temp_goal).r.svFlags |= SVF_NOCLIENT;

        if (*newent).client.is_null() {
            crate::g_main::Com_Printf(&format!(
                "{}ERROR: NPC BG_Alloc client failed\n",
                latin1_to_string(S_COLOR_RED.to_bytes())
            ));
            if (*ent).spawnflags & NSF_DROP_TO_FLOOR != 0 {
                crate::g_utils::G_SetOrigin(&mut *(ent), save_org);
            }
            return newent;
        }

        // This zeroes byte-wise, over `sizeof(gclient_t)`.
        // `client` is `*mut c_void` here, so a typed `write_bytes` would zero only one byte
        // (`size_of::<c_void>()`), not the whole struct that C's `memset(newent->client, 0,
        // sizeof(*newent->client))` zeroes.
        // The backing storage, from `G_CreateFakeClient` through `BG_Alloc`, is also aligned to only 4 bytes,
        // below `gclient_t`'s pointer-field alignment.
        core::ptr::write_bytes(
            (*newent).client as *mut u8,
            0,
            core::mem::size_of::<gclient_t>(),
        );

        (*newent).playerState = &mut (*((*newent).client)).ps as *mut playerState_t;

        if (*ent).NPC_type.is_none() {
            (*ent).NPC_type = Some("random".to_owned());
        } else {
            // Raven does `Q_strlwr(G_NewString(NPC_type))`.
            // This lowercases the owned name in place, ASCII only, matching `Q_strlwr`.
            let lowered = (*ent).NPC_type.as_deref().unwrap().to_ascii_lowercase();
            (*ent).NPC_type = Some(lowered);
        }

        if (*ent).r.svFlags & SVF_NO_BASIC_SOUNDS != 0 {
            (*newent).r.svFlags |= SVF_NO_BASIC_SOUNDS;
        }
        if (*ent).r.svFlags & SVF_NO_COMBAT_SOUNDS != 0 {
            (*newent).r.svFlags |= SVF_NO_COMBAT_SOUNDS;
        }
        if (*ent).r.svFlags & SVF_NO_EXTRA_SOUNDS != 0 {
            (*newent).r.svFlags |= SVF_NO_EXTRA_SOUNDS;
        }

        if (*ent).message.is_some() {
            // Raven aliased the one `message` allocation.
            // The owned `String` clone is content-identical, because no pointer-identity compares exist.
            (*newent).message = (*ent).message.clone();
            (*newent).flags |= FL_NO_KNOCKBACK;
        }

        if Q_stricmp(&(*ent).classname_str(), "NPC_Vehicle") == 0 {
            let mut callbacks = crate::bg_channel::GameCallbacksImpl {
                // SEAM-BG-REENTRY (DEC-28, sanctioned).
                // GameCallbacksImpl.world is a `*mut GameWorld` field aliasing bg_state.
                // A raw store is required for bg-seam re-entry.
                world: ctx.world_raw(),
                engine: ctx.engine,
            };
            // `NPC_type` is `Some` here, already normalized above.
            // This materializes a `CString` for the vehicle-loader consumers that still take `*const c_char`,
            // in the bg tier and `G_Create*NPC`.
            let npc_type_c = cstr((*ent).NPC_type.as_deref().unwrap());
            let i_veh_index = BG_VehicleGetIndex(
                npc_type_c.as_ptr(),
                &mut ctx.world.bg_state,
                &crate::bg_channel::GameBgTraps::new(ctx.engine),
                &mut callbacks,
            );

            if i_veh_index == VEHICLE_NONE {
                crate::g_utils::G_FreeEntity(ctx, ctx.entity_id_of(newent));
                crate::g_utils::G_FreeEntity(ctx, ctx.entity_id_of(ent));
                return core::ptr::null_mut();
            }

            match (&ctx.world.bg_state.g_vehicleInfo)[i_veh_index as usize].r#type {
                VH_ANIMAL => {
                    crate::AnimalNPC::G_CreateAnimalNPC(
                        ctx,
                        &mut (*newent).m_pVehicle,
                        npc_type_c.as_ptr(),
                    );
                }
                VH_SPEEDER => {
                    crate::SpeederNPC::G_CreateSpeederNPC(
                        ctx,
                        &mut (*newent).m_pVehicle,
                        npc_type_c.as_ptr(),
                    );
                }
                VH_FIGHTER => {
                    crate::FighterNPC::G_CreateFighterNPC(
                        ctx,
                        &mut (*newent).m_pVehicle,
                        npc_type_c.as_ptr(),
                    );
                }
                VH_WALKER => {
                    crate::WalkerNPC::G_CreateWalkerNPC(
                        ctx,
                        &mut (*newent).m_pVehicle,
                        npc_type_c.as_ptr(),
                    );
                }
                _ => {
                    crate::g_main::Com_Printf(&format!(
                        "{} ERROR: Couldn't spawn NPC {}\n",
                        latin1_to_string(S_COLOR_RED.to_bytes()),
                        (*ent).NPC_type.as_deref().unwrap_or("")
                    ));
                    crate::g_utils::G_FreeEntity(ctx, ctx.entity_id_of(newent));
                    crate::g_utils::G_FreeEntity(ctx, ctx.entity_id_of(ent));
                    return core::ptr::null_mut();
                }
            }

            (*((*newent).m_pVehicle)).m_vOrientation =
                &mut (*((*newent).client)).ps.vehOrientation[0] as *mut f32;

            // This overlay-casts to `mp_bg`'s `bgEntity_t`.
            // This crate's own `bgEntity_t` name is the prelude's `gentity_t` alias.
            (*((*newent).m_pVehicle)).m_pParentEntity =
                newent as *mut mp_bg::public::bg_entity::bgEntity_t;
            crate::veh_dispatch::initialize(ctx, (*newent).m_pVehicle);

            crate::veh_dispatch::register_assets(ctx, (*newent).m_pVehicle);
            (*((*newent).client)).NPC_class = CLASS_VEHICLE;
            if (&ctx.world.bg_state.g_vehicleInfo)[i_veh_index as usize].r#type == VH_FIGHTER {
                (*newent).flags |= FL_NO_KNOCKBACK | FL_SHIELDED | FL_DMG_BY_HEAVY_WEAP_ONLY;
            }
            (*(*((*newent).m_pVehicle)).m_vOrientation.add(YAW as usize)) =
                (*ent).s.angles[YAW as usize];
            *(*((*newent).m_pVehicle)).m_vOrientation.add(PITCH as usize) = 0.0;
            *(*((*newent).m_pVehicle)).m_vOrientation.add(ROLL as usize) = 0.0;
            let orient: vec3_t = [
                *(*((*newent).m_pVehicle)).m_vOrientation.add(0),
                *(*((*newent).m_pVehicle)).m_vOrientation.add(1),
                *(*((*newent).m_pVehicle)).m_vOrientation.add(2),
            ];
            crate::g_utils::G_SetAngles(&mut *(newent), orient);
            SetClientViewAngle(&mut *newent, orient);

            (*newent).fly_sound_debounce_time = (*ent).fly_sound_debounce_time;
            (*newent).damage = (*ent).damage;
            (*newent).speed = (*ent).speed;
            (*newent).healingclass = (*ent).healingclass.clone();
            (*newent).healingsound = (*ent).healingsound.clone();
            (*newent).healingrate = (*ent).healingrate;
            (*newent).model2 = (*ent).model2.clone();
        } else {
            (*((*newent).client)).ps.weapon = WP_NONE;
        }

        crate::q_math::_VectorCopy((*ent).s.origin, &mut (*newent).s.origin);
        crate::q_math::_VectorCopy((*ent).s.origin, &mut (*((*newent).client)).ps.origin);
        crate::q_math::_VectorCopy((*ent).s.origin, &mut (*newent).r.currentOrigin);
        crate::g_utils::G_SetOrigin(&mut *(newent), (*ent).s.origin);
        let npc_type_owned = (*ent).NPC_type.clone().unwrap_or_default();
        if crate::NPC_stats::NPC_ParseParms(ctx, &npc_type_owned, ctx.entity_id_of(newent).unwrap())
            == qfalse
        {
            crate::g_main::Com_Printf(&format!(
                "{} ERROR: Couldn't spawn NPC {}\n",
                latin1_to_string(S_COLOR_RED.to_bytes()),
                npc_type_owned
            ));
            crate::g_utils::G_FreeEntity(ctx, ctx.entity_id_of(newent));
            crate::g_utils::G_FreeEntity(ctx, ctx.entity_id_of(ent));
            return core::ptr::null_mut();
        }

        if let Some(npc_type) = (*ent).NPC_type.as_deref() {
            if Q_stricmp(npc_type, "kyle") == 0 {
                (*((*newent).NPC)).aiFlags |= NPCAI_MATCHPLAYERWEAPON;
            } else if Q_stricmp(npc_type, "test") == 0 {
                let base = ctx.world.g_entities.as_mut_ptr();
                for n in 0..1 {
                    let e = base.add(n as usize);
                    if (*e).s.eType != (ET_NPC) as i32 && !(*e).client.is_null() {
                        crate::q_math::_VectorCopy((*e).s.origin, &mut (*newent).s.origin);
                        (*((*newent).client)).playerTeam = (*((*e).client)).playerTeam;
                        (*newent).s.teamowner = (*((*e).client)).playerTeam;
                        break;
                    }
                }
                (*((*newent).NPC)).defaultBehavior = bState_t::BS_WAIT;
                (*((*newent).NPC)).behaviorState = bState_t::BS_WAIT;
                ctx.ent_set(__teid16, PrefixSet::ClassnameStatic(c"NPC"));
            }
        }

        if (*newent).health == 0 {
            (*newent).health = (*ent).health;
        }
        // `NPC_targetname` is an owned `String`, where an empty string stands for absent.
        // `targetname` and `script_targetname` are PREFIX slots the engine reads.
        // Raven aliased one pool allocation into both, so the two `set` writes are content-identical, because
        // no pointer-identity compare exists.
        // An empty string maps to NULL, matching Raven's NULL-pointer alias when the field is unset.
        {
            let npc_targetname = (*ent).NPC_targetname.clone();
            if npc_targetname.is_empty() {
                ctx.ent_set(__teid16, PrefixSet::ScriptTargetname(None));
                ctx.ent_set(__teid16, PrefixSet::Targetname(None));
            } else {
                ctx.ent_set(__teid16, PrefixSet::ScriptTargetname(Some(&npc_targetname)));
                ctx.ent_set(__teid16, PrefixSet::Targetname(Some(&npc_targetname)));
            }
        }
        // `NPC_target` and `target` are now both owned, as `String`/`Option<String>`, where an empty string
        // or `None` stands for absent.
        // The copy is a plain owned move, per the string migration's ruling C: only `message` translates
        // `\n`, because NPC target names carry no escapes.
        (*newent).target = {
            let npc_target = (*ent).NPC_target.clone();
            if npc_target.is_empty() {
                None
            } else {
                Some(npc_target)
            }
        };
        (*newent).target2 = (*ent).target2.clone();
        (*newent).target3 = (*ent).target3.clone();
        (*newent).target4 = (*ent).target4.clone();
        (*newent).wait = (*ent).wait;

        let mut index = BSET_FIRST;
        while index < NUM_BSETS {
            if (*ent).behavior_set_str(index as usize).is_some() {
                (*newent).alias_from(&*ent, PrefixSlot::BehaviorSet(index as usize));
            }
            index += 1;
        }

        ctx.ent_set(__teid16, PrefixSet::ClassnameStatic(c"NPC"));
        (*newent).NPC_type = (*ent).NPC_type.clone();
        trap::UnlinkEntity(
            ctx.engine,
            mp_abi::game::syscalls::G_UNLINKENTITY::GUnlinkentityArgs::new(newent.cast()),
        );

        crate::q_math::_VectorCopy((*ent).s.angles, &mut (*newent).s.angles);
        crate::q_math::_VectorCopy((*ent).s.angles, &mut (*newent).r.currentAngles);
        crate::q_math::_VectorCopy((*ent).s.angles, &mut (*((*newent).client)).ps.viewangles);
        (*((*newent).NPC)).desiredYaw = (*ent).s.angles[YAW as usize];

        trap::LinkEntity(
            ctx.engine,
            mp_abi::game::syscalls::G_LINKENTITY::GLinkentityArgs::new(newent.cast()),
        );
        (*newent).spawnflags = (*ent).spawnflags;

        if (*ent).paintarget.is_some() {
            (*newent).paintarget = (*ent).paintarget.clone();
        }
        if (*ent).opentarget.is_some() {
            (*newent).opentarget = (*ent).opentarget.clone();
        }

        (*newent).s.eType = ET_NPC as c_int;

        if !(*ent).parms.is_null() {
            for parm_num in 0..MAX_PARMS {
                // Raven's `parm[parmNum]` null check is always true, because it is a char array and never
                // NULL.
                // Only the `[0]` emptiness check survives.
                let p = &(*(*ent).parms).parm[parm_num as usize];
                if p[0] != 0 {
                    Q3_SetParm(
                        ctx,
                        (*newent).s.number,
                        (parm_num) as i32,
                        p.as_ptr() as *const c_char,
                    );
                }
            }
        }

        (*newent).s.pos.trType = TR_INTERPOLATE;
        (*newent).s.pos.trTime = ctx.world.level.time;
        crate::q_math::_VectorCopy((*newent).r.currentOrigin, &mut (*newent).s.pos.trBase);
        (*newent).s.pos.trDelta = [0.0; 3];
        (*newent).s.pos.trDuration = 0;
        (*newent).s.apos.trType = TR_INTERPOLATE;
        (*newent).s.apos.trTime = ctx.world.level.time;
        crate::q_math::_VectorCopy((*newent).s.angles, &mut (*newent).s.apos.trBase);
        (*newent).s.apos.trDelta = [0.0; 3];
        (*newent).s.apos.trDuration = 0;

        (*((*newent).NPC)).combatPoint = -1;

        (*newent).flags |= FL_NOTARGET;
        (*newent).s.eFlags |= EF_NODRAW;

        (*newent).think = Some(EntThink::NPC_Begin).into();
        (*newent).nextthink = ctx.world.level.time + FRAMETIME;
        NPC_DefaultScriptFlags(&*newent);

        (*newent).s.shouldtarget = (*ent).s.shouldtarget;
        (*newent).s.teamowner = (*ent).s.teamowner;
        (*newent).alliedTeam = (*ent).alliedTeam;
        (*newent).teamnodmg = (*ent).teamnodmg;
        if let Some(team) = (*ent).team.as_deref().filter(|s| !s.is_empty()) {
            (*((*newent).client)).sess.sessionTeam = atoi_bytes(team.as_bytes());
        } else if (*newent).s.teamowner != TEAM_FREE {
            (*((*newent).client)).sess.sessionTeam = (*newent).s.teamowner;
        } else if (*newent).alliedTeam != TEAM_FREE {
            (*((*newent).client)).sess.sessionTeam = (*newent).alliedTeam;
        } else if (*newent).teamnodmg != TEAM_FREE {
            (*((*newent).client)).sess.sessionTeam = (*newent).teamnodmg;
        } else {
            (*((*newent).client)).sess.sessionTeam = TEAM_FREE;
        }
        (*((*newent).client)).ps.persistant[PERS_TEAM as usize] =
            (*((*newent).client)).sess.sessionTeam;

        trap::LinkEntity(
            ctx.engine,
            mp_abi::game::syscalls::G_LINKENTITY::GLinkentityArgs::new(newent.cast()),
        );

        if (*ent).use_.is_none() {
            if (*ent).target.is_some() {
                crate::g_utils::G_UseTargets(ctx, ctx.entity_id_of(ent), ctx.entity_id_of(ent));
            }
            if let Some(ct) = (*ent).closetarget.clone() {
                // `closetarget` is an owned `Option<String>` (G4).
                // When set, it bridges into the owned `target`, and both get `\n`-translated at spawn.
                (*newent).target = Some(ct);
            }
            ctx.ent_set(ctx.entity_id_of(ent).unwrap(), PrefixSet::Targetname(None));
            crate::g_utils::G_FreeEntity(ctx, ctx.entity_id_of(ent));
        }

        if (*ent).spawnflags & NSF_DROP_TO_FLOOR != 0 {
            crate::g_utils::G_SetOrigin(&mut *(ent), save_org);
        }

        newent
    }
}

/// Raven `NPC_Spawn_Go`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:1765-1768`
pub fn NPC_Spawn_Go(ctx: &mut GameContext, ent: EntityId) {
    // This is a thin pass-through. The `EntityId` forwards directly, with no re-derive.
    NPC_Spawn_Do(ctx, ent);
}

/// Raven `NPC_ShySpawn`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:1814-1831`
pub fn NPC_ShySpawn(ctx: &mut GameContext, ent: EntityId) {
    let nt = ctx.world.level.time + SHY_THINK_TIME;
    ctx.world.entity_mut(ent).nextthink = nt;
    ctx.world.entity_mut(ent).think = Some(EntThink::NPC_ShySpawn).into();

    // `g_entities[0]` is player 0, a real client slot.
    let player0 = EntityId(0);
    let p0_origin = ctx.world.entity(player0).r.currentOrigin;
    let ent_origin = ctx.world.entity(ent).r.currentOrigin;
    if crate::q_math::DistanceSquared(p0_origin, ent_origin) <= SHY_SPAWN_DISTANCE_SQR as vec_t {
        return;
    }

    if crate::NPC_senses::InFOV(ctx, Some(ent), player0, 80, 64) != 0 {
        let ent_origin = ctx.world.entity(ent).r.currentOrigin;
        if crate::NPC_utils::NPC_ClearLOS2(ctx, Some(player0), ent_origin) != 0 {
            return;
        }
    }

    ctx.world.entity_mut(ent).think = FnId::NONE;
    ctx.world.entity_mut(ent).nextthink = 0;

    NPC_Spawn_Go(ctx, ent);
}

/// Raven `NPC_Spawn`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:1839-1873`
pub fn NPC_Spawn(
    ctx: &mut GameContext,
    ent: EntityId,
    other: Option<EntityId>,
    activator: Option<EntityId>,
) {
    // `other` and `activator` go unused in the body. This matches Raven's use-handler signature.
    if ctx.world.entity(ent).delay != 0 {
        if ctx.world.entity(ent).spawnflags & 2048 != 0 {
            ctx.world.entity_mut(ent).think = Some(EntThink::NPC_ShySpawn).into();
        } else {
            ctx.world.entity_mut(ent).think = Some(EntThink::NPC_Spawn_Go).into();
        }
        let nt = ctx.world.level.time + ctx.world.entity(ent).delay;
        ctx.world.entity_mut(ent).nextthink = nt;
    } else {
        if ctx.world.entity(ent).spawnflags & 2048 != 0 {
            NPC_ShySpawn(ctx, ent);
        } else {
            NPC_Spawn_Do(ctx, ent);
        }
    }
}

/// Raven `NPC_PrecacheType`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:1961-1971`
pub fn NPC_PrecacheType(ctx: &mut GameContext, NPC_type: &str) {
    let fakespawner = crate::g_utils::G_Spawn(ctx);
    if let Some(id) = Some(fakespawner) {
        ctx.world.entity_mut(id).NPC_type = Some(NPC_type.to_owned());
        crate::NPC_stats::NPC_Precache(ctx, id);
        crate::g_utils::G_FreeEntity(ctx, Some(id));
    }
}

/// Raven `SP_NPC_spawner`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:1973-2068`
pub fn SP_NPC_spawner(ctx: &mut GameContext, self_: EntityId) {
    let mut t: c_int = 0;

    if ctx.world.cvars.g_allowNPC.integer == 0 {
        ctx.world.entity_mut(self_).think = Some(EntThink::G_FreeEntity).into();
        let time = ctx.world.level.time;
        ctx.world.entity_mut(self_).nextthink = time;
        return;
    }
    if ctx
        .world
        .entity(self_)
        .fullname_str()
        .map_or(true, |s| s.is_empty())
    {
        ctx.ent_set(self_, PrefixSet::FullName(Some("Humanoid Lifeform")));
    }

    if ctx.world.entity(self_).count == 0 {
        ctx.world.entity_mut(self_).count = 1;
    }

    {
        let mut garbage: c_int = 0;
        let no_basic = cstr("noBasicSounds");
        let zero = cstr("0");
        if crate::g_spawn::G_SpawnInt(ctx, no_basic.as_ptr(), zero.as_ptr(), &mut garbage) != 0 {
            ctx.world.entity_mut(self_).r.svFlags |= SVF_NO_BASIC_SOUNDS;
        }
        let no_combat = cstr("noCombatSounds");
        if crate::g_spawn::G_SpawnInt(ctx, no_combat.as_ptr(), zero.as_ptr(), &mut garbage) != 0 {
            ctx.world.entity_mut(self_).r.svFlags |= SVF_NO_COMBAT_SOUNDS;
        }
        let no_extra = cstr("noExtraSounds");
        if crate::g_spawn::G_SpawnInt(ctx, no_extra.as_ptr(), zero.as_ptr(), &mut garbage) != 0 {
            ctx.world.entity_mut(self_).r.svFlags |= SVF_NO_EXTRA_SOUNDS;
        }
    }

    if ctx.world.entity(self_).wait == (0) as f32 {
        ctx.world.entity_mut(self_).wait = (500) as f32;
    } else {
        ctx.world.entity_mut(self_).wait *= (1000) as f32;
    }

    ctx.world.entity_mut(self_).delay *= 1000;

    let showhealth = cstr("showhealth");
    let zero = cstr("0");
    crate::g_spawn::G_SpawnInt(ctx, showhealth.as_ptr(), zero.as_ptr(), &mut t);
    if t != 0 {
        ctx.world.entity_mut(self_).s.shouldtarget = qtrue;
    }

    let npc_type = ctx.world.entity(self_).NPC_type.clone();
    crate::NPC_stats::NPC_PrecacheAnimationCFG(npc_type.as_deref().unwrap_or(""));

    crate::NPC_stats::NPC_Precache(ctx, self_);

    if ctx.world.entity(self_).targetname_str().is_some() {
        ctx.world.entity_mut(self_).use_ = Some(EntUse::NPC_Spawn).into();
    } else {
        ctx.world.entity_mut(self_).think = Some(EntThink::NPC_Spawn_Go).into();
        let nt = ctx.world.level.time + START_TIME_REMOVE_ENTS + 50;
        ctx.world.entity_mut(self_).nextthink = nt;
    }
}

/// Raven `NPC_VehiclePrecache`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:2103-2173`
pub fn NPC_VehiclePrecache(ctx: &mut GameContext, spawner: EntityId) -> qboolean {
    unsafe {
        // `NPC_type` is `Option<String>`, where `None` stands for Raven's NULL.
        // This passes a NULL pointer through to the bg-tier loader when unset, and a `CString` otherwise.
        let sp_npc_type = ctx.world.entity(spawner).NPC_type.clone();
        let sp_npc_type_c = sp_npc_type.as_deref().map(cstr);
        let sp_npc_type_ptr = sp_npc_type_c
            .as_ref()
            .map_or(core::ptr::null(), |c| c.as_ptr());
        let mut callbacks = crate::bg_channel::GameCallbacksImpl {
            // SEAM-BG-REENTRY (DEC-28, sanctioned).
            // GameCallbacksImpl.world is a `*mut GameWorld` field aliasing bg_state.
            // A raw store is required for bg-seam re-entry.
            world: ctx.world_raw(),
            engine: ctx.engine,
        };
        let i_veh_index = BG_VehicleGetIndex(
            sp_npc_type_ptr,
            &mut ctx.world.bg_state,
            &crate::bg_channel::GameBgTraps::new(ctx.engine),
            &mut callbacks,
        );
        if i_veh_index == VEHICLE_NONE {
            return qfalse;
        }

        G_ModelIndex(ctx, &format!("${}", sp_npc_type.as_deref().unwrap_or("")));

        let p_veh_info = &(&ctx.world.bg_state.g_vehicleInfo)[i_veh_index as usize];
        if !p_veh_info.model.is_null() && *p_veh_info.model != 0 {
            let mut temp_g2: *mut c_void = core::ptr::null_mut();
            let mut skin: c_int = 0;
            if !p_veh_info.skin.is_null() && *p_veh_info.skin != 0 {
                let path = format!(
                    "models/players/{}/model_{}.skin",
                    cstr_to_str(p_veh_info.model as *const c_char),
                    cstr_to_str(p_veh_info.skin as *const c_char)
                );
                skin = trap::R_RegisterSkin(ctx.engine, &path);
            }
            let glm_path = format!(
                "models/players/{}/model.glm",
                cstr_to_str(p_veh_info.model as *const c_char)
            );
            trap::G2API_InitGhoul2Model(
                ctx.engine,
                &mut temp_g2 as *mut *mut c_void,
                &glm_path,
                0,
                skin,
                0,
                0,
                0,
            );
            if !temp_g2.is_null() {
                let gla_name = trap::G2API_GetGLAName(ctx.engine, temp_g2, 0, 1024);

                if !gla_name.is_empty() {
                    if let Some(slash_pos) = gla_name.rfind('/') {
                        let anim_path = cstr(&format!("{}/animation.cfg", &gla_name[..slash_pos]));

                        let traps = crate::bg_channel::GameBgTraps::new(ctx.engine);
                        // SEAM-BG-REENTRY (DEC-28, sanctioned).
                        // GameCallbacksImpl.world is a `*mut GameWorld` field aliasing bg_state.
                        // A raw store is required for bg-seam re-entry.
                        // The `world_raw()` accessor feeds the store.
                        let mut callbacks = crate::bg_channel::GameCallbacksImpl {
                            world: ctx.world_raw(),
                            engine: ctx.engine,
                        };
                        mp_bg::bg_panimate::BG_ParseAnimationFile(
                            &mut ctx.world.bg_state,
                            &traps,
                            &mut callbacks,
                            anim_path.as_ptr(),
                            core::ptr::null_mut(),
                            qfalse,
                        );
                    }
                }
                trap::G2API_CleanGhoul2Models(
                    ctx.engine,
                    mp_abi::game::syscalls::G_G2_CLEANMODELS::GG2CleanmodelsArgs::new(
                        &mut temp_g2 as *mut *mut c_void,
                    ),
                );
            }
        }

        // Raven prefers the spawner's `model2`, and falls back to the vehicle's `droidNPC`.
        // An empty string stands for absent in both: `model2` is owned, and `droidNPC` must be non-null and
        // non-empty, per Raven's explicit guard.
        let sp_model2 = ctx.world.entity(spawner).model2.clone();
        let droid_npc_type = if !sp_model2.is_empty() {
            sp_model2
        } else if !(&ctx.world.bg_state.g_vehicleInfo)[i_veh_index as usize]
            .droidNPC
            .is_null()
            && *(&ctx.world.bg_state.g_vehicleInfo)[i_veh_index as usize].droidNPC != 0
        {
            cstr_to_str(
                (&ctx.world.bg_state.g_vehicleInfo)[i_veh_index as usize].droidNPC as *const c_char,
            )
        } else {
            String::new()
        };

        if !droid_npc_type.is_empty() {
            if Q_stricmp("random", &droid_npc_type) == 0
                || Q_stricmp("default", &droid_npc_type) == 0
            {
                NPC_PrecacheType(ctx, "r2d2");
                NPC_PrecacheType(ctx, "r5d2");
            } else {
                NPC_PrecacheType(ctx, &droid_npc_type);
            }
        }
        qtrue
    }
}

/// Raven `NPC_VehicleSpawnUse`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:2175-2186`
pub fn NPC_VehicleSpawnUse(
    ctx: &mut GameContext,
    self_: EntityId,
    other: Option<EntityId>,
    activator: Option<EntityId>,
) {
    // `other` and `activator` go unused in the body. This matches Raven's use-handler signature.
    if ctx.world.entity(self_).delay != 0 {
        ctx.world.entity_mut(self_).think = Some(EntThink::G_VehicleSpawn).into();
        let nt = ctx.world.level.time + ctx.world.entity(self_).delay;
        ctx.world.entity_mut(self_).nextthink = nt;
    } else {
        crate::g_vehicles::G_VehicleSpawn(ctx, self_);
    }
}

/// Raven `SP_NPC_Vehicle`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:2188-2253`
pub fn SP_NPC_Vehicle(ctx: &mut GameContext, self_: EntityId) {
    let mut drop_time: f32 = 0.0;
    let mut t: c_int = 0;
    if ctx.world.entity(self_).NPC_type.is_none() {
        ctx.world.entity_mut(self_).NPC_type = Some("swoop".to_owned());
    }

    if ctx.world.entity(self_).classname_str().is_empty() {
        ctx.ent_set(self_, PrefixSet::ClassnameStatic(c"NPC_Vehicle"));
    }

    if ctx.world.entity(self_).wait == (0) as f32 {
        ctx.world.entity_mut(self_).wait = (500) as f32;
    } else {
        ctx.world.entity_mut(self_).wait *= (1000) as f32;
    }
    ctx.world.entity_mut(self_).delay *= 1000;

    let origin = ctx.world.entity(self_).s.origin;
    crate::g_utils::G_SetOrigin(ctx.world.entity_mut(self_), origin);
    let angles = ctx.world.entity(self_).s.angles;
    crate::g_utils::G_SetAngles(ctx.world.entity_mut(self_), angles);
    let drop_time_key = cstr("dropTime");
    let zero_f = cstr("0");
    crate::g_spawn::G_SpawnFloat(ctx, drop_time_key.as_ptr(), zero_f.as_ptr(), &mut drop_time);
    if drop_time != 0.0 {
        ctx.world.entity_mut(self_).fly_sound_debounce_time =
            (drop_time as f64 * 1000.0).ceil() as c_int;
    }

    let showhealth = cstr("showhealth");
    let zero = cstr("0");
    crate::g_spawn::G_SpawnInt(ctx, showhealth.as_ptr(), zero.as_ptr(), &mut t);
    if t != 0 {
        ctx.world.entity_mut(self_).s.shouldtarget = qtrue;
    }

    if ctx.world.entity(self_).targetname_str().is_some() {
        if NPC_VehiclePrecache(ctx, self_) == qfalse {
            crate::g_utils::G_FreeEntity(ctx, Some(self_));
            return;
        }
        ctx.world.entity_mut(self_).use_ = Some(EntUse::NPC_VehicleSpawnUse).into();
    } else {
        if ctx.world.entity(self_).delay != 0 {
            if NPC_VehiclePrecache(ctx, self_) == qfalse {
                crate::g_utils::G_FreeEntity(ctx, Some(self_));
                return;
            }
            ctx.world.entity_mut(self_).think = Some(EntThink::G_VehicleSpawn).into();
            let nt = ctx.world.level.time + ctx.world.entity(self_).delay;
            ctx.world.entity_mut(self_).nextthink = nt;
        } else {
            crate::g_vehicles::G_VehicleSpawn(ctx, self_);
        }
    }
}

/// Raven `SP_NPC_Kyle`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:2338-2345`
pub fn SP_NPC_Kyle(ctx: &mut GameContext, self_: EntityId) {
    ctx.world.entity_mut(self_).NPC_type = Some("Kyle".to_owned());
    WP_SetSaberModel(None, CLASS_KYLE);
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Lando`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:2354-2359`
pub fn SP_NPC_Lando(ctx: &mut GameContext, self_: EntityId) {
    ctx.world.entity_mut(self_).NPC_type = Some("Lando".to_owned());
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Jan`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:2368-2373`
pub fn SP_NPC_Jan(ctx: &mut GameContext, self_: EntityId) {
    ctx.world.entity_mut(self_).NPC_type = Some("Jan".to_owned());
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Luke`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:2382-2389`
pub fn SP_NPC_Luke(ctx: &mut GameContext, self_: EntityId) {
    ctx.world.entity_mut(self_).NPC_type = Some("Luke".to_owned());
    WP_SetSaberModel(None, CLASS_LUKE);
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_MonMothma`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:2398-2403`
pub fn SP_NPC_MonMothma(ctx: &mut GameContext, self_: EntityId) {
    ctx.world.entity_mut(self_).NPC_type = Some("MonMothma".to_owned());
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Tavion`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:2412-2419`
pub fn SP_NPC_Tavion(ctx: &mut GameContext, self_: EntityId) {
    ctx.world.entity_mut(self_).NPC_type = Some("Tavion".to_owned());
    WP_SetSaberModel(None, CLASS_TAVION);
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Tavion_New`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:2432-2448`
pub fn SP_NPC_Tavion_New(ctx: &mut GameContext, self_: EntityId) {
    let e = ctx.world.entity_mut(self_);
    if e.spawnflags & 1 != 0 {
        e.NPC_type = Some("tavion_scepter".to_owned());
    } else if e.spawnflags & 2 != 0 {
        e.NPC_type = Some("tavion_sith_sword".to_owned());
    } else {
        e.NPC_type = Some("tavion_new".to_owned());
    }
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Alora`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:2460-2472`
pub fn SP_NPC_Alora(ctx: &mut GameContext, self_: EntityId) {
    let e = ctx.world.entity_mut(self_);
    if e.spawnflags & 1 != 0 {
        e.NPC_type = Some("alora_dual".to_owned());
    } else {
        e.NPC_type = Some("alora".to_owned());
    }
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Reborn_New`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:2487-2524`
pub fn SP_NPC_Reborn_New(ctx: &mut GameContext, self_: EntityId) {
    let e = ctx.world.entity_mut(self_);
    if e.NPC_type.is_none() {
        if e.spawnflags & 4 != 0 {
            if e.spawnflags & 1 != 0 {
                e.NPC_type = Some("reborn_dual2".to_owned());
            } else if e.spawnflags & 2 != 0 {
                e.NPC_type = Some("reborn_staff2".to_owned());
            } else {
                e.NPC_type = Some("reborn_new2".to_owned());
            }
        } else {
            if e.spawnflags & 1 != 0 {
                e.NPC_type = Some("reborn_dual".to_owned());
            } else if e.spawnflags & 2 != 0 {
                e.NPC_type = Some("reborn_staff".to_owned());
            } else {
                e.NPC_type = Some("reborn_new".to_owned());
            }
        }
    }
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Cultist_Saber`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:2542-2593`
pub fn SP_NPC_Cultist_Saber(ctx: &mut GameContext, self_: EntityId) {
    let e = ctx.world.entity_mut(self_);
    if e.NPC_type.is_none() {
        if e.spawnflags & 1 != 0 {
            e.NPC_type = if e.spawnflags & 8 != 0 {
                Some("cultist_saber_med_throw".to_owned())
            } else {
                Some("cultist_saber_med".to_owned())
            };
        } else if e.spawnflags & 2 != 0 {
            e.NPC_type = if e.spawnflags & 8 != 0 {
                Some("cultist_saber_strong_throw".to_owned())
            } else {
                Some("cultist_saber_strong".to_owned())
            };
        } else if e.spawnflags & 2 != 0 {
            e.NPC_type = if e.spawnflags & 8 != 0 {
                Some("cultist_saber_all_throw".to_owned())
            } else {
                Some("cultist_saber_all".to_owned())
            };
        } else {
            e.NPC_type = if e.spawnflags & 8 != 0 {
                Some("cultist_saber_throw".to_owned())
            } else {
                Some("cultist_saber".to_owned())
            };
        }
    }
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Cultist_Saber_Powers`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:2611-2662`
pub fn SP_NPC_Cultist_Saber_Powers(ctx: &mut GameContext, self_: EntityId) {
    let e = ctx.world.entity_mut(self_);
    if e.NPC_type.is_none() {
        if e.spawnflags & 1 != 0 {
            e.NPC_type = if e.spawnflags & 8 != 0 {
                Some("cultist_saber_med_throw2".to_owned())
            } else {
                Some("cultist_saber_med2".to_owned())
            };
        } else if e.spawnflags & 2 != 0 {
            e.NPC_type = if e.spawnflags & 8 != 0 {
                Some("cultist_saber_strong_throw2".to_owned())
            } else {
                Some("cultist_saber_strong2".to_owned())
            };
        } else if e.spawnflags & 2 != 0 {
            e.NPC_type = if e.spawnflags & 8 != 0 {
                Some("cultist_saber_all_throw2".to_owned())
            } else {
                Some("cultist_saber_all2".to_owned())
            };
        } else {
            e.NPC_type = if e.spawnflags & 8 != 0 {
                Some("cultist_saber_throw".to_owned())
            } else {
                Some("cultist_saber2".to_owned())
            };
        }
    }
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Cultist`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:2679-2725`
pub fn SP_NPC_Cultist(ctx: &mut GameContext, self_: EntityId) {
    if ctx.world.entity(self_).NPC_type.is_none() {
        if ctx.world.entity(self_).spawnflags & 1 != 0 {
            ctx.world.entity_mut(self_).NPC_type = None;
            ctx.world.entity_mut(self_).spawnflags = 0;
            match ctx.world.bg_state.rng.Q_irand(0, 2) {
                0 => ctx.world.entity_mut(self_).spawnflags |= 1,
                1 => ctx.world.entity_mut(self_).spawnflags |= 2,
                2 => ctx.world.entity_mut(self_).spawnflags |= 4,
                _ => {}
            }
            if ctx.world.bg_state.rng.Q_irand(0, 1) != 0 {
                ctx.world.entity_mut(self_).spawnflags |= 8;
            }
            SP_NPC_Cultist_Saber(ctx, self_);
            return;
        } else if ctx.world.entity(self_).spawnflags & 2 != 0 {
            ctx.world.entity_mut(self_).NPC_type = Some("cultist_grip".to_owned());
        } else if ctx.world.entity(self_).spawnflags & 4 != 0 {
            ctx.world.entity_mut(self_).NPC_type = Some("cultist_lightning".to_owned());
        } else if ctx.world.entity(self_).spawnflags & 8 != 0 {
            ctx.world.entity_mut(self_).NPC_type = Some("cultist_drain".to_owned());
        } else {
            ctx.world.entity_mut(self_).NPC_type = Some("cultist".to_owned());
        }
    }
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Cultist_Commando`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:2737-2744`
pub fn SP_NPC_Cultist_Commando(ctx: &mut GameContext, self_: EntityId) {
    let e = ctx.world.entity_mut(self_);
    if e.NPC_type.is_none() {
        e.NPC_type = Some("cultistcommando".to_owned());
    }
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Cultist_Destroyer`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:2755-2759`
pub fn SP_NPC_Cultist_Destroyer(ctx: &mut GameContext, self_: EntityId) {
    ctx.world.entity_mut(self_).NPC_type = Some("cultist".to_owned());
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Reelo`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:2768-2773`
pub fn SP_NPC_Reelo(ctx: &mut GameContext, self_: EntityId) {
    ctx.world.entity_mut(self_).NPC_type = Some("Reelo".to_owned());
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Galak`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:2784-2797`
pub fn SP_NPC_Galak(ctx: &mut GameContext, self_: EntityId) {
    if ctx.world.entity(self_).spawnflags & 1 != 0 {
        ctx.world.entity_mut(self_).NPC_type = Some("Galak_Mech".to_owned());
        crate::NPC_AI_GalakMech::NPC_GalakMech_Precache(ctx);
    } else {
        ctx.world.entity_mut(self_).NPC_type = Some("Galak".to_owned());
    }
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Desann`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:2806-2813`
pub fn SP_NPC_Desann(ctx: &mut GameContext, self_: EntityId) {
    ctx.world.entity_mut(self_).NPC_type = Some("Desann".to_owned());
    WP_SetSaberModel(None, CLASS_DESANN);
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Bartender`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:2822-2827`
pub fn SP_NPC_Bartender(ctx: &mut GameContext, self_: EntityId) {
    ctx.world.entity_mut(self_).NPC_type = Some("Bartender".to_owned());
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_MorganKatarn`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:2836-2841`
pub fn SP_NPC_MorganKatarn(ctx: &mut GameContext, self_: EntityId) {
    ctx.world.entity_mut(self_).NPC_type = Some("MorganKatarn".to_owned());
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Jedi`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:2857-2887`
pub fn SP_NPC_Jedi(ctx: &mut GameContext, self_: EntityId) {
    if ctx.world.entity(self_).NPC_type.is_none() {
        if ctx.world.entity(self_).spawnflags & 1 != 0 {
            ctx.world.entity_mut(self_).NPC_type = Some("jeditrainer".to_owned());
        } else if ctx.world.bg_state.rng.Q_irand(0, 1) != 0 {
            ctx.world.entity_mut(self_).NPC_type = Some("Jedi".to_owned());
        } else {
            ctx.world.entity_mut(self_).NPC_type = Some("Jedi2".to_owned());
        }
    }
    WP_SetSaberModel(None, CLASS_JEDI);
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Prisoner`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:2896-2911`
pub fn SP_NPC_Prisoner(ctx: &mut GameContext, self_: EntityId) {
    if ctx.world.entity(self_).NPC_type.is_none() {
        let t = if ctx.world.bg_state.rng.Q_irand(0, 1) != 0 {
            Some("Prisoner".to_owned())
        } else {
            Some("Prisoner2".to_owned())
        };
        ctx.world.entity_mut(self_).NPC_type = t;
    }
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Rebel`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:2920-2935`
pub fn SP_NPC_Rebel(ctx: &mut GameContext, self_: EntityId) {
    if ctx.world.entity(self_).NPC_type.is_none() {
        let t = if ctx.world.bg_state.rng.Q_irand(0, 1) != 0 {
            Some("Rebel".to_owned())
        } else {
            Some("Rebel2".to_owned())
        };
        ctx.world.entity_mut(self_).NPC_type = t;
    }
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Stormtrooper`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:2955-2986`
pub fn SP_NPC_Stormtrooper(ctx: &mut GameContext, self_: EntityId) {
    if ctx.world.entity(self_).spawnflags & 8 != 0 {
        ctx.world.entity_mut(self_).NPC_type = Some("rockettrooper".to_owned());
    } else if ctx.world.entity(self_).spawnflags & 4 != 0 {
        ctx.world.entity_mut(self_).NPC_type = Some("stofficeralt".to_owned());
    } else if ctx.world.entity(self_).spawnflags & 2 != 0 {
        ctx.world.entity_mut(self_).NPC_type = Some("stcommander".to_owned());
    } else if ctx.world.entity(self_).spawnflags & 1 != 0 {
        ctx.world.entity_mut(self_).NPC_type = Some("stofficer".to_owned());
    } else {
        let t = if ctx.world.bg_state.rng.Q_irand(0, 1) != 0 {
            Some("StormTrooper".to_owned())
        } else {
            Some("StormTrooper2".to_owned())
        };
        ctx.world.entity_mut(self_).NPC_type = t;
    }
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_StormtrooperOfficer`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:2987-2991`
pub fn SP_NPC_StormtrooperOfficer(ctx: &mut GameContext, self_: EntityId) {
    ctx.world.entity_mut(self_).spawnflags |= 1;
    SP_NPC_Stormtrooper(ctx, self_);
}

/// Raven `SP_NPC_Snowtrooper`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3001-3006`
pub fn SP_NPC_Snowtrooper(ctx: &mut GameContext, self_: EntityId) {
    ctx.world.entity_mut(self_).NPC_type = Some("snowtrooper".to_owned());
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Tie_Pilot`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3016-3021`
pub fn SP_NPC_Tie_Pilot(ctx: &mut GameContext, self_: EntityId) {
    ctx.world.entity_mut(self_).NPC_type = Some("stormpilot".to_owned());
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Ugnaught`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3030-3045`
pub fn SP_NPC_Ugnaught(ctx: &mut GameContext, self_: EntityId) {
    if ctx.world.entity(self_).NPC_type.is_none() {
        let t = if ctx.world.bg_state.rng.Q_irand(0, 1) != 0 {
            Some("Ugnaught".to_owned())
        } else {
            Some("Ugnaught2".to_owned())
        };
        ctx.world.entity_mut(self_).NPC_type = t;
    }
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Jawa`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3056-3071`
pub fn SP_NPC_Jawa(ctx: &mut GameContext, self_: EntityId) {
    let e = ctx.world.entity_mut(self_);
    if e.NPC_type.is_none() {
        e.NPC_type = if e.spawnflags & 1 != 0 {
            Some("jawa_armed".to_owned())
        } else {
            Some("jawa".to_owned())
        };
    }
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Gran`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3084-3110`
pub fn SP_NPC_Gran(ctx: &mut GameContext, self_: EntityId) {
    if ctx.world.entity(self_).NPC_type.is_none() {
        if ctx.world.entity(self_).spawnflags & 1 != 0 {
            ctx.world.entity_mut(self_).NPC_type = Some("granshooter".to_owned());
        } else if ctx.world.entity(self_).spawnflags & 2 != 0 {
            ctx.world.entity_mut(self_).NPC_type = Some("granboxer".to_owned());
        } else {
            let t = if ctx.world.bg_state.rng.Q_irand(0, 1) != 0 {
                Some("gran".to_owned())
            } else {
                Some("gran2".to_owned())
            };
            ctx.world.entity_mut(self_).NPC_type = t;
        }
    }
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Rodian`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3121-3136`
pub fn SP_NPC_Rodian(ctx: &mut GameContext, self_: EntityId) {
    let e = ctx.world.entity_mut(self_);
    if e.NPC_type.is_none() {
        e.NPC_type = if e.spawnflags & 1 != 0 {
            Some("rodian2".to_owned())
        } else {
            Some("rodian".to_owned())
        };
    }
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Weequay`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3145-3167`
pub fn SP_NPC_Weequay(ctx: &mut GameContext, self_: EntityId) {
    if ctx.world.entity(self_).NPC_type.is_none() {
        let t = match ctx.world.bg_state.rng.Q_irand(0, 3) {
            0 => Some("Weequay".to_owned()),
            1 => Some("Weequay2".to_owned()),
            2 => Some("Weequay3".to_owned()),
            _ => Some("Weequay4".to_owned()),
        };
        ctx.world.entity_mut(self_).NPC_type = t;
    }
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Trandoshan`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3176-3184`
pub fn SP_NPC_Trandoshan(ctx: &mut GameContext, self_: EntityId) {
    let e = ctx.world.entity_mut(self_);
    if e.NPC_type.is_none() {
        e.NPC_type = Some("Trandoshan".to_owned());
    }
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Tusken`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3193-3208`
pub fn SP_NPC_Tusken(ctx: &mut GameContext, self_: EntityId) {
    let e = ctx.world.entity_mut(self_);
    if e.NPC_type.is_none() {
        e.NPC_type = if e.spawnflags & 1 != 0 {
            Some("tuskensniper".to_owned())
        } else {
            Some("tusken".to_owned())
        };
    }
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Noghri`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3217-3225`
pub fn SP_NPC_Noghri(ctx: &mut GameContext, self_: EntityId) {
    let e = ctx.world.entity_mut(self_);
    if e.NPC_type.is_none() {
        e.NPC_type = Some("noghri".to_owned());
    }
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_SwampTrooper`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3235-3250`
pub fn SP_NPC_SwampTrooper(ctx: &mut GameContext, self_: EntityId) {
    let e = ctx.world.entity_mut(self_);
    if e.NPC_type.is_none() {
        e.NPC_type = if e.spawnflags & 1 != 0 {
            Some("SwampTrooper2".to_owned())
        } else {
            Some("SwampTrooper".to_owned())
        };
    }
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Imperial`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3267-3301`
pub fn SP_NPC_Imperial(ctx: &mut GameContext, self_: EntityId) {
    let e = ctx.world.entity_mut(self_);
    if e.NPC_type.is_none() {
        if e.spawnflags & 1 != 0 {
            e.NPC_type = Some("ImpOfficer".to_owned());
        } else if e.spawnflags & 2 != 0 {
            e.NPC_type = Some("ImpCommander".to_owned());
        } else {
            e.NPC_type = Some("Imperial".to_owned());
        }
    }
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_ImpWorker`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3310-3329`
pub fn SP_NPC_ImpWorker(ctx: &mut GameContext, self_: EntityId) {
    if ctx.world.entity(self_).NPC_type.is_none() {
        if ctx.world.bg_state.rng.Q_irand(0, 2) == 0 {
            ctx.world.entity_mut(self_).NPC_type = Some("ImpWorker".to_owned());
        } else if ctx.world.bg_state.rng.Q_irand(0, 1) != 0 {
            ctx.world.entity_mut(self_).NPC_type = Some("ImpWorker2".to_owned());
        } else {
            ctx.world.entity_mut(self_).NPC_type = Some("ImpWorker3".to_owned());
        }
    }
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_BespinCop`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3338-3353`
pub fn SP_NPC_BespinCop(ctx: &mut GameContext, self_: EntityId) {
    if ctx.world.entity(self_).NPC_type.is_none() {
        let t = if ctx.world.bg_state.rng.Q_irand(0, 1) == 0 {
            Some("BespinCop".to_owned())
        } else {
            Some("BespinCop2".to_owned())
        };
        ctx.world.entity_mut(self_).NPC_type = t;
    }
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Reborn`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3372-3400`
pub fn SP_NPC_Reborn(ctx: &mut GameContext, self_: EntityId) {
    let e = ctx.world.entity_mut(self_);
    if e.NPC_type.is_none() {
        if e.spawnflags & 1 != 0 {
            e.NPC_type = Some("rebornforceuser".to_owned());
        } else if e.spawnflags & 2 != 0 {
            e.NPC_type = Some("rebornfencer".to_owned());
        } else if e.spawnflags & 4 != 0 {
            e.NPC_type = Some("rebornacrobat".to_owned());
        } else if e.spawnflags & 8 != 0 {
            e.NPC_type = Some("rebornboss".to_owned());
        } else {
            e.NPC_type = Some("reborn".to_owned());
        }
    }
    WP_SetSaberModel(None, CLASS_REBORN);
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_ShadowTrooper`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3409-3427`
pub fn SP_NPC_ShadowTrooper(ctx: &mut GameContext, self_: EntityId) {
    if ctx.world.entity(self_).NPC_type.is_none() {
        let t = if ctx.world.bg_state.rng.Q_irand(0, 1) == 0 {
            Some("ShadowTrooper".to_owned())
        } else {
            Some("ShadowTrooper2".to_owned())
        };
        ctx.world.entity_mut(self_).NPC_type = t;
    }
    crate::NPC_AI_Jedi::NPC_ShadowTrooper_Precache(ctx);
    WP_SetSaberModel(None, CLASS_SHADOWTROOPER);
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Monster_Murjj`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3439-3444`
pub fn SP_NPC_Monster_Murjj(ctx: &mut GameContext, self_: EntityId) {
    ctx.world.entity_mut(self_).NPC_type = Some("Murjj".to_owned());
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Monster_Swamp`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3453-3458`
pub fn SP_NPC_Monster_Swamp(ctx: &mut GameContext, self_: EntityId) {
    ctx.world.entity_mut(self_).NPC_type = Some("Swamp".to_owned());
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Monster_Howler`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3467-3472`
pub fn SP_NPC_Monster_Howler(ctx: &mut GameContext, self_: EntityId) {
    let s = Some("howler".to_owned());
    ctx.world.entity_mut(self_).NPC_type = s;
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_MineMonster`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3481-3487`
pub fn SP_NPC_MineMonster(ctx: &mut GameContext, self_: EntityId) {
    let s = Some("minemonster".to_owned());
    ctx.world.entity_mut(self_).NPC_type = s;
    SP_NPC_spawner(ctx, self_);
    NPC_MineMonster_Precache(ctx);
}

/// Raven `SP_NPC_Monster_Claw`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3496-3501`
pub fn SP_NPC_Monster_Claw(ctx: &mut GameContext, self_: EntityId) {
    let s = Some("Claw".to_owned());
    ctx.world.entity_mut(self_).NPC_type = s;
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Monster_Glider`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3510-3515`
pub fn SP_NPC_Monster_Glider(ctx: &mut GameContext, self_: EntityId) {
    let s = Some("Glider".to_owned());
    ctx.world.entity_mut(self_).NPC_type = s;
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Monster_Flier2`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3524-3529`
pub fn SP_NPC_Monster_Flier2(ctx: &mut GameContext, self_: EntityId) {
    let s = Some("Flier2".to_owned());
    ctx.world.entity_mut(self_).NPC_type = s;
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Monster_Lizard`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3538-3543`
pub fn SP_NPC_Monster_Lizard(ctx: &mut GameContext, self_: EntityId) {
    let s = Some("Lizard".to_owned());
    ctx.world.entity_mut(self_).NPC_type = s;
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Monster_Fish`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3552-3557`
pub fn SP_NPC_Monster_Fish(ctx: &mut GameContext, self_: EntityId) {
    let s = Some("Fish".to_owned());
    ctx.world.entity_mut(self_).NPC_type = s;
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Monster_Wampa`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3568-3575`
pub fn SP_NPC_Monster_Wampa(ctx: &mut GameContext, self_: EntityId) {
    let s = Some("wampa".to_owned());
    ctx.world.entity_mut(self_).NPC_type = s;
    NPC_Wampa_Precache(ctx);
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Monster_Rancor`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3584-3589`
pub fn SP_NPC_Monster_Rancor(ctx: &mut GameContext, self_: EntityId) {
    let s = Some("rancor".to_owned());
    ctx.world.entity_mut(self_).NPC_type = s;
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Droid_Interrogator`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3602-3609`
pub fn SP_NPC_Droid_Interrogator(ctx: &mut GameContext, self_: EntityId) {
    let s = Some("interrogator".to_owned());
    ctx.world.entity_mut(self_).NPC_type = s;
    SP_NPC_spawner(ctx, self_);
    NPC_Interrogator_Precache(ctx, Some(self_));
}

/// Raven `SP_NPC_Droid_Probe`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3620-3627`
pub fn SP_NPC_Droid_Probe(ctx: &mut GameContext, self_: EntityId) {
    let s = Some("probe".to_owned());
    ctx.world.entity_mut(self_).NPC_type = s;
    SP_NPC_spawner(ctx, self_);
    NPC_Probe_Precache(ctx);
}

/// Raven `SP_NPC_Droid_Mark1`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3639-3646`
pub fn SP_NPC_Droid_Mark1(ctx: &mut GameContext, self_: EntityId) {
    let s = Some("mark1".to_owned());
    ctx.world.entity_mut(self_).NPC_type = s;
    SP_NPC_spawner(ctx, self_);
    NPC_Mark1_Precache(ctx);
}

/// Raven `SP_NPC_Droid_Mark2`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3658-3665`
pub fn SP_NPC_Droid_Mark2(ctx: &mut GameContext, self_: EntityId) {
    let s = Some("mark2".to_owned());
    ctx.world.entity_mut(self_).NPC_type = s;
    SP_NPC_spawner(ctx, self_);
    NPC_Mark2_Precache(ctx);
}

/// Raven `SP_NPC_Droid_ATST`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3674-3688`
pub fn SP_NPC_Droid_ATST(ctx: &mut GameContext, self_: EntityId) {
    let s = if ctx.world.entity(self_).spawnflags & 1 != 0 {
        Some("atst_vehicle".to_owned())
    } else {
        Some("atst".to_owned())
    };
    ctx.world.entity_mut(self_).NPC_type = s;
    SP_NPC_spawner(ctx, self_);
    NPC_ATST_Precache(ctx);
}

/// Raven `SP_NPC_Droid_Remote`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3699-3706`
pub fn SP_NPC_Droid_Remote(ctx: &mut GameContext, self_: EntityId) {
    let s = Some("remote".to_owned());
    ctx.world.entity_mut(self_).NPC_type = s;
    SP_NPC_spawner(ctx, self_);
    NPC_Remote_Precache(ctx);
}

/// Raven `SP_NPC_Droid_Seeker`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3717-3724`
pub fn SP_NPC_Droid_Seeker(ctx: &mut GameContext, self_: EntityId) {
    let s = Some("seeker".to_owned());
    ctx.world.entity_mut(self_).NPC_type = s;
    SP_NPC_spawner(ctx, self_);
    NPC_Seeker_Precache(ctx);
}

/// Raven `SP_NPC_Droid_Sentry`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3735-3742`
pub fn SP_NPC_Droid_Sentry(ctx: &mut GameContext, self_: EntityId) {
    let s = Some("sentry".to_owned());
    ctx.world.entity_mut(self_).NPC_type = s;
    SP_NPC_spawner(ctx, self_);
    NPC_Sentry_Precache(ctx);
}

/// Raven `SP_NPC_Droid_Gonk`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3755-3763`
pub fn SP_NPC_Droid_Gonk(ctx: &mut GameContext, self_: EntityId) {
    let s = Some("gonk".to_owned());
    ctx.world.entity_mut(self_).NPC_type = s;
    SP_NPC_spawner(ctx, self_);
    NPC_Gonk_Precache(ctx);
}

/// Raven `SP_NPC_Droid_Mouse`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3776-3785`
pub fn SP_NPC_Droid_Mouse(ctx: &mut GameContext, self_: EntityId) {
    let s = Some("mouse".to_owned());
    ctx.world.entity_mut(self_).NPC_type = s;
    SP_NPC_spawner(ctx, self_);
    NPC_Mouse_Precache(ctx);
}

/// Raven `SP_NPC_Droid_R2D2`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3798-3812`
pub fn SP_NPC_Droid_R2D2(ctx: &mut GameContext, self_: EntityId) {
    let s = if ctx.world.entity(self_).spawnflags & 1 != 0 {
        Some("r2d2_imp".to_owned())
    } else {
        Some("r2d2".to_owned())
    };
    ctx.world.entity_mut(self_).NPC_type = s;
    SP_NPC_spawner(ctx, self_);
    NPC_R2D2_Precache(ctx);
}

/// Raven `SP_NPC_Droid_R5D2`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3826-3840`
pub fn SP_NPC_Droid_R5D2(ctx: &mut GameContext, self_: EntityId) {
    let s = if ctx.world.entity(self_).spawnflags & 1 != 0 {
        Some("r5d2_imp".to_owned())
    } else {
        Some("r5d2".to_owned())
    };
    ctx.world.entity_mut(self_).NPC_type = s;
    SP_NPC_spawner(ctx, self_);
    NPC_R5D2_Precache(ctx);
}

/// Raven `SP_NPC_Droid_Protocol`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3851-3864`
pub fn SP_NPC_Droid_Protocol(ctx: &mut GameContext, self_: EntityId) {
    let s = if ctx.world.entity(self_).spawnflags & 1 != 0 {
        Some("protocol_imp".to_owned())
    } else {
        Some("protocol".to_owned())
    };
    ctx.world.entity_mut(self_).NPC_type = s;
    SP_NPC_spawner(ctx, self_);
    NPC_Protocol_Precache(ctx);
}

/// Raven `NPC_SpawnType`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3872-4018`
pub fn NPC_SpawnType(
    ctx: &mut GameContext,
    ent: Option<EntityId>,
    npc_type: &str,
    targetname: Option<&str>,
    isVehicle: qboolean,
) -> *mut gentity_t {
    // `ent` is `Option<EntityId>`, and the body null-checks it, then re-derives it to a raw pointer, with the
    // body preserved verbatim, as open work.
    // `npc_type` is a name string (`&str`).
    // Raven's nullable `targetname` becomes `Option<&str>`, so the "only set NPC_targetname when non-NULL"
    // distinction is preserved.
    // The return stays a raw `*mut gentity_t`, because return-type conversion is a later pass.
    let ent: *mut gentity_t = ent.map_or(core::ptr::null_mut(), |i| ctx.entity_mut(i));
    let npc_spawner_eid = G_Spawn(ctx);
    let npc_spawner = ctx.entity_mut(npc_spawner_eid) as *mut gentity_t;

    if npc_spawner.is_null() {
        Com_Printf("NPC_Spawn Error: Out of entities!\n");
        return std::ptr::null_mut();
    }

    unsafe {
        (*npc_spawner).think = Some(EntThink::G_FreeEntity).into();
        (*npc_spawner).nextthink = ctx.world.level.time + FRAMETIME;
    }

    if npc_type.is_empty() {
        Com_Printf("Error, expected one of:\n NPC spawn [NPC type (from ext_data/NPCs)]\n NPC spawn vehicle [VEH type (from ext_data/vehicles)]\n");
        return std::ptr::null_mut();
    }

    if ent.is_null() || unsafe { (*ent).client.is_null() } {
        return std::ptr::null_mut();
    }

    // Spawn it at spot of first player
    let mut forward = [0.0f32; 3];
    let mut end = [0.0f32; 3];
    let mut trace: trace_t = unsafe { core::mem::zeroed() };

    AngleVectors(
        unsafe { (*((*ent).client)).ps.viewangles },
        Some(&mut forward),
        None,
        None,
    );
    let _ = VectorNormalize(&mut forward);

    unsafe {
        let ent_origin = (*ent).r.currentOrigin;
        end[0] = ent_origin[0] + 64.0 * forward[0];
        end[1] = ent_origin[1] + 64.0 * forward[1];
        end[2] = ent_origin[2] + 64.0 * forward[2];
    }

    unsafe {
        trap::Trace(
            ctx.engine,
            mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
                &mut trace as *mut trace_t,
                &(*ent).r.currentOrigin as *const vec3_t,
                core::ptr::null(),
                core::ptr::null(),
                &end as *const vec3_t,
                0,
                MASK_SOLID,
            ),
        );

        end = trace.endpos;
        end[2] -= 24.0;

        trap::Trace(
            ctx.engine,
            mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
                &mut trace as *mut trace_t,
                &trace.endpos as *const vec3_t,
                core::ptr::null(),
                core::ptr::null(),
                &end as *const vec3_t,
                0,
                MASK_SOLID,
            ),
        );

        end = trace.endpos;
        end[2] += 24.0;
    }

    G_SetOrigin(unsafe { &mut *npc_spawner }, end);

    unsafe {
        (*npc_spawner).s.origin = (*npc_spawner).r.currentOrigin;
        (*npc_spawner).s.angles[1] = (*((*ent).client)).ps.viewangles[1];
    }

    trap::LinkEntity(
        ctx.engine,
        mp_abi::game::syscalls::G_LINKENTITY::GLinkentityArgs::new(npc_spawner.cast()),
    );

    unsafe {
        (*npc_spawner).NPC_type = Some(npc_type.to_owned());

        if let Some(targetname) = targetname {
            (*npc_spawner).NPC_targetname = targetname.to_owned();
        }

        (*npc_spawner).count = 1;
        (*npc_spawner).delay = 0;

        if isVehicle != 0 {
            ctx.ent_set(npc_spawner_eid, PrefixSet::Classname("NPC_Vehicle"));
        }
    }

    //call precache funcs for James' builds
    let npc_type_str = npc_type;

    if Q_stricmp("gonk", npc_type_str) == 0 {
        NPC_Gonk_Precache(ctx);
    } else if Q_stricmp("mouse", npc_type_str) == 0 {
        NPC_Mouse_Precache(ctx);
    } else if Q_strncmp("r2d2", npc_type_str, 4) == 0 {
        NPC_R2D2_Precache(ctx);
    } else if Q_stricmp("atst", npc_type_str) == 0 {
        NPC_ATST_Precache(ctx);
    } else if Q_strncmp("r5d2", npc_type_str, 4) == 0 {
        NPC_R5D2_Precache(ctx);
    } else if Q_stricmp("mark1", npc_type_str) == 0 {
        NPC_Mark1_Precache(ctx);
    } else if Q_stricmp("mark2", npc_type_str) == 0 {
        NPC_Mark2_Precache(ctx);
    } else if Q_stricmp("interrogator", npc_type_str) == 0 {
        NPC_Interrogator_Precache(ctx, None);
    } else if Q_stricmp("probe", npc_type_str) == 0 {
        NPC_Probe_Precache(ctx);
    } else if Q_stricmp("seeker", npc_type_str) == 0 {
        NPC_Seeker_Precache(ctx);
    } else if Q_stricmp("remote", npc_type_str) == 0 {
        NPC_Remote_Precache(ctx);
    } else if Q_strncmp("shadowtrooper", npc_type_str, 13) == 0 {
        NPC_ShadowTrooper_Precache(ctx);
    } else if Q_stricmp("minemonster", npc_type_str) == 0 {
        NPC_MineMonster_Precache(ctx);
    } else if Q_stricmp("howler", npc_type_str) == 0 {
        NPC_Howler_Precache();
    } else if Q_stricmp("sentry", npc_type_str) == 0 {
        NPC_Sentry_Precache(ctx);
    } else if Q_stricmp("protocol", npc_type_str) == 0 {
        NPC_Protocol_Precache(ctx);
    } else if Q_stricmp("galak_mech", npc_type_str) == 0 {
        NPC_GalakMech_Precache(ctx);
    } else if Q_stricmp("wampa", npc_type_str) == 0 {
        NPC_Wampa_Precache(ctx);
    }

    NPC_Spawn_Do(ctx, npc_spawner_eid)
}

/// Raven `NPC_Spawn_f`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:4020-4039`
pub fn NPC_Spawn_f(ctx: &mut GameContext, ent: EntityId) {
    let mut npc_type: String;
    let targetname: String;
    let mut is_vehicle = 0u32;

    let arg2 = trap::Argv(ctx.engine, 2, 1024);
    npc_type = strncpyz_string(arg2.as_bytes(), 1024);

    if Q_stricmp("vehicle", &npc_type) == 0 {
        is_vehicle = 1;
        let arg3 = trap::Argv(ctx.engine, 3, 1024);
        npc_type = strncpyz_string(arg3.as_bytes(), 1024);
        let arg4 = trap::Argv(ctx.engine, 4, 1024);
        targetname = strncpyz_string(arg4.as_bytes(), 1024);
    } else {
        let arg3 = trap::Argv(ctx.engine, 3, 1024);
        targetname = strncpyz_string(arg3.as_bytes(), 1024);
    }

    NPC_SpawnType(
        ctx,
        Some(ent),
        &npc_type,
        Some(&targetname),
        (is_vehicle) as i32,
    );
}

/// Raven `NPC_Kill_f`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:4045-4170`
pub fn NPC_Kill_f(ctx: &mut GameContext) {
    // Raven's `TeamNames[TEAM_NUM_TEAMS]` (`NPC_stats.c:133`) holds the NPC `team_t` names.
    // The many commented-out Trek-era entries collapse to these three.
    const TEAM_NAMES: [&str; TEAM_NUM_TEAMS as usize] = ["", "player", "enemy", "neutral"];
    let mut kill_team: team_t = TEAM_FREE;
    let mut kill_non_sf = 0u32;

    let arg2 = trap::Argv(ctx.engine, 2, 1024);
    let mut name = strncpyz_string(arg2.as_bytes(), 1024);

    if name.is_empty() {
        Com_Printf("Error, Expected:\n");
        Com_Printf("NPC kill '[NPC targetname]' - kills NPCs with certain targetname\n");
        Com_Printf("or\n");
        Com_Printf("NPC kill 'all' - kills all NPCs\n");
        Com_Printf("or\n");
        Com_Printf("NPC team '[teamname]' - kills all NPCs of a certain team ('nonally' is all but your allies)\n");
        return;
    }

    if Q_stricmp("team", &name) == 0 {
        let arg3 = trap::Argv(ctx.engine, 3, 1024);
        name = strncpyz_string(arg3.as_bytes(), 1024);

        if name.is_empty() {
            Com_Printf("NPC_Kill Error: 'npc kill team' requires a team name!\n");
            Com_Printf("Valid team names are:\n");
            for n in (TEAM_FREE + 1)..TEAM_NUM_TEAMS {
                // Raven's `TeamNames[]` (`NPC_stats.c:133`) holds the NPC `team_t` names.
                Com_Printf(&format!("{}\n", TEAM_NAMES[n as usize]));
            }
            Com_Printf("nonally - kills all but your teammates\n");
            return;
        }

        if Q_stricmp("nonally", &name) == 0 {
            kill_non_sf = 1;
        } else {
            kill_team =
                GetIDForString(TeamTable.as_ptr() as *mut stringID_table_t, &name) as team_t;

            if kill_team == TEAM_FREE {
                Com_Printf(&format!("NPC_Kill Error: team '{}' not recognized\n", name));
                Com_Printf("Valid team names are:\n");
                for n in (TEAM_FREE + 1)..TEAM_NUM_TEAMS {
                    // Raven's `TeamNames[]` (`NPC_stats.c:133`) holds the NPC `team_t` names.
                    Com_Printf(&format!("{}\n", TEAM_NAMES[n as usize]));
                }
                Com_Printf("nonally - kills all but your teammates\n");
                return;
            }
        }
    }

    for n in 1..ENTITYNUM_MAX_NORMAL {
        let player = ctx.world.g_entities.get_mut(n as usize);
        if player.is_none() {
            continue;
        }
        let player = player.unwrap();

        if player.inuse == qfalse {
            continue;
        }

        if kill_non_sf != 0 {
            if !player.client.is_null() {
                if unsafe { (*(player.client)).playerTeam } != NPCTEAM_PLAYER {
                    Com_Printf(&format!(
                        "Killing NPC {} named {}\n",
                        player.NPC_type.as_deref().unwrap_or(""),
                        player.targetname_str().unwrap_or_default()
                    ));
                    player.health = 0;

                    if let Some(die_fn) = player.die.get() {
                        if !player.client.is_null() {
                            let health = unsafe { (*(player.client)).pers.maxHealth };
                            let self_ = player as *mut gentity_t;
                            dispatch_die(
                                ctx,
                                die_fn,
                                self_,
                                self_,
                                self_,
                                health,
                                MOD_UNKNOWN as c_int,
                            );
                        }
                    }
                }
            } else if player.NPC_type.is_some()
                && !player.classname_str().is_empty()
                && Q_stricmp("NPC_starfleet", &player.classname_str()) != 0
            {
                Com_Printf(&format!(
                    "Removing NPC spawner {} with NPC named {}\n",
                    player.NPC_type.as_deref().unwrap_or(""),
                    player.NPC_targetname.clone()
                ));
                // This raw pointer cast ends the `player` borrow before re-entering `ctx`.
                let player_ptr = player as *mut gentity_t;
                G_FreeEntity(ctx, ctx.entity_id_of(player_ptr));
            }
        } else if !player.NPC.is_null() && !player.client.is_null() {
            if kill_team != TEAM_FREE {
                if unsafe { (*(player.client)).playerTeam } == kill_team {
                    Com_Printf(&format!(
                        "Killing NPC {} named {}\n",
                        player.NPC_type.as_deref().unwrap_or(""),
                        player.targetname_str().unwrap_or_default()
                    ));
                    player.health = 0;
                    if let Some(die_fn) = player.die.get() {
                        let health = unsafe { (*(player.client)).pers.maxHealth };
                        let self_ = player as *mut gentity_t;
                        dispatch_die(
                            ctx,
                            die_fn,
                            self_,
                            self_,
                            self_,
                            health,
                            MOD_UNKNOWN as c_int,
                        );
                    }
                }
            } else if player
                .targetname_str()
                .as_deref()
                .is_some_and(|tn| Q_stricmp(&name, tn) == 0)
                || Q_stricmp("all", &name) == 0
            {
                Com_Printf(&format!(
                    "Killing NPC {} named {}\n",
                    player.NPC_type.as_deref().unwrap_or(""),
                    player.targetname_str().unwrap_or_default()
                ));
                player.health = 0;
                unsafe {
                    (*(player.client)).ps.stats[STAT_HEALTH as usize] = 0;
                }
                if let Some(die_fn) = player.die.get() {
                    let self_ = player as *mut gentity_t;
                    dispatch_die(ctx, die_fn, self_, self_, self_, 100, MOD_UNKNOWN as c_int);
                }
            }
        }
    }
}

/// Raven `NPC_PrintScore`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:4172-4175`
pub fn NPC_PrintScore(ctx: &mut GameContext, ent: EntityId) {
    let targetname = ctx.world.entity(ent).targetname_str().unwrap_or_default();
    // The pool client deref stays raw: a copied pointer value in a tight `unsafe` block.
    let client = ctx.world.entity(ent).client;
    let score = unsafe { (*client).ps.persistant[PERS_SCORE as usize] };
    Com_Printf(&format!("{targetname}: {score}\n"));
}

/// Raven `Cmd_NPC_f`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:4183-4243`
pub fn Cmd_NPC_f(ctx: &mut GameContext, ent: EntityId) {
    let cmd = trap::Argv(ctx.engine, 1, 1024);

    if cmd.is_empty() {
        Com_Printf("Valid NPC commands are:\n");
        Com_Printf(" spawn [NPC type (from NCPCs.cfg)]\n");
        Com_Printf(" kill [NPC targetname] or [all(kills all NPCs)] or 'team [teamname]'\n");
        Com_Printf(" showbounds (draws exact bounding boxes of NPCs)\n");
        Com_Printf(" score [NPC targetname] (prints number of kills per NPC)\n");
    } else if Q_stricmp("spawn", &cmd) == 0 {
        NPC_Spawn_f(ctx, ent);
    } else if Q_stricmp("kill", &cmd) == 0 {
        NPC_Kill_f(ctx);
    } else if Q_stricmp("showbounds", &cmd) == 0 {
        ctx.world.globals.showBBoxes = if ctx.world.globals.showBBoxes != 0 {
            0
        } else {
            1
        };
    } else if Q_stricmp("score", &cmd) == 0 {
        let arg2 = trap::Argv(ctx.engine, 2, 1024);
        let cmd2 = strncpyz_string(arg2.as_bytes(), 1024);

        if cmd2.is_empty() {
            // Show the score for all NPCs
            Com_Printf("SCORE LIST:\n");
            for i in 0..ENTITYNUM_WORLD as usize {
                let player = ctx.world.g_entities.get(i);
                if player.is_none() || player.unwrap().client.is_null() {
                    continue;
                }
                NPC_PrintScore(ctx, ctx.entity_id_of(player.unwrap()).unwrap());
            }
        } else {
            let found_ent = G_Find(
                ctx,
                ctx.entity_id_of(std::ptr::null_mut()),
                EntFindField::Targetname,
                &cmd2,
            );
            if !found_ent.is_null() && !unsafe { (*found_ent).client.is_null() } {
                NPC_PrintScore(ctx, ctx.entity_id_of(found_ent).unwrap());
            } else {
                Com_Printf(&format!("ERROR: NPC score - no such NPC {}\n", cmd2));
            }
        }
    }
}
