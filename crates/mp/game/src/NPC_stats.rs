// PORT-COMPLETE: NPC_stats.c
//! FAITHFUL port of `oracle/codemp/game/NPC_stats.c`.
//!
//! Filled by the jampgame mega-pass; functions reach file-scope game state
//! (`level`, `g_entities`, cvars) and engine traps through the threaded
//! `GameContext`/`GameWorld` handle.
//!
//! Safe-state migration **Stage 1**: entity-pointer params are `EntityId` /
//! `Option<EntityId>` handles (§B5), not raw `gentity_t*`; ctx-free leaf helpers
//! take `&mut`/`&gentity_t`. Bodies re-derive the raw pointers verbatim at the
//! top (`// STAGE-1:` markers) — Stage-2 debt. Callers bridge at the boundary
//! via `ctx.entity_id_of(ptr)`.
#![allow(non_snake_case, unused, clippy::all)]

use crate::bg_channel::GameBgTraps;
use crate::q_shared::COM_BeginParseSession;
use crate::prelude::*;
use crate::g_utils::G_ModelIndex;
use crate::g_utils::G_SoundIndex;
use native_string::Q_stricmp;
use mp_bg::bg_saberLoad::qstricmp_eq;

// `DEFAULT_MINS_2`/`DEFAULT_MAXS_2` canonical in `mp_bg::public::viewheight`
// (`c_int`, cast here to match the `vec3_t` components they seed).
// Source: `oracle/codemp/game/bg_public.h:41-42`
const DEFAULT_MINS_2: f32 = mp_bg::public::viewheight::DEFAULT_MINS_2 as f32;
const DEFAULT_MAXS_2: f32 = mp_bg::public::viewheight::DEFAULT_MAXS_2 as f32;
// Raven `CROUCH_MAXS_2` (`bg_public.h`) — redeclared locally per the existing
// per-file-local-const convention (no canonical shared export).
const CROUCH_MAXS_2: f32 = 16.0;

/// Raven `MAX_NPC_DATA_SIZE` — maximum size for NPC stat loading buffer.
/// Source: `oracle/codemp/game/NPC_stats.c:236`
const MAX_NPC_DATA_SIZE: c_int = 0x20000;

/// Raven `BSTable` — `bState_t` name/id lookup table (internal-only bStates
/// past `BS_CINEMATIC` are not name-lookupable, per the oracle table).
///
/// Source: `oracle/codemp/game/NPC_stats.c:88-99`
pub static BSTable: [stringID_table_t; 11] = [
    stringID_table_t {
        name: c"BS_DEFAULT".as_ptr() as *mut c_char,
        id: bState_t::BS_DEFAULT as c_int,
    },
    stringID_table_t {
        name: c"BS_ADVANCE_FIGHT".as_ptr() as *mut c_char,
        id: bState_t::BS_ADVANCE_FIGHT as c_int,
    },
    stringID_table_t {
        name: c"BS_SLEEP".as_ptr() as *mut c_char,
        id: bState_t::BS_SLEEP as c_int,
    },
    stringID_table_t {
        name: c"BS_FOLLOW_LEADER".as_ptr() as *mut c_char,
        id: bState_t::BS_FOLLOW_LEADER as c_int,
    },
    stringID_table_t {
        name: c"BS_JUMP".as_ptr() as *mut c_char,
        id: bState_t::BS_JUMP as c_int,
    },
    stringID_table_t {
        name: c"BS_SEARCH".as_ptr() as *mut c_char,
        id: bState_t::BS_SEARCH as c_int,
    },
    stringID_table_t {
        name: c"BS_WANDER".as_ptr() as *mut c_char,
        id: bState_t::BS_WANDER as c_int,
    },
    stringID_table_t {
        name: c"BS_NOCLIP".as_ptr() as *mut c_char,
        id: bState_t::BS_NOCLIP as c_int,
    },
    stringID_table_t {
        name: c"BS_REMOVE".as_ptr() as *mut c_char,
        id: bState_t::BS_REMOVE as c_int,
    },
    stringID_table_t {
        name: c"BS_CINEMATIC".as_ptr() as *mut c_char,
        id: bState_t::BS_CINEMATIC as c_int,
    },
    stringID_table_t {
        name: c"".as_ptr() as *mut c_char,
        id: -1,
    },
];

/// Raven `TeamTable` — NPC-team name/id lookup.
///
/// Source: `oracle/codemp/game/NPC_stats.c:14-20`
pub static TeamTable: [stringID_table_t; 5] = [
    stringID_table_t {
        name: c"NPCTEAM_FREE".as_ptr() as *mut c_char,
        id: NPCTEAM_FREE as c_int,
    },
    stringID_table_t {
        name: c"NPCTEAM_PLAYER".as_ptr() as *mut c_char,
        id: NPCTEAM_PLAYER as c_int,
    },
    stringID_table_t {
        name: c"NPCTEAM_ENEMY".as_ptr() as *mut c_char,
        id: NPCTEAM_ENEMY as c_int,
    },
    stringID_table_t {
        name: c"NPCTEAM_NEUTRAL".as_ptr() as *mut c_char,
        id: NPCTEAM_NEUTRAL as c_int,
    },
    stringID_table_t {
        name: c"".as_ptr() as *mut c_char,
        id: -1,
    }, // terminator: Raven's `"", -1`
];

/// Raven `ClassTable` — NPC-class name/id lookup (order must match the `class_t`
/// enum in `teams.h`).
///
/// Source: `oracle/codemp/game/NPC_stats.c:23-86`
pub static ClassTable: [stringID_table_t; 57] = [
    stringID_table_t {
        name: c"CLASS_NONE".as_ptr() as *mut c_char,
        id: CLASS_NONE as c_int,
    },
    stringID_table_t {
        name: c"CLASS_ATST".as_ptr() as *mut c_char,
        id: CLASS_ATST as c_int,
    },
    stringID_table_t {
        name: c"CLASS_BARTENDER".as_ptr() as *mut c_char,
        id: CLASS_BARTENDER as c_int,
    },
    stringID_table_t {
        name: c"CLASS_BESPIN_COP".as_ptr() as *mut c_char,
        id: CLASS_BESPIN_COP as c_int,
    },
    stringID_table_t {
        name: c"CLASS_CLAW".as_ptr() as *mut c_char,
        id: CLASS_CLAW as c_int,
    },
    stringID_table_t {
        name: c"CLASS_COMMANDO".as_ptr() as *mut c_char,
        id: CLASS_COMMANDO as c_int,
    },
    stringID_table_t {
        name: c"CLASS_DESANN".as_ptr() as *mut c_char,
        id: CLASS_DESANN as c_int,
    },
    stringID_table_t {
        name: c"CLASS_FISH".as_ptr() as *mut c_char,
        id: CLASS_FISH as c_int,
    },
    stringID_table_t {
        name: c"CLASS_FLIER2".as_ptr() as *mut c_char,
        id: CLASS_FLIER2 as c_int,
    },
    stringID_table_t {
        name: c"CLASS_GALAK".as_ptr() as *mut c_char,
        id: CLASS_GALAK as c_int,
    },
    stringID_table_t {
        name: c"CLASS_GLIDER".as_ptr() as *mut c_char,
        id: CLASS_GLIDER as c_int,
    },
    stringID_table_t {
        name: c"CLASS_GONK".as_ptr() as *mut c_char,
        id: CLASS_GONK as c_int,
    },
    stringID_table_t {
        name: c"CLASS_GRAN".as_ptr() as *mut c_char,
        id: CLASS_GRAN as c_int,
    },
    stringID_table_t {
        name: c"CLASS_HOWLER".as_ptr() as *mut c_char,
        id: CLASS_HOWLER as c_int,
    },
    // ENUM2STRING(CLASS_RANCOR) — commented out in the oracle here (line 40); the
    // active CLASS_RANCOR entry appears near the end (line 83).
    stringID_table_t {
        name: c"CLASS_IMPERIAL".as_ptr() as *mut c_char,
        id: CLASS_IMPERIAL as c_int,
    },
    stringID_table_t {
        name: c"CLASS_IMPWORKER".as_ptr() as *mut c_char,
        id: CLASS_IMPWORKER as c_int,
    },
    stringID_table_t {
        name: c"CLASS_INTERROGATOR".as_ptr() as *mut c_char,
        id: CLASS_INTERROGATOR as c_int,
    },
    stringID_table_t {
        name: c"CLASS_JAN".as_ptr() as *mut c_char,
        id: CLASS_JAN as c_int,
    },
    stringID_table_t {
        name: c"CLASS_JEDI".as_ptr() as *mut c_char,
        id: CLASS_JEDI as c_int,
    },
    stringID_table_t {
        name: c"CLASS_KYLE".as_ptr() as *mut c_char,
        id: CLASS_KYLE as c_int,
    },
    stringID_table_t {
        name: c"CLASS_LANDO".as_ptr() as *mut c_char,
        id: CLASS_LANDO as c_int,
    },
    stringID_table_t {
        name: c"CLASS_LIZARD".as_ptr() as *mut c_char,
        id: CLASS_LIZARD as c_int,
    },
    stringID_table_t {
        name: c"CLASS_LUKE".as_ptr() as *mut c_char,
        id: CLASS_LUKE as c_int,
    },
    stringID_table_t {
        name: c"CLASS_MARK1".as_ptr() as *mut c_char,
        id: CLASS_MARK1 as c_int,
    },
    stringID_table_t {
        name: c"CLASS_MARK2".as_ptr() as *mut c_char,
        id: CLASS_MARK2 as c_int,
    },
    stringID_table_t {
        name: c"CLASS_GALAKMECH".as_ptr() as *mut c_char,
        id: CLASS_GALAKMECH as c_int,
    },
    stringID_table_t {
        name: c"CLASS_MINEMONSTER".as_ptr() as *mut c_char,
        id: CLASS_MINEMONSTER as c_int,
    },
    stringID_table_t {
        name: c"CLASS_MONMOTHA".as_ptr() as *mut c_char,
        id: CLASS_MONMOTHA as c_int,
    },
    stringID_table_t {
        name: c"CLASS_MORGANKATARN".as_ptr() as *mut c_char,
        id: CLASS_MORGANKATARN as c_int,
    },
    stringID_table_t {
        name: c"CLASS_MOUSE".as_ptr() as *mut c_char,
        id: CLASS_MOUSE as c_int,
    },
    stringID_table_t {
        name: c"CLASS_MURJJ".as_ptr() as *mut c_char,
        id: CLASS_MURJJ as c_int,
    },
    stringID_table_t {
        name: c"CLASS_PRISONER".as_ptr() as *mut c_char,
        id: CLASS_PRISONER as c_int,
    },
    stringID_table_t {
        name: c"CLASS_PROBE".as_ptr() as *mut c_char,
        id: CLASS_PROBE as c_int,
    },
    stringID_table_t {
        name: c"CLASS_PROTOCOL".as_ptr() as *mut c_char,
        id: CLASS_PROTOCOL as c_int,
    },
    stringID_table_t {
        name: c"CLASS_R2D2".as_ptr() as *mut c_char,
        id: CLASS_R2D2 as c_int,
    },
    stringID_table_t {
        name: c"CLASS_R5D2".as_ptr() as *mut c_char,
        id: CLASS_R5D2 as c_int,
    },
    stringID_table_t {
        name: c"CLASS_REBEL".as_ptr() as *mut c_char,
        id: CLASS_REBEL as c_int,
    },
    stringID_table_t {
        name: c"CLASS_REBORN".as_ptr() as *mut c_char,
        id: CLASS_REBORN as c_int,
    },
    stringID_table_t {
        name: c"CLASS_REELO".as_ptr() as *mut c_char,
        id: CLASS_REELO as c_int,
    },
    stringID_table_t {
        name: c"CLASS_REMOTE".as_ptr() as *mut c_char,
        id: CLASS_REMOTE as c_int,
    },
    stringID_table_t {
        name: c"CLASS_RODIAN".as_ptr() as *mut c_char,
        id: CLASS_RODIAN as c_int,
    },
    stringID_table_t {
        name: c"CLASS_SEEKER".as_ptr() as *mut c_char,
        id: CLASS_SEEKER as c_int,
    },
    stringID_table_t {
        name: c"CLASS_SENTRY".as_ptr() as *mut c_char,
        id: CLASS_SENTRY as c_int,
    },
    stringID_table_t {
        name: c"CLASS_SHADOWTROOPER".as_ptr() as *mut c_char,
        id: CLASS_SHADOWTROOPER as c_int,
    },
    stringID_table_t {
        name: c"CLASS_STORMTROOPER".as_ptr() as *mut c_char,
        id: CLASS_STORMTROOPER as c_int,
    },
    stringID_table_t {
        name: c"CLASS_SWAMP".as_ptr() as *mut c_char,
        id: CLASS_SWAMP as c_int,
    },
    stringID_table_t {
        name: c"CLASS_SWAMPTROOPER".as_ptr() as *mut c_char,
        id: CLASS_SWAMPTROOPER as c_int,
    },
    stringID_table_t {
        name: c"CLASS_TAVION".as_ptr() as *mut c_char,
        id: CLASS_TAVION as c_int,
    },
    stringID_table_t {
        name: c"CLASS_TRANDOSHAN".as_ptr() as *mut c_char,
        id: CLASS_TRANDOSHAN as c_int,
    },
    stringID_table_t {
        name: c"CLASS_UGNAUGHT".as_ptr() as *mut c_char,
        id: CLASS_UGNAUGHT as c_int,
    },
    stringID_table_t {
        name: c"CLASS_JAWA".as_ptr() as *mut c_char,
        id: CLASS_JAWA as c_int,
    },
    stringID_table_t {
        name: c"CLASS_WEEQUAY".as_ptr() as *mut c_char,
        id: CLASS_WEEQUAY as c_int,
    },
    stringID_table_t {
        name: c"CLASS_BOBAFETT".as_ptr() as *mut c_char,
        id: CLASS_BOBAFETT as c_int,
    },
    // ENUM2STRING(CLASS_ROCKETTROOPER) — commented out in the oracle (line 80).
    // ENUM2STRING(CLASS_PLAYER) — commented out in the oracle (line 81).
    stringID_table_t {
        name: c"CLASS_VEHICLE".as_ptr() as *mut c_char,
        id: CLASS_VEHICLE as c_int,
    },
    stringID_table_t {
        name: c"CLASS_RANCOR".as_ptr() as *mut c_char,
        id: CLASS_RANCOR as c_int,
    },
    stringID_table_t {
        name: c"CLASS_WAMPA".as_ptr() as *mut c_char,
        id: CLASS_WAMPA as c_int,
    },
    stringID_table_t {
        name: c"".as_ptr() as *mut c_char,
        id: -1,
    }, // terminator: Raven's `"", -1`
];

/// Raven `NPC_ReactionTime`.
///
/// Source: `oracle/codemp/game/NPC_stats.c:220-223`
pub fn NPC_ReactionTime(ctx: &mut GameContext) -> c_int {
    // `NPCInfo` (`NPC.c:34`) is a real `*mut gNPC_t` field on `GameGlobals` now
    // (pass-2 backfill) — deref straight through.
    unsafe { 200 * (6 - (*ctx.world.globals.NPCInfo).stats.reactions) }
}

/// Raven `TranslateRankName`.
///
/// Raven: `Should be used to determine pip bolt-ons` (see the commented-out
/// `TranslateRankName` doc block above the live definition).
/// Source: `oracle/codemp/game/NPC_stats.c:287-330`
pub fn TranslateRankName(name: *const c_char) -> rank_t {
    unsafe {
        if qstricmp_eq(name, c"civilian") {
            return RANK_CIVILIAN;
        }
        if qstricmp_eq(name, c"crewman") {
            return RANK_CREWMAN;
        }
        if qstricmp_eq(name, c"ensign") {
            return RANK_ENSIGN;
        }
        if qstricmp_eq(name, c"ltjg") {
            return RANK_LT_JG;
        }
        if qstricmp_eq(name, c"lt") {
            return RANK_LT;
        }
        if qstricmp_eq(name, c"ltcomm") {
            return RANK_LT_COMM;
        }
        if qstricmp_eq(name, c"commander") {
            return RANK_COMMANDER;
        }
        if qstricmp_eq(name, c"captain") {
            return RANK_CAPTAIN;
        }
    }
    RANK_CIVILIAN
}

/// Raven `G_ParseAnimFileSet`.
///
/// Source: `oracle/codemp/game/NPC_stats.c:424-437`
pub fn G_ParseAnimFileSet(
    ctx: &mut GameContext,
    filename: *const c_char,
    animCFG: *const c_char,
    animFileIndex: *mut c_int,
) -> qboolean {
    // Raven: `*animFileIndex = BG_ParseAnimationFile(filename, NULL, qfalse);`
    // "if it's humanoid we should have it cached and return it, if it is not
    // it will be loaded (unless it's also cached already)". `animCFG` is
    // unused by the live body (matches oracle — it is only forwarded by the
    // (disabled) caller, not read here).
    let _ = animCFG;

    unsafe {
        let traps = crate::bg_channel::GameBgTraps::new(ctx.engine);
        let mut callbacks = crate::bg_channel::GameCallbacksImpl {
            // STAGE-2b: irreducible — `GameCallbacksImpl.world` is a `*mut GameWorld` bg-seam field; a raw store is required.
            world: ctx.world_raw(),
            engine: ctx.engine,
        };
        *animFileIndex = mp_bg::bg_panimate::BG_ParseAnimationFile(
            &mut ctx.world.bg_state,
            &traps,
            &mut callbacks,
            filename,
            std::ptr::null_mut(),
            qfalse,
        );
        if *animFileIndex == -1 {
            return 0; // qfalse
        }
    }
    // "I guess this isn't really even needed game-side." — BG_ParseAnimationSndFile
    // call stays commented out per oracle.
    1 // qtrue
}

/// Raven `NPC_PrecacheAnimationCFG`.
///
/// Raven: entire body is compiled out (`#if 0 //rwwFIXMEFIXME: Actually
/// precache stuff here.` ... `#endif`) — the live function is a no-op.
/// Source: `oracle/codemp/game/NPC_stats.c:439-548`
pub fn NPC_PrecacheAnimationCFG(NPC_type: &str) {
    let _ = NPC_type;
    // Deliberate no-op: matches the oracle's `#if 0`-disabled body verbatim.
}

/// Raven `NPC_PrecacheWeapons`.
///
/// Source: `oracle/codemp/game/NPC_stats.c:551-591`
pub fn NPC_PrecacheWeapons(
    ctx: &mut GameContext,
    playerTeam: team_t,
    spawnflags: c_int,
    NPCtype: &str,
) {
    use mp_bg::weapons::weapon_t::{WP_NUM_WEAPONS, WP_SABER};

    let weapons = crate::NPC_spawn::NPC_WeaponsForTeam(playerTeam, spawnflags, NPCtype);

    let mut curWeap = WP_SABER;
    while curWeap < WP_NUM_WEAPONS {
        if weapons & (1 << curWeap) != 0 {
            let item = mp_bg::bg_misc::BG_FindItemForWeapon(curWeap);
            crate::g_items::RegisterItem(ctx, item);
        }
        curWeap += 1;
    }
    // The `#if 0 //rwwFIXMEFIXME: actually precache weapons here` block (the
    // ghoul2 in-hand/in-world weapon-model precache) is dead in the oracle —
    // dropped per porting-rules §17/§20 (dead disabled surface, not ported
    // speculatively).
}

/// Raven `NPC_Precache`.
///
/// Source: `oracle/codemp/game/NPC_stats.c:599-873`
pub fn NPC_Precache(ctx: &mut GameContext, spawner: EntityId) {
    use mp_bg::weapons::weapon_t::{weapon_t, WP_NONE, WP_NUM_WEAPONS};

    unsafe {
        // `spawner` stays an `EntityId`; entity fields go through
        // `ctx.world.entity(spawner)` accessor borrows (2c). Its pool `client`
        // (`gClPtrs`) is read via the safe borrow, then dereffed raw.
        let mut player_team: team_t = NPCTEAM_FREE;
        let mut md3_model: qboolean = 0; // qfalse
        let mut custom_skin: [c_char; MAX_QPATH as usize] = [0; MAX_QPATH as usize];
        let mut sound: [c_char; MAX_QPATH as usize] = [0; MAX_QPATH as usize];
        let mut playerModel: [c_char; MAX_QPATH as usize] = [0; MAX_QPATH as usize];

        if Q_stricmp("random", ctx.world.entity(spawner).NPC_type.as_deref().unwrap_or("")) == 0 {
            //sorry, can't precache a random just yet
            return;
        }
        write_cstr_field(&mut custom_skin, "default");

        let npc_parms: *const c_char = (&ctx.world.globals.NPCParms) as *const _ as *const c_char;
        let mut p: *const c_char = npc_parms;
        COM_BeginParseSession(&mut ctx.world.bg_state.qs, &ctx.world.globals.NPCFile);

        // look for the right NPC
        loop {
            if p.is_null() {
                break;
            }
            let token = crate::q_shared::COM_ParseExt(
                &mut ctx.world.bg_state.qs,
                &mut p as *mut *const c_char,
                1,
            );
            if *token == 0 {
                return;
            }
            if Q_stricmp(&cstr_to_str(token), ctx.world.entity(spawner).NPC_type.as_deref().unwrap_or("")) == 0 {
                break;
            }
            crate::q_shared::SkipBracedSection(
                &mut ctx.world.bg_state.qs,
                &mut p as *mut *const c_char,
            );
        }

        if p.is_null() {
            return;
        }

        if mp_bg::bg_saberLoad::BG_ParseLiteral(
            &mut ctx.world.bg_state.qs,
            &mut p as *mut *const c_char,
            cstr("{").as_ptr(),
            &crate::bg_channel::GameBgTraps::new(ctx.engine),
        ) != 0
        {
            return;
        }

        // parse the NPC info block
        loop {
            let token = crate::q_shared::COM_ParseExt(
                &mut ctx.world.bg_state.qs,
                &mut p as *mut *const c_char,
                1,
            );
            if *token == 0 {
                let msg = format!(
                    "ERROR: unexpected EOF while parsing '{}'\n",
                    ctx.world.entity(spawner).NPC_type.as_deref().unwrap_or("")
                );
                crate::g_main::Com_Printf(&msg);
                return;
            }

            if qstricmp_eq(token, c"}") {
                break;
            }

            // headmodel
            if qstricmp_eq(token, c"headmodel") {
                let mut value: *const c_char = std::ptr::null();
                if crate::q_shared::COM_ParseString(
                    &mut ctx.world.bg_state.qs,
                    &mut p as *mut *const c_char,
                    &mut value,
                ) != 0
                {
                    continue;
                }
                if Q_stricmp("none", &cstr_to_str(value)) == 0 {
                    // (nothing — headModelName not wired yet, matches oracle's
                    // commented-out Q_strncpyz)
                }
                md3_model = 1;
                continue;
            }

            // torsomodel
            if qstricmp_eq(token, c"torsomodel") {
                let mut value: *const c_char = std::ptr::null();
                if crate::q_shared::COM_ParseString(
                    &mut ctx.world.bg_state.qs,
                    &mut p as *mut *const c_char,
                    &mut value,
                ) != 0
                {
                    continue;
                }
                if Q_stricmp("none", &cstr_to_str(value)) == 0 {
                    // (nothing — torsoModelName not wired yet)
                }
                md3_model = 1;
                continue;
            }

            // legsmodel
            if qstricmp_eq(token, c"legsmodel") {
                let mut value: *const c_char = std::ptr::null();
                if crate::q_shared::COM_ParseString(
                    &mut ctx.world.bg_state.qs,
                    &mut p as *mut *const c_char,
                    &mut value,
                ) != 0
                {
                    continue;
                }
                md3_model = 1;
                continue;
            }

            // playerModel
            if qstricmp_eq(token, c"playerModel") {
                let mut value: *const c_char = std::ptr::null();
                if crate::q_shared::COM_ParseString(
                    &mut ctx.world.bg_state.qs,
                    &mut p as *mut *const c_char,
                    &mut value,
                ) != 0
                {
                    continue;
                }
                crate::q_shared::Q_strncpyz(
                    playerModel.as_mut_ptr(),
                    value,
                    playerModel.len() as c_int,
                );
                md3_model = 0;
                continue;
            }

            // customSkin
            if qstricmp_eq(token, c"customSkin") {
                let mut value: *const c_char = std::ptr::null();
                if crate::q_shared::COM_ParseString(
                    &mut ctx.world.bg_state.qs,
                    &mut p as *mut *const c_char,
                    &mut value,
                ) != 0
                {
                    continue;
                }
                crate::q_shared::Q_strncpyz(
                    custom_skin.as_mut_ptr(),
                    value,
                    custom_skin.len() as c_int,
                );
                continue;
            }

            // playerTeam
            if qstricmp_eq(token, c"playerTeam") {
                let mut value: *const c_char = std::ptr::null();
                if crate::q_shared::COM_ParseString(
                    &mut ctx.world.bg_state.qs,
                    &mut p as *mut *const c_char,
                    &mut value,
                ) != 0
                {
                    continue;
                }
                // Raven bug (transcribed faithfully): sprintf's from `token`
                // (still "playerTeam"), not the just-parsed `value`.
                let tk = format!("NPC{}", cstr_to_str(token));
                let tk_c = cstr(&tk);
                player_team = crate::q_shared::GetIDForString(
                    TeamTable.as_ptr() as *mut stringID_table_t,
                    tk_c.as_ptr(),
                );
                continue;
            }

            // snd
            if qstricmp_eq(token, c"snd") {
                let mut value: *const c_char = std::ptr::null();
                if crate::q_shared::COM_ParseString(
                    &mut ctx.world.bg_state.qs,
                    &mut p as *mut *const c_char,
                    &mut value,
                ) != 0
                {
                    continue;
                }
                if ctx.world.entity(spawner).r.svFlags & SVF_NO_BASIC_SOUNDS == 0 {
                    crate::q_shared::Q_strncpyz(sound.as_mut_ptr(), value, sound.len() as c_int);
                    let sound_s = cstr_to_str(sound.as_ptr());
                    let trimmed = sound_s.split('/').next().unwrap_or(&sound_s);
                    let idx_s = format!("*${}", trimmed);
                    ctx.world.entity_mut(spawner).s.csSounds_Std =
                        G_SoundIndex(&idx_s);
                }
                continue;
            }

            // sndcombat
            if qstricmp_eq(token, c"sndcombat") {
                let mut value: *const c_char = std::ptr::null();
                if crate::q_shared::COM_ParseString(
                    &mut ctx.world.bg_state.qs,
                    &mut p as *mut *const c_char,
                    &mut value,
                ) != 0
                {
                    continue;
                }
                if ctx.world.entity(spawner).r.svFlags & SVF_NO_COMBAT_SOUNDS == 0 {
                    crate::q_shared::Q_strncpyz(sound.as_mut_ptr(), value, sound.len() as c_int);
                    let sound_s = cstr_to_str(sound.as_ptr());
                    let trimmed = sound_s.split('/').next().unwrap_or(&sound_s);
                    let idx_s = format!("*${}", trimmed);
                    ctx.world.entity_mut(spawner).s.csSounds_Combat =
                        G_SoundIndex(&idx_s);
                }
                continue;
            }

            // sndextra
            if qstricmp_eq(token, c"sndextra") {
                let mut value: *const c_char = std::ptr::null();
                if crate::q_shared::COM_ParseString(
                    &mut ctx.world.bg_state.qs,
                    &mut p as *mut *const c_char,
                    &mut value,
                ) != 0
                {
                    continue;
                }
                if ctx.world.entity(spawner).r.svFlags & SVF_NO_EXTRA_SOUNDS == 0 {
                    crate::q_shared::Q_strncpyz(sound.as_mut_ptr(), value, sound.len() as c_int);
                    let sound_s = cstr_to_str(sound.as_ptr());
                    let trimmed = sound_s.split('/').next().unwrap_or(&sound_s);
                    let idx_s = format!("*${}", trimmed);
                    ctx.world.entity_mut(spawner).s.csSounds_Extra =
                        G_SoundIndex(&idx_s);
                }
                continue;
            }

            // sndjedi
            if qstricmp_eq(token, c"sndjedi") {
                let mut value: *const c_char = std::ptr::null();
                if crate::q_shared::COM_ParseString(
                    &mut ctx.world.bg_state.qs,
                    &mut p as *mut *const c_char,
                    &mut value,
                ) != 0
                {
                    continue;
                }
                if ctx.world.entity(spawner).r.svFlags & SVF_NO_EXTRA_SOUNDS == 0 {
                    crate::q_shared::Q_strncpyz(sound.as_mut_ptr(), value, sound.len() as c_int);
                    let sound_s = cstr_to_str(sound.as_ptr());
                    let trimmed = sound_s.split('/').next().unwrap_or(&sound_s);
                    let idx_s = format!("*${}", trimmed);
                    ctx.world.entity_mut(spawner).s.csSounds_Jedi =
                        G_SoundIndex(&idx_s);
                }
                continue;
            }

            if qstricmp_eq(token, c"weapon") {
                let mut value: *const c_char = std::ptr::null();
                if crate::q_shared::COM_ParseString(
                    &mut ctx.world.bg_state.qs,
                    &mut p as *mut *const c_char,
                    &mut value,
                ) != 0
                {
                    continue;
                }
                let cur_weap = crate::q_shared::GetIDForString(
                    mp_bg::bg_saga::WPTable.as_ptr() as *mut stringID_table_t,
                    value,
                );
                if cur_weap > WP_NONE && cur_weap < WP_NUM_WEAPONS {
                    crate::g_items::RegisterItem(
                        ctx,
                        mp_bg::bg_misc::BG_FindItemForWeapon(cur_weap as weapon_t),
                    );
                }
                continue;
            }

            // (unrecognized token inside this block falls through — the
            // oracle loop has no `else` catch-all here, matching its while(1),
            // so an unrecognized token is ignored and the loop continues,
            // rather than exiting early)
            continue;
        }

        // If we're not a vehicle, then an error here would be valid...
        // FLAG: `spawner.client` is a `BG_Alloc`'d pool `gClPtrs` client
        // (`g_utils.c:430`), not a `level.clients` slot; read the pointer via the
        // safe borrow, deref raw (recipe 2b/2c).
        let client_ptr = ctx.world.entity(spawner).client;
        if client_ptr.is_null() || (*client_ptr).NPC_class != CLASS_VEHICLE {
            if md3_model != 0 {
                crate::g_main::Com_Printf(
                    "MD3 model using NPCs are not supported in MP\n",
                );
            } else {
                //if we have a model/skin then index them so they'll be registered immediately
                //when the client gets a configstring update.
                let mut model_name = format!(
                    "models/players/{}/model.glm",
                    cstr_to_str(playerModel.as_ptr())
                );
                if custom_skin[0] != 0 {
                    //append it after a *
                    model_name.push_str(&format!("*{}", cstr_to_str(custom_skin.as_ptr())));
                }
                G_ModelIndex(&model_name);
            }
        }

        //precache this NPC's possible weapons
        let spawner_spawnflags = ctx.world.entity(spawner).spawnflags;
        let spawner_npc_type = ctx.world.entity(spawner).NPC_type.clone();
        crate::NPC_stats::NPC_PrecacheWeapons(
            ctx,
            player_team,
            spawner_spawnflags,
            spawner_npc_type.as_deref().unwrap_or(""),
        );

        //	CG_RegisterNPCCustomSounds( &ci );
        //	CG_RegisterNPCEffects( playerTeam );
        //rwwFIXMEFIXME: same
        //FIXME: Look for a "sounds" directory and precache death, pain, alert sounds
    }
}

/// Raven `NPC_ParseParms`.
///
/// Source: `oracle/codemp/game/NPC_stats.c:974-3233`
pub fn NPC_ParseParms(ctx: &mut GameContext, NPCName_in: &str, NPC: EntityId) -> qboolean {
    use crate::client::render_info::renderInfo_t;
    use crate::npc::g_npcstats_e::gNPCstats_t;
    use mp_bg::weapons::weapon_t::{weapon_t, WP_NONE, WP_NUM_WEAPONS};

    unsafe {
        // `NPC` stays an `EntityId`; entity fields go through
        // `ctx.world.entity(NPC)` accessor borrows (2c).
        let mut NPCName = NPCName_in;

        let mut sound: [c_char; MAX_QPATH] = [0; MAX_QPATH];
        let mut playerModel: [c_char; MAX_QPATH] = [0; MAX_QPATH];
        let mut customSkin: [c_char; MAX_QPATH] = [0; MAX_QPATH];
        // FLAG: `NPC.client`/`NPC.NPC` are `BG_Alloc`'d pool `gClPtrs`/`gNPC_t`
        // (`g_utils.c:430`), not `level.clients`/accessor-backed; read the raw
        // pointer values via the safe borrow, deref raw below (recipe 2b/2c).
        let client_ptr = ctx.world.entity(NPC).client;
        let ri: *mut renderInfo_t = &mut (*client_ptr).renderInfo as *mut renderInfo_t;
        let mut stats: *mut gNPCstats_t = std::ptr::null_mut();
        let mut md3Model: qboolean = 1; // qtrue
        let mut surfOff: [c_char; 1024] = [0; 1024];
        let mut surfOn: [c_char; 1024] = [0; 1024];
        let parsingPlayer: qboolean =
            if ctx.world.entity(NPC).s.number == 0 && !client_ptr.is_null() {
                1
            } else {
                0
            };
        let mut localPlayerMins: vec3_t = [-15.0, -15.0, DEFAULT_MINS_2];
        let mut localPlayerMaxs: vec3_t = [15.0, 15.0, DEFAULT_MAXS_2];
        let mut npcSaber1: c_int = 0;
        let mut npcSaber2: c_int = 0;

        write_cstr_field(&mut customSkin, "default");
        if NPCName.is_empty() {
            NPCName = "Player";
        }

        let npc_ptr = ctx.world.entity(NPC).NPC;
        if !npc_ptr.is_null() {
            stats = &mut (*npc_ptr).stats as *mut gNPCstats_t;
            // fill in defaults
            (*stats).aggression = 3;
            (*stats).aim = 3;
            (*stats).earshot = 1024.0;
            (*stats).evasion = 3;
            (*stats).hfov = 90;
            (*stats).intelligence = 3;
            (*stats).r#move = 3;
            (*stats).reactions = 3;
            (*stats).vfov = 60;
            (*stats).vigilance = 0.1f32;
            (*stats).visrange = 1024.0;

            (*stats).health = 0;

            (*stats).yawSpeed = 90.0;
            (*stats).walkSpeed = 90;
            (*stats).runSpeed = 300;
            (*stats).acceleration = 15; //Increase/descrease speed this much per frame (20fps)
        } else {
            stats = std::ptr::null_mut();
        }

        //Set defaults
        //FIXME: should probably put default torso and head models, but what about enemies
        //that don't have any- like Stasis?
        //Q_strncpyz( ri->headModelName,	DEFAULT_HEADMODEL,  sizeof(ri->headModelName),	qtrue);
        //Q_strncpyz( ri->torsoModelName, DEFAULT_TORSOMODEL, sizeof(ri->torsoModelName),	qtrue);
        //Q_strncpyz( ri->legsModelName,	DEFAULT_LEGSMODEL,  sizeof(ri->legsModelName),	qtrue);
        surfOff = [0; 1024];
        surfOn = [0; 1024];

        (*ri).headYawRangeLeft = 80;
        (*ri).headYawRangeRight = 80;
        (*ri).headPitchRangeUp = 45;
        (*ri).headPitchRangeDown = 45;
        (*ri).torsoYawRangeLeft = 60;
        (*ri).torsoYawRangeRight = 60;
        (*ri).torsoPitchRangeUp = 30;
        (*ri).torsoPitchRangeDown = 50;

        {
            let e = ctx.world.entity_mut(NPC);
            e.r.mins = localPlayerMins;
            e.r.maxs = localPlayerMaxs;
        }
        (*client_ptr).ps.crouchheight = CROUCH_MAXS_2 as c_int;
        (*client_ptr).ps.standheight = DEFAULT_MAXS_2 as c_int;

        (*client_ptr).ps.customRGBA[0] = 255;
        (*client_ptr).ps.customRGBA[1] = 255;
        (*client_ptr).ps.customRGBA[2] = 255;
        (*client_ptr).ps.customRGBA[3] = 255;

        if Q_stricmp("random", NPCName) == 0 {
            //Randomly assemble a starfleet guy
            //NPC_BuildRandom( NPC );
            crate::g_main::Com_Printf("RANDOM NPC NOT SUPPORTED IN MP\n");
            return 0; // qfalse
        }

        let npc_parms: *const c_char = (&ctx.world.globals.NPCParms) as *const _ as *const c_char;
        let mut p: *const c_char = npc_parms;
        COM_BeginParseSession(&mut ctx.world.bg_state.qs, &ctx.world.globals.NPCFile);

        // look for the right NPC
        loop {
            if p.is_null() {
                return 0;
            }
            let token = crate::q_shared::COM_ParseExt(
                &mut ctx.world.bg_state.qs,
                &mut p as *mut *const c_char,
                1,
            );
            if *token == 0 {
                return 0;
            }
            if Q_stricmp(&cstr_to_str(token), NPCName) == 0 {
                break;
            }
            crate::q_shared::SkipBracedSection(
                &mut ctx.world.bg_state.qs,
                &mut p as *mut *const c_char,
            );
        }
        if p.is_null() {
            return 0;
        }

        if mp_bg::bg_saberLoad::BG_ParseLiteral(
            &mut ctx.world.bg_state.qs,
            &mut p as *mut *const c_char,
            cstr("{").as_ptr(),
            &crate::bg_channel::GameBgTraps::new(ctx.engine),
        ) != 0
        {
            return 0;
        }

        // parse the NPC info block
        'parse: loop {
            let token = crate::q_shared::COM_ParseExt(
                &mut ctx.world.bg_state.qs,
                &mut p as *mut *const c_char,
                1,
            );
            if *token == 0 {
                let msg = format!(
                    "ERROR: unexpected EOF while parsing '{}'\n",
                    NPCName
                );
                crate::g_main::Com_Printf(&msg);
                return 0;
            }

            if qstricmp_eq(token, c"}") {
                break;
            }

            //===MODEL PROPERTIES===========================================================
            // custom color
            if qstricmp_eq(token, c"customRGBA") {
                let mut value: *const c_char = std::ptr::null();
                if crate::q_shared::COM_ParseString(
                    &mut ctx.world.bg_state.qs,
                    &mut p as *mut *const c_char,
                    &mut value,
                ) != 0
                {
                    continue;
                }
                if Q_stricmp(&cstr_to_str(value), "random") == 0 {
                    (*client_ptr).ps.customRGBA[0] = ctx.world.bg_state.rng.Q_irand(0, 255);
                    (*client_ptr).ps.customRGBA[1] = ctx.world.bg_state.rng.Q_irand(0, 255);
                    (*client_ptr).ps.customRGBA[2] = ctx.world.bg_state.rng.Q_irand(0, 255);
                    (*client_ptr).ps.customRGBA[3] = 255;
                } else {
                    (*client_ptr).ps.customRGBA[0] = atoi(value);

                    let mut n0: c_int = 0;
                    if crate::q_shared::COM_ParseInt(
                        &mut ctx.world.bg_state.qs,
                        &mut p as *mut *const c_char,
                        &mut n0,
                    ) != 0
                    {
                        continue;
                    }
                    (*client_ptr).ps.customRGBA[1] = n0;

                    if crate::q_shared::COM_ParseInt(
                        &mut ctx.world.bg_state.qs,
                        &mut p as *mut *const c_char,
                        &mut n0,
                    ) != 0
                    {
                        continue;
                    }
                    (*client_ptr).ps.customRGBA[2] = n0;

                    if crate::q_shared::COM_ParseInt(
                        &mut ctx.world.bg_state.qs,
                        &mut p as *mut *const c_char,
                        &mut n0,
                    ) != 0
                    {
                        continue;
                    }
                    (*client_ptr).ps.customRGBA[3] = n0;
                }
                continue;
            }

            // headmodel
            if qstricmp_eq(token, c"headmodel") {
                let mut value: *const c_char = std::ptr::null();
                if crate::q_shared::COM_ParseString(
                    &mut ctx.world.bg_state.qs,
                    &mut p as *mut *const c_char,
                    &mut value,
                ) != 0
                {
                    continue;
                }
                if Q_stricmp("none", &cstr_to_str(value)) == 0 {
                    //Zero the head clamp range so the torso & legs don't lag behind
                    (*ri).headYawRangeLeft = 0;
                    (*ri).headYawRangeRight = 0;
                    (*ri).headPitchRangeUp = 0;
                    (*ri).headPitchRangeDown = 0;
                }
                continue;
            }

            // torsomodel
            if qstricmp_eq(token, c"torsomodel") {
                let mut value: *const c_char = std::ptr::null();
                if crate::q_shared::COM_ParseString(
                    &mut ctx.world.bg_state.qs,
                    &mut p as *mut *const c_char,
                    &mut value,
                ) != 0
                {
                    continue;
                }
                if Q_stricmp("none", &cstr_to_str(value)) == 0 {
                    //Zero the torso clamp range so the legs don't lag behind
                    (*ri).torsoYawRangeLeft = 0;
                    (*ri).torsoYawRangeRight = 0;
                    (*ri).torsoPitchRangeUp = 0;
                    (*ri).torsoPitchRangeDown = 0;
                }
                continue;
            }

            // legsmodel
            if qstricmp_eq(token, c"legsmodel") {
                let mut value: *const c_char = std::ptr::null();
                if crate::q_shared::COM_ParseString(
                    &mut ctx.world.bg_state.qs,
                    &mut p as *mut *const c_char,
                    &mut value,
                ) != 0
                {
                    continue;
                }
                continue;
            }

            // playerModel
            if qstricmp_eq(token, c"playerModel") {
                let mut value: *const c_char = std::ptr::null();
                if crate::q_shared::COM_ParseString(
                    &mut ctx.world.bg_state.qs,
                    &mut p as *mut *const c_char,
                    &mut value,
                ) != 0
                {
                    continue;
                }
                crate::q_shared::Q_strncpyz(
                    playerModel.as_mut_ptr(),
                    value,
                    playerModel.len() as c_int,
                );
                md3Model = 0;
                continue;
            }

            // customSkin
            if qstricmp_eq(token, c"customSkin") {
                let mut value: *const c_char = std::ptr::null();
                if crate::q_shared::COM_ParseString(
                    &mut ctx.world.bg_state.qs,
                    &mut p as *mut *const c_char,
                    &mut value,
                ) != 0
                {
                    continue;
                }
                crate::q_shared::Q_strncpyz(
                    customSkin.as_mut_ptr(),
                    value,
                    customSkin.len() as c_int,
                );
                continue;
            }

            // surfOff
            if qstricmp_eq(token, c"surfOff") {
                let mut value: *const c_char = std::ptr::null();
                if crate::q_shared::COM_ParseString(
                    &mut ctx.world.bg_state.qs,
                    &mut p as *mut *const c_char,
                    &mut value,
                ) != 0
                {
                    continue;
                }
                if surfOff[0] != 0 {
                    crate::q_shared::Q_strcat(
                        surfOff.as_mut_ptr(),
                        surfOff.len() as c_int,
                        cstr(",").as_ptr(),
                    );
                    crate::q_shared::Q_strcat(surfOff.as_mut_ptr(), surfOff.len() as c_int, value);
                } else {
                    crate::q_shared::Q_strncpyz(
                        surfOff.as_mut_ptr(),
                        value,
                        surfOff.len() as c_int,
                    );
                }
                continue;
            }

            // surfOn
            if qstricmp_eq(token, c"surfOn") {
                let mut value: *const c_char = std::ptr::null();
                if crate::q_shared::COM_ParseString(
                    &mut ctx.world.bg_state.qs,
                    &mut p as *mut *const c_char,
                    &mut value,
                ) != 0
                {
                    continue;
                }
                if surfOn[0] != 0 {
                    crate::q_shared::Q_strcat(
                        surfOn.as_mut_ptr(),
                        surfOn.len() as c_int,
                        cstr(",").as_ptr(),
                    );
                    crate::q_shared::Q_strcat(surfOn.as_mut_ptr(), surfOn.len() as c_int, value);
                } else {
                    crate::q_shared::Q_strncpyz(surfOn.as_mut_ptr(), value, surfOn.len() as c_int);
                }
                continue;
            }

            // headYawRangeLeft / Right / headPitchRangeUp / Down /
            // torsoYawRangeLeft / Right / torsoPitchRangeUp / Down — all share
            // the same shape (parse int, reject negative, store on `ri`).
            macro_rules! range_field {
                ($lit:literal, $field:ident) => {
                    if qstricmp_eq(token, $lit) {
                        let mut n0: c_int = 0;
                        if crate::q_shared::COM_ParseInt(
                            &mut ctx.world.bg_state.qs,
                            &mut p as *mut *const c_char,
                            &mut n0,
                        ) != 0
                        {
                            crate::q_shared::SkipRestOfLine(
                                &mut ctx.world.bg_state.qs,
                                &mut p as *mut *const c_char,
                            );
                            continue 'parse;
                        }
                        if n0 < 0 {
                            let msg = format!(
                                "WARNING: bad {} in NPC '{}'\n",
                                cstr_to_str(token),
                                NPCName
                            );
                            crate::g_main::Com_Printf(&msg);
                            continue 'parse;
                        }
                        (*ri).$field = n0;
                        continue 'parse;
                    }
                };
            }
            range_field!(c"headYawRangeLeft", headYawRangeLeft);
            range_field!(c"headYawRangeRight", headYawRangeRight);
            range_field!(c"headPitchRangeUp", headPitchRangeUp);
            range_field!(c"headPitchRangeDown", headPitchRangeDown);
            range_field!(c"torsoYawRangeLeft", torsoYawRangeLeft);
            range_field!(c"torsoYawRangeRight", torsoYawRangeRight);
            range_field!(c"torsoPitchRangeUp", torsoPitchRangeUp);
            range_field!(c"torsoPitchRangeDown", torsoPitchRangeDown);

            // Uniform XYZ scale
            if qstricmp_eq(token, c"scale") {
                let mut n0: c_int = 0;
                if crate::q_shared::COM_ParseInt(
                    &mut ctx.world.bg_state.qs,
                    &mut p as *mut *const c_char,
                    &mut n0,
                ) != 0
                {
                    crate::q_shared::SkipRestOfLine(
                        &mut ctx.world.bg_state.qs,
                        &mut p as *mut *const c_char,
                    );
                    continue;
                }
                if n0 < 0 {
                    let msg = format!(
                        "bad {} in NPC '{}'\n",
                        cstr_to_str(token),
                        NPCName
                    );
                    crate::g_main::Com_Printf(&msg);
                    continue;
                }
                if n0 != 100 {
                    (*client_ptr).ps.iModelScale = n0; //so the client knows
                    if n0 >= 1024 {
                        crate::g_main::Com_Printf(
                            "WARNING: MP does not support scaling up to or over 1024%\n",
                        );
                        n0 = 1023;
                    }

                    let scale = n0 as f32 / 100.0f32;
                    ctx.world.entity_mut(NPC).modelScale = [scale, scale, scale];
                }
                continue;
            }

            // X/Y/Z scale — unsupported in MP, parsed and discarded.
            macro_rules! scale_axis {
                ($lit:literal) => {
                    if qstricmp_eq(token, $lit) {
                        let mut n0: c_int = 0;
                        if crate::q_shared::COM_ParseInt(
                            &mut ctx.world.bg_state.qs,
                            &mut p as *mut *const c_char,
                            &mut n0,
                        ) != 0
                        {
                            crate::q_shared::SkipRestOfLine(
                                &mut ctx.world.bg_state.qs,
                                &mut p as *mut *const c_char,
                            );
                            continue 'parse;
                        }
                        if n0 < 0 {
                            let msg = format!(
                                "bad {} in NPC '{}'\n",
                                cstr_to_str(token),
                                NPCName
                            );
                            crate::g_main::Com_Printf(&msg);
                            continue 'parse;
                        }
                        if n0 != 100 {
                            crate::g_main::Com_Printf(
                                "MP doesn't support xyz scaling, use 'scale'.\n",
                            );
                        }
                        continue 'parse;
                    }
                };
            }
            scale_axis!(c"scaleX");
            scale_axis!(c"scaleY");
            scale_axis!(c"scaleZ");

            //===AI STATS=====================================================================
            if parsingPlayer == 0 {
                // int-valued 1-5-range NPC stats (aggression/aim/evasion/
                // intelligence/move/reactions) — same shape.
                macro_rules! stat_1_5 {
                    ($lit:literal, $field:ident) => {
                        if qstricmp_eq(token, $lit) {
                            let mut n0: c_int = 0;
                            if crate::q_shared::COM_ParseInt(
                                &mut ctx.world.bg_state.qs,
                                &mut p as *mut *const c_char,
                                &mut n0,
                            ) != 0
                            {
                                crate::q_shared::SkipRestOfLine(
                                    &mut ctx.world.bg_state.qs,
                                    &mut p as *mut *const c_char,
                                );
                                continue 'parse;
                            }
                            if n0 < 1 || n0 > 5 {
                                let msg = format!(
                                    "bad {} in NPC '{}'\n",
                                    cstr_to_str(token),
                                    NPCName
                                );
                                crate::g_main::Com_Printf(&msg);
                                continue 'parse;
                            }
                            if !npc_ptr.is_null() {
                                (*stats).$field = n0;
                            }
                            continue 'parse;
                        }
                    };
                }
                stat_1_5!(c"aggression", aggression);
                stat_1_5!(c"aim", aim);
                // earshot (float)
                if qstricmp_eq(token, c"earshot") {
                    let mut f0: f32 = 0.0;
                    if crate::q_shared::COM_ParseFloat(
                        &mut ctx.world.bg_state.qs,
                        &mut p as *mut *const c_char,
                        &mut f0,
                    ) != 0
                    {
                        crate::q_shared::SkipRestOfLine(
                            &mut ctx.world.bg_state.qs,
                            &mut p as *mut *const c_char,
                        );
                        continue;
                    }
                    if f0 < 0.0f32 {
                        let msg = format!(
                            "bad {} in NPC '{}'\n",
                            cstr_to_str(token),
                            NPCName
                        );
                        crate::g_main::Com_Printf(&msg);
                        continue;
                    }
                    if !npc_ptr.is_null() {
                        (*stats).earshot = f0;
                    }
                    continue;
                }
                stat_1_5!(c"evasion", evasion);
                // hfov
                if qstricmp_eq(token, c"hfov") {
                    let mut n0: c_int = 0;
                    if crate::q_shared::COM_ParseInt(
                        &mut ctx.world.bg_state.qs,
                        &mut p as *mut *const c_char,
                        &mut n0,
                    ) != 0
                    {
                        crate::q_shared::SkipRestOfLine(
                            &mut ctx.world.bg_state.qs,
                            &mut p as *mut *const c_char,
                        );
                        continue;
                    }
                    if n0 < 30 || n0 > 180 {
                        let msg = format!(
                            "bad {} in NPC '{}'\n",
                            cstr_to_str(token),
                            NPCName
                        );
                        crate::g_main::Com_Printf(&msg);
                        continue;
                    }
                    if !npc_ptr.is_null() {
                        (*stats).hfov = n0; // / 2;	//FIXME: Why was this being done?!
                    }
                    continue;
                }
                stat_1_5!(c"intelligence", intelligence);
                stat_1_5!(c"move", r#move);
                stat_1_5!(c"reactions", reactions);
                // shootDistance (float)
                if qstricmp_eq(token, c"shootDistance") {
                    let mut f0: f32 = 0.0;
                    if crate::q_shared::COM_ParseFloat(
                        &mut ctx.world.bg_state.qs,
                        &mut p as *mut *const c_char,
                        &mut f0,
                    ) != 0
                    {
                        crate::q_shared::SkipRestOfLine(
                            &mut ctx.world.bg_state.qs,
                            &mut p as *mut *const c_char,
                        );
                        continue;
                    }
                    if f0 < 0.0f32 {
                        let msg = format!(
                            "bad {} in NPC '{}'\n",
                            cstr_to_str(token),
                            NPCName
                        );
                        crate::g_main::Com_Printf(&msg);
                        continue;
                    }
                    if !npc_ptr.is_null() {
                        (*stats).shootDistance = f0;
                    }
                    continue;
                }
                // vfov
                if qstricmp_eq(token, c"vfov") {
                    let mut n0: c_int = 0;
                    if crate::q_shared::COM_ParseInt(
                        &mut ctx.world.bg_state.qs,
                        &mut p as *mut *const c_char,
                        &mut n0,
                    ) != 0
                    {
                        crate::q_shared::SkipRestOfLine(
                            &mut ctx.world.bg_state.qs,
                            &mut p as *mut *const c_char,
                        );
                        continue;
                    }
                    if n0 < 30 || n0 > 180 {
                        let msg = format!(
                            "bad {} in NPC '{}'\n",
                            cstr_to_str(token),
                            NPCName
                        );
                        crate::g_main::Com_Printf(&msg);
                        continue;
                    }
                    if !npc_ptr.is_null() {
                        (*stats).vfov = n0 / 2;
                    }
                    continue;
                }
                // vigilance (float)
                if qstricmp_eq(token, c"vigilance") {
                    let mut f0: f32 = 0.0;
                    if crate::q_shared::COM_ParseFloat(
                        &mut ctx.world.bg_state.qs,
                        &mut p as *mut *const c_char,
                        &mut f0,
                    ) != 0
                    {
                        crate::q_shared::SkipRestOfLine(
                            &mut ctx.world.bg_state.qs,
                            &mut p as *mut *const c_char,
                        );
                        continue;
                    }
                    if f0 < 0.0f32 {
                        let msg = format!(
                            "bad {} in NPC '{}'\n",
                            cstr_to_str(token),
                            NPCName
                        );
                        crate::g_main::Com_Printf(&msg);
                        continue;
                    }
                    if !npc_ptr.is_null() {
                        (*stats).vigilance = f0;
                    }
                    continue;
                }
                // visrange (float)
                if qstricmp_eq(token, c"visrange") {
                    let mut f0: f32 = 0.0;
                    if crate::q_shared::COM_ParseFloat(
                        &mut ctx.world.bg_state.qs,
                        &mut p as *mut *const c_char,
                        &mut f0,
                    ) != 0
                    {
                        crate::q_shared::SkipRestOfLine(
                            &mut ctx.world.bg_state.qs,
                            &mut p as *mut *const c_char,
                        );
                        continue;
                    }
                    if f0 < 0.0f32 {
                        let msg = format!(
                            "bad {} in NPC '{}'\n",
                            cstr_to_str(token),
                            NPCName
                        );
                        crate::g_main::Com_Printf(&msg);
                        continue;
                    }
                    if !npc_ptr.is_null() {
                        (*stats).visrange = f0;
                    }
                    continue;
                }
                // race — commented out in oracle, dropped per §17/§20.

                // rank
                if qstricmp_eq(token, c"rank") {
                    let mut value: *const c_char = std::ptr::null();
                    if crate::q_shared::COM_ParseString(
                        &mut ctx.world.bg_state.qs,
                        &mut p as *mut *const c_char,
                        &mut value,
                    ) != 0
                    {
                        continue;
                    }
                    if !npc_ptr.is_null() {
                        (*npc_ptr).rank = TranslateRankName(value);
                    }
                    continue;
                }
            }

            // health
            if qstricmp_eq(token, c"health") {
                let mut n0: c_int = 0;
                if crate::q_shared::COM_ParseInt(
                    &mut ctx.world.bg_state.qs,
                    &mut p as *mut *const c_char,
                    &mut n0,
                ) != 0
                {
                    crate::q_shared::SkipRestOfLine(
                        &mut ctx.world.bg_state.qs,
                        &mut p as *mut *const c_char,
                    );
                    continue;
                }
                if n0 < 0 {
                    let msg = format!(
                        "WARNING: bad {} in NPC '{}'\n",
                        cstr_to_str(token),
                        NPCName
                    );
                    crate::g_main::Com_Printf(&msg);
                    continue;
                }
                if !npc_ptr.is_null() {
                    (*stats).health = n0;
                } else if parsingPlayer != 0 {
                    (*client_ptr).ps.stats[crate::prelude::STAT_MAX_HEALTH as usize] = n0;
                    (*client_ptr).pers.maxHealth = n0;
                }
                continue;
            }

            // fullName
            if qstricmp_eq(token, c"fullName") {
                let mut value: *const c_char = std::ptr::null();
                if crate::q_shared::COM_ParseString(
                    &mut ctx.world.bg_state.qs,
                    &mut p as *mut *const c_char,
                    &mut value,
                ) != 0
                {
                    continue;
                }
                let full_name = unsafe { cstr_to_str(value) };
                ctx.ent_set(NPC, PrefixSet::FullName(Some(&full_name)));
                continue;
            }

            // playerTeam
            if qstricmp_eq(token, c"playerTeam") {
                let mut value: *const c_char = std::ptr::null();
                if crate::q_shared::COM_ParseString(
                    &mut ctx.world.bg_state.qs,
                    &mut p as *mut *const c_char,
                    &mut value,
                ) != 0
                {
                    continue;
                }
                let tk = format!("NPC{}", cstr_to_str(token));
                let tk_c = cstr(&tk);
                let team_id = crate::q_shared::GetIDForString(
                    TeamTable.as_ptr() as *mut stringID_table_t,
                    tk_c.as_ptr(),
                );
                (*client_ptr).playerTeam = team_id;
                ctx.world.entity_mut(NPC).s.teamowner = team_id;
                continue;
            }

            // enemyTeam
            if qstricmp_eq(token, c"enemyTeam") {
                let mut value: *const c_char = std::ptr::null();
                if crate::q_shared::COM_ParseString(
                    &mut ctx.world.bg_state.qs,
                    &mut p as *mut *const c_char,
                    &mut value,
                ) != 0
                {
                    continue;
                }
                let tk = format!("NPC{}", cstr_to_str(token));
                let tk_c = cstr(&tk);
                (*client_ptr).enemyTeam = crate::q_shared::GetIDForString(
                    TeamTable.as_ptr() as *mut stringID_table_t,
                    tk_c.as_ptr(),
                );
                continue;
            }

            // class
            if qstricmp_eq(token, c"class") {
                let mut value: *const c_char = std::ptr::null();
                if crate::q_shared::COM_ParseString(
                    &mut ctx.world.bg_state.qs,
                    &mut p as *mut *const c_char,
                    &mut value,
                ) != 0
                {
                    continue;
                }
                let class_id = crate::q_shared::GetIDForString(
                    ClassTable.as_ptr() as *mut stringID_table_t,
                    value,
                );
                // Divergence (§19): Raven stores GetIDForString's -1 miss (SP-only
                // class names) straight into the enum; `class_t` can't hold -1, and
                // no MP code compares against CLASS_NONE, so a miss clamps to it.
                // `s.NPC_class` below keeps the raw -1 exactly as Raven.
                (*client_ptr).NPC_class =
                    if class_id >= 0 && class_id < class_t::CLASS_NUM_CLASSES as c_int {
                        core::mem::transmute::<c_int, class_t>(class_id)
                    } else {
                        class_t::CLASS_NONE
                    };
                ctx.world.entity_mut(NPC).s.NPC_class = class_id; //we actually only need this value now, but at the moment I don't feel like changing the 200+ references to client->NPC_class.

                // No md3's for vehicles.
                if (*client_ptr).NPC_class == CLASS_VEHICLE {
                    if ctx.world.entity(NPC).m_pVehicle.is_null() {
                        //you didn't spawn this guy right!
                        let msg = format!(
                            "ERROR: Tried to spawn a vehicle NPC ({}) without using NPC_Vehicle or 'NPC spawn vehicle <vehiclename>'!!!  Bad, bad, bad!  Shame on you!\n",
                            NPCName
                        );
                        crate::g_main::Com_Printf(&msg);
                        return 0; // qfalse
                    }
                    md3Model = 0;
                }

                continue;
            }

            // dismemberment probabilities — parsed, never applied (MP doesn't
            // support dismemberment; matches oracle's commented-out fields).
            macro_rules! dismember_stub {
                ($lit:literal) => {
                    if qstricmp_eq(token, $lit) {
                        let mut n0: c_int = 0;
                        if crate::q_shared::COM_ParseInt(
                            &mut ctx.world.bg_state.qs,
                            &mut p as *mut *const c_char,
                            &mut n0,
                        ) != 0
                        {
                            crate::q_shared::SkipRestOfLine(
                                &mut ctx.world.bg_state.qs,
                                &mut p as *mut *const c_char,
                            );
                            continue 'parse;
                        }
                        if n0 < 0 {
                            let msg = format!(
                                "bad {} in NPC '{}'\n",
                                cstr_to_str(token),
                                NPCName
                            );
                            crate::g_main::Com_Printf(&msg);
                            continue 'parse;
                        }
                        //rwwFIXMEFIXME: support for this?
                        continue 'parse;
                    }
                };
            }
            dismember_stub!(c"dismemberProbHead");
            dismember_stub!(c"dismemberProbArms");
            dismember_stub!(c"dismemberProbHands");
            dismember_stub!(c"dismemberProbWaist");
            dismember_stub!(c"dismemberProbLegs");

            //===MOVEMENT STATS============================================================
            if qstricmp_eq(token, c"width") {
                let mut n0: c_int = 0;
                if crate::q_shared::COM_ParseInt(
                    &mut ctx.world.bg_state.qs,
                    &mut p as *mut *const c_char,
                    &mut n0,
                ) != 0
                {
                    continue;
                }
                {
                    let e = ctx.world.entity_mut(NPC);
                    e.r.mins[0] = -(n0 as f32);
                    e.r.mins[1] = -(n0 as f32);
                    e.r.maxs[0] = n0 as f32;
                    e.r.maxs[1] = n0 as f32;
                }
                continue;
            }

            if qstricmp_eq(token, c"height") {
                let mut n0: c_int = 0;
                if crate::q_shared::COM_ParseInt(
                    &mut ctx.world.bg_state.qs,
                    &mut p as *mut *const c_char,
                    &mut n0,
                ) != 0
                {
                    continue;
                }
                let vehForHeight = ctx.world.entity(NPC).m_pVehicle;
                if (*client_ptr).NPC_class == CLASS_VEHICLE
                    && !ctx.world.entity(NPC).m_pVehicle.is_null()
                    && !(*vehForHeight).m_pVehicleInfo.is_null()
                    && (*(*vehForHeight).m_pVehicleInfo).r#type == VH_FIGHTER
                {
                    // a flying vehicle's origin must be centered in bbox and it should spawn on the ground
                    // Raven `maxs[2] = ps.standheight = (n/2.0f)`: the chained assign stores
                    // through int `standheight` first, so `maxs[2]` gets the truncated `floor(n/2)`.
                    (*client_ptr).ps.standheight = (n0 as f32 / 2.0f32) as c_int;
                    let standheight_f = (*client_ptr).ps.standheight as f32;
                    ctx.world.entity_mut(NPC).r.maxs[2] = standheight_f;
                    let maxs2 = ctx.world.entity(NPC).r.maxs[2];
                    ctx.world.entity_mut(NPC).r.mins[2] = -maxs2;
                    let mins2 = ctx.world.entity(NPC).r.mins[2];
                    ctx.world.entity_mut(NPC).s.origin[2] += (DEFAULT_MINS_2 - mins2) + 0.125f32;
                    let origin = ctx.world.entity(NPC).s.origin;
                    crate::q_math::_VectorCopy(origin, &mut (*client_ptr).ps.origin);
                    crate::q_math::_VectorCopy(
                        origin,
                        &mut ctx.world.entity_mut(NPC).r.currentOrigin,
                    );
                    crate::g_utils::G_SetOrigin(ctx.world.entity_mut(NPC), origin);
                    let np = ctx.world.entity_mut(NPC) as *mut gentity_t;
                    trap::LinkEntity(
                        ctx.engine,
                        mp_abi::game::syscalls::G_LINKENTITY::GLinkentityArgs::new(np.cast()),
                    );
                } else {
                    ctx.world.entity_mut(NPC).r.mins[2] = DEFAULT_MINS_2; //Cannot change
                    ctx.world.entity_mut(NPC).r.maxs[2] = n0 as f32 + DEFAULT_MINS_2;
                    let maxs2 = ctx.world.entity(NPC).r.maxs[2];
                    (*client_ptr).ps.standheight = maxs2 as c_int;
                }
                ctx.world.entity_mut(NPC).radius = n0 as f32;
                continue;
            }

            if qstricmp_eq(token, c"crouchheight") {
                let mut n0: c_int = 0;
                if crate::q_shared::COM_ParseInt(
                    &mut ctx.world.bg_state.qs,
                    &mut p as *mut *const c_char,
                    &mut n0,
                ) != 0
                {
                    continue;
                }
                (*client_ptr).ps.crouchheight = n0 + DEFAULT_MINS_2 as c_int;
                continue;
            }

            if parsingPlayer == 0 {
                if qstricmp_eq(token, c"movetype") {
                    let mut value: *const c_char = std::ptr::null();
                    if crate::q_shared::COM_ParseString(
                        &mut ctx.world.bg_state.qs,
                        &mut p as *mut *const c_char,
                        &mut value,
                    ) != 0
                    {
                        continue;
                    }
                    if Q_stricmp("flyswim", &cstr_to_str(value)) == 0 {
                        (*client_ptr).ps.eFlags2 |= mp_bg::public::entity_effects::EF2_FLYING;
                    }
                    //NPC->client->moveType = (movetype_t)MoveTypeNameToEnum(value);
                    //rwwFIXMEFIXME: support for movetypes
                    continue;
                }

                // yawSpeed (float-valued stat stored from an int token)
                if qstricmp_eq(token, c"yawSpeed") {
                    let mut n0: c_int = 0;
                    if crate::q_shared::COM_ParseInt(
                        &mut ctx.world.bg_state.qs,
                        &mut p as *mut *const c_char,
                        &mut n0,
                    ) != 0
                    {
                        crate::q_shared::SkipRestOfLine(
                            &mut ctx.world.bg_state.qs,
                            &mut p as *mut *const c_char,
                        );
                        continue;
                    }
                    if n0 <= 0 {
                        let msg = format!(
                            "bad {} in NPC '{}'\n",
                            cstr_to_str(token),
                            NPCName
                        );
                        crate::g_main::Com_Printf(&msg);
                        continue;
                    }
                    if !npc_ptr.is_null() {
                        (*stats).yawSpeed = n0 as f32;
                    }
                    continue;
                }

                // walkSpeed / runSpeed / acceleration — int stats, reject < 0.
                macro_rules! stat_nonneg {
                    ($lit:literal, $field:ident) => {
                        if qstricmp_eq(token, $lit) {
                            let mut n0: c_int = 0;
                            if crate::q_shared::COM_ParseInt(
                                &mut ctx.world.bg_state.qs,
                                &mut p as *mut *const c_char,
                                &mut n0,
                            ) != 0
                            {
                                crate::q_shared::SkipRestOfLine(
                                    &mut ctx.world.bg_state.qs,
                                    &mut p as *mut *const c_char,
                                );
                                continue 'parse;
                            }
                            if n0 < 0 {
                                let msg = format!(
                                    "WARNING: bad {} in NPC '{}'\n",
                                    cstr_to_str(token),
                                    NPCName
                                );
                                crate::g_main::Com_Printf(&msg);
                                continue 'parse;
                            }
                            if !npc_ptr.is_null() {
                                (*stats).$field = n0;
                            }
                            continue 'parse;
                        }
                    };
                }
                stat_nonneg!(c"walkSpeed", walkSpeed);
                stat_nonneg!(c"runSpeed", runSpeed);
                if qstricmp_eq(token, c"acceleration") {
                    let mut n0: c_int = 0;
                    if crate::q_shared::COM_ParseInt(
                        &mut ctx.world.bg_state.qs,
                        &mut p as *mut *const c_char,
                        &mut n0,
                    ) != 0
                    {
                        crate::q_shared::SkipRestOfLine(
                            &mut ctx.world.bg_state.qs,
                            &mut p as *mut *const c_char,
                        );
                        continue;
                    }
                    if n0 < 0 {
                        let msg = format!(
                            "WARNING: bad {} in NPC '{}'\n",
                            cstr_to_str(token),
                            NPCName
                        );
                        crate::g_main::Com_Printf(&msg);
                        continue;
                    }
                    if !npc_ptr.is_null() {
                        (*stats).acceleration = n0;
                    }
                    continue;
                }
                //sex - skip in MP
                if qstricmp_eq(token, c"sex") {
                    crate::q_shared::SkipRestOfLine(
                        &mut ctx.world.bg_state.qs,
                        &mut p as *mut *const c_char,
                    );
                    continue;
                }
                //===MISC===============================================================================
                // default behavior
                if qstricmp_eq(token, c"behavior") {
                    let mut n0: c_int = 0;
                    if crate::q_shared::COM_ParseInt(
                        &mut ctx.world.bg_state.qs,
                        &mut p as *mut *const c_char,
                        &mut n0,
                    ) != 0
                    {
                        crate::q_shared::SkipRestOfLine(
                            &mut ctx.world.bg_state.qs,
                            &mut p as *mut *const c_char,
                        );
                        continue;
                    }
                    if n0 < BS_DEFAULT as c_int || n0 >= NUM_BSTATES as c_int {
                        let msg = format!(
                            "WARNING: bad {} in NPC '{}'\n",
                            cstr_to_str(token),
                            NPCName
                        );
                        crate::g_main::Com_Printf(&msg);
                        continue;
                    }
                    if !npc_ptr.is_null() {
                        (*npc_ptr).defaultBehavior = core::mem::transmute::<c_int, bState_t>(n0);
                    }
                    continue;
                }
            }

            // snd / sndcombat / sndextra / sndjedi — parsed for their
            // directory prefix but not applied (client-side sound-dir fields
            // are not ported; matches oracle's commented-out ci-> stores).
            macro_rules! snd_dir {
                ($lit:literal, $flag:expr) => {
                    if qstricmp_eq(token, $lit) {
                        let mut value: *const c_char = std::ptr::null();
                        if crate::q_shared::COM_ParseString(
                            &mut ctx.world.bg_state.qs,
                            &mut p as *mut *const c_char,
                            &mut value,
                        ) != 0
                        {
                            continue 'parse;
                        }
                        if ctx.world.entity(NPC).r.svFlags & $flag == 0 {
                            crate::q_shared::Q_strncpyz(
                                sound.as_mut_ptr(),
                                value,
                                sound.len() as c_int,
                            );
                            //	ci->customBasicSoundDir = G_NewString( sound );
                            //rwwFIXMEFIXME: Hooray for violating client server rules
                        }
                        continue 'parse;
                    }
                };
            }
            snd_dir!(c"snd", SVF_NO_BASIC_SOUNDS);
            snd_dir!(c"sndcombat", SVF_NO_COMBAT_SOUNDS);
            snd_dir!(c"sndextra", SVF_NO_EXTRA_SOUNDS);
            snd_dir!(c"sndjedi", SVF_NO_EXTRA_SOUNDS);

            //New NPC/jedi stats:
            //starting weapon
            if qstricmp_eq(token, c"weapon") {
                let mut value: *const c_char = std::ptr::null();
                if crate::q_shared::COM_ParseString(
                    &mut ctx.world.bg_state.qs,
                    &mut p as *mut *const c_char,
                    &mut value,
                ) != 0
                {
                    continue;
                }
                //FIXME: need to precache the weapon, too?  (in above func)
                let weap = crate::q_shared::GetIDForString(
                    mp_bg::bg_saga::WPTable.as_ptr() as *mut stringID_table_t,
                    value,
                );
                if weap >= WP_NONE && weap <= (WP_NUM_WEAPONS as c_int) {
                    (*client_ptr).ps.weapon = weap;
                    (*client_ptr).ps.stats[crate::prelude::STAT_WEAPONS as usize] |=
                        1 << (*client_ptr).ps.weapon;
                    if weap > WP_NONE {
                        //	RegisterItem( FindItemForWeapon( (weapon_t)(NPC->client->ps.weapon) ) );	//precache the weapon
                        (*client_ptr).ps.ammo
                            [weaponData[(*client_ptr).ps.weapon as usize].ammoIndex as usize] = 100;
                        //FIXME: max ammo!
                    }
                }
                continue;
            }

            if parsingPlayer == 0 {
                //altFire
                if qstricmp_eq(token, c"altFire") {
                    let mut n0: c_int = 0;
                    if crate::q_shared::COM_ParseInt(
                        &mut ctx.world.bg_state.qs,
                        &mut p as *mut *const c_char,
                        &mut n0,
                    ) != 0
                    {
                        crate::q_shared::SkipRestOfLine(
                            &mut ctx.world.bg_state.qs,
                            &mut p as *mut *const c_char,
                        );
                        continue;
                    }
                    if !npc_ptr.is_null() {
                        if n0 != 0 {
                            (*npc_ptr).scriptFlags |= crate::npc::script_flags::SCF_ALT_FIRE;
                        }
                    }
                    continue;
                }
                //Other unique behaviors/numbers that are currently hardcoded?
            }

            //force powers
            let fp = crate::q_shared::GetIDForString(
                mp_bg::bg_saga::FPTable.as_ptr() as *mut stringID_table_t,
                token,
            );
            if fp >= FP_FIRST && fp < mp_qshared::shared::force_powers::NUM_FORCE_POWERS {
                let mut n0: c_int = 0;
                if crate::q_shared::COM_ParseInt(
                    &mut ctx.world.bg_state.qs,
                    &mut p as *mut *const c_char,
                    &mut n0,
                ) != 0
                {
                    crate::q_shared::SkipRestOfLine(
                        &mut ctx.world.bg_state.qs,
                        &mut p as *mut *const c_char,
                    );
                    continue;
                }
                //FIXME: need to precache the fx, too?  (in above func)
                //cap
                if n0 > 5 {
                    n0 = 5;
                } else if n0 < 0 {
                    n0 = 0;
                }
                if n0 != 0 {
                    //set
                    (*client_ptr).ps.fd.forcePowersKnown |= 1 << fp;
                } else {
                    //clear
                    (*client_ptr).ps.fd.forcePowersKnown &= !(1 << fp);
                }
                (*client_ptr).ps.fd.forcePowerLevel[fp as usize] = n0;
                continue;
            }

            //max force power
            if qstricmp_eq(token, c"forcePowerMax") {
                let mut n0: c_int = 0;
                if crate::q_shared::COM_ParseInt(
                    &mut ctx.world.bg_state.qs,
                    &mut p as *mut *const c_char,
                    &mut n0,
                ) != 0
                {
                    crate::q_shared::SkipRestOfLine(
                        &mut ctx.world.bg_state.qs,
                        &mut p as *mut *const c_char,
                    );
                    continue;
                }
                (*client_ptr).ps.fd.forcePowerMax = n0;
                continue;
            }

            //force regen rate - default is 100ms
            if qstricmp_eq(token, c"forceRegenRate") {
                let mut n0: c_int = 0;
                if crate::q_shared::COM_ParseInt(
                    &mut ctx.world.bg_state.qs,
                    &mut p as *mut *const c_char,
                    &mut n0,
                ) != 0
                {
                    crate::q_shared::SkipRestOfLine(
                        &mut ctx.world.bg_state.qs,
                        &mut p as *mut *const c_char,
                    );
                    continue;
                }
                //NPC->client->ps.forcePowerRegenRate = n;
                //rwwFIXMEFIXME: support this?
                continue;
            }

            //force regen amount - default is 1 (points per second)
            if qstricmp_eq(token, c"forceRegenAmount") {
                let mut n0: c_int = 0;
                if crate::q_shared::COM_ParseInt(
                    &mut ctx.world.bg_state.qs,
                    &mut p as *mut *const c_char,
                    &mut n0,
                ) != 0
                {
                    crate::q_shared::SkipRestOfLine(
                        &mut ctx.world.bg_state.qs,
                        &mut p as *mut *const c_char,
                    );
                    continue;
                }
                //NPC->client->ps.forcePowerRegenAmount = n;
                //rwwFIXMEFIXME: support this?
                continue;
            }

            //have a sabers.cfg and just name your saber in your NPCs.cfg/ICARUS script
            //saber name
            if qstricmp_eq(token, c"saber") {
                let mut value: *const c_char = std::ptr::null();
                if crate::q_shared::COM_ParseString(
                    &mut ctx.world.bg_state.qs,
                    &mut p as *mut *const c_char,
                    &mut value,
                ) != 0
                {
                    continue;
                }

                let bg = &mut ctx.world.bg_state;
                let saber_name = mp_bg::bg_misc::BG_TempAlloc(4096, bg) as *mut c_char; //G_NewString( value );
                crate::q_shared::Q_strncpyz(saber_name, value, 4096);

                let mut callbacks = crate::bg_channel::GameCallbacksImpl {
                    // SEAM-BG-REENTRY (DEC-28, sanctioned) — GameCallbacksImpl.world is a `*mut GameWorld`
                    // field aliasing bg_state; a raw store is required (bg-seam re-entry).
                    world: ctx.world_raw(),
                    engine: ctx.engine,
                };
                mp_bg::bg_saberLoad::WP_SaberParseParms(
                    &cstr_to_str(saber_name),
                    &mut (*client_ptr).saber[0] as *mut saberInfo_t,
                    &mut ctx.world.bg_state,
                    &GameBgTraps::new(ctx.engine),
                    &mut callbacks,
                );
                let idx_s = format!("@{}", cstr_to_str(saber_name));
                npcSaber1 = G_ModelIndex(&idx_s);

                mp_bg::bg_misc::BG_TempFree(4096, &mut ctx.world.bg_state);
                continue;
            }

            //second saber name
            if qstricmp_eq(token, c"saber2") {
                let mut value: *const c_char = std::ptr::null();
                if crate::q_shared::COM_ParseString(
                    &mut ctx.world.bg_state.qs,
                    &mut p as *mut *const c_char,
                    &mut value,
                ) != 0
                {
                    continue;
                }

                if (*client_ptr).saber[0].saberFlags & SFL_TWO_HANDED == 0 {
                    //can't use a second saber if first one is a two-handed saber...?
                    let bg = &mut ctx.world.bg_state;
                    let saber_name = mp_bg::bg_misc::BG_TempAlloc(4096, bg) as *mut c_char; //G_NewString( value );
                    crate::q_shared::Q_strncpyz(saber_name, value, 4096);

                    let mut callbacks = crate::bg_channel::GameCallbacksImpl {
                        // SEAM-BG-REENTRY (DEC-28, sanctioned) — GameCallbacksImpl.world is a `*mut GameWorld`
                        // field aliasing bg_state; a raw store is required (bg-seam re-entry).
                        world: ctx.world_raw(),
                        engine: ctx.engine,
                    };
                    mp_bg::bg_saberLoad::WP_SaberParseParms(
                        &cstr_to_str(saber_name),
                        &mut (*client_ptr).saber[1] as *mut saberInfo_t,
                        &mut ctx.world.bg_state,
                        &GameBgTraps::new(ctx.engine),
                        &mut callbacks,
                    );
                    if (*client_ptr).saber[1].saberFlags & SFL_TWO_HANDED != 0 {
                        //tsk tsk, can't use a twoHanded saber as second saber
                        mp_bg::bg_saberLoad::WP_RemoveSaber(
                            (*client_ptr).saber.as_mut_ptr(),
                            1,
                            &mut callbacks,
                        );
                    } else {
                        //NPC->client->ps.dualSabers = qtrue;
                        let idx_s = format!("@{}", cstr_to_str(saber_name));
                        npcSaber2 = G_ModelIndex(&idx_s);
                    }
                    mp_bg::bg_misc::BG_TempFree(4096, &mut ctx.world.bg_state);
                }
                continue;
            }

            // saberColor / saberColor2..8 — set-all-blades vs single-blade
            // color, mirrored for saber[1] as saber2Color / saber2Color2..8.
            if qstricmp_eq(token, c"saberColor") {
                let mut value: *const c_char = std::ptr::null();
                if crate::q_shared::COM_ParseString(
                    &mut ctx.world.bg_state.qs,
                    &mut p as *mut *const c_char,
                    &mut value,
                ) != 0
                {
                    continue;
                }
                if !client_ptr.is_null() {
                    let color =
                        mp_bg::bg_saberLoad::TranslateSaberColor(value, &mut ctx.world.bg_state);
                    for bi in 0..MAX_BLADES {
                        (*client_ptr).saber[0].blade[bi].color = color;
                    }
                }
                continue;
            }
            macro_rules! saber_color_n {
                ($lit:literal, $saber_idx:expr, $blade_idx:expr) => {
                    if qstricmp_eq(token, $lit) {
                        let mut value: *const c_char = std::ptr::null();
                        if crate::q_shared::COM_ParseString(
                            &mut ctx.world.bg_state.qs,
                            &mut p as *mut *const c_char,
                            &mut value,
                        ) != 0
                        {
                            continue 'parse;
                        }
                        if !client_ptr.is_null() {
                            (*client_ptr).saber[$saber_idx].blade[$blade_idx].color =
                                mp_bg::bg_saberLoad::TranslateSaberColor(
                                    value,
                                    &mut ctx.world.bg_state,
                                );
                        }
                        continue 'parse;
                    }
                };
            }
            saber_color_n!(c"saberColor2", 0, 1);
            saber_color_n!(c"saberColor3", 0, 2);
            saber_color_n!(c"saberColor4", 0, 3);
            saber_color_n!(c"saberColor5", 0, 4);
            saber_color_n!(c"saberColor6", 0, 5);
            saber_color_n!(c"saberColor7", 0, 6);
            saber_color_n!(c"saberColor8", 0, 7);
            if qstricmp_eq(token, c"saber2Color") {
                let mut value: *const c_char = std::ptr::null();
                if crate::q_shared::COM_ParseString(
                    &mut ctx.world.bg_state.qs,
                    &mut p as *mut *const c_char,
                    &mut value,
                ) != 0
                {
                    continue;
                }
                if !client_ptr.is_null() {
                    let color =
                        mp_bg::bg_saberLoad::TranslateSaberColor(value, &mut ctx.world.bg_state);
                    for bi in 0..MAX_BLADES {
                        (*client_ptr).saber[1].blade[bi].color = color;
                    }
                }
                continue;
            }
            saber_color_n!(c"saber2Color2", 1, 1);
            saber_color_n!(c"saber2Color3", 1, 2);
            saber_color_n!(c"saber2Color4", 1, 3);
            saber_color_n!(c"saber2Color5", 1, 4);
            saber_color_n!(c"saber2Color6", 1, 5);
            saber_color_n!(c"saber2Color7", 1, 6);
            saber_color_n!(c"saber2Color8", 1, 7);

            // saberLength / saberLength2..8, saber2Length / saber2Length2..8
            if qstricmp_eq(token, c"saberLength") {
                let mut f0: f32 = 0.0;
                if crate::q_shared::COM_ParseFloat(
                    &mut ctx.world.bg_state.qs,
                    &mut p as *mut *const c_char,
                    &mut f0,
                ) != 0
                {
                    crate::q_shared::SkipRestOfLine(
                        &mut ctx.world.bg_state.qs,
                        &mut p as *mut *const c_char,
                    );
                    continue;
                }
                if f0 < 4.0f32 {
                    f0 = 4.0f32;
                }
                for bi in 0..MAX_BLADES {
                    (*client_ptr).saber[0].blade[bi].lengthMax = f0;
                }
                continue;
            }
            macro_rules! saber_length_n {
                ($lit:literal, $saber_idx:expr, $blade_idx:expr) => {
                    if qstricmp_eq(token, $lit) {
                        let mut f0: f32 = 0.0;
                        if crate::q_shared::COM_ParseFloat(
                            &mut ctx.world.bg_state.qs,
                            &mut p as *mut *const c_char,
                            &mut f0,
                        ) != 0
                        {
                            crate::q_shared::SkipRestOfLine(
                                &mut ctx.world.bg_state.qs,
                                &mut p as *mut *const c_char,
                            );
                            continue 'parse;
                        }
                        if f0 < 4.0f32 {
                            f0 = 4.0f32;
                        }
                        (*client_ptr).saber[$saber_idx].blade[$blade_idx].lengthMax = f0;
                        continue 'parse;
                    }
                };
            }
            saber_length_n!(c"saberLength2", 0, 1);
            saber_length_n!(c"saberLength3", 0, 2);
            saber_length_n!(c"saberLength4", 0, 3);
            saber_length_n!(c"saberLength5", 0, 4);
            saber_length_n!(c"saberLength6", 0, 5);
            saber_length_n!(c"saberLength7", 0, 6);
            saber_length_n!(c"saberLength8", 0, 7);
            if qstricmp_eq(token, c"saber2Length") {
                let mut f0: f32 = 0.0;
                if crate::q_shared::COM_ParseFloat(
                    &mut ctx.world.bg_state.qs,
                    &mut p as *mut *const c_char,
                    &mut f0,
                ) != 0
                {
                    crate::q_shared::SkipRestOfLine(
                        &mut ctx.world.bg_state.qs,
                        &mut p as *mut *const c_char,
                    );
                    continue;
                }
                if f0 < 4.0f32 {
                    f0 = 4.0f32;
                }
                for bi in 0..MAX_BLADES {
                    (*client_ptr).saber[1].blade[bi].lengthMax = f0;
                }
                continue;
            }
            saber_length_n!(c"saber2Length2", 1, 1);
            saber_length_n!(c"saber2Length3", 1, 2);
            saber_length_n!(c"saber2Length4", 1, 3);
            saber_length_n!(c"saber2Length5", 1, 4);
            saber_length_n!(c"saber2Length6", 1, 5);
            saber_length_n!(c"saber2Length7", 1, 6);
            saber_length_n!(c"saber2Length8", 1, 7);

            // saberRadius / saberRadius2..8, saber2Radius / saber2Radius2..8
            if qstricmp_eq(token, c"saberRadius") {
                let mut f0: f32 = 0.0;
                if crate::q_shared::COM_ParseFloat(
                    &mut ctx.world.bg_state.qs,
                    &mut p as *mut *const c_char,
                    &mut f0,
                ) != 0
                {
                    crate::q_shared::SkipRestOfLine(
                        &mut ctx.world.bg_state.qs,
                        &mut p as *mut *const c_char,
                    );
                    continue;
                }
                if f0 < 0.25f32 {
                    f0 = 0.25f32;
                }
                for bi in 0..MAX_BLADES {
                    (*client_ptr).saber[0].blade[bi].radius = f0;
                }
                continue;
            }
            macro_rules! saber_radius_n {
                ($lit:literal, $saber_idx:expr, $blade_idx:expr) => {
                    if qstricmp_eq(token, $lit) {
                        let mut f0: f32 = 0.0;
                        if crate::q_shared::COM_ParseFloat(
                            &mut ctx.world.bg_state.qs,
                            &mut p as *mut *const c_char,
                            &mut f0,
                        ) != 0
                        {
                            crate::q_shared::SkipRestOfLine(
                                &mut ctx.world.bg_state.qs,
                                &mut p as *mut *const c_char,
                            );
                            continue 'parse;
                        }
                        if f0 < 0.25f32 {
                            f0 = 0.25f32;
                        }
                        (*client_ptr).saber[$saber_idx].blade[$blade_idx].radius = f0;
                        continue 'parse;
                    }
                };
            }
            saber_radius_n!(c"saberRadius2", 0, 1);
            saber_radius_n!(c"saberRadius3", 0, 2);
            saber_radius_n!(c"saberRadius4", 0, 3);
            saber_radius_n!(c"saberRadius5", 0, 4);
            saber_radius_n!(c"saberRadius6", 0, 5);
            saber_radius_n!(c"saberRadius7", 0, 6);
            saber_radius_n!(c"saberRadius8", 0, 7);
            if qstricmp_eq(token, c"saber2Radius") {
                let mut f0: f32 = 0.0;
                if crate::q_shared::COM_ParseFloat(
                    &mut ctx.world.bg_state.qs,
                    &mut p as *mut *const c_char,
                    &mut f0,
                ) != 0
                {
                    crate::q_shared::SkipRestOfLine(
                        &mut ctx.world.bg_state.qs,
                        &mut p as *mut *const c_char,
                    );
                    continue;
                }
                if f0 < 0.25f32 {
                    f0 = 0.25f32;
                }
                for bi in 0..MAX_BLADES {
                    (*client_ptr).saber[1].blade[bi].radius = f0;
                }
                continue;
            }
            saber_radius_n!(c"saber2Radius2", 1, 1);
            saber_radius_n!(c"saber2Radius3", 1, 2);
            saber_radius_n!(c"saber2Radius4", 1, 3);
            saber_radius_n!(c"saber2Radius5", 1, 4);
            saber_radius_n!(c"saber2Radius6", 1, 5);
            saber_radius_n!(c"saber2Radius7", 1, 6);
            saber_radius_n!(c"saber2Radius8", 1, 7);

            //ADD:
            //saber sounds (on, off, loop)
            //loop sound (like Vader's breathing or droid bleeps, etc.)

            //starting saber style
            if qstricmp_eq(token, c"saberStyle") {
                let mut n0: c_int = 0;
                if crate::q_shared::COM_ParseInt(
                    &mut ctx.world.bg_state.qs,
                    &mut p as *mut *const c_char,
                    &mut n0,
                ) != 0
                {
                    crate::q_shared::SkipRestOfLine(
                        &mut ctx.world.bg_state.qs,
                        &mut p as *mut *const c_char,
                    );
                    continue;
                }
                //cap
                if n0 < 0 {
                    n0 = 0;
                } else if n0 > 5 {
                    n0 = 5;
                }
                (*client_ptr).ps.fd.saberAnimLevel = n0;
                /*
                if ( parsingPlayer )
                {
                    cg.saberAnimLevelPending = n;
                }
                */
                continue;
            }

            if parsingPlayer == 0 {
                let msg = format!(
                    "WARNING: unknown keyword '{}' while parsing '{}'\n",
                    cstr_to_str(token),
                    NPCName
                );
                crate::g_main::Com_Printf(&msg);
            }
            crate::q_shared::SkipRestOfLine(
                &mut ctx.world.bg_state.qs,
                &mut p as *mut *const c_char,
            );
        }

        /*
        Ghoul2 Insert Start
        */
        if md3Model == 0 {
            let mut set_type_back: qboolean = 0; // qfalse

            if npcSaber1 == 0 {
                //use "kyle" for a default then
                npcSaber1 = G_ModelIndex("@Kyle");
                let mut callbacks = crate::bg_channel::GameCallbacksImpl {
                    // SEAM-BG-REENTRY (DEC-28, sanctioned) — GameCallbacksImpl.world is a `*mut GameWorld`
                    // field aliasing bg_state; a raw store is required (bg-seam re-entry).
                    world: ctx.world_raw(),
                    engine: ctx.engine,
                };
                mp_bg::bg_saberLoad::WP_SaberParseParms(
                    "Kyle",
                    &mut (*client_ptr).saber[0] as *mut saberInfo_t,
                    &mut ctx.world.bg_state,
                    &GameBgTraps::new(ctx.engine),
                    &mut callbacks,
                );
            }

            {
                let e = ctx.world.entity_mut(NPC);
                e.s.npcSaber1 = npcSaber1;
                e.s.npcSaber2 = npcSaber2;
            }

            if customSkin[0] == 0 {
                write_cstr_field(&mut customSkin, "default");
            }

            if !client_ptr.is_null() && (*client_ptr).NPC_class == CLASS_VEHICLE {
                //vehicles want their names fed in as models
                //we put the $ in front to indicate a name and not a model
                write_cstr_field(&mut playerModel, &format!("${}", NPCName));
            }
            crate::g_client::SetupGameGhoul2Model(
                ctx,
                NPC,
                playerModel.as_mut_ptr(),
                customSkin.as_mut_ptr(),
            );

            if ctx.world.entity(NPC).NPC_type.is_none() {
                //just do this for now so NPC_Precache can see the name.
                ctx.world.entity_mut(NPC).NPC_type = Some(NPCName.to_owned());
                set_type_back = 1;
            }

            NPC_Precache(ctx, NPC); //this will just soundindex some values for sounds on the client,

            if set_type_back != 0 {
                //don't want this being set if we aren't ready yet.
                ctx.world.entity_mut(NPC).NPC_type = None;
            }
        } else {
            crate::g_main::Com_Printf("MD3 MODEL NPC'S ARE NOT SUPPORTED IN MP!\n");
            return 0; // qfalse
        }
        /*
        Ghoul2 Insert End
        */
        /*
        if(	NPCsPrecached )
        {//Spawning in after initial precache, our models are precached, we just need to set our clientInfo
            CG_RegisterClientModels( NPC->s.number );
            CG_RegisterNPCCustomSounds( ci );
            CG_RegisterNPCEffects( NPC->client->playerTeam );
        }
        */
        //rwwFIXMEFIXME: Do something here I guess to properly precache stuff.

        1 // qtrue
    }
}

/// Raven `NPC_LoadParms`.
///
/// Source: `oracle/codemp/game/NPC_stats.c:3241-3302`
pub fn NPC_LoadParms(ctx: &mut GameContext) {
    unsafe {
        // The `_XBOX` malloc/free of `npcParseBuffer` is dead on this platform
        // and is dropped per porting-rules §20.
        let mut totallen: c_int = 0;
        let mainblocklen: c_int = 0;
        let _ = mainblocklen;

        // Raven: `marker = NPCParms + totallen; *marker = 0;`
        let npc_parms: *mut c_char = (&mut ctx.world.globals.NPCParms) as *mut _ as *mut c_char;
        let mut marker: *mut c_char = npc_parms.offset(totallen as isize);
        *marker = 0;

        // Raven: `char npcExtensionListBuf[2048];`
        let mut npc_extension_list_buf: [c_char; 2048] = [0; 2048];
        let file_cnt = trap::FS_GetFileList(
            ctx.engine,
            "ext_data/NPCs",
            ".npc",
            std::slice::from_raw_parts_mut(
                npc_extension_list_buf.as_mut_ptr() as *mut u8,
                npc_extension_list_buf.len(),
            ),
        );

        let npc_parse_buffer: *mut c_char =
            (&mut ctx.world.globals.npcParseBuffer) as *mut _ as *mut c_char;

        let mut hold_char: *mut c_char = npc_extension_list_buf.as_mut_ptr();
        let mut i: c_int = 0;
        while i < file_cnt {
            let npc_ext_fn_len = std::ffi::CStr::from_ptr(hold_char).to_bytes().len() as c_int;

            let path = format!("ext_data/NPCs/{}", cstr_to_str(hold_char));
            let mut f: fileHandle_t = 0;
            let mut len = trap::FS_FOpenFile(ctx.engine, &path, &mut f, FS_READ);

            if len == -1 {
                crate::g_main::Com_Printf("error reading file\n");
            } else {
                if totallen + len >= MAX_NPC_DATA_SIZE {
                    crate::g_main::G_Error(
                        ctx,
                        "NPC extensions (*.npc) are too large",
                    );
                }
                trap::FS_Read(
                    ctx.engine,
                    std::slice::from_raw_parts_mut(npc_parse_buffer as *mut u8, len as usize),
                    f,
                );
                *npc_parse_buffer.offset(len as isize) = 0;

                len = crate::q_shared::COM_Compress(npc_parse_buffer);

                crate::q_shared::Q_strcat(marker, MAX_NPC_DATA_SIZE - totallen, npc_parse_buffer);
                crate::q_shared::Q_strcat(
                    marker,
                    MAX_NPC_DATA_SIZE - totallen,
                    cstr("\n").as_ptr(),
                );
                len += 1;
                trap::FS_FCloseFile(ctx.engine, f);

                totallen += len;
                marker = npc_parms.offset(totallen as isize);
            }

            i += 1;
            hold_char = hold_char.offset((npc_ext_fn_len + 1) as isize);
        }
    }
}
