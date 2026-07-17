// PORT-COMPLETE: g_team.c (PrintMsg dropped — dead, zero live callers)
//! FAITHFUL port of `oracle/codemp/game/g_team.c`.
//!
//! Functions that reach file-scope game state (`level`, `teamgame`,
//! `g_entities`, cvars) or an engine trap thread the `GameContext` handle.
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;

use core::ffi::CStr;

use crate::ent_id::resolve;
use crate::entity::flags::{FL_DROPPED_ITEM, FL_FORCE_GESTURE};
use crate::g_combat::AddScore;
use crate::g_items::RespawnItem;
use crate::g_main::CalculateRanks;
use crate::g_utils::{G_Find, G_FreeEntity, G_TempEntity};
use mp_bg::bg_lib::qsort;
use mp_bg::public::ctf_msg::ctfMsg_t;
use mp_bg::public::entity_event::entity_event_t;
use mp_bg::public::global_team_sound::global_team_sound_t;
use mp_bg::public::powerup::{PW_BLUEFLAG, PW_NEUTRALFLAG, PW_REDFLAG};
use mp_qshared::shared::flag_status::{FLAG_ATBASE, FLAG_DROPPED};
use mp_qshared::shared::MAX_CLIENTS;

// Raven `qboolean` is `c_int`; keep the source spelling at assignment sites.
// Source: `oracle/codemp/game/q_shared.h`

// Raven color escape `#define`s (porting-rules §C8: `#define` -> `const`).
// Relocated to the qshared tier (`q_shared.h` is a shared header) for the bg
// crate; re-exported here so game importers and the prelude keep resolving
// `crate::g_team::S_COLOR_*` unchanged. Canonical home:
// `mp_qshared::shared::q_color`.
// Source: `oracle/codemp/game/q_shared.h:1145-1167`
pub use mp_qshared::shared::q_color::{
    S_COLOR_BLUE, S_COLOR_GREEN, S_COLOR_RED, S_COLOR_WHITE, S_COLOR_YELLOW,
};

// Raven color index `#define`s (porting-rules §C8: `#define` -> `const`).
// Source: `oracle/codemp/game/q_shared.h:1150-1157`
pub const COLOR_BLACK: c_int = '0' as c_int;
pub const COLOR_RED: c_int = '1' as c_int;
pub const COLOR_GREEN: c_int = '2' as c_int;
pub const COLOR_YELLOW: c_int = '3' as c_int;
pub const COLOR_BLUE: c_int = '4' as c_int;
pub const COLOR_CYAN: c_int = '5' as c_int;
pub const COLOR_MAGENTA: c_int = '6' as c_int;
pub const COLOR_WHITE: c_int = '7' as c_int;

// Raven team indices (`q_shared.h`).
const TEAM_FREE: c_int = 0;
const TEAM_RED: c_int = 1;
const TEAM_BLUE: c_int = 2;
const TEAM_SPECTATOR: c_int = 3;

// Siege team indices — canonical `#define`s in bg_saga.h (SIEGETEAM_TEAM1==1,
// SIEGETEAM_TEAM2==2). Local decls were off-by-one (0/1), breaking the
// `team == SIEGETEAM_TEAM1` spawn-class selection in SelectRandomTeamSpawnPoint.
// Source: `oracle/codemp/game/bg_saga.h:3-4`
use mp_bg::saga::siege_team_t::{SIEGETEAM_TEAM1, SIEGETEAM_TEAM2};

// Game state constant (oracle/codemp/game/g_team.c:974)
const TEAM_BEGIN: c_int = 0;

// `SVF_BROADCAST` (svflags #define) is canonical in `g_public_consts` and
// reaches here via the prelude glob (`pub use crate::g_public_consts::*`).
// Source: `oracle/codemp/game/g_public.h:20`

// CTF scoring bonuses (porting-rules §C8: `#define` -> `const`).
// Source: `oracle/codemp/game/g_team.h:4-15`
const CTF_CAPTURE_BONUS: c_int = 100;
const CTF_TEAM_BONUS: c_int = 25;
const CTF_RECOVERY_BONUS: c_int = 10;
const CTF_FLAG_BONUS: c_int = 10;
const CTF_FRAG_CARRIER_BONUS: c_int = 20;
const CTF_CARRIER_DANGER_PROTECT_BONUS: c_int = 5;
const CTF_CARRIER_PROTECT_BONUS: c_int = 2;
const CTF_FLAG_DEFENSE_BONUS: c_int = 10;
const CTF_RETURN_FLAG_ASSIST_BONUS: c_int = 10;
const CTF_FRAG_CARRIER_ASSIST_BONUS: c_int = 10;

/// `FOFS(x)` — byte offset of field `x` within `gentity_t` (Raven macro,
/// `g_local.h`). Used as the `fieldofs` argument to `G_Find`.
#[inline]
fn fofs_classname() -> c_int {
    core::mem::offset_of!(gentity_t, classname) as c_int
}

/// Raven `Team_InitGame`.
///
/// Source: `oracle/codemp/game/g_team.c:22-35`
pub fn Team_InitGame(ctx: &mut GameContext) {
    ctx.world.globals.teamgame = Default::default();

    let gametype = ctx.world.cvars.g_gametype.integer;
    if gametype == GT_CTF as c_int || gametype == GT_CTY as c_int {
        ctx.world.globals.teamgame.redStatus = -1; // Invalid to force update
        ctx.world.globals.teamgame.blueStatus = -1;
        Team_SetFlagStatus(ctx, TEAM_RED, FLAG_ATBASE);
        Team_SetFlagStatus(ctx, TEAM_BLUE, FLAG_ATBASE);
    }
}

/// Raven `OtherTeam`.
///
/// Source: `oracle/codemp/game/g_team.c:37-43`
pub fn OtherTeam(team: c_int) -> c_int {
    if team == TEAM_RED {
        TEAM_BLUE
    } else if team == TEAM_BLUE {
        TEAM_RED
    } else {
        team
    }
}

/// Raven `TeamName`.
///
/// Source: `oracle/codemp/game/g_team.c:45-53`
pub fn TeamName(team: c_int) -> *const c_char {
    if team == TEAM_RED {
        c"RED".as_ptr()
    } else if team == TEAM_BLUE {
        c"BLUE".as_ptr()
    } else if team == TEAM_SPECTATOR {
        c"SPECTATOR".as_ptr()
    } else {
        c"FREE".as_ptr()
    }
}

/// Raven `OtherTeamName`.
///
/// Source: `oracle/codemp/game/g_team.c:55-63`
pub fn OtherTeamName(team: c_int) -> *const c_char {
    if team == TEAM_RED {
        c"BLUE".as_ptr()
    } else if team == TEAM_BLUE {
        c"RED".as_ptr()
    } else if team == TEAM_SPECTATOR {
        c"SPECTATOR".as_ptr()
    } else {
        c"FREE".as_ptr()
    }
}

/// Raven `TeamColorString`.
///
/// Source: `oracle/codemp/game/g_team.c:65-73`
pub fn TeamColorString(team: c_int) -> *const c_char {
    if team == TEAM_RED {
        S_COLOR_RED.as_ptr()
    } else if team == TEAM_BLUE {
        S_COLOR_BLUE.as_ptr()
    } else if team == TEAM_SPECTATOR {
        S_COLOR_YELLOW.as_ptr()
    } else {
        S_COLOR_WHITE.as_ptr()
    }
}

/// Raven `PrintCTFMessage`.
///
/// plIndex used to print pl->client->pers.netname; teamIndex used to print
/// team name.
/// Source: `oracle/codemp/game/g_team.c:100-132`
pub fn PrintCTFMessage(ctx: &mut GameContext, plIndex: c_int, teamIndex: c_int, ctfMessage: c_int) {
    // MAX_CLIENTS not threaded through this packet's resolved surface; use
    // the Raven literal directly (g_team.c:106 hardcodes the same +1 idiom).
    let plIndex = if plIndex == -1 { 32 + 1 } else { plIndex };
    let teamIndex = if teamIndex == -1 { 50 } else { teamIndex };

    let te_id = G_TempEntity(ctx, [0.0, 0.0, 0.0], entity_event_t::EV_CTFMESSAGE as c_int);
    let te = ctx.world.entity_mut(te_id);
    te.r.svFlags |= SVF_BROADCAST;
    te.s.eventParm = ctfMessage;
    te.s.trickedentindex = plIndex;
    if ctfMessage == ctfMsg_t::CTFMESSAGE_PLAYER_CAPTURED_FLAG as c_int {
        te.s.trickedentindex2 = if teamIndex == TEAM_RED {
            TEAM_BLUE
        } else {
            TEAM_RED
        };
    } else {
        te.s.trickedentindex2 = teamIndex;
    }
}

/// Raven `AddTeamScore`.
///
/// Source: `oracle/codemp/game/g_team.c:142-179`
pub fn AddTeamScore(ctx: &mut GameContext, origin: vec3_t, team: c_int, score: c_int) {
    let te_id = G_TempEntity(ctx, origin, entity_event_t::EV_GLOBAL_TEAM_SOUND as c_int);

    let red = ctx.world.level.teamScores[TEAM_RED as usize];
    let blue = ctx.world.level.teamScores[TEAM_BLUE as usize];
    let event_parm = if team == TEAM_RED {
        if red + score == blue {
            global_team_sound_t::GTS_TEAMS_ARE_TIED as c_int
        } else if red <= blue && red + score > blue {
            global_team_sound_t::GTS_REDTEAM_TOOK_LEAD as c_int
        } else {
            global_team_sound_t::GTS_REDTEAM_SCORED as c_int
        }
    } else {
        if blue + score == red {
            global_team_sound_t::GTS_TEAMS_ARE_TIED as c_int
        } else if blue <= red && blue + score > red {
            global_team_sound_t::GTS_BLUETEAM_TOOK_LEAD as c_int
        } else {
            global_team_sound_t::GTS_BLUETEAM_SCORED as c_int
        }
    };

    let te = ctx.world.entity_mut(te_id);
    te.r.svFlags |= SVF_BROADCAST;
    te.s.eventParm = event_parm;

    ctx.world.level.teamScores[team as usize] += score;
}

/// Raven `OnSameTeam`.
///
/// Source: `oracle/codemp/game/g_team.c:187-276`
pub fn OnSameTeam(
    ctx: &mut GameContext,
    ent1: Option<EntityId>,
    ent2: Option<EntityId>,
) -> qboolean {
    // Option<EntityId> params. Entity fields route through the checked
    // arena accessor; the `.client` pointer is dereffed raw (see FLAG below).
    let (Some(ent1), Some(ent2)) = (ent1, ent2) else {
        return qfalse;
    };
    // FLAG (task #7): `.client` may be an NPC/vehicle pool `gclient_t` (`gClPtrs`,
    // g_utils.c:430), not a `level.clients` slot — read the pointer via the safe
    // entity borrow and deref it raw exactly as Raven does. (recipe 2c)
    let c1 = ctx.world.entity(ent1).client;
    let c2 = ctx.world.entity(ent2).client;
    unsafe {
        if c1.is_null() || c2.is_null() {
            return qfalse;
        }

        let gametype = ctx.world.cvars.g_gametype.integer;

        if gametype == GT_POWERDUEL as c_int {
            if (*c1).sess.duelTeam == (*c2).sess.duelTeam {
                return qtrue;
            }
            return qfalse;
        }

        if gametype == GT_SINGLE_PLAYER as c_int {
            let ent1IsBot = if ctx.world.entity(ent1).r.svFlags & SVF_BOT != 0 {
                qtrue
            } else {
                qfalse
            };
            let ent2IsBot = if ctx.world.entity(ent2).r.svFlags & SVF_BOT != 0 {
                qtrue
            } else {
                qfalse
            };

            if (ent1IsBot != 0 && ent2IsBot != 0) || (ent1IsBot == 0 && ent2IsBot == 0) {
                return qtrue;
            }
            return qfalse;
        }

        if gametype < GT_TEAM as c_int {
            return qfalse;
        }

        if ctx.world.entity(ent1).s.eType == entityType_t::ET_NPC as c_int
            && ctx.world.entity(ent1).s.NPC_class as c_int == class_t::CLASS_VEHICLE as c_int
            && !c1.is_null()
            && (*c1).sess.sessionTeam as c_int != TEAM_FREE
            && !c2.is_null()
            && (*c1).sess.sessionTeam as c_int == (*c2).sess.sessionTeam as c_int
        {
            return qtrue;
        }
        if ctx.world.entity(ent2).s.eType == entityType_t::ET_NPC as c_int
            && ctx.world.entity(ent2).s.NPC_class as c_int == class_t::CLASS_VEHICLE as c_int
            && !c2.is_null()
            && (*c2).sess.sessionTeam as c_int != TEAM_FREE
            && !c1.is_null()
            && (*c2).sess.sessionTeam as c_int == (*c1).sess.sessionTeam as c_int
        {
            return qtrue;
        }

        if (*c1).sess.sessionTeam as c_int == TEAM_FREE
            && (*c2).sess.sessionTeam as c_int == TEAM_FREE
            && ctx.world.entity(ent1).s.eType == entityType_t::ET_NPC as c_int
            && ctx.world.entity(ent2).s.eType == entityType_t::ET_NPC as c_int
        {
            return qfalse;
        }

        if ctx.world.entity(ent1).s.eType == entityType_t::ET_NPC as c_int
            && ctx.world.entity(ent2).s.eType == entityType_t::ET_PLAYER as c_int
        {
            if G_CheckVehicleNPCTeamDamage(Some(ctx.world.entity(ent1))) != 0 {
                if (*c1).sess.sessionTeam as c_int == (*c2).sess.sessionTeam as c_int
                    || ctx.world.entity(ent1).teamnodmg == (*c2).sess.sessionTeam as c_int
                {
                    return qtrue;
                }
            }
            return qfalse;
        } else if ctx.world.entity(ent1).s.eType == entityType_t::ET_PLAYER as c_int
            && ctx.world.entity(ent2).s.eType == entityType_t::ET_NPC as c_int
        {
            return qfalse;
        }

        if (*c1).sess.sessionTeam as c_int == (*c2).sess.sessionTeam as c_int {
            return qtrue;
        }

        qfalse
    }
}

/// Raven `Team_SetFlagStatus`.
///
/// Source: `oracle/codemp/game/g_team.c:281-318`
pub fn Team_SetFlagStatus(ctx: &mut GameContext, team: c_int, status: flagStatus_t) {
    unsafe {
        let mut modified = qfalse;

        match team {
            TEAM_RED => {
                if ctx.world.globals.teamgame.redStatus != status {
                    ctx.world.globals.teamgame.redStatus = status;
                    modified = qtrue;
                }
            }
            TEAM_BLUE => {
                if ctx.world.globals.teamgame.blueStatus != status {
                    ctx.world.globals.teamgame.blueStatus = status;
                    modified = qtrue;
                }
            }
            TEAM_FREE => {
                if ctx.world.globals.teamgame.flagStatus != status {
                    ctx.world.globals.teamgame.flagStatus = status;
                    modified = qtrue;
                }
            }
            _ => {}
        }

        if modified != 0 {
            let ctfFlagStatusRemap: &[u8] = &[b'0', b'1', b'*', b'*', b'2'];
            // §19: oracle reads `char st[4]` uninitialized outside CTF/CTY (UB);
            // port zero-inits, sending "". Source: g_team.c:308-314
            let mut st: [c_char; 4] = [0; 4];

            if ctx.world.cvars.g_gametype.integer == GT_CTF as c_int
                || ctx.world.cvars.g_gametype.integer == GT_CTY as c_int
            {
                st[0] = ctfFlagStatusRemap[ctx.world.globals.teamgame.redStatus as usize] as c_char;
                st[1] =
                    ctfFlagStatusRemap[ctx.world.globals.teamgame.blueStatus as usize] as c_char;
                st[2] = 0;
            }

            trap::SetConfigstring(
                ctx.engine,
                mp_abi::game::syscalls::G_SET_CONFIGSTRING::GSetConfigstringArgs::new(
                    CS_FLAGSTATUS,
                    cstr(&cstr_to_str(st.as_ptr())),
                ),
            );
        }
    }
}

/// Raven `Team_CheckDroppedItem`.
///
/// Source: `oracle/codemp/game/g_team.c:320-330`
pub fn Team_CheckDroppedItem(ctx: &mut GameContext, dropped: EntityId) {
    let item = ctx.world.entity(dropped).item;
    // Only flag items reach here (LaunchItem's CTF branch).
    let ItemKind::Team(giTag) = item.unwrap().item().kind else {
        unreachable!("Team_CheckDroppedItem on non-flag item");
    };
    if giTag == PW_REDFLAG {
        Team_SetFlagStatus(ctx, TEAM_RED, FLAG_DROPPED);
    } else if giTag == PW_BLUEFLAG {
        Team_SetFlagStatus(ctx, TEAM_BLUE, FLAG_DROPPED);
    } else if giTag == PW_NEUTRALFLAG {
        Team_SetFlagStatus(ctx, TEAM_FREE, FLAG_DROPPED);
    }
}

/// Raven `Team_ForceGesture`.
///
/// Source: `oracle/codemp/game/g_team.c:337-352`
pub fn Team_ForceGesture(ctx: &mut GameContext, team: c_int) {
    // Oracle loops fixed `MAX_CLIENTS`, not `g_maxclients.integer`.
    // Source: g_team.c:341
    for i in 0..MAX_CLIENTS {
        let id = EntityId(i as u32);
        if ctx.world.entity(id).inuse == 0 {
            continue;
        }
        // FLAG (task #7): `.client` read via the safe entity borrow, dereffed raw
        // as Raven does. (recipe 2c)
        let client = ctx.world.entity(id).client;
        if client.is_null() {
            continue;
        }
        if unsafe { (*client).sess.sessionTeam } as c_int != team {
            continue;
        }

        ctx.world.entity_mut(id).flags |= FL_FORCE_GESTURE;
    }
}

/// Raven `Team_FragBonuses`.
///
/// Source: `oracle/codemp/game/g_team.c:363-534`
pub fn Team_FragBonuses(
    ctx: &mut GameContext,
    targ: EntityId,
    inflictor: Option<EntityId>,
    attacker: Option<EntityId>,
) {
    // EntityId targ + Option<EntityId> inflictor/attacker (`inflictor`
    // unused, as in Raven). Entity fields route through the checked arena
    // accessor; `.client` pointers are dereffed raw (see FLAG below).
    let Some(attacker) = attacker else {
        return;
    };

    // no bonus for fragging yourself or team mates
    if ctx.world.entity(targ).client.is_null()
        || ctx.world.entity(attacker).client.is_null()
        || targ == attacker
        || OnSameTeam(ctx, Some(targ), Some(attacker)) != 0
    {
        return;
    }

    // FLAG (task #7): `.client` read via the safe entity borrow, dereffed raw as
    // Raven does (may be an NPC pool `gclient_t`, `gClPtrs`). (recipe 2c)
    let targ_cl = ctx.world.entity(targ).client;
    let attacker_cl = ctx.world.entity(attacker).client;
    // Entity origins are not mutated in this function; read once.
    let targ_origin = ctx.world.entity(targ).r.currentOrigin;
    let attacker_origin = ctx.world.entity(attacker).r.currentOrigin;

    unsafe {
        let team = (*targ_cl).sess.sessionTeam as c_int;
        let otherteam = OtherTeam(team);
        if otherteam < 0 {
            return; // whoever died isn't on a team
        }

        // same team, if the flag at base, check to he has the enemy flag
        let (flag_pw, enemy_flag_pw) = if team == TEAM_RED {
            (PW_REDFLAG, PW_BLUEFLAG)
        } else {
            (PW_BLUEFLAG, PW_REDFLAG)
        };

        // did the attacker frag the flag carrier?
        // Oracle sets `tokens = 0` here (g_team.c:394) and never changes it, so the
        // `if (tokens)` block below is dead. Preserved as a dead branch (porting-rules §20).
        let tokens = 0;
        if (*targ_cl).ps.powerups[enemy_flag_pw as usize] != 0 {
            (*attacker_cl).pers.teamState.lastfraggedcarrier = ctx.world.level.time as f32;
            AddScore(ctx, attacker, targ_origin, CTF_FRAG_CARRIER_BONUS);
            (*attacker_cl).pers.teamState.fragcarrier += 1;
            let num = ctx.world.entity(attacker).s.number;
            PrintCTFMessage(
                ctx,
                num,
                team,
                ctfMsg_t::CTFMESSAGE_FRAGGED_FLAG_CARRIER as c_int,
            );

            // the target had the flag, clear the hurt carrier field on the other team
            let max_clients = ctx.world.cvars.g_maxclients.integer;
            for i in 0..max_clients {
                let eid = EntityId(i as u32);
                let ec = ctx.world.entity(eid).client;
                if ctx.world.entity(eid).inuse != 0 && (*ec).sess.sessionTeam as c_int == otherteam
                {
                    (*ec).pers.teamState.lasthurtcarrier = 0.0;
                }
            }
            return;
        }

        // did the attacker frag a head carrier? other->client->ps.generic1
        // Dead branch in oracle: `tokens` is always 0 (see above). g_team.c:413-429.
        if tokens != 0 {
            (*attacker_cl).pers.teamState.lastfraggedcarrier = ctx.world.level.time as f32;
            AddScore(
                ctx,
                attacker,
                targ_origin,
                CTF_FRAG_CARRIER_BONUS * tokens * tokens,
            );
            (*attacker_cl).pers.teamState.fragcarrier += 1;

            // the target had the flag, clear the hurt carrier field on the other team
            let max_clients = ctx.world.cvars.g_maxclients.integer;
            for i in 0..max_clients {
                let eid = EntityId(i as u32);
                let ec = ctx.world.entity(eid).client;
                if ctx.world.entity(eid).inuse != 0 && (*ec).sess.sessionTeam as c_int == otherteam
                {
                    (*ec).pers.teamState.lasthurtcarrier = 0.0;
                }
            }
            return;
        }

        if (*targ_cl).pers.teamState.lasthurtcarrier != 0.0
            && (ctx.world.level.time as f32) - (*targ_cl).pers.teamState.lasthurtcarrier < 8000.0 // CTF_CARRIER_DANGER_PROTECT_TIMEOUT
            && (*attacker_cl).ps.powerups[flag_pw as usize] == 0
        {
            // attacker is on the same team as the flag carrier and fragged a guy who hurt our flag carrier
            AddScore(ctx, attacker, targ_origin, CTF_CARRIER_DANGER_PROTECT_BONUS);

            (*attacker_cl).pers.teamState.carrierdefense += 1;
            (*targ_cl).pers.teamState.lasthurtcarrier = 0.0;

            (*attacker_cl).ps.persistant[persEnum_t::PERS_DEFEND_COUNT as usize] += 1;
            let _team = (*attacker_cl).sess.sessionTeam as c_int;
            (*attacker_cl).rewardTime = ctx.world.level.time + 2000;

            return;
        }

        if (*targ_cl).pers.teamState.lasthurtcarrier != 0.0
            && (ctx.world.level.time as f32) - (*targ_cl).pers.teamState.lasthurtcarrier < 8000.0
        // CTF_CARRIER_DANGER_PROTECT_TIMEOUT
        {
            // attacker is on the same team as the skull carrier and
            AddScore(ctx, attacker, targ_origin, CTF_CARRIER_DANGER_PROTECT_BONUS);

            (*attacker_cl).pers.teamState.carrierdefense += 1;
            (*targ_cl).pers.teamState.lasthurtcarrier = 0.0;

            (*attacker_cl).ps.persistant[persEnum_t::PERS_DEFEND_COUNT as usize] += 1;
            let _team = (*attacker_cl).sess.sessionTeam as c_int;
            (*attacker_cl).rewardTime = ctx.world.level.time + 2000;

            return;
        }

        // flag and flag carrier area defense bonuses

        // we have to find the flag and carrier entities

        // find the flag
        let c = if (*attacker_cl).sess.sessionTeam as c_int == TEAM_RED {
            c"team_CTF_redflag"
        } else if (*attacker_cl).sess.sessionTeam as c_int == TEAM_BLUE {
            c"team_CTF_blueflag"
        } else {
            return;
        };

        // find attacker's team's flag carrier
        let mut carrier: Option<EntityId> = None;
        let max_clients = ctx.world.cvars.g_maxclients.integer;
        for i in 0..max_clients {
            let cid = EntityId(i as u32);
            carrier = Some(cid);
            let cl = ctx.world.entity(cid).client;
            if ctx.world.entity(cid).inuse != 0 && (*cl).ps.powerups[flag_pw as usize] != 0 {
                break;
            }
            carrier = None;
        }

        let mut flag: Option<EntityId> = None;
        loop {
            let f = G_Find(ctx, flag, fofs_classname(), c.as_ptr());
            flag = ctx.entity_id_of(f);
            if f.is_null() {
                break;
            }
            if ctx.world.entity(flag.unwrap()).flags & FL_DROPPED_ITEM == 0 {
                break;
            }
        }

        let Some(flag) = flag else {
            return; // can't find attacker's flag
        };
        let flag_origin = ctx.world.entity(flag).r.currentOrigin;

        // ok we have the attackers flag and a pointer to the carrier

        // check to see if we are defending the base's flag
        let mut v1 = [0.0f32; 3];
        let mut v2 = [0.0f32; 3];
        v1[0] = targ_origin[0] - flag_origin[0];
        v1[1] = targ_origin[1] - flag_origin[1];
        v1[2] = targ_origin[2] - flag_origin[2];
        v2[0] = attacker_origin[0] - flag_origin[0];
        v2[1] = attacker_origin[1] - flag_origin[1];
        v2[2] = attacker_origin[2] - flag_origin[2];

        let v1_len = (v1[0] * v1[0] + v1[1] * v1[1] + v1[2] * v1[2]).sqrt();
        let v2_len = (v2[0] * v2[0] + v2[1] * v2[1] + v2[2] * v2[2]).sqrt();

        // CTF_TARGET_PROTECT_RADIUS = 1000 (g_team.h:17)
        if ((v1_len < 1000.0
            && trap::InPVS(
                ctx.engine,
                mp_abi::game::syscalls::G_IN_PVS::GInPvsArgs::new(
                    &flag_origin as *const vec3_t,
                    &targ_origin as *const vec3_t,
                ),
            ) != 0)
            || (v2_len < 1000.0
                && trap::InPVS(
                    ctx.engine,
                    mp_abi::game::syscalls::G_IN_PVS::GInPvsArgs::new(
                        &flag_origin as *const vec3_t,
                        &attacker_origin as *const vec3_t,
                    ),
                ) != 0))
            && (*attacker_cl).sess.sessionTeam as c_int != (*targ_cl).sess.sessionTeam as c_int
        {
            // we defended the base flag
            AddScore(ctx, attacker, targ_origin, CTF_FLAG_DEFENSE_BONUS);
            (*attacker_cl).pers.teamState.basedefense += 1;

            (*attacker_cl).ps.persistant[persEnum_t::PERS_DEFEND_COUNT as usize] += 1;
            (*attacker_cl).rewardTime = ctx.world.level.time + 2000;

            return;
        }

        if let Some(carrier) = carrier.filter(|&c| c != attacker) {
            let carrier_origin = ctx.world.entity(carrier).r.currentOrigin;
            // Oracle typo (g_team.c:517-518): VectorSubtract writes v1 on BOTH lines,
            // so v2 is never recomputed here and stays stale (attacker-flag from the
            // base-flag block above). Preserved verbatim (porting-rules §19).
            v1[0] = targ_origin[0] - carrier_origin[0];
            v1[1] = targ_origin[1] - carrier_origin[1];
            v1[2] = targ_origin[2] - carrier_origin[2];
            v1[0] = attacker_origin[0] - carrier_origin[0];
            v1[1] = attacker_origin[1] - carrier_origin[1];
            v1[2] = attacker_origin[2] - carrier_origin[2];

            let v1_len = (v1[0] * v1[0] + v1[1] * v1[1] + v1[2] * v1[2]).sqrt();
            let v2_len = (v2[0] * v2[0] + v2[1] * v2[1] + v2[2] * v2[2]).sqrt();

            // CTF_ATTACKER_PROTECT_RADIUS = 1000 (g_team.h:18)
            if ((v1_len < 1000.0
                && trap::InPVS(
                    ctx.engine,
                    mp_abi::game::syscalls::G_IN_PVS::GInPvsArgs::new(
                        &carrier_origin as *const vec3_t,
                        &targ_origin as *const vec3_t,
                    ),
                ) != 0)
                || (v2_len < 1000.0
                    && trap::InPVS(
                        ctx.engine,
                        mp_abi::game::syscalls::G_IN_PVS::GInPvsArgs::new(
                            &carrier_origin as *const vec3_t,
                            &attacker_origin as *const vec3_t,
                        ),
                    ) != 0))
                && (*attacker_cl).sess.sessionTeam as c_int != (*targ_cl).sess.sessionTeam as c_int
            {
                AddScore(ctx, attacker, targ_origin, CTF_CARRIER_PROTECT_BONUS);
                (*attacker_cl).pers.teamState.carrierdefense += 1;

                (*attacker_cl).ps.persistant[persEnum_t::PERS_DEFEND_COUNT as usize] += 1;
                (*attacker_cl).rewardTime = ctx.world.level.time + 2000;

                return;
            }
        }
    }
}

/// Raven `Team_CheckHurtCarrier`.
///
/// Source: `oracle/codemp/game/g_team.c:544-565`
pub fn Team_CheckHurtCarrier(
    ctx: &mut GameContext,
    targ: Option<EntityId>,
    attacker: Option<EntityId>,
) {
    // Option<EntityId> params. `.client` pointers dereffed raw (FLAG).
    let (Some(targ), Some(attacker)) = (targ, attacker) else {
        return;
    };
    if ctx.world.entity(targ).client.is_null() || ctx.world.entity(attacker).client.is_null() {
        return;
    }

    // FLAG (task #7): `.client` read via the safe entity borrow, dereffed raw as
    // Raven does. (recipe 2c)
    let targ_cl = ctx.world.entity(targ).client;
    let attacker_cl = ctx.world.entity(attacker).client;
    let time = ctx.world.level.time as f32;
    unsafe {
        let flag_pw = if (*targ_cl).sess.sessionTeam as c_int == TEAM_RED {
            PW_BLUEFLAG
        } else {
            PW_REDFLAG
        };

        // flags
        if (*targ_cl).ps.powerups[flag_pw as usize] != 0
            && (*targ_cl).sess.sessionTeam as c_int != (*attacker_cl).sess.sessionTeam as c_int
        {
            (*attacker_cl).pers.teamState.lasthurtcarrier = time;
        }

        // skulls
        if (*targ_cl).ps.generic1 != 0
            && (*targ_cl).sess.sessionTeam as c_int != (*attacker_cl).sess.sessionTeam as c_int
        {
            (*attacker_cl).pers.teamState.lasthurtcarrier = time;
        }
    }
}

/// Raven `Team_ResetFlag`.
///
/// Source: `oracle/codemp/game/g_team.c:568-599`
pub fn Team_ResetFlag(ctx: &mut GameContext, team: c_int) -> Option<EntityId> {
    let classname: &CStr = if team == TEAM_RED {
        c"team_CTF_redflag"
    } else if team == TEAM_BLUE {
        c"team_CTF_blueflag"
    } else if team == TEAM_FREE {
        c"team_CTF_neutralflag"
    } else {
        return None;
    };

    let mut ent: Option<EntityId> = None;
    let mut rent: Option<EntityId> = None;
    loop {
        let e = G_Find(ctx, ent, fofs_classname(), classname.as_ptr());
        ent = ctx.entity_id_of(e);
        if e.is_null() {
            break;
        }
        let ent_id = ent.unwrap();
        if ctx.world.entity(ent_id).flags & FL_DROPPED_ITEM != 0 {
            G_FreeEntity(ctx, ent);
        } else {
            rent = ent;
            RespawnItem(ctx, ent_id);
        }
    }

    Team_SetFlagStatus(ctx, team, FLAG_ATBASE);

    rent
}

/// Raven `Team_ResetFlags`.
///
/// Source: `oracle/codemp/game/g_team.c:601-606`
pub fn Team_ResetFlags(ctx: &mut GameContext) {
    if ctx.world.cvars.g_gametype.integer == GT_CTF as c_int
        || ctx.world.cvars.g_gametype.integer == GT_CTY as c_int
    {
        Team_ResetFlag(ctx, TEAM_RED);
        Team_ResetFlag(ctx, TEAM_BLUE);
    }
}

/// Raven `Team_ReturnFlagSound`.
///
/// Source: `oracle/codemp/game/g_team.c:608-624`
pub fn Team_ReturnFlagSound(ctx: &mut GameContext, ent: Option<EntityId>, team: c_int) {
    let Some(ent) = ent else {
        // G_Printf(ctx, "Warning:  NULL passed to Team_ReturnFlagSound\n") —
        // logging trap not resolved in this packet's call surface; behavior
        // (early return) preserved, message dropped.
        return;
    };

    let trbase = ctx.world.entity(ent).s.pos.trBase;
    let te_id = G_TempEntity(ctx, trbase, entity_event_t::EV_GLOBAL_TEAM_SOUND as c_int);
    let te = ctx.world.entity_mut(te_id);
    te.s.eventParm = if team == TEAM_BLUE {
        global_team_sound_t::GTS_RED_RETURN as c_int
    } else {
        global_team_sound_t::GTS_BLUE_RETURN as c_int
    };
    te.r.svFlags |= SVF_BROADCAST;
}

/// Raven `Team_TakeFlagSound`.
///
/// Source: `oracle/codemp/game/g_team.c:626-662`
pub fn Team_TakeFlagSound(ctx: &mut GameContext, ent: Option<EntityId>, team: c_int) {
    let Some(ent) = ent else {
        // G_Printf ("Warning:  NULL passed to Team_TakeFlagSound\n");
        return;
    };

    // only play sound when the flag was at the base
    // or not picked up the last 10 seconds
    match team {
        TEAM_RED => {
            if ctx.world.globals.teamgame.blueStatus != FLAG_ATBASE {
                if ctx.world.globals.teamgame.blueTakenTime > ctx.world.level.time - 10000 {
                    return;
                }
            }
            ctx.world.globals.teamgame.blueTakenTime = ctx.world.level.time;
        }
        TEAM_BLUE => {
            if ctx.world.globals.teamgame.redStatus != FLAG_ATBASE {
                if ctx.world.globals.teamgame.redTakenTime > ctx.world.level.time - 10000 {
                    return;
                }
            }
            ctx.world.globals.teamgame.redTakenTime = ctx.world.level.time;
        }
        _ => {}
    }

    let trbase = ctx.world.entity(ent).s.pos.trBase;
    let te_id = G_TempEntity(ctx, trbase, entity_event_t::EV_GLOBAL_TEAM_SOUND as c_int);
    let te = ctx.world.entity_mut(te_id);
    if team == TEAM_BLUE {
        te.s.eventParm = global_team_sound_t::GTS_RED_TAKEN as c_int;
    } else {
        te.s.eventParm = global_team_sound_t::GTS_BLUE_TAKEN as c_int;
    }
    te.r.svFlags |= SVF_BROADCAST;
}

/// Raven `Team_CaptureFlagSound`.
///
/// Source: `oracle/codemp/game/g_team.c:664-680`
pub fn Team_CaptureFlagSound(ctx: &mut GameContext, ent: Option<EntityId>, team: c_int) {
    let Some(ent) = ent else {
        // G_Printf(ctx, "Warning:  NULL passed to Team_CaptureFlagSound\n") —
        // logging trap not resolved in this packet's call surface.
        return;
    };

    let trbase = ctx.world.entity(ent).s.pos.trBase;
    let te_id = G_TempEntity(ctx, trbase, entity_event_t::EV_GLOBAL_TEAM_SOUND as c_int);
    let te = ctx.world.entity_mut(te_id);
    te.s.eventParm = if team == TEAM_BLUE {
        global_team_sound_t::GTS_BLUE_CAPTURE as c_int
    } else {
        global_team_sound_t::GTS_RED_CAPTURE as c_int
    };
    te.r.svFlags |= SVF_BROADCAST;
}

/// Raven `Team_ReturnFlag`.
///
/// Source: `oracle/codemp/game/g_team.c:682-691`
pub fn Team_ReturnFlag(ctx: &mut GameContext, team: c_int) {
    let flag = Team_ResetFlag(ctx, team);
    Team_ReturnFlagSound(ctx, flag, team);
    if team == TEAM_FREE {
        // PrintMsg(NULL, "The flag has returned!\n") — dead (StringEd-only
        // client-side messaging, g_team.c:685).
    } else {
        // flag should always have team in normal CTF
        PrintCTFMessage(ctx, -1, team, ctfMsg_t::CTFMESSAGE_FLAG_RETURNED as c_int);
    }
}

/// Raven `Team_FreeEntity`.
///
/// Source: `oracle/codemp/game/g_team.c:693-703`
pub fn Team_FreeEntity(ctx: &mut GameContext, ent: EntityId) {
    let item = ctx.world.entity(ent).item;
    // Only flag items reach here (G_RunItem's nodrop branch Team match).
    let ItemKind::Team(giTag) = item.unwrap().item().kind else {
        unreachable!("Team_FreeEntity on non-flag item");
    };
    if giTag == PW_REDFLAG {
        Team_ReturnFlag(ctx, TEAM_RED);
    } else if giTag == PW_BLUEFLAG {
        Team_ReturnFlag(ctx, TEAM_BLUE);
    } else if giTag == PW_NEUTRALFLAG {
        Team_ReturnFlag(ctx, TEAM_FREE);
    }
}

/// Raven `Team_DroppedFlagThink`.
///
/// Automatically set in `Launch_Item` if the item is one of the flags. Flags
/// are unique in that if they are dropped, the base flag must be respawned
/// when they time out. Stored as a fn pointer (`EntThink`) — the fn-ID enum
/// wiring for the assignment site is separate from this body.
/// Source: `oracle/codemp/game/g_team.c:714-729`
pub fn Team_DroppedFlagThink(ctx: &mut GameContext, ent: EntityId) {
    let item = ctx.world.entity(ent).item;
    // Only flag items carry this think (LaunchItem's CTF branch).
    let ItemKind::Team(giTag) = item.unwrap().item().kind else {
        unreachable!("Team_DroppedFlagThink on non-flag item");
    };
    let team = if giTag == PW_REDFLAG {
        TEAM_RED
    } else if giTag == PW_BLUEFLAG {
        TEAM_BLUE
    } else {
        TEAM_FREE
    };

    // Team_ResetFlag will delete this entity.
    let flag = Team_ResetFlag(ctx, team);
    Team_ReturnFlagSound(ctx, flag, team);
}

/// Raven `Team_TouchOurFlag`.
///
/// Source: `oracle/codemp/game/g_team.c:737-825`
pub fn Team_TouchOurFlag(
    ctx: &mut GameContext,
    ent: EntityId,
    other: EntityId,
    team: c_int,
) -> c_int {
    // EntityId params. Entity fields route through the checked arena
    // accessor; the `.client` pointer is dereffed raw (see FLAG below).
    // FLAG (task #7): `.client` read via the safe entity borrow, dereffed raw as
    // Raven does. (recipe 2c)
    let cl = ctx.world.entity(other).client;
    let num = ctx.world.entity(other).s.number;
    // `ent` (the flag) origins are not mutated before use; read once.
    let ent_origin = ctx.world.entity(ent).r.currentOrigin;
    let ent_trbase = ctx.world.entity(ent).s.pos.trBase;
    let ent_dropped = ctx.world.entity(ent).flags & FL_DROPPED_ITEM != 0;

    unsafe {
        let enemy_flag = if (*cl).sess.sessionTeam as c_int == TEAM_RED {
            PW_BLUEFLAG
        } else {
            PW_REDFLAG
        };

        if ent_dropped {
            // flag is not at home, return it by teleporting it back
            PrintCTFMessage(
                ctx,
                num,
                team,
                ctfMsg_t::CTFMESSAGE_PLAYER_RETURNED_FLAG as c_int,
            );
            AddScore(ctx, other, ent_origin, CTF_RECOVERY_BONUS);
            (*cl).pers.teamState.flagrecovery += 1;
            (*cl).pers.teamState.lastreturnedflag = ctx.world.level.time as f32;
            let reset_flag_id = Team_ResetFlag(ctx, team);
            Team_ReturnFlagSound(ctx, reset_flag_id, team);
            return 0;
        }

        // the flag is at home base. if the player has the enemy flag, he's just won!
        if (*cl).ps.powerups[enemy_flag as usize] == 0 {
            return 0; // We don't have the flag
        }

        PrintCTFMessage(
            ctx,
            num,
            team,
            ctfMsg_t::CTFMESSAGE_PLAYER_CAPTURED_FLAG as c_int,
        );

        (*cl).ps.powerups[enemy_flag as usize] = 0;

        ctx.world.globals.teamgame.last_flag_capture = ctx.world.level.time as f32;
        ctx.world.globals.teamgame.last_capture_team = team;

        // Increase the team's score
        let cl_team = (*cl).sess.sessionTeam as c_int;
        AddTeamScore(ctx, ent_trbase, cl_team, 1);

        (*cl).pers.teamState.captures += 1;
        (*cl).rewardTime = ctx.world.level.time + 2000; // REWARD_SPRITE_TIME
        (*cl).ps.persistant[persEnum_t::PERS_CAPTURES as usize] += 1;

        // other gets another 10 frag bonus
        AddScore(ctx, other, ent_origin, CTF_CAPTURE_BONUS);

        Team_CaptureFlagSound(ctx, Some(ent), team);

        // Ok, let's do the player loop, hand out the bonuses
        let max_clients = ctx.world.cvars.g_maxclients.integer;
        for i in 0..max_clients {
            let pid = EntityId(i as u32);
            if ctx.world.entity(pid).inuse == 0 {
                continue;
            }

            // FLAG (task #7): player `.client` dereffed raw as Raven does. (recipe 2c)
            let pcl = ctx.world.entity(pid).client;
            if (*pcl).sess.sessionTeam as c_int != (*cl).sess.sessionTeam as c_int {
                (*pcl).pers.teamState.lasthurtcarrier = -5.0;
            } else if (*pcl).sess.sessionTeam as c_int == (*cl).sess.sessionTeam as c_int {
                if pid != other {
                    AddScore(ctx, pid, ent_origin, CTF_TEAM_BONUS);
                }
                // award extra points for capture assists
                if (*pcl).pers.teamState.lastreturnedflag + 10000.0 > ctx.world.level.time as f32 {
                    // CTF_RETURN_FLAG_ASSIST_TIMEOUT = 10000 (g_team.h:22)
                    AddScore(ctx, pid, ent_origin, CTF_RETURN_FLAG_ASSIST_BONUS);
                    (*cl).pers.teamState.assists += 1;

                    (*pcl).ps.persistant[persEnum_t::PERS_ASSIST_COUNT as usize] += 1;
                    (*pcl).rewardTime = ctx.world.level.time + 2000;
                } else if (*pcl).pers.teamState.lastfraggedcarrier + 10000.0
                    > ctx.world.level.time as f32
                {
                    // CTF_FRAG_CARRIER_ASSIST_TIMEOUT = 10000 (g_team.h:21)
                    AddScore(ctx, pid, ent_origin, CTF_FRAG_CARRIER_ASSIST_BONUS);
                    (*cl).pers.teamState.assists += 1;
                    (*pcl).ps.persistant[persEnum_t::PERS_ASSIST_COUNT as usize] += 1;
                    (*pcl).rewardTime = ctx.world.level.time + 2000;
                }
            }
        }
        Team_ResetFlags(ctx);

        CalculateRanks(ctx);

        0 // Do not respawn this automatically
    }
}

/// Raven `Team_TouchEnemyFlag`.
///
/// Source: `oracle/codemp/game/g_team.c:827-846`
pub fn Team_TouchEnemyFlag(
    ctx: &mut GameContext,
    ent: EntityId,
    other: EntityId,
    team: c_int,
) -> c_int {
    // EntityId params. Entity fields route through the checked arena
    // accessor; the `.client` pointer is dereffed raw (see FLAG below).
    // FLAG (task #7): `.client` read via the safe entity borrow, dereffed raw as
    // Raven does. (recipe 2c)
    let cl = ctx.world.entity(other).client;
    let num = ctx.world.entity(other).s.number;
    let ent_origin = ctx.world.entity(ent).r.currentOrigin;
    unsafe {
        PrintCTFMessage(
            ctx,
            num,
            team,
            ctfMsg_t::CTFMESSAGE_PLAYER_GOT_FLAG as c_int,
        );

        if team == TEAM_RED {
            (*cl).ps.powerups[PW_REDFLAG as usize] = c_int::MAX; // flags never expire
        } else {
            (*cl).ps.powerups[PW_BLUEFLAG as usize] = c_int::MAX;
        }

        Team_SetFlagStatus(ctx, team, 1); // FLAG_TAKEN

        AddScore(ctx, other, ent_origin, CTF_FLAG_BONUS);
        (*cl).pers.teamState.flagsince = ctx.world.level.time as f32;
        Team_TakeFlagSound(ctx, Some(ent), team);

        -1 // Do not respawn this automatically, but do delete it if it was FL_DROPPED
    }
}

/// Raven `Pickup_Team`.
///
/// Source: `oracle/codemp/game/g_team.c:848-871`
pub fn Pickup_Team(ctx: &mut GameContext, ent: EntityId, other: EntityId) -> c_int {
    // FLAG (task #7): `.classname` is a `*mut c_char` (bg/spawn string), read raw
    // via the safe entity borrow. (recipe 2c)
    let classname_ptr = ctx.world.entity(ent).classname;
    let classname = unsafe { CStr::from_ptr(classname_ptr) };
    let team = if classname == c"team_CTF_redflag" {
        TEAM_RED
    } else if classname == c"team_CTF_blueflag" {
        TEAM_BLUE
    } else if classname == c"team_CTF_neutralflag" {
        TEAM_FREE
    } else {
        // PrintMsg(other, "Don't know what team the flag is on.\n") — dead.
        return 0;
    };

    // FLAG (task #7): `.client` read via the safe entity borrow, dereffed raw. (recipe 2c)
    let cl = ctx.world.entity(other).client;
    let cl_team = unsafe { (*cl).sess.sessionTeam } as c_int;
    if team == cl_team {
        Team_TouchOurFlag(ctx, ent, other, team)
    } else {
        Team_TouchEnemyFlag(ctx, ent, other, team)
    }
}

/// Raven `Team_GetLocation`.
///
/// Source: `oracle/codemp/game/g_team.c:880-909`
pub fn Team_GetLocation(ctx: &mut GameContext, ent: EntityId) -> Option<EntityId> {
    let mut best: Option<EntityId> = None;
    let mut bestlen: f32 = 3.0 * 8192.0 * 8192.0;

    let origin = ctx.world.entity(ent).r.currentOrigin;

    // `level.locationHead` is a raw `*mut gentity_t` chain-head; convert it to a
    // handle and walk `nextTrain` (already `Option<EntityId>`) through the arena.
    let mut eloc = ctx.entity_id_of(ctx.world.level.locationHead);
    while let Some(eid) = eloc {
        let eloc_origin = ctx.world.entity(eid).r.currentOrigin;
        let len = (origin[0] - eloc_origin[0]) * (origin[0] - eloc_origin[0])
            + (origin[1] - eloc_origin[1]) * (origin[1] - eloc_origin[1])
            + (origin[2] - eloc_origin[2]) * (origin[2] - eloc_origin[2]);

        if len > bestlen {
            eloc = ctx.world.entity(eid).nextTrain;
            continue;
        }

        if trap::InPVS(
            ctx.engine,
            mp_abi::game::syscalls::G_IN_PVS::GInPvsArgs::new(
                &origin as *const vec3_t,
                &eloc_origin as *const vec3_t,
            ),
        ) == 0
        {
            eloc = ctx.world.entity(eid).nextTrain;
            continue;
        }

        bestlen = len;
        best = Some(eid);
        eloc = ctx.world.entity(eid).nextTrain;
    }

    best
}

/// Raven `Team_GetLocationMsg`.
///
/// Source: `oracle/codemp/game/g_team.c:919-938`
pub fn Team_GetLocationMsg(
    ctx: &mut GameContext,
    ent: EntityId,
    loc: *mut c_char,
    loclen: c_int,
) -> qboolean {
    let Some(best) = Team_GetLocation(ctx, ent) else {
        return qfalse;
    };

    // FLAG (task #7): `.message` is a `*mut c_char` (spawn string), read raw via
    // the safe entity borrow. (recipe 2c)
    let message_ptr = ctx.world.entity(best).message;
    let message = unsafe { CStr::from_ptr(message_ptr) }.to_string_lossy();

    // Oracle gates on the original `best->count`, then clamps and writes the
    // clamped value back into the entity (g_team.c:928-933).
    let formatted = if ctx.world.entity(best).count != 0 {
        if ctx.world.entity(best).count < 0 {
            ctx.world.entity_mut(best).count = 0;
        }
        if ctx.world.entity(best).count > 7 {
            ctx.world.entity_mut(best).count = 7;
        }
        let count = ctx.world.entity(best).count;
        format!("^{}{}{}", (count as u8 + b'0') as char, message, "^7")
    } else {
        message.to_string()
    };

    // FLAG (task #7): `loc` is an engine-owned out-buffer; the raw slice write
    // stays raw. (recipe 2c)
    let loc_slice = unsafe { core::slice::from_raw_parts_mut(loc, loclen as usize) };
    write_cstr_field(loc_slice, &formatted);

    qtrue
}

/// Raven `SelectRandomTeamSpawnPoint`.
///
/// Source: `oracle/codemp/game/g_team.c:951-1040`
pub fn SelectRandomTeamSpawnPoint(
    ctx: &mut GameContext,
    teamstate: c_int,
    team: team_t,
    siegeClass: c_int,
) -> Option<EntityId> {
    let classname: &CStr = if ctx.world.cvars.g_gametype.integer == GT_SIEGE as c_int {
        if team == SIEGETEAM_TEAM1 as team_t {
            c"info_player_siegeteam1"
        } else {
            c"info_player_siegeteam2"
        }
    } else {
        if teamstate == TEAM_BEGIN {
            if team == TEAM_RED as team_t {
                c"team_CTF_redplayer"
            } else if team == TEAM_BLUE as team_t {
                c"team_CTF_blueplayer"
            } else {
                return None;
            }
        } else {
            if team == TEAM_RED as team_t {
                c"team_CTF_redspawn"
            } else if team == TEAM_BLUE as team_t {
                c"team_CTF_bluespawn"
            } else {
                return None;
            }
        }
    };

    let mustBeEnabled = ctx.world.cvars.g_gametype.integer == GT_SIEGE as c_int;

    // MAX_TEAM_SPAWN_POINTS = 32 (g_team.c:950)
    let mut count: c_int = 0;
    let mut spots: [Option<EntityId>; 32] = [None; 32];
    let mut spot: Option<EntityId> = None;

    loop {
        // Oracle's `while ((spot = G_Find(...)) != NULL)` — break on null before
        // taking an id (the STAGE-1 body unwrapped pre-null-check, which would
        // panic on the terminating iteration; oracle just exits the loop).
        let s = G_Find(ctx, spot, fofs_classname(), classname.as_ptr());
        spot = ctx.entity_id_of(s);
        if s.is_null() {
            break;
        }
        let spot_id = spot.unwrap();

        if SpotWouldTelefrag(ctx, spot_id) != 0 {
            continue;
        }

        if mustBeEnabled && ctx.world.entity(spot_id).genericValue1 == 0 {
            continue;
        }

        spots[count as usize] = spot;
        count += 1;
        if count == 32 {
            break;
        }
    }

    if count == 0 {
        let s = G_Find(ctx, None, fofs_classname(), classname.as_ptr());
        return ctx.entity_id_of(s);
    }

    if ctx.world.cvars.g_gametype.integer == GT_SIEGE as c_int && siegeClass >= 0 {
        let mut class_spots: [Option<EntityId>; 32] = [None; 32];
        let mut class_count: c_int = 0;
        let mut i: c_int = 0;

        while i < count {
            if let Some(sid) = spots[i as usize] {
                // FLAG (task #7): `.idealclass` is a `*mut c_char` (spawn string);
                // read the pointer via the safe entity borrow, deref raw. (recipe 2c)
                let idealclass = ctx.world.entity(sid).idealclass;
                if !idealclass.is_null() && unsafe { *idealclass } != 0 {
                    let bg_classes = &ctx.world.bg_state.bgSiegeClasses;
                    let class_name =
                        unsafe { CStr::from_ptr(bg_classes[siegeClass as usize].name.as_ptr()) };
                    let spot_class = unsafe { CStr::from_ptr(idealclass) };
                    if Q_stricmp(class_name.as_ptr(), spot_class.as_ptr()) == 0 {
                        class_spots[class_count as usize] = spots[i as usize];
                        class_count += 1;
                    }
                }
            }
            i += 1;
        }

        if class_count > 0 {
            let selection = (ctx.world.bg_state.rng.rand() % class_count) as usize;
            // Oracle returns `spots[selection]` here (g_team.c:1034), not
            // `classSpots[selection]` — a Raven bug (selection is classCount-bounded
            // but indexes the full spots array). Preserved (porting-rules §19).
            return spots[selection];
        }
    }

    let selection = (ctx.world.bg_state.rng.rand() % count) as usize;
    spots[selection]
}

/// Raven `SelectCTFSpawnPoint`.
///
/// Source: `oracle/codemp/game/g_team.c:1049-1063`
pub fn SelectCTFSpawnPoint(
    ctx: &mut GameContext,
    team: team_t,
    teamstate: c_int,
    origin: &mut vec3_t,
    angles: &mut vec3_t,
) -> *mut gentity_t {
    let Some(spot) = SelectRandomTeamSpawnPoint(ctx, teamstate, team, -1) else {
        return SelectSpawnPoint(ctx, [0.0, 0.0, 0.0], origin, angles, team);
    };

    let s_origin = ctx.world.entity(spot).s.origin;
    let s_angles = ctx.world.entity(spot).s.angles;
    crate::q_math::_VectorCopy(s_origin, origin);
    (*origin)[2] += 9.0;
    crate::q_math::_VectorCopy(s_angles, angles);

    // External callers (`g_client.rs`) consume a raw `*mut gentity_t`; reconstruct
    // it from the handle at the return boundary (a cast, not a deref).
    ctx.world.entity_mut(spot) as *mut gentity_t
}

/// Raven `SelectSiegeSpawnPoint`.
///
/// Source: `oracle/codemp/game/g_team.c:1071-1085`
pub fn SelectSiegeSpawnPoint(
    ctx: &mut GameContext,
    siegeClass: c_int,
    team: team_t,
    teamstate: c_int,
    origin: &mut vec3_t,
    angles: &mut vec3_t,
) -> *mut gentity_t {
    let Some(spot) = SelectRandomTeamSpawnPoint(ctx, teamstate, team, siegeClass) else {
        return SelectSpawnPoint(ctx, [0.0, 0.0, 0.0], origin, angles, team);
    };

    let s_origin = ctx.world.entity(spot).s.origin;
    let s_angles = ctx.world.entity(spot).s.angles;
    crate::q_math::_VectorCopy(s_origin, origin);
    (*origin)[2] += 9.0;
    crate::q_math::_VectorCopy(s_angles, angles);

    // External callers (`g_client.rs`) consume a raw `*mut gentity_t`; reconstruct
    // it from the handle at the return boundary (a cast, not a deref).
    ctx.world.entity_mut(spot) as *mut gentity_t
}

/// Raven `SortClients`.
///
/// Source: `oracle/codemp/game/g_team.c:1089-1091`
pub fn SortClients(a: *const c_void, b: *const c_void) -> c_int {
    unsafe { *(a as *const c_int) - *(b as *const c_int) }
}

/// Raven `TeamplayInfoMessage`.
///
/// Source: `oracle/codemp/game/g_team.c:1103-1159`
pub fn TeamplayInfoMessage(ctx: &mut GameContext, ent: EntityId) {
    // FLAG (task #7): `.client` read via the safe entity borrow, dereffed raw as
    // Raven does. (recipe 2c)
    let ent_cl = ctx.world.entity(ent).client;
    unsafe {
        if (*ent_cl).pers.teamInfo == 0 {
            return;
        }
        let ent_team = (*ent_cl).sess.sessionTeam as c_int;

        // TEAM_MAXOVERLAY = 32 (bg_public.h:1031)
        let mut clients: [c_int; 32] = [0; 32];
        let mut cnt: c_int = 0;
        let max_clients = ctx.world.cvars.g_maxclients.integer;

        for i in 0..max_clients {
            if cnt >= 32 {
                break;
            }
            let pnum = ctx.world.level.sortedClients[i as usize];
            let pid = EntityId(pnum as u32);
            let pcl = ctx.world.entity(pid).client;
            if ctx.world.entity(pid).inuse != 0 && (*pcl).sess.sessionTeam as c_int == ent_team {
                clients[cnt as usize] = pnum;
                cnt += 1;
            }
        }

        qsort(
            clients.as_mut_ptr() as *mut c_void,
            cnt as usize,
            core::mem::size_of::<c_int>(),
            SortClients as *mut c_void,
        );

        let mut string: [c_char; 8192] = [0; 8192];
        let mut stringlength: usize = 0;

        // Oracle re-initializes `cnt = 0` in the second loop header (g_team.c:1134).
        cnt = 0;
        for i in 0..max_clients {
            if cnt >= 32 {
                break;
            }
            let pid = EntityId(i as u32);
            let pcl = ctx.world.entity(pid).client;
            if ctx.world.entity(pid).inuse != 0 && (*pcl).sess.sessionTeam as c_int == ent_team {
                let h = (*pcl).ps.stats[STAT_HEALTH as usize];
                let a = (*pcl).ps.stats[STAT_ARMOR as usize];
                let h = if h < 0 { 0 } else { h };
                let a = if a < 0 { 0 } else { a };
                let powerups = ctx.world.entity(pid).s.powerups;

                let entry = format!(
                    " {} {} {} {} {} {}",
                    i,
                    (*pcl).pers.teamState.location,
                    h,
                    a,
                    (*pcl).ps.weapon,
                    powerups
                );
                let entry_bytes = entry.as_bytes();
                let entry_len = entry_bytes.len();

                if stringlength + entry_len > 8192 {
                    break;
                }

                for j in 0..entry_len {
                    string[stringlength + j] = entry_bytes[j] as c_char;
                }
                stringlength += entry_len;
                cnt += 1;
            }
        }

        let ent_idx = ent.index();
        let cmd = format!(
            "tinfo {} {}",
            cnt,
            String::from_iter(string[0..stringlength].iter().map(|&c| c as u8 as char))
        );
        trap::SendServerCommand(
            ctx.engine,
            mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                ent_idx as c_int,
                cstr(&cmd),
            ),
        );
    }
}

/// Raven `CheckTeamStatus`.
///
/// Source: `oracle/codemp/game/g_team.c:1161-1202`
pub fn CheckTeamStatus(ctx: &mut GameContext) {
    const TEAM_LOCATION_UPDATE_TIME: c_int = 1000;

    if ctx.world.level.time - ctx.world.level.lastTeamLocationTime <= TEAM_LOCATION_UPDATE_TIME {
        return;
    }

    ctx.world.level.lastTeamLocationTime = ctx.world.level.time;

    // FLAG (task #7): `.client` read via the safe entity borrow, dereffed raw as
    // Raven does. (recipe 2c)
    let max_clients = ctx.world.cvars.g_maxclients.integer;
    for i in 0..max_clients {
        let eid = EntityId(i as u32);
        let client = ctx.world.entity(eid).client;

        if client.is_null() {
            continue;
        }

        if unsafe { (*client).pers.connected } != CON_CONNECTED as c_int {
            continue;
        }

        let team = unsafe { (*client).sess.sessionTeam } as c_int;
        if ctx.world.entity(eid).inuse != 0 && (team == TEAM_RED || team == TEAM_BLUE) {
            let loc = Team_GetLocation(ctx, eid);
            let location = match loc {
                Some(lid) => ctx.world.entity(lid).health,
                None => 0,
            };
            unsafe {
                (*client).pers.teamState.location = location;
            }
        }
    }

    for i in 0..max_clients {
        let eid = EntityId(i as u32);
        // Oracle's second loop derefs `ent->client` without a null guard.
        let client = ctx.world.entity(eid).client;

        if unsafe { (*client).pers.connected } != CON_CONNECTED as c_int {
            continue;
        }

        let team = unsafe { (*client).sess.sessionTeam } as c_int;
        if ctx.world.entity(eid).inuse != 0 && (team == TEAM_RED || team == TEAM_BLUE) {
            TeamplayInfoMessage(ctx, eid);
        }
    }
}

/// Raven `SP_team_CTF_redplayer`.
///
/// Raven: empty — spawn markers carry no runtime behavior; classname alone
/// is read by `SelectRandomTeamSpawnPoint`.
/// Source: `oracle/codemp/game/g_team.c:1209-1210`
pub fn SP_team_CTF_redplayer(ent: &gentity_t) {}

/// Raven `SP_team_CTF_blueplayer`.
///
/// Raven: empty — spawn markers carry no runtime behavior.
/// Source: `oracle/codemp/game/g_team.c:1216-1217`
pub fn SP_team_CTF_blueplayer(ent: &gentity_t) {}

/// Raven `SP_team_CTF_redspawn`.
///
/// Raven: empty — spawn markers carry no runtime behavior.
/// Source: `oracle/codemp/game/g_team.c:1224-1225`
pub fn SP_team_CTF_redspawn(ent: &gentity_t) {}

/// Raven `SP_team_CTF_bluespawn`.
///
/// Raven: empty — spawn markers carry no runtime behavior.
/// Source: `oracle/codemp/game/g_team.c:1231-1232`
pub fn SP_team_CTF_bluespawn(ent: &gentity_t) {}
