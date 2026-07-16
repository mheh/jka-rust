// PORT-COMPLETE: NPC_spawn.c 5/85
//! FAITHFUL port of `oracle/codemp/game/NPC_spawn.c`.
//!
//! Filled by the jampgame mega-pass; functions reach file-scope game state
//! (`level`, `g_entities`, cvars) and engine traps through the threaded
//! `GameContext`/`GameWorld` handle.
//!
//! Safe-state migration **Stage 1**: entity-pointer params are `EntityId` /
//! `Option<EntityId>` handles (§B5), not raw `gentity_t*`; ctx-free leaf helpers
//! take `&mut`/`&gentity_t`.
//!
//! Safe-state migration **Stage 2b** (body sweep): every world reach is a
//! checked `ctx.world.…` borrow — the transitional `(*ctx.world_raw())` raw-deref
//! regime is retired. One `world_raw()` use survives (irreducible): the raw
//! `*mut GameWorld` field of `GameCallbacksImpl` fed to `BG_ParseAnimationFile`.
//!
//! Safe-state campaign **2c** (deref regime): per-body `gentity_t` derefs go
//! through `ctx.world.entity()`/`entity_mut()` accessors at point of use; the
//! fn-top `ctx.entity_mut()` re-derives are gone. Pool clients (`ent.client`)
//! and `gNPC_t` (`ent.NPC` / `globals.NPCInfo`) have no accessor, so those
//! derefs stay raw in tight `unsafe` blocks through a copied pointer value
//! (recipe 2b/2c). `NPC_Spawn_Do`/`NPC_SpawnType` keep the Stage-1 re-derive:
//! each cross-copies a fresh `G_Spawn` entity with the spawner (two live
//! entities, rule 4) — left as Stage-2 debt. This file is referee-blind —
//! parity rests on the compile + golden suite.
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;
// Dedupe SVF_NOCLIENT glob ambiguity (g_items::* / g_public_consts::* both
// define it): the canonical home is g_public_consts, per house convention.
use crate::ent_fn_enums::dispatch_die;
use crate::g_ICARUScb::G_DebugPrint;
use crate::g_ICARUScb::Q3_SetParm;
use crate::g_public_consts::SVF_NOCLIENT;
use crate::q_shared::FOFS_targetname;
use crate::NPC_stats::TeamTable;
use mp_qshared::common::mp::gentity::BSET_FIRST;

// Unported types referenced in this file (need porting before this compiles):
// PAIN_FUNC, TOUCH_FUNC

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

// PORT-NOTE(packet-contract): this packet's manifest rows list callee
// names (in-module callees / bg_ callees / traps / globals) but never give the
// resolved Rust signature of any out-of-file symbol (no `## RESOLVED` /
// `prelude` section), unlike what the task brief promises. Nearly every
// function in this file (NPC_Begin, NPC_Spawn_Do, the ~60 SP_NPC_* spawn-field
// setters, vehicle/precache plumbing, Cmd_NPC_f, …) reaches `level`,
// `g_entities`, `G_Spawn`/`G_FreeEntity`, NPC.cfg parsing, ICARUS, vehicle
// vtables, and ~213 fn-pointer targets whose concrete Rust
// shapes/signatures are not given here and this protocol forbids exploring
// the tree to find them. Parking those bodies rather than inventing
// signatures. Only the handful of functions below are fully self-contained
// (pure arithmetic / string compares / no external state) and are ported
// faithfully.

/// Raven `WP_SetSaberModel`.
///
/// Raven: rwwFIXMEFIXME: Do something here, need to let the client know.
/// Source: `oracle/codemp/game/NPC_spawn.c:90-94`
pub fn WP_SetSaberModel(client: Option<&mut gclient_t>, npcClass: class_t) -> c_int {
    // Ctx-free leaf; body ignores `client` (all callers pass `None`).
    let _ = client;
    1
}

/// Raven `NPC_PainFunc`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:103-189`
pub fn NPC_PainFunc(ent: &gentity_t) -> Option<crate::ent_fn_enums::EntPain> {
    // Raven returns the selected pain fn-ptr; fn-ptr fields are the
    // `Option<EntPain>` fn-ID enum directly (no *mut c_void encoding).
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
    // Raven always returns `NPC_Touch`, returned here as `Option<EntTouch>` directly.
    Some(crate::ent_fn_enums::EntTouch::NPC_Touch)
}

/// Raven `NPC_SetMiscDefaultData`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:215-501`
pub fn NPC_SetMiscDefaultData(ctx: &mut GameContext, ent: EntityId) {
    // Pool client + gNPC_t + vehicle derefs stay raw (recipe 2b/2c): copied
    // pointer values, tight unsafe. Entity fields go through the world accessor.
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
            let surf = cstr("head_hatchcover");
            let ghoul2 = ctx.world.entity(ent).ghoul2;
            trap::G2API_SetSurfaceOnOff(
                ctx.engine,
                mp_abi::game::syscalls::G_G2_SETSURFACEONOFF::GG2SetsurfaceonoffArgs::new(
                    ghoul2,
                    surf.as_ptr(),
                    0,
                ),
            );
        }
        let wampa = cstr("wampa");
        if crate::q_shared::Q_stricmp(wampa.as_ptr(), ctx.world.entity(ent).NPC_type) == 0 {
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
        let yoda = cstr("Yoda");
        if crate::q_shared::Q_stricmp(yoda.as_ptr(), ctx.world.entity(ent).NPC_type) == 0 {
            (*npc).scriptFlags |= SCF_NO_FORCE;
        }
        let emperor = cstr("emperor");
        if crate::q_shared::Q_stricmp(emperor.as_ptr(), ctx.world.entity(ent).NPC_type) == 0 {
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
                                // officers/thermal alt-fire: commented out in oracle
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
                let gonk = cstr("gonk");
                if crate::q_shared::Q_stricmp(ctx.world.entity(ent).NPC_type, gonk.as_ptr()) == 0 {
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
                    // Oracle switch: `default:` falls into `case WP_BLASTER:`, so
                    // ST_ClearTimers runs for WP_BLASTER and any weapon not in the
                    // explicit no-op case set. Source: oracle/codemp/game/NPC_spawn.c:412-458
                    match (*client).ps.weapon {
                        WP_BRYAR_PISTOL | WP_DISRUPTOR | WP_BOWCASTER | WP_REPEATER | WP_DEMP2
                        | WP_FLECHETTE | WP_ROCKET_LAUNCHER | WP_THERMAL | WP_STUN_BATON => {}
                        _ => {
                            crate::NPC_AI_Stormtrooper::ST_ClearTimers(ctx, ent);
                        }
                    }
                    let galak_mech = cstr("galak_mech");
                    if crate::q_shared::Q_stricmp(
                        ctx.world.entity(ent).NPC_type,
                        galak_mech.as_ptr(),
                    ) == 0
                    {
                        crate::NPC_AI_GalakMech::NPC_GalakMech_Init(ctx, ent);
                    }
                }
            }
            _ => {}
        }

        if (*client).NPC_class == CLASS_SEEKER && ctx.world.entity(ent).activator.is_some() {
            // teams already set correctly
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
pub fn NPC_WeaponsForTeam(team: team_t, spawnflags: c_int, NPC_type: *const c_char) -> c_int {
    // Faithful transcription of the C string-compare cascade. Q_stricmp /
    // Q_strncmp are pure string helpers (bg-shared); NPC_type is a NUL
    // terminated C string handed to us by the (unported) caller — safe to
    // read here since we only borrow it for comparisons.
    let name = unsafe {
        if NPC_type.is_null() {
            std::borrow::Cow::Borrowed("")
        } else {
            std::ffi::CStr::from_ptr(NPC_type).to_string_lossy()
        }
    };
    let name = name.as_ref();

    let stricmp = |a: &str, b: &str| a.eq_ignore_ascii_case(b);
    // Q_strncmp is case-SENSITIVE (unlike Q_stricmp); compare prefixes exactly.
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
    // Pool client + gNPC_t derefs stay raw (recipe 2b/2c): copied pointer values.
    let client = ctx.world.entity(ent).client;
    let npc = ctx.world.entity(ent).NPC;
    let mut bestWeap: c_int = WP_NONE;
    let player_team = unsafe { (*client).playerTeam };
    let spawnflags = ctx.world.entity(ent).spawnflags;
    let npc_type = ctx.world.entity(ent).NPC_type;
    let weapons = NPC_WeaponsForTeam(player_team, spawnflags, npc_type as *const c_char);

    unsafe {
        (*client).ps.stats[STAT_WEAPONS as usize] = 0;
        let mut curWeap = WP_SABER;
        while curWeap < WP_NUM_WEAPONS {
            if weapons & (1 << curWeap) != 0 {
                (*client).ps.stats[STAT_WEAPONS as usize] |= 1 << curWeap;
                // PORT-NOTE(weaponData): `weaponData` global table isn't resolved
                // in this packet; ammoIndex lookup left as the literal Raven
                // subject until that global lands.
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
    // Empty body in the oracle (effect hook, never filled in).
    let _ = ent;
}

/// Raven `NPC_SetFX_SpawnStates`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:817-823`
pub fn NPC_SetFX_SpawnStates(ctx: &mut GameContext, ent: EntityId) {
    // Pool client + gNPC_t derefs stay raw (recipe 2b/2c): copied pointer values.
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
    // Pool client + gNPC_t derefs stay raw (recipe 2b/2c): copied pointer values.
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
                    let target3 = ctx.world.entity(ent).target3;
                    let t3 = if target3.is_null() {
                        String::new()
                    } else {
                        cstr_to_str(target3 as *const c_char)
                    };
                    let targetname = ctx.world.entity(ent).targetname;
                    let tn = if targetname.is_null() {
                        String::new()
                    } else {
                        cstr_to_str(targetname as *const c_char)
                    };
                    G_DebugPrint(
                        ctx,
                        WL_DEBUG as i32,
                        cstr(&format!(
                            "NPC {} could not spawn, firing target3 ({}) and removing self\n",
                            tn, t3
                        ))
                        .as_ptr(),
                    );
                    let t3ptr = ctx.world.entity(ent).target3;
                    crate::g_utils::G_UseTargets2(
                        ctx,
                        Some(ent),
                        Some(ent),
                        t3ptr as *const c_char,
                    );
                    ctx.world.entity_mut(ent).think = Some(EntThink::G_FreeEntity).into();
                    let nt = ctx.world.level.time + 100;
                    ctx.world.entity_mut(ent).nextthink = nt;
                } else {
                    let targetname = ctx.world.entity(ent).targetname;
                    let tn = if targetname.is_null() {
                        String::new()
                    } else {
                        cstr_to_str(targetname as *const c_char)
                    };
                    let wait = ctx.world.entity(ent).wait;
                    G_DebugPrint(
                        ctx,
                        WL_DEBUG as i32,
                        cstr(&format!(
                            "NPC {} could not spawn, waiting {:.2} secs to try again\n",
                            tn,
                            wait as f32 / 1000.0f32
                        ))
                        .as_ptr(),
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

        let rodian = cstr("rodian");
        if crate::q_shared::Q_stricmp(rodian.as_ptr(), ctx.world.entity(ent).NPC_type) == 0 {
            match ctx.world.cvars.g_spskill.integer {
                0 => (*npc).stats.aim = (1.0) as i32,
                1 => (*npc).stats.aim = (ctx.world.bg_state.rng.Q_irand(2, 3) as f32) as i32,
                2 => (*npc).stats.aim = (ctx.world.bg_state.rng.Q_irand(3, 4) as f32) as i32,
                _ => {}
            }
        } else {
            let rodian2 = cstr("rodian2");
            if (*client).NPC_class == CLASS_STORMTROOPER
                || (*client).NPC_class == CLASS_SWAMPTROOPER
                || (*client).NPC_class == CLASS_IMPWORKER
                || crate::q_shared::Q_stricmp(rodian2.as_ptr(), ctx.world.entity(ent).NPC_type) == 0
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
        ctx.world.entity_mut(ent).classname = c"NPC".as_ptr() as *mut c_char;
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
            // already have owner set
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
        // gNPC_t (NPCInfo) has no accessor; deref stays raw (recipe 2c).
        let npc_info = ctx.world.globals.NPCInfo;
        (*npc_info).timeOfDeath = 0;
        (*npc_info).shotTime = 0;
        crate::NPC_goal::NPC_ClearGoal(ctx);
        NPC_ChangeWeapon((*client).ps.weapon);

        ctx.world.entity_mut(ent).pain = Some(crate::ent_fn_enums::EntPain::NPC_Pain).into();
        // pain/touch fn-ID enums assigned straight from the selector fns.
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
        // Raven `VectorCopy` is a macro; here it copies the `int angles[3]`.
        ucmd.angles = (*client).pers.cmd.angles;

        (*client).ps.groundEntityNum = ENTITYNUM_NONE;

        // NPCAI_MATCHPLAYERWEAPON: G_MatchPlayerWeapon commented out in oracle.

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
                // g_entities[0].client stats bump: commented out / no-op in oracle
            }
        }
        ctx.world.entity_mut(ent).waypoint = WAYPOINT_NONE;
        (*npc).homeWp = WAYPOINT_NONE;

        // FLAG (rule 4): two-entity copy — a fresh droid NPC (`droid_ent`, raw
        // `*mut gentity_t` from NPC_SpawnType) is cross-copied with the parent
        // `ent`. Parent fields are read into locals (ctx borrow ends before each
        // raw droid write); `droid_ent`/`veh`/pool-clients stay raw. Left as
        // Stage-2 debt — the copy-out/copy-in pilot shape does not apply here.
        let veh = ctx.world.entity(ent).m_pVehicle;
        if !veh.is_null() {
            if (*veh).m_iDroidUnitTag != -1 {
                let mut droid_npc_type: *mut c_char = core::ptr::null_mut();
                let model2 = ctx.world.entity(ent).model2;
                if !model2.is_null() && *model2.as_ref().unwrap_or(&0) != 0 {
                    droid_npc_type = model2;
                } else if !(*(*veh).m_pVehicleInfo).droidNPC.is_null() {
                    droid_npc_type = (*(*veh).m_pVehicleInfo).droidNPC;
                }

                if !droid_npc_type.is_null() {
                    let random_s = cstr("random");
                    let default_s = cstr("default");
                    if crate::q_shared::Q_stricmp(
                        random_s.as_ptr(),
                        droid_npc_type as *const c_char,
                    ) == 0
                        || crate::q_shared::Q_stricmp(
                            default_s.as_ptr(),
                            droid_npc_type as *const c_char,
                        ) == 0
                    {
                        droid_npc_type = if ctx.world.bg_state.rng.Q_irand(0, 1) != 0 {
                            cstr("r2d2").into_raw()
                        } else {
                            cstr("r5d2").into_raw()
                        };
                    }
                    let droid_ent = NPC_SpawnType(
                        ctx,
                        Some(ent),
                        droid_npc_type,
                        core::ptr::null_mut(),
                        qfalse,
                    );
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
                            // `Vehicle_t.m_pDroidUnit` is `mp_bg`'s own `bgEntity_t`; this crate's
                            // `bgEntity_t` name is the `gentity_t` alias (prelude), so the overlay
                            // cast targets the bg type fully qualified.
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
            // `gNPC_t` holds a `*mut AIGroupInfo_t` field (align 8); pad to an
            // 8-byte boundary first (see `BG_AllocPad8`) so every `(*ptr).field`
            // access downstream is safely dereferenceable.
            crate::bg_misc::BG_AllocPad8(&mut ctx.world.bg_state);
            (&mut ctx.world.globals.gNPCPtrs)[entNum as usize] = BG_Alloc(
                core::mem::size_of::<gNPC_t>() as c_int,
                &mut ctx.world.bg_state,
            ) as *mut gNPC_t;
        }

        let ptr = (&ctx.world.globals.gNPCPtrs)[entNum as usize];

        if !ptr.is_null() {
            // Byte-wise, like C's memset: `ptr` is BG_Alloc pool storage (4-byte
            // aligned only), not guaranteed 8-aligned for gNPC_t's pointer field.
            core::ptr::write_bytes(ptr as *mut u8, 0, core::mem::size_of::<gNPC_t>());
        }

        ptr
    }
}

/// Raven `NPC_DefaultScriptFlags`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:1356-1364`
pub fn NPC_DefaultScriptFlags(ent: &gentity_t) {
    // Ctx-free leaf takes `&gentity_t`; the caller's `ent.is_null()` guard is
    // vacuous behind a reference (dropped), the `NPC` guard is preserved.
    let npc = ent.NPC;
    if npc.is_null() {
        return;
    }
    // gNPC_t deref stays raw (recipe 2c): copied pointer value, tight unsafe.
    unsafe {
        (*npc).scriptFlags = SCF_CHASE_ENEMIES | SCF_LOOK_FOR_ENEMIES;
    }
}

/// Raven `NPC_Spawn_Do`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:1377-1763`
pub fn NPC_Spawn_Do(ctx: &mut GameContext, ent: EntityId) -> *mut gentity_t {
    unsafe {
        // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
        // Return stays raw `*mut gentity_t` (return conversion is a later pass).
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

        newent = crate::g_utils::G_Spawn(ctx);

        if newent.is_null() {
            crate::g_main::Com_Printf(
                cstr(&format!(
                    "{}ERROR: NPC G_Spawn failed\n",
                    S_COLOR_RED.to_string_lossy()
                ))
                .as_ptr(),
            );
            return core::ptr::null_mut();
        }

        (*newent).fullName = (*ent).fullName;

        (*newent).NPC = New_NPC_t(ctx, (*newent).s.number);
        if (*newent).NPC.is_null() {
            crate::g_main::Com_Printf(
                cstr(&format!(
                    "{}ERROR: NPC G_Alloc NPC failed\n",
                    S_COLOR_RED.to_string_lossy()
                ))
                .as_ptr(),
            );
            // Raven: goto finish; (unreachable `return NULL;` right after — the
            // goto always wins). Preserve control-flow, not shape (§C10).
            if (*ent).spawnflags & NSF_DROP_TO_FLOOR != 0 {
                crate::g_utils::G_SetOrigin(&mut *(ent), save_org);
            }
            return newent;
        }

        crate::g_utils::G_CreateFakeClient(ctx, (*newent).s.number, &mut (*newent).client);

        (*((*newent).NPC)).tempGoal = ent_id_opt(
            ctx.world.g_entities.as_mut_ptr(),
            crate::g_utils::G_Spawn(ctx),
        );

        if (*((*newent).NPC)).tempGoal.is_none() {
            // Oracle nulls NPC and `goto finish` — the finish path returns the
            // (non-null) newent; the `return NULL` after the goto is unreachable.
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
        (*temp_goal).classname = c"NPC_goal".as_ptr() as *mut c_char;
        (*temp_goal).parent = Some(ent_id(ctx.world.g_entities.as_mut_ptr(), newent));
        (*temp_goal).r.svFlags |= SVF_NOCLIENT;

        if (*newent).client.is_null() {
            crate::g_main::Com_Printf(
                cstr(&format!(
                    "{}ERROR: NPC BG_Alloc client failed\n",
                    S_COLOR_RED.to_string_lossy()
                ))
                .as_ptr(),
            );
            if (*ent).spawnflags & NSF_DROP_TO_FLOOR != 0 {
                crate::g_utils::G_SetOrigin(&mut *(ent), save_org);
            }
            return newent;
        }

        // Byte-wise over sizeof(gclient_t): `client` is `*mut c_void` here, so a
        // typed write_bytes would zero only 1 byte (size_of::<c_void>()), not the
        // whole struct C's `memset(newent->client, 0, sizeof(*newent->client))`
        // zeroes; the backing storage (G_CreateFakeClient -> BG_Alloc) is also
        // only 4-byte aligned, below gclient_t's pointer-field alignment.
        core::ptr::write_bytes(
            (*newent).client as *mut u8,
            0,
            core::mem::size_of::<gclient_t>(),
        );

        (*newent).playerState = &mut (*((*newent).client)).ps as *mut playerState_t;

        if (*ent).NPC_type.is_null() {
            (*ent).NPC_type = c"random".as_ptr() as *mut c_char;
        } else {
            (*ent).NPC_type = crate::q_shared::Q_strlwr(crate::g_spawn::G_NewString(
                ctx,
                (*ent).NPC_type as *const c_char,
            ));
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

        if !(*ent).message.is_null() {
            (*newent).message = (*ent).message;
            (*newent).flags |= FL_NO_KNOCKBACK;
        }

        let npc_vehicle = cstr("NPC_Vehicle");
        if crate::q_shared::Q_stricmp((*ent).classname as *const c_char, npc_vehicle.as_ptr()) == 0
        {
            let mut callbacks = crate::bg_channel::GameCallbacksImpl {
                // SEAM-BG-REENTRY (DEC-28, sanctioned) — GameCallbacksImpl.world is a `*mut GameWorld`
                // field aliasing bg_state; a raw store is required (bg-seam re-entry).
                world: ctx.world_raw(),
                engine: ctx.engine,
            };
            let i_veh_index = BG_VehicleGetIndex(
                (*ent).NPC_type as *const c_char,
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
                        (*ent).NPC_type as *const c_char,
                    );
                }
                VH_SPEEDER => {
                    crate::SpeederNPC::G_CreateSpeederNPC(
                        ctx,
                        &mut (*newent).m_pVehicle,
                        (*ent).NPC_type as *const c_char,
                    );
                }
                VH_FIGHTER => {
                    crate::FighterNPC::G_CreateFighterNPC(
                        ctx,
                        &mut (*newent).m_pVehicle,
                        (*ent).NPC_type as *const c_char,
                    );
                }
                VH_WALKER => {
                    crate::WalkerNPC::G_CreateWalkerNPC(
                        ctx,
                        &mut (*newent).m_pVehicle,
                        (*ent).NPC_type as *const c_char,
                    );
                }
                _ => {
                    crate::g_main::Com_Printf(
                        cstr(&format!(
                            "{} ERROR: Couldn't spawn NPC {}\n",
                            S_COLOR_RED.to_string_lossy(),
                            cstr_to_str((*ent).NPC_type as *const c_char)
                        ))
                        .as_ptr(),
                    );
                    crate::g_utils::G_FreeEntity(ctx, ctx.entity_id_of(newent));
                    crate::g_utils::G_FreeEntity(ctx, ctx.entity_id_of(ent));
                    return core::ptr::null_mut();
                }
            }

            (*((*newent).m_pVehicle)).m_vOrientation =
                &mut (*((*newent).client)).ps.vehOrientation[0] as *mut f32;

            // Overlay cast to `mp_bg`'s `bgEntity_t` (this crate's `bgEntity_t` is the
            // prelude `gentity_t` alias).
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
            (*newent).healingclass = (*ent).healingclass;
            (*newent).healingsound = (*ent).healingsound;
            (*newent).healingrate = (*ent).healingrate;
            (*newent).model2 = (*ent).model2;
        } else {
            (*((*newent).client)).ps.weapon = WP_NONE;
        }

        crate::q_math::_VectorCopy((*ent).s.origin, &mut (*newent).s.origin);
        crate::q_math::_VectorCopy((*ent).s.origin, &mut (*((*newent).client)).ps.origin);
        crate::q_math::_VectorCopy((*ent).s.origin, &mut (*newent).r.currentOrigin);
        crate::g_utils::G_SetOrigin(&mut *(newent), (*ent).s.origin);
        if crate::NPC_stats::NPC_ParseParms(
            ctx,
            (*ent).NPC_type as *const c_char,
            ctx.entity_id_of(newent).unwrap(),
        ) == qfalse
        {
            crate::g_main::Com_Printf(
                cstr(&format!(
                    "{} ERROR: Couldn't spawn NPC {}\n",
                    S_COLOR_RED.to_string_lossy(),
                    cstr_to_str((*ent).NPC_type as *const c_char)
                ))
                .as_ptr(),
            );
            crate::g_utils::G_FreeEntity(ctx, ctx.entity_id_of(newent));
            crate::g_utils::G_FreeEntity(ctx, ctx.entity_id_of(ent));
            return core::ptr::null_mut();
        }

        if !(*ent).NPC_type.is_null() {
            let kyle = cstr("kyle");
            let test = cstr("test");
            if crate::q_shared::Q_stricmp((*ent).NPC_type as *const c_char, kyle.as_ptr()) == 0 {
                (*((*newent).NPC)).aiFlags |= NPCAI_MATCHPLAYERWEAPON;
            } else if crate::q_shared::Q_stricmp((*ent).NPC_type as *const c_char, test.as_ptr())
                == 0
            {
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
                (*newent).classname = c"NPC".as_ptr() as *mut c_char;
            }
        }

        if (*newent).health == 0 {
            (*newent).health = (*ent).health;
        }
        (*newent).script_targetname = (*ent).NPC_targetname;
        (*newent).targetname = (*ent).NPC_targetname;
        (*newent).target = (*ent).NPC_target;
        (*newent).target2 = (*ent).target2;
        (*newent).target3 = (*ent).target3;
        (*newent).target4 = (*ent).target4;
        (*newent).wait = (*ent).wait;

        let mut index = BSET_FIRST;
        while index < NUM_BSETS {
            if !(*ent).behaviorSet[index as usize].is_null() {
                (*newent).behaviorSet[index as usize] = (*ent).behaviorSet[index as usize];
            }
            index += 1;
        }

        (*newent).classname = c"NPC".as_ptr() as *mut c_char;
        (*newent).NPC_type = (*ent).NPC_type;
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

        if !(*ent).paintarget.is_null() {
            (*newent).paintarget = (*ent).paintarget;
        }
        if !(*ent).opentarget.is_null() {
            (*newent).opentarget = (*ent).opentarget;
        }

        (*newent).s.eType = ET_NPC as c_int;

        if !(*ent).parms.is_null() {
            for parm_num in 0..MAX_PARMS {
                // Raven's `parm[parmNum]` null arm is constant-true (char array,
                // never NULL); only the `[0]` emptiness check survives.
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
        if !(*ent).team.is_null() && *(*ent).team != 0 {
            (*((*newent).client)).sess.sessionTeam = atoi((*ent).team as *const c_char);
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
            if !(*ent).target.is_null() {
                crate::g_utils::G_UseTargets(ctx, ctx.entity_id_of(ent), ctx.entity_id_of(ent));
            }
            if !(*ent).closetarget.is_null() {
                (*newent).target = (*ent).closetarget;
            }
            (*ent).targetname = core::ptr::null_mut();
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
    // STAGE-1: thin pass-through — `EntityId` forwarded directly (no re-derive).
    NPC_Spawn_Do(ctx, ent);
}

/// Raven `NPC_ShySpawn`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:1814-1831`
pub fn NPC_ShySpawn(ctx: &mut GameContext, ent: EntityId) {
    let nt = ctx.world.level.time + SHY_THINK_TIME;
    ctx.world.entity_mut(ent).nextthink = nt;
    ctx.world.entity_mut(ent).think = Some(EntThink::NPC_ShySpawn).into();

    // g_entities[0] is player 0, a real client slot (recipe 2b).
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
    // `other`/`activator` are unused by the body (Raven use-handler signature).
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
pub fn NPC_PrecacheType(ctx: &mut GameContext, NPC_type: *mut c_char) {
    let fakespawner = crate::g_utils::G_Spawn(ctx);
    if let Some(id) = ctx.entity_id_of(fakespawner) {
        ctx.world.entity_mut(id).NPC_type = NPC_type;
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
    let full_name = ctx.world.entity(self_).fullName;
    if full_name.is_null() || unsafe { *full_name == 0 } {
        ctx.world.entity_mut(self_).fullName = c"Humanoid Lifeform".as_ptr() as *mut c_char;
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

    let npc_type = ctx.world.entity(self_).NPC_type;
    crate::NPC_stats::NPC_PrecacheAnimationCFG(npc_type as *const c_char);

    crate::NPC_stats::NPC_Precache(ctx, self_);

    if !ctx.world.entity(self_).targetname.is_null() {
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
        let mut droid_npc_type: *const c_char = core::ptr::null();
        let sp_npc_type = ctx.world.entity(spawner).NPC_type;
        let mut callbacks = crate::bg_channel::GameCallbacksImpl {
            // SEAM-BG-REENTRY (DEC-28, sanctioned) — GameCallbacksImpl.world is a `*mut GameWorld`
            // field aliasing bg_state; a raw store is required (bg-seam re-entry).
            world: ctx.world_raw(),
            engine: ctx.engine,
        };
        let i_veh_index = BG_VehicleGetIndex(
            sp_npc_type as *const c_char,
            &mut ctx.world.bg_state,
            &crate::bg_channel::GameBgTraps::new(ctx.engine),
            &mut callbacks,
        );
        if i_veh_index == VEHICLE_NONE {
            return qfalse;
        }

        crate::g_utils::G_ModelIndex(
            cstr(&format!("${}", cstr_to_str(sp_npc_type as *const c_char))).as_ptr(),
        );

        let p_veh_info = &(&ctx.world.bg_state.g_vehicleInfo)[i_veh_index as usize];
        if !p_veh_info.model.is_null() && *p_veh_info.model != 0 {
            let mut temp_g2: *mut c_void = core::ptr::null_mut();
            let mut skin: c_int = 0;
            if !p_veh_info.skin.is_null() && *p_veh_info.skin != 0 {
                let path = cstr(&format!(
                    "models/players/{}/model_{}.skin",
                    cstr_to_str(p_veh_info.model as *const c_char),
                    cstr_to_str(p_veh_info.skin as *const c_char)
                ));
                skin = trap::R_RegisterSkin(
                    ctx.engine,
                    mp_abi::game::syscalls::G_R_REGISTERSKIN::GRRegisterskinArgs::new(path),
                );
            }
            let glm_path = cstr(&format!(
                "models/players/{}/model.glm",
                cstr_to_str(p_veh_info.model as *const c_char)
            ));
            trap::G2API_InitGhoul2Model(
                ctx.engine,
                mp_abi::game::syscalls::G_G2_INITGHOUL2MODEL::GG2Initghoul2ModelArgs::new(
                    &mut temp_g2 as *mut *mut c_void,
                    glm_path,
                    0,
                    skin,
                    0,
                    0,
                    0,
                ),
            );
            if !temp_g2.is_null() {
                let mut gla_name: [c_char; 1024] = [0; 1024];
                gla_name[0] = 0;
                trap::G2API_GetGLAName(
                    ctx.engine,
                    mp_abi::game::syscalls::G_G2_GETGLANAME::GG2GetglanameArgs::new(
                        temp_g2,
                        0,
                        gla_name.as_mut_ptr(),
                    ),
                );

                if gla_name[0] != 0 {
                    let slash = crate::q_shared::Q_strrchr(gla_name.as_ptr(), '/' as c_int);
                    if !slash.is_null() {
                        let anim_cfg = cstr("/animation.cfg");
                        let n = crate::q_shared::Q_strlen(anim_cfg.as_ptr());
                        core::ptr::copy_nonoverlapping(anim_cfg.as_ptr(), slash, n as usize + 1);

                        let traps = crate::bg_channel::GameBgTraps::new(ctx.engine);
                        // STAGE-2b: irreducible — `GameCallbacksImpl.world` is a raw
                        // `*mut GameWorld` field fed by the `world_raw()` accessor.
                        let mut callbacks = crate::bg_channel::GameCallbacksImpl {
                            world: ctx.world_raw(),
                            engine: ctx.engine,
                        };
                        crate::bg_panimate::BG_ParseAnimationFile(
                            &mut ctx.world.bg_state,
                            &traps,
                            &mut callbacks,
                            gla_name.as_ptr(),
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

        let sp_model2 = ctx.world.entity(spawner).model2;
        if !sp_model2.is_null() && *sp_model2 != 0 {
            droid_npc_type = sp_model2 as *const c_char;
        } else if !(&ctx.world.bg_state.g_vehicleInfo)[i_veh_index as usize]
            .droidNPC
            .is_null()
            && *(&ctx.world.bg_state.g_vehicleInfo)[i_veh_index as usize].droidNPC != 0
        {
            droid_npc_type =
                (&ctx.world.bg_state.g_vehicleInfo)[i_veh_index as usize].droidNPC as *const c_char;
        }

        if !droid_npc_type.is_null() {
            let random_s = cstr("random");
            let default_s = cstr("default");
            if crate::q_shared::Q_stricmp(random_s.as_ptr(), droid_npc_type) == 0
                || crate::q_shared::Q_stricmp(default_s.as_ptr(), droid_npc_type) == 0
            {
                NPC_PrecacheType(ctx, cstr("r2d2").into_raw());
                NPC_PrecacheType(ctx, cstr("r5d2").into_raw());
            } else {
                NPC_PrecacheType(ctx, droid_npc_type as *mut c_char);
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
    // `other`/`activator` are unused by the body (Raven use-handler signature).
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
    if ctx.world.entity(self_).NPC_type.is_null() {
        ctx.world.entity_mut(self_).NPC_type = c"swoop".as_ptr() as *mut c_char;
    }

    if ctx.world.entity(self_).classname.is_null() {
        ctx.world.entity_mut(self_).classname = c"NPC_Vehicle".as_ptr() as *mut c_char;
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

    if !ctx.world.entity(self_).targetname.is_null() {
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
    ctx.world.entity_mut(self_).NPC_type = c"Kyle".as_ptr() as *mut c_char;
    WP_SetSaberModel(None, CLASS_KYLE);
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Lando`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:2354-2359`
pub fn SP_NPC_Lando(ctx: &mut GameContext, self_: EntityId) {
    ctx.world.entity_mut(self_).NPC_type = c"Lando".as_ptr() as *mut c_char;
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Jan`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:2368-2373`
pub fn SP_NPC_Jan(ctx: &mut GameContext, self_: EntityId) {
    ctx.world.entity_mut(self_).NPC_type = c"Jan".as_ptr() as *mut c_char;
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Luke`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:2382-2389`
pub fn SP_NPC_Luke(ctx: &mut GameContext, self_: EntityId) {
    ctx.world.entity_mut(self_).NPC_type = c"Luke".as_ptr() as *mut c_char;
    WP_SetSaberModel(None, CLASS_LUKE);
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_MonMothma`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:2398-2403`
pub fn SP_NPC_MonMothma(ctx: &mut GameContext, self_: EntityId) {
    ctx.world.entity_mut(self_).NPC_type = c"MonMothma".as_ptr() as *mut c_char;
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Tavion`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:2412-2419`
pub fn SP_NPC_Tavion(ctx: &mut GameContext, self_: EntityId) {
    ctx.world.entity_mut(self_).NPC_type = c"Tavion".as_ptr() as *mut c_char;
    WP_SetSaberModel(None, CLASS_TAVION);
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Tavion_New`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:2432-2448`
pub fn SP_NPC_Tavion_New(ctx: &mut GameContext, self_: EntityId) {
    let e = ctx.world.entity_mut(self_);
    if e.spawnflags & 1 != 0 {
        e.NPC_type = c"tavion_scepter".as_ptr() as *mut c_char;
    } else if e.spawnflags & 2 != 0 {
        e.NPC_type = c"tavion_sith_sword".as_ptr() as *mut c_char;
    } else {
        e.NPC_type = c"tavion_new".as_ptr() as *mut c_char;
    }
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Alora`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:2460-2472`
pub fn SP_NPC_Alora(ctx: &mut GameContext, self_: EntityId) {
    let e = ctx.world.entity_mut(self_);
    if e.spawnflags & 1 != 0 {
        e.NPC_type = c"alora_dual".as_ptr() as *mut c_char;
    } else {
        e.NPC_type = c"alora".as_ptr() as *mut c_char;
    }
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Reborn_New`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:2487-2524`
pub fn SP_NPC_Reborn_New(ctx: &mut GameContext, self_: EntityId) {
    let e = ctx.world.entity_mut(self_);
    if e.NPC_type.is_null() {
        if e.spawnflags & 4 != 0 {
            if e.spawnflags & 1 != 0 {
                e.NPC_type = c"reborn_dual2".as_ptr() as *mut c_char;
            } else if e.spawnflags & 2 != 0 {
                e.NPC_type = c"reborn_staff2".as_ptr() as *mut c_char;
            } else {
                e.NPC_type = c"reborn_new2".as_ptr() as *mut c_char;
            }
        } else {
            if e.spawnflags & 1 != 0 {
                e.NPC_type = c"reborn_dual".as_ptr() as *mut c_char;
            } else if e.spawnflags & 2 != 0 {
                e.NPC_type = c"reborn_staff".as_ptr() as *mut c_char;
            } else {
                e.NPC_type = c"reborn_new".as_ptr() as *mut c_char;
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
    if e.NPC_type.is_null() {
        if e.spawnflags & 1 != 0 {
            e.NPC_type = if e.spawnflags & 8 != 0 {
                c"cultist_saber_med_throw".as_ptr() as *mut c_char
            } else {
                c"cultist_saber_med".as_ptr() as *mut c_char
            };
        } else if e.spawnflags & 2 != 0 {
            e.NPC_type = if e.spawnflags & 8 != 0 {
                c"cultist_saber_strong_throw".as_ptr() as *mut c_char
            } else {
                c"cultist_saber_strong".as_ptr() as *mut c_char
            };
        } else if e.spawnflags & 2 != 0 {
            e.NPC_type = if e.spawnflags & 8 != 0 {
                c"cultist_saber_all_throw".as_ptr() as *mut c_char
            } else {
                c"cultist_saber_all".as_ptr() as *mut c_char
            };
        } else {
            e.NPC_type = if e.spawnflags & 8 != 0 {
                c"cultist_saber_throw".as_ptr() as *mut c_char
            } else {
                c"cultist_saber".as_ptr() as *mut c_char
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
    if e.NPC_type.is_null() {
        if e.spawnflags & 1 != 0 {
            e.NPC_type = if e.spawnflags & 8 != 0 {
                c"cultist_saber_med_throw2".as_ptr() as *mut c_char
            } else {
                c"cultist_saber_med2".as_ptr() as *mut c_char
            };
        } else if e.spawnflags & 2 != 0 {
            e.NPC_type = if e.spawnflags & 8 != 0 {
                c"cultist_saber_strong_throw2".as_ptr() as *mut c_char
            } else {
                c"cultist_saber_strong2".as_ptr() as *mut c_char
            };
        } else if e.spawnflags & 2 != 0 {
            e.NPC_type = if e.spawnflags & 8 != 0 {
                c"cultist_saber_all_throw2".as_ptr() as *mut c_char
            } else {
                c"cultist_saber_all2".as_ptr() as *mut c_char
            };
        } else {
            e.NPC_type = if e.spawnflags & 8 != 0 {
                c"cultist_saber_throw".as_ptr() as *mut c_char
            } else {
                c"cultist_saber2".as_ptr() as *mut c_char
            };
        }
    }
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Cultist`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:2679-2725`
pub fn SP_NPC_Cultist(ctx: &mut GameContext, self_: EntityId) {
    if ctx.world.entity(self_).NPC_type.is_null() {
        if ctx.world.entity(self_).spawnflags & 1 != 0 {
            ctx.world.entity_mut(self_).NPC_type = core::ptr::null_mut();
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
            ctx.world.entity_mut(self_).NPC_type = c"cultist_grip".as_ptr() as *mut c_char;
        } else if ctx.world.entity(self_).spawnflags & 4 != 0 {
            ctx.world.entity_mut(self_).NPC_type = c"cultist_lightning".as_ptr() as *mut c_char;
        } else if ctx.world.entity(self_).spawnflags & 8 != 0 {
            ctx.world.entity_mut(self_).NPC_type = c"cultist_drain".as_ptr() as *mut c_char;
        } else {
            ctx.world.entity_mut(self_).NPC_type = c"cultist".as_ptr() as *mut c_char;
        }
    }
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Cultist_Commando`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:2737-2744`
pub fn SP_NPC_Cultist_Commando(ctx: &mut GameContext, self_: EntityId) {
    let e = ctx.world.entity_mut(self_);
    if e.NPC_type.is_null() {
        e.NPC_type = c"cultistcommando".as_ptr() as *mut c_char;
    }
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Cultist_Destroyer`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:2755-2759`
pub fn SP_NPC_Cultist_Destroyer(ctx: &mut GameContext, self_: EntityId) {
    ctx.world.entity_mut(self_).NPC_type = c"cultist".as_ptr() as *mut c_char;
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Reelo`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:2768-2773`
pub fn SP_NPC_Reelo(ctx: &mut GameContext, self_: EntityId) {
    ctx.world.entity_mut(self_).NPC_type = c"Reelo".as_ptr() as *mut c_char;
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Galak`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:2784-2797`
pub fn SP_NPC_Galak(ctx: &mut GameContext, self_: EntityId) {
    if ctx.world.entity(self_).spawnflags & 1 != 0 {
        ctx.world.entity_mut(self_).NPC_type = c"Galak_Mech".as_ptr() as *mut c_char;
        crate::NPC_AI_GalakMech::NPC_GalakMech_Precache(ctx);
    } else {
        ctx.world.entity_mut(self_).NPC_type = c"Galak".as_ptr() as *mut c_char;
    }
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Desann`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:2806-2813`
pub fn SP_NPC_Desann(ctx: &mut GameContext, self_: EntityId) {
    ctx.world.entity_mut(self_).NPC_type = c"Desann".as_ptr() as *mut c_char;
    WP_SetSaberModel(None, CLASS_DESANN);
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Bartender`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:2822-2827`
pub fn SP_NPC_Bartender(ctx: &mut GameContext, self_: EntityId) {
    ctx.world.entity_mut(self_).NPC_type = c"Bartender".as_ptr() as *mut c_char;
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_MorganKatarn`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:2836-2841`
pub fn SP_NPC_MorganKatarn(ctx: &mut GameContext, self_: EntityId) {
    ctx.world.entity_mut(self_).NPC_type = c"MorganKatarn".as_ptr() as *mut c_char;
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Jedi`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:2857-2887`
pub fn SP_NPC_Jedi(ctx: &mut GameContext, self_: EntityId) {
    if ctx.world.entity(self_).NPC_type.is_null() {
        if ctx.world.entity(self_).spawnflags & 1 != 0 {
            ctx.world.entity_mut(self_).NPC_type = c"jeditrainer".as_ptr() as *mut c_char;
        } else if ctx.world.bg_state.rng.Q_irand(0, 1) != 0 {
            ctx.world.entity_mut(self_).NPC_type = c"Jedi".as_ptr() as *mut c_char;
        } else {
            ctx.world.entity_mut(self_).NPC_type = c"Jedi2".as_ptr() as *mut c_char;
        }
    }
    WP_SetSaberModel(None, CLASS_JEDI);
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Prisoner`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:2896-2911`
pub fn SP_NPC_Prisoner(ctx: &mut GameContext, self_: EntityId) {
    if ctx.world.entity(self_).NPC_type.is_null() {
        let t = if ctx.world.bg_state.rng.Q_irand(0, 1) != 0 {
            c"Prisoner".as_ptr() as *mut c_char
        } else {
            c"Prisoner2".as_ptr() as *mut c_char
        };
        ctx.world.entity_mut(self_).NPC_type = t;
    }
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Rebel`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:2920-2935`
pub fn SP_NPC_Rebel(ctx: &mut GameContext, self_: EntityId) {
    if ctx.world.entity(self_).NPC_type.is_null() {
        let t = if ctx.world.bg_state.rng.Q_irand(0, 1) != 0 {
            c"Rebel".as_ptr() as *mut c_char
        } else {
            c"Rebel2".as_ptr() as *mut c_char
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
        ctx.world.entity_mut(self_).NPC_type = c"rockettrooper".as_ptr() as *mut c_char;
    } else if ctx.world.entity(self_).spawnflags & 4 != 0 {
        ctx.world.entity_mut(self_).NPC_type = c"stofficeralt".as_ptr() as *mut c_char;
    } else if ctx.world.entity(self_).spawnflags & 2 != 0 {
        ctx.world.entity_mut(self_).NPC_type = c"stcommander".as_ptr() as *mut c_char;
    } else if ctx.world.entity(self_).spawnflags & 1 != 0 {
        ctx.world.entity_mut(self_).NPC_type = c"stofficer".as_ptr() as *mut c_char;
    } else {
        let t = if ctx.world.bg_state.rng.Q_irand(0, 1) != 0 {
            c"StormTrooper".as_ptr() as *mut c_char
        } else {
            c"StormTrooper2".as_ptr() as *mut c_char
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
    ctx.world.entity_mut(self_).NPC_type = c"snowtrooper".as_ptr() as *mut c_char;
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Tie_Pilot`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3016-3021`
pub fn SP_NPC_Tie_Pilot(ctx: &mut GameContext, self_: EntityId) {
    ctx.world.entity_mut(self_).NPC_type = c"stormpilot".as_ptr() as *mut c_char;
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Ugnaught`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3030-3045`
pub fn SP_NPC_Ugnaught(ctx: &mut GameContext, self_: EntityId) {
    if ctx.world.entity(self_).NPC_type.is_null() {
        let t = if ctx.world.bg_state.rng.Q_irand(0, 1) != 0 {
            c"Ugnaught".as_ptr() as *mut c_char
        } else {
            c"Ugnaught2".as_ptr() as *mut c_char
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
    if e.NPC_type.is_null() {
        e.NPC_type = if e.spawnflags & 1 != 0 {
            c"jawa_armed".as_ptr() as *mut c_char
        } else {
            c"jawa".as_ptr() as *mut c_char
        };
    }
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Gran`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3084-3110`
pub fn SP_NPC_Gran(ctx: &mut GameContext, self_: EntityId) {
    if ctx.world.entity(self_).NPC_type.is_null() {
        if ctx.world.entity(self_).spawnflags & 1 != 0 {
            ctx.world.entity_mut(self_).NPC_type = c"granshooter".as_ptr() as *mut c_char;
        } else if ctx.world.entity(self_).spawnflags & 2 != 0 {
            ctx.world.entity_mut(self_).NPC_type = c"granboxer".as_ptr() as *mut c_char;
        } else {
            let t = if ctx.world.bg_state.rng.Q_irand(0, 1) != 0 {
                c"gran".as_ptr() as *mut c_char
            } else {
                c"gran2".as_ptr() as *mut c_char
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
    if e.NPC_type.is_null() {
        e.NPC_type = if e.spawnflags & 1 != 0 {
            c"rodian2".as_ptr() as *mut c_char
        } else {
            c"rodian".as_ptr() as *mut c_char
        };
    }
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Weequay`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3145-3167`
pub fn SP_NPC_Weequay(ctx: &mut GameContext, self_: EntityId) {
    if ctx.world.entity(self_).NPC_type.is_null() {
        let t = match ctx.world.bg_state.rng.Q_irand(0, 3) {
            0 => c"Weequay".as_ptr() as *mut c_char,
            1 => c"Weequay2".as_ptr() as *mut c_char,
            2 => c"Weequay3".as_ptr() as *mut c_char,
            _ => c"Weequay4".as_ptr() as *mut c_char,
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
    if e.NPC_type.is_null() {
        e.NPC_type = c"Trandoshan".as_ptr() as *mut c_char;
    }
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Tusken`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3193-3208`
pub fn SP_NPC_Tusken(ctx: &mut GameContext, self_: EntityId) {
    let e = ctx.world.entity_mut(self_);
    if e.NPC_type.is_null() {
        e.NPC_type = if e.spawnflags & 1 != 0 {
            c"tuskensniper".as_ptr() as *mut c_char
        } else {
            c"tusken".as_ptr() as *mut c_char
        };
    }
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Noghri`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3217-3225`
pub fn SP_NPC_Noghri(ctx: &mut GameContext, self_: EntityId) {
    let e = ctx.world.entity_mut(self_);
    if e.NPC_type.is_null() {
        e.NPC_type = c"noghri".as_ptr() as *mut c_char;
    }
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_SwampTrooper`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3235-3250`
pub fn SP_NPC_SwampTrooper(ctx: &mut GameContext, self_: EntityId) {
    let e = ctx.world.entity_mut(self_);
    if e.NPC_type.is_null() {
        e.NPC_type = if e.spawnflags & 1 != 0 {
            c"SwampTrooper2".as_ptr() as *mut c_char
        } else {
            c"SwampTrooper".as_ptr() as *mut c_char
        };
    }
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Imperial`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3267-3301`
pub fn SP_NPC_Imperial(ctx: &mut GameContext, self_: EntityId) {
    let e = ctx.world.entity_mut(self_);
    if e.NPC_type.is_null() {
        if e.spawnflags & 1 != 0 {
            e.NPC_type = c"ImpOfficer".as_ptr() as *mut c_char;
        } else if e.spawnflags & 2 != 0 {
            e.NPC_type = c"ImpCommander".as_ptr() as *mut c_char;
        } else {
            e.NPC_type = c"Imperial".as_ptr() as *mut c_char;
        }
    }
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_ImpWorker`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3310-3329`
pub fn SP_NPC_ImpWorker(ctx: &mut GameContext, self_: EntityId) {
    if ctx.world.entity(self_).NPC_type.is_null() {
        if ctx.world.bg_state.rng.Q_irand(0, 2) == 0 {
            ctx.world.entity_mut(self_).NPC_type = c"ImpWorker".as_ptr() as *mut c_char;
        } else if ctx.world.bg_state.rng.Q_irand(0, 1) != 0 {
            ctx.world.entity_mut(self_).NPC_type = c"ImpWorker2".as_ptr() as *mut c_char;
        } else {
            ctx.world.entity_mut(self_).NPC_type = c"ImpWorker3".as_ptr() as *mut c_char;
        }
    }
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_BespinCop`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3338-3353`
pub fn SP_NPC_BespinCop(ctx: &mut GameContext, self_: EntityId) {
    if ctx.world.entity(self_).NPC_type.is_null() {
        let t = if ctx.world.bg_state.rng.Q_irand(0, 1) == 0 {
            c"BespinCop".as_ptr() as *mut c_char
        } else {
            c"BespinCop2".as_ptr() as *mut c_char
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
    if e.NPC_type.is_null() {
        if e.spawnflags & 1 != 0 {
            e.NPC_type = c"rebornforceuser".as_ptr() as *mut c_char;
        } else if e.spawnflags & 2 != 0 {
            e.NPC_type = c"rebornfencer".as_ptr() as *mut c_char;
        } else if e.spawnflags & 4 != 0 {
            e.NPC_type = c"rebornacrobat".as_ptr() as *mut c_char;
        } else if e.spawnflags & 8 != 0 {
            e.NPC_type = c"rebornboss".as_ptr() as *mut c_char;
        } else {
            e.NPC_type = c"reborn".as_ptr() as *mut c_char;
        }
    }
    WP_SetSaberModel(None, CLASS_REBORN);
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_ShadowTrooper`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3409-3427`
pub fn SP_NPC_ShadowTrooper(ctx: &mut GameContext, self_: EntityId) {
    if ctx.world.entity(self_).NPC_type.is_null() {
        let t = if ctx.world.bg_state.rng.Q_irand(0, 1) == 0 {
            c"ShadowTrooper".as_ptr() as *mut c_char
        } else {
            c"ShadowTrooper2".as_ptr() as *mut c_char
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
    ctx.world.entity_mut(self_).NPC_type = c"Murjj".as_ptr() as *mut c_char;
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Monster_Swamp`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3453-3458`
pub fn SP_NPC_Monster_Swamp(ctx: &mut GameContext, self_: EntityId) {
    ctx.world.entity_mut(self_).NPC_type = c"Swamp".as_ptr() as *mut c_char;
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Monster_Howler`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3467-3472`
pub fn SP_NPC_Monster_Howler(ctx: &mut GameContext, self_: EntityId) {
    let s = G_NewString(ctx, c"howler".as_ptr() as *const c_char);
    ctx.world.entity_mut(self_).NPC_type = s;
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_MineMonster`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3481-3487`
pub fn SP_NPC_MineMonster(ctx: &mut GameContext, self_: EntityId) {
    let s = G_NewString(ctx, c"minemonster".as_ptr() as *const c_char);
    ctx.world.entity_mut(self_).NPC_type = s;
    SP_NPC_spawner(ctx, self_);
    NPC_MineMonster_Precache(ctx);
}

/// Raven `SP_NPC_Monster_Claw`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3496-3501`
pub fn SP_NPC_Monster_Claw(ctx: &mut GameContext, self_: EntityId) {
    let s = G_NewString(ctx, c"Claw".as_ptr() as *const c_char);
    ctx.world.entity_mut(self_).NPC_type = s;
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Monster_Glider`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3510-3515`
pub fn SP_NPC_Monster_Glider(ctx: &mut GameContext, self_: EntityId) {
    let s = G_NewString(ctx, c"Glider".as_ptr() as *const c_char);
    ctx.world.entity_mut(self_).NPC_type = s;
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Monster_Flier2`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3524-3529`
pub fn SP_NPC_Monster_Flier2(ctx: &mut GameContext, self_: EntityId) {
    let s = G_NewString(ctx, c"Flier2".as_ptr() as *const c_char);
    ctx.world.entity_mut(self_).NPC_type = s;
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Monster_Lizard`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3538-3543`
pub fn SP_NPC_Monster_Lizard(ctx: &mut GameContext, self_: EntityId) {
    let s = G_NewString(ctx, c"Lizard".as_ptr() as *const c_char);
    ctx.world.entity_mut(self_).NPC_type = s;
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Monster_Fish`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3552-3557`
pub fn SP_NPC_Monster_Fish(ctx: &mut GameContext, self_: EntityId) {
    let s = G_NewString(ctx, c"Fish".as_ptr() as *const c_char);
    ctx.world.entity_mut(self_).NPC_type = s;
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Monster_Wampa`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3568-3575`
pub fn SP_NPC_Monster_Wampa(ctx: &mut GameContext, self_: EntityId) {
    let s = G_NewString(ctx, c"wampa".as_ptr() as *const c_char);
    ctx.world.entity_mut(self_).NPC_type = s;
    NPC_Wampa_Precache(ctx);
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Monster_Rancor`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3584-3589`
pub fn SP_NPC_Monster_Rancor(ctx: &mut GameContext, self_: EntityId) {
    let s = G_NewString(ctx, c"rancor".as_ptr() as *const c_char);
    ctx.world.entity_mut(self_).NPC_type = s;
    SP_NPC_spawner(ctx, self_);
}

/// Raven `SP_NPC_Droid_Interrogator`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3602-3609`
pub fn SP_NPC_Droid_Interrogator(ctx: &mut GameContext, self_: EntityId) {
    let s = G_NewString(ctx, c"interrogator".as_ptr() as *const c_char);
    ctx.world.entity_mut(self_).NPC_type = s;
    SP_NPC_spawner(ctx, self_);
    NPC_Interrogator_Precache(ctx, Some(self_));
}

/// Raven `SP_NPC_Droid_Probe`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3620-3627`
pub fn SP_NPC_Droid_Probe(ctx: &mut GameContext, self_: EntityId) {
    let s = G_NewString(ctx, c"probe".as_ptr() as *const c_char);
    ctx.world.entity_mut(self_).NPC_type = s;
    SP_NPC_spawner(ctx, self_);
    NPC_Probe_Precache(ctx);
}

/// Raven `SP_NPC_Droid_Mark1`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3639-3646`
pub fn SP_NPC_Droid_Mark1(ctx: &mut GameContext, self_: EntityId) {
    let s = G_NewString(ctx, c"mark1".as_ptr() as *const c_char);
    ctx.world.entity_mut(self_).NPC_type = s;
    SP_NPC_spawner(ctx, self_);
    NPC_Mark1_Precache(ctx);
}

/// Raven `SP_NPC_Droid_Mark2`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3658-3665`
pub fn SP_NPC_Droid_Mark2(ctx: &mut GameContext, self_: EntityId) {
    let s = G_NewString(ctx, c"mark2".as_ptr() as *const c_char);
    ctx.world.entity_mut(self_).NPC_type = s;
    SP_NPC_spawner(ctx, self_);
    NPC_Mark2_Precache(ctx);
}

/// Raven `SP_NPC_Droid_ATST`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3674-3688`
pub fn SP_NPC_Droid_ATST(ctx: &mut GameContext, self_: EntityId) {
    let s = if ctx.world.entity(self_).spawnflags & 1 != 0 {
        G_NewString(ctx, c"atst_vehicle".as_ptr() as *const c_char)
    } else {
        G_NewString(ctx, c"atst".as_ptr() as *const c_char)
    };
    ctx.world.entity_mut(self_).NPC_type = s;
    SP_NPC_spawner(ctx, self_);
    NPC_ATST_Precache(ctx);
}

/// Raven `SP_NPC_Droid_Remote`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3699-3706`
pub fn SP_NPC_Droid_Remote(ctx: &mut GameContext, self_: EntityId) {
    let s = G_NewString(ctx, c"remote".as_ptr() as *const c_char);
    ctx.world.entity_mut(self_).NPC_type = s;
    SP_NPC_spawner(ctx, self_);
    NPC_Remote_Precache(ctx);
}

/// Raven `SP_NPC_Droid_Seeker`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3717-3724`
pub fn SP_NPC_Droid_Seeker(ctx: &mut GameContext, self_: EntityId) {
    let s = G_NewString(ctx, c"seeker".as_ptr() as *const c_char);
    ctx.world.entity_mut(self_).NPC_type = s;
    SP_NPC_spawner(ctx, self_);
    NPC_Seeker_Precache(ctx);
}

/// Raven `SP_NPC_Droid_Sentry`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3735-3742`
pub fn SP_NPC_Droid_Sentry(ctx: &mut GameContext, self_: EntityId) {
    let s = G_NewString(ctx, c"sentry".as_ptr() as *const c_char);
    ctx.world.entity_mut(self_).NPC_type = s;
    SP_NPC_spawner(ctx, self_);
    NPC_Sentry_Precache(ctx);
}

/// Raven `SP_NPC_Droid_Gonk`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3755-3763`
pub fn SP_NPC_Droid_Gonk(ctx: &mut GameContext, self_: EntityId) {
    let s = G_NewString(ctx, c"gonk".as_ptr() as *const c_char);
    ctx.world.entity_mut(self_).NPC_type = s;
    SP_NPC_spawner(ctx, self_);
    NPC_Gonk_Precache(ctx);
}

/// Raven `SP_NPC_Droid_Mouse`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3776-3785`
pub fn SP_NPC_Droid_Mouse(ctx: &mut GameContext, self_: EntityId) {
    let s = G_NewString(ctx, c"mouse".as_ptr() as *const c_char);
    ctx.world.entity_mut(self_).NPC_type = s;
    SP_NPC_spawner(ctx, self_);
    NPC_Mouse_Precache(ctx);
}

/// Raven `SP_NPC_Droid_R2D2`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:3798-3812`
pub fn SP_NPC_Droid_R2D2(ctx: &mut GameContext, self_: EntityId) {
    let s = if ctx.world.entity(self_).spawnflags & 1 != 0 {
        G_NewString(ctx, c"r2d2_imp".as_ptr() as *const c_char)
    } else {
        G_NewString(ctx, c"r2d2".as_ptr() as *const c_char)
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
        G_NewString(ctx, c"r5d2_imp".as_ptr() as *const c_char)
    } else {
        G_NewString(ctx, c"r5d2".as_ptr() as *const c_char)
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
        G_NewString(ctx, c"protocol_imp".as_ptr() as *const c_char)
    } else {
        G_NewString(ctx, c"protocol".as_ptr() as *const c_char)
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
    npc_type: *mut c_char,
    targetname: *mut c_char,
    isVehicle: qboolean,
) -> *mut gentity_t {
    // STAGE-1: `ent` is `Option<EntityId>` (the body null-checks it); re-derived
    // to a raw pointer, verbatim body preserved (Stage-2 debt). `npc_type`/
    // `targetname` are C strings and stay raw; the return stays raw `*mut
    // gentity_t` (return conversion is a later pass).
    let ent: *mut gentity_t = ent.map_or(core::ptr::null_mut(), |i| ctx.entity_mut(i));
    let npc_spawner = G_Spawn(ctx);

    if npc_spawner.is_null() {
        Com_Printf(c"NPC_Spawn Error: Out of entities!\n".as_ptr() as *const c_char);
        return std::ptr::null_mut();
    }

    unsafe {
        (*npc_spawner).think = Some(EntThink::G_FreeEntity).into();
        (*npc_spawner).nextthink = ctx.world.level.time + FRAMETIME;
    }

    if npc_type.is_null() {
        return std::ptr::null_mut();
    }

    unsafe {
        if (*npc_type) == b'\0' as c_char {
            Com_Printf(c"Error, expected one of:\n NPC spawn [NPC type (from ext_data/NPCs)]\n NPC spawn vehicle [VEH type (from ext_data/vehicles)]\n".as_ptr() as *const c_char);
            return std::ptr::null_mut();
        }
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
        (*npc_spawner).NPC_type = G_NewString(ctx, npc_type);

        if !targetname.is_null() {
            (*npc_spawner).NPC_targetname = G_NewString(ctx, targetname);
        }

        (*npc_spawner).count = 1;
        (*npc_spawner).delay = 0;

        if isVehicle != 0 {
            (*npc_spawner).classname = G_NewString(ctx, c"NPC_Vehicle".as_ptr() as *const c_char);
        }
    }

    // Call precache funcs
    let npc_type_str = unsafe {
        if npc_type.is_null() {
            ""
        } else {
            std::ffi::CStr::from_ptr(npc_type).to_str().unwrap_or("")
        }
    };

    if Q_stricmp(c"gonk".as_ptr() as *const c_char, npc_type) == 0 {
        NPC_Gonk_Precache(ctx);
    } else if Q_stricmp(c"mouse".as_ptr() as *const c_char, npc_type) == 0 {
        NPC_Mouse_Precache(ctx);
    } else if Q_strncmp(c"r2d2".as_ptr() as *const c_char, npc_type, 4) == 0 {
        NPC_R2D2_Precache(ctx);
    } else if Q_stricmp(c"atst".as_ptr() as *const c_char, npc_type) == 0 {
        NPC_ATST_Precache(ctx);
    } else if Q_strncmp(c"r5d2".as_ptr() as *const c_char, npc_type, 4) == 0 {
        NPC_R5D2_Precache(ctx);
    } else if Q_stricmp(c"mark1".as_ptr() as *const c_char, npc_type) == 0 {
        NPC_Mark1_Precache(ctx);
    } else if Q_stricmp(c"mark2".as_ptr() as *const c_char, npc_type) == 0 {
        NPC_Mark2_Precache(ctx);
    } else if Q_stricmp(c"interrogator".as_ptr() as *const c_char, npc_type) == 0 {
        NPC_Interrogator_Precache(ctx, None);
    } else if Q_stricmp(c"probe".as_ptr() as *const c_char, npc_type) == 0 {
        NPC_Probe_Precache(ctx);
    } else if Q_stricmp(c"seeker".as_ptr() as *const c_char, npc_type) == 0 {
        NPC_Seeker_Precache(ctx);
    } else if Q_stricmp(c"remote".as_ptr() as *const c_char, npc_type) == 0 {
        NPC_Remote_Precache(ctx);
    } else if Q_strncmp(c"shadowtrooper".as_ptr() as *const c_char, npc_type, 13) == 0 {
        NPC_ShadowTrooper_Precache(ctx);
    } else if Q_stricmp(c"minemonster".as_ptr() as *const c_char, npc_type) == 0 {
        NPC_MineMonster_Precache(ctx);
    } else if Q_stricmp(c"howler".as_ptr() as *const c_char, npc_type) == 0 {
        NPC_Howler_Precache();
    } else if Q_stricmp(c"sentry".as_ptr() as *const c_char, npc_type) == 0 {
        NPC_Sentry_Precache(ctx);
    } else if Q_stricmp(c"protocol".as_ptr() as *const c_char, npc_type) == 0 {
        NPC_Protocol_Precache(ctx);
    } else if Q_stricmp(c"galak_mech".as_ptr() as *const c_char, npc_type) == 0 {
        NPC_GalakMech_Precache(ctx);
    } else if Q_stricmp(c"wampa".as_ptr() as *const c_char, npc_type) == 0 {
        NPC_Wampa_Precache(ctx);
    }

    NPC_Spawn_Do(ctx, ctx.entity_id_of(npc_spawner).unwrap())
}

/// Raven `NPC_Spawn_f`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:4020-4039`
pub fn NPC_Spawn_f(ctx: &mut GameContext, ent: EntityId) {
    let mut npc_type: [u8; 1024] = [0; 1024];
    let mut targetname: [u8; 1024] = [0; 1024];
    let mut is_vehicle = 0u32;

    trap::Argv(
        ctx.engine,
        mp_abi::game::syscalls::G_ARGV::GArgvArgs::new(
            2,
            npc_type.as_mut_ptr() as *mut c_char,
            1024,
        ),
    );

    if Q_stricmp(
        c"vehicle".as_ptr() as *const c_char,
        npc_type.as_ptr() as *const c_char,
    ) == 0
    {
        is_vehicle = 1;
        trap::Argv(
            ctx.engine,
            mp_abi::game::syscalls::G_ARGV::GArgvArgs::new(
                3,
                npc_type.as_mut_ptr() as *mut c_char,
                1024,
            ),
        );
        trap::Argv(
            ctx.engine,
            mp_abi::game::syscalls::G_ARGV::GArgvArgs::new(
                4,
                targetname.as_mut_ptr() as *mut c_char,
                1024,
            ),
        );
    } else {
        trap::Argv(
            ctx.engine,
            mp_abi::game::syscalls::G_ARGV::GArgvArgs::new(
                3,
                targetname.as_mut_ptr() as *mut c_char,
                1024,
            ),
        );
    }

    NPC_SpawnType(
        ctx,
        Some(ent),
        npc_type.as_mut_ptr() as *mut c_char,
        targetname.as_mut_ptr() as *mut c_char,
        (is_vehicle) as i32,
    );
}

/// Raven `NPC_Kill_f`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:4045-4170`
pub fn NPC_Kill_f(ctx: &mut GameContext) {
    // Raven `TeamNames[TEAM_NUM_TEAMS]` (NPC_stats.c:133), the NPC team_t names
    // (the many commented-out Trek-era entries collapse to these three).
    const TEAM_NAMES: [&str; TEAM_NUM_TEAMS as usize] = ["", "player", "enemy", "neutral"];
    let mut name: [u8; 1024] = [0; 1024];
    let mut kill_team: team_t = TEAM_FREE;
    let mut kill_non_sf = 0u32;

    trap::Argv(
        ctx.engine,
        mp_abi::game::syscalls::G_ARGV::GArgvArgs::new(2, name.as_mut_ptr() as *mut c_char, 1024),
    );

    if name[0] == b'\0' as u8 {
        Com_Printf(c"Error, Expected:\n".as_ptr() as *const c_char);
        Com_Printf(
            c"NPC kill '[NPC targetname]' - kills NPCs with certain targetname\n".as_ptr()
                as *const c_char,
        );
        Com_Printf(c"or\n".as_ptr() as *const c_char);
        Com_Printf(c"NPC kill 'all' - kills all NPCs\n".as_ptr() as *const c_char);
        Com_Printf(c"or\n".as_ptr() as *const c_char);
        Com_Printf(c"NPC team '[teamname]' - kills all NPCs of a certain team ('nonally' is all but your allies)\n".as_ptr() as *const c_char);
        return;
    }

    if Q_stricmp(
        c"team".as_ptr() as *const c_char,
        name.as_ptr() as *const c_char,
    ) == 0
    {
        trap::Argv(
            ctx.engine,
            mp_abi::game::syscalls::G_ARGV::GArgvArgs::new(
                3,
                name.as_mut_ptr() as *mut c_char,
                1024,
            ),
        );

        if name[0] == b'\0' as u8 {
            Com_Printf(
                c"NPC_Kill Error: 'npc kill team' requires a team name!\n".as_ptr()
                    as *const c_char,
            );
            Com_Printf(c"Valid team names are:\n".as_ptr() as *const c_char);
            for n in (TEAM_FREE + 1)..TEAM_NUM_TEAMS {
                // Raven `TeamNames[]` (NPC_stats.c:133) — the NPC team_t names.
                Com_Printf(cstr(&format!("{}\n", TEAM_NAMES[n as usize])).as_ptr());
            }
            Com_Printf(c"nonally - kills all but your teammates\n".as_ptr() as *const c_char);
            return;
        }

        if Q_stricmp(
            c"nonally".as_ptr() as *const c_char,
            name.as_ptr() as *const c_char,
        ) == 0
        {
            kill_non_sf = 1;
        } else {
            kill_team = GetIDForString(
                TeamTable.as_ptr() as *mut stringID_table_t,
                name.as_ptr() as *const c_char,
            ) as team_t;

            if kill_team == TEAM_FREE {
                Com_Printf(
                    cstr(&format!(
                        "NPC_Kill Error: team '{}' not recognized\n",
                        unsafe { cstr_to_str(name.as_ptr() as *const c_char) }
                    ))
                    .as_ptr(),
                );
                Com_Printf(c"Valid team names are:\n".as_ptr() as *const c_char);
                for n in (TEAM_FREE + 1)..TEAM_NUM_TEAMS {
                    // Raven `TeamNames[]` (NPC_stats.c:133) — the NPC team_t names.
                    Com_Printf(cstr(&format!("{}\n", TEAM_NAMES[n as usize])).as_ptr());
                }
                Com_Printf(c"nonally - kills all but your teammates\n".as_ptr() as *const c_char);
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
                    Com_Printf(
                        cstr(&format!(
                            "Killing NPC {} named {}\n",
                            unsafe { cstr_to_str(player.NPC_type as *const c_char) },
                            unsafe { cstr_to_str(player.targetname as *const c_char) }
                        ))
                        .as_ptr(),
                    );
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
            } else if !player.NPC_type.is_null() && !player.classname.is_null() {
                unsafe {
                    if (*player.classname) != b'\0' as c_char
                        && Q_stricmp(c"NPC_starfleet".as_ptr() as *const c_char, player.classname)
                            != 0
                    {
                        Com_Printf(
                            cstr(&format!(
                                "Removing NPC spawner {} with NPC named {}\n",
                                cstr_to_str(player.NPC_type as *const c_char),
                                cstr_to_str(player.NPC_targetname as *const c_char)
                            ))
                            .as_ptr(),
                        );
                        // STAGE-1: raw pointer cast ends the `player` borrow before
                        // re-entering `ctx` (Stage-2 debt).
                        let player_ptr = player as *mut gentity_t;
                        G_FreeEntity(ctx, ctx.entity_id_of(player_ptr));
                    }
                }
            }
        } else if !player.NPC.is_null() && !player.client.is_null() {
            if kill_team != TEAM_FREE {
                if unsafe { (*(player.client)).playerTeam } == kill_team {
                    Com_Printf(
                        cstr(&format!(
                            "Killing NPC {} named {}\n",
                            unsafe { cstr_to_str(player.NPC_type as *const c_char) },
                            unsafe { cstr_to_str(player.targetname as *const c_char) }
                        ))
                        .as_ptr(),
                    );
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
            } else if (!player.targetname.is_null()
                && Q_stricmp(name.as_ptr() as *const c_char, player.targetname) == 0)
                || Q_stricmp(
                    c"all".as_ptr() as *const c_char,
                    name.as_ptr() as *const c_char,
                ) == 0
            {
                Com_Printf(
                    cstr(&format!(
                        "Killing NPC {} named {}\n",
                        unsafe { cstr_to_str(player.NPC_type as *const c_char) },
                        unsafe { cstr_to_str(player.targetname as *const c_char) }
                    ))
                    .as_ptr(),
                );
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
    let targetname = ctx.world.entity(ent).targetname;
    // Pool client deref stays raw (recipe 2b): copied pointer value, tight unsafe.
    let client = ctx.world.entity(ent).client;
    let score = unsafe { (*client).ps.persistant[PERS_SCORE as usize] };
    Com_Printf(
        cstr(&format!(
            "{}: {}\n",
            unsafe { cstr_to_str(targetname as *const c_char) },
            score
        ))
        .as_ptr(),
    );
}

/// Raven `Cmd_NPC_f`.
///
/// Source: `oracle/codemp/game/NPC_spawn.c:4183-4243`
pub fn Cmd_NPC_f(ctx: &mut GameContext, ent: EntityId) {
    let mut cmd: [u8; 1024] = [0; 1024];

    trap::Argv(
        ctx.engine,
        mp_abi::game::syscalls::G_ARGV::GArgvArgs::new(1, cmd.as_mut_ptr() as *mut c_char, 1024),
    );

    if cmd[0] == b'\0' as u8 {
        Com_Printf(c"Valid NPC commands are:\n".as_ptr() as *const c_char);
        Com_Printf(c" spawn [NPC type (from NCPCs.cfg)]\n".as_ptr() as *const c_char);
        Com_Printf(
            c" kill [NPC targetname] or [all(kills all NPCs)] or 'team [teamname]'\n".as_ptr()
                as *const c_char,
        );
        Com_Printf(c" showbounds (draws exact bounding boxes of NPCs)\n".as_ptr() as *const c_char);
        Com_Printf(
            c" score [NPC targetname] (prints number of kills per NPC)\n".as_ptr() as *const c_char,
        );
    } else if Q_stricmp(
        c"spawn".as_ptr() as *const c_char,
        cmd.as_ptr() as *const c_char,
    ) == 0
    {
        NPC_Spawn_f(ctx, ent);
    } else if Q_stricmp(
        c"kill".as_ptr() as *const c_char,
        cmd.as_ptr() as *const c_char,
    ) == 0
    {
        NPC_Kill_f(ctx);
    } else if Q_stricmp(
        c"showbounds".as_ptr() as *const c_char,
        cmd.as_ptr() as *const c_char,
    ) == 0
    {
        ctx.world.globals.showBBoxes = if ctx.world.globals.showBBoxes != 0 {
            0
        } else {
            1
        };
    } else if Q_stricmp(
        c"score".as_ptr() as *const c_char,
        cmd.as_ptr() as *const c_char,
    ) == 0
    {
        let mut cmd2: [u8; 1024] = [0; 1024];
        trap::Argv(
            ctx.engine,
            mp_abi::game::syscalls::G_ARGV::GArgvArgs::new(
                2,
                cmd2.as_mut_ptr() as *mut c_char,
                1024,
            ),
        );

        if cmd2[0] == b'\0' as u8 {
            // Show the score for all NPCs
            Com_Printf(c"SCORE LIST:\n".as_ptr() as *const c_char);
            for i in 0..ENTITYNUM_WORLD as usize {
                let player = ctx.world.g_entities.get(i);
                if player.is_none() || player.unwrap().client.is_null() {
                    continue;
                }
                NPC_PrintScore(ctx, ctx.entity_id_of(player.unwrap()).unwrap());
            }
        } else {
            // Find specific NPC
            let found_ent = G_Find(
                ctx,
                ctx.entity_id_of(std::ptr::null_mut()),
                FOFS_targetname,
                cmd2.as_ptr() as *const c_char,
            );
            if !found_ent.is_null() && !unsafe { (*found_ent).client.is_null() } {
                NPC_PrintScore(ctx, ctx.entity_id_of(found_ent).unwrap());
            } else {
                Com_Printf(
                    cstr(&format!("ERROR: NPC score - no such NPC {}\n", unsafe {
                        cstr_to_str(cmd2.as_ptr() as *const c_char)
                    }))
                    .as_ptr(),
                );
            }
        }
    }
}
