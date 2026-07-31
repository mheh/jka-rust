// PORT-COMPLETE: g_session.c
//! FAITHFUL port of `oracle/codemp/game/g_session.c`.
//!
//! Filled by the jampgame mega-pass; functions reach file-scope game state
//! (`level`, `g_entities`, cvars) and engine traps through the threaded
//! `GameContext`/`GameWorld` handle.
#![allow(non_snake_case, unused, clippy::all)]

use crate::client::spectator_state::spectatorState_t;
use crate::client::spectator_state::spectatorState_t::*;
use crate::g_main::{G_PowerDuelCount, G_Printf};
use crate::prelude::*;
use mp_bg::public::duel_team::duelTeam_t::*;
use native_string::atoi;
use native_string::latin1_to_string;
use native_string::strncpyz_string;

/// Raven `G_WriteClientSessionData`.
///
/// Source: `oracle/codemp/game/g_session.c:23-96`
pub fn G_WriteClientSessionData(ctx: &mut GameContext, client: usize) {
    // Raven copies each session string into a scratch buffer and converts its
    // spaces to char(1) — siege class names contain spaces, but the session
    // cvar is space-separated. The `String` fields have no interior NUL (they
    // are filled from NUL-terminated sources), so iterating all bytes matches
    // Raven's `while (buf[i])` walk exactly.
    let (mut siege_class, mut saber_type, mut saber2_type) = {
        let c = ctx.world.client(client);
        (
            c.sess.siegeClass.clone().into_bytes(),
            c.sess.saberType.clone().into_bytes(),
            c.sess.saber2Type.clone().into_bytes(),
        )
    };

    for b in siege_class.iter_mut() {
        if *b == b' ' {
            *b = 1;
        }
    }

    if siege_class.is_empty() {
        // make sure there's at least something
        siege_class = b"none".to_vec();
    }

    for b in saber_type.iter_mut() {
        if *b == b' ' {
            *b = 1;
        }
    }

    for b in saber2_type.iter_mut() {
        if *b == b' ' {
            *b = 1;
        }
    }

    // Space (0x20) becomes 0x01 so the space-separated session cvar parses. The
    // Latin-1 decode keeps every byte, so the cvar text matches Raven's `%s`.
    let siege_class_str = latin1_to_string(&siege_class);
    let saber_type_str = latin1_to_string(&saber_type);
    let saber2_type_str = latin1_to_string(&saber2_type);

    // `client - level.clients` recomputes to the client index `client` (both
    // alias `world.clients`), so the session cvar name uses it directly.
    let s = {
        let c = ctx.world.client(client);
        format!(
            "{} {} {} {} {} {} {} {} {} {} {} {} {} {} {}",
            c.sess.sessionTeam as i32,
            c.sess.spectatorTime,
            c.sess.spectatorState as i32,
            c.sess.spectatorClient,
            c.sess.wins,
            c.sess.losses,
            c.sess.teamLeader as i32,
            c.sess.setForce,
            c.sess.saberLevel,
            c.sess.selectedFP,
            c.sess.duelTeam,
            c.sess.siegeDesiredTeam,
            siege_class_str,
            saber_type_str,
            saber2_type_str
        )
    };

    let var = format!("session{}", client);
    trap::Cvar_Set(ctx.engine, &var, &s);
}

/// Raven `G_ReadSessionData`.
///
/// Source: `oracle/codemp/game/g_session.c:105-177`
pub fn G_ReadSessionData(ctx: &mut GameContext, client: usize) {
    // `client - level.clients` recomputes to the client index `client`.
    let var = format!("session{}", client);
    let s_str = trap::Cvar_VariableStringBuffer(ctx.engine, &var, MAX_STRING_CHARS as usize);

    let parts: Vec<&str> = s_str.split_whitespace().collect();

    let mut idx = 0;
    // §19: C's sscanf leaves these uninitialized when the session string has
    // fewer than the expected tokens and then assigns that garbage (UB); a
    // short string reads as 0 here instead.
    let mut session_team: i32 = 0;
    let mut spectator_state: i32 = 0;
    let mut team_leader: i32 = 0;

    let c = ctx.world.client_mut(client);

    if idx < parts.len() {
        session_team = parts[idx].parse().unwrap_or(0);
        idx += 1;
    }
    if idx < parts.len() {
        c.sess.spectatorTime = parts[idx].parse().unwrap_or(0);
        idx += 1;
    }
    if idx < parts.len() {
        spectator_state = parts[idx].parse().unwrap_or(0);
        idx += 1;
    }
    if idx < parts.len() {
        c.sess.spectatorClient = parts[idx].parse().unwrap_or(0);
        idx += 1;
    }
    if idx < parts.len() {
        c.sess.wins = parts[idx].parse().unwrap_or(0);
        idx += 1;
    }
    if idx < parts.len() {
        c.sess.losses = parts[idx].parse().unwrap_or(0);
        idx += 1;
    }
    if idx < parts.len() {
        team_leader = parts[idx].parse().unwrap_or(0);
        idx += 1;
    }
    if idx < parts.len() {
        c.sess.setForce = parts[idx].parse().unwrap_or(0);
        idx += 1;
    }
    if idx < parts.len() {
        c.sess.saberLevel = parts[idx].parse().unwrap_or(0);
        idx += 1;
    }
    if idx < parts.len() {
        c.sess.selectedFP = parts[idx].parse().unwrap_or(0);
        idx += 1;
    }
    if idx < parts.len() {
        c.sess.duelTeam = parts[idx].parse().unwrap_or(0);
        idx += 1;
    }
    if idx < parts.len() {
        c.sess.siegeDesiredTeam = parts[idx].parse().unwrap_or(0);
        idx += 1;
    }
    if idx < parts.len() {
        c.sess.siegeClass = strncpyz_string(parts[idx].as_bytes(), 64);
        idx += 1;
    }
    if idx < parts.len() {
        c.sess.saberType = strncpyz_string(parts[idx].as_bytes(), 64);
        idx += 1;
    }
    if idx < parts.len() {
        c.sess.saber2Type = strncpyz_string(parts[idx].as_bytes(), 64);
    }

    // Convert the char(1) placeholders back to spaces, as the session data was
    // written that way (0x01 and ' ' are single ASCII bytes, so `replace` on
    // the char is byte-identical to Raven's in-place `buf[i] == 1 -> ' '`).
    c.sess.siegeClass = c.sess.siegeClass.replace('\u{1}', " ");
    c.sess.saberType = c.sess.saberType.replace('\u{1}', " ");
    c.sess.saber2Type = c.sess.saber2Type.replace('\u{1}', " ");

    c.sess.sessionTeam = session_team as team_t;
    // spectatorState_t is `#[repr(i32)]`; the sscanf'd int transmutes to it.
    c.sess.spectatorState =
        unsafe { core::mem::transmute::<i32, spectatorState_t>(spectator_state) };
    c.sess.teamLeader = if team_leader != 0 { qtrue } else { qfalse };

    c.ps.fd.saberAnimLevel = c.sess.saberLevel;
    c.ps.fd.saberDrawAnimLevel = c.sess.saberLevel;
    c.ps.fd.forcePowerSelected = c.sess.selectedFP;
}

/// Raven `G_InitSessionData`.
///
/// Source: `oracle/codemp/game/g_session.c:187-282`
pub fn G_InitSessionData(ctx: &mut GameContext, client: usize, userinfo: &str, isBot: qboolean) {
    // `client - level.clients` recomputes to the client index `client`.
    let client_id = EntityId(client as u32);

    ctx.world.client_mut(client).sess.siegeDesiredTeam = TEAM_FREE;

    if ctx.world.cvars.g_gametype.integer >= GT_TEAM as i32 {
        if ctx.world.cvars.g_teamAutoJoin.integer != 0 {
            let team = PickTeam(ctx, -1);
            ctx.world.client_mut(client).sess.sessionTeam = team;
            BroadcastTeamChange(ctx, client_id, -1);
        } else {
            if isBot == qfalse {
                ctx.world.client_mut(client).sess.sessionTeam = TEAM_SPECTATOR;
            } else {
                let value = Info_ValueForKey(userinfo, "team");
                let value_char = value.chars().next().unwrap_or('\0');
                if value_char == 'r' || value_char == 'R' {
                    ctx.world.client_mut(client).sess.sessionTeam = TEAM_RED;
                } else if value_char == 'b' || value_char == 'B' {
                    ctx.world.client_mut(client).sess.sessionTeam = TEAM_BLUE;
                } else {
                    let team = PickTeam(ctx, -1);
                    ctx.world.client_mut(client).sess.sessionTeam = team;
                }
                BroadcastTeamChange(ctx, client_id, -1);
            }
        }
    } else {
        let value = Info_ValueForKey(userinfo, "team");
        let value_char = value.chars().next().unwrap_or('\0');
        if value_char == 's' {
            ctx.world.client_mut(client).sess.sessionTeam = TEAM_SPECTATOR;
        } else {
            match ctx.world.cvars.g_gametype.integer {
                x if x == GT_DUEL as i32 => {
                    if ctx.world.level.numNonSpectatorClients >= 2 {
                        ctx.world.client_mut(client).sess.sessionTeam = TEAM_SPECTATOR;
                    } else {
                        ctx.world.client_mut(client).sess.sessionTeam = TEAM_FREE;
                    }
                }
                x if x == GT_POWERDUEL as i32 => {
                    let mut loners: c_int = 0;
                    let mut doubles: c_int = 0;
                    G_PowerDuelCount(ctx, &mut loners, &mut doubles, qtrue);
                    if doubles == 0 || loners > (doubles / 2) {
                        ctx.world.client_mut(client).sess.duelTeam = DUELTEAM_DOUBLE as c_int;
                    } else {
                        ctx.world.client_mut(client).sess.duelTeam = DUELTEAM_LONE as c_int;
                    }
                    ctx.world.client_mut(client).sess.sessionTeam = TEAM_SPECTATOR;
                }
                _ => {
                    if ctx.world.cvars.g_maxGameClients.integer > 0
                        && ctx.world.level.numNonSpectatorClients
                            >= ctx.world.cvars.g_maxGameClients.integer
                    {
                        ctx.world.client_mut(client).sess.sessionTeam = TEAM_SPECTATOR;
                    } else {
                        ctx.world.client_mut(client).sess.sessionTeam = TEAM_FREE;
                    }
                }
            }
        }
    }

    ctx.world.client_mut(client).sess.spectatorState = SPECTATOR_FREE;
    let time = ctx.world.level.time;
    ctx.world.client_mut(client).sess.spectatorTime = time;

    {
        // Raven clears each with `sess.X[0] = 0` (empty C string); `.clear()`
        // is the byte-equivalent empty-`String`.
        let c = ctx.world.client_mut(client);
        c.sess.siegeClass.clear();
        c.sess.saberType.clear();
        c.sess.saber2Type.clear();
    }

    G_WriteClientSessionData(ctx, client);
}

/// Raven `G_InitWorldSession`.
///
/// Source: `oracle/codemp/game/g_session.c:291-304`
pub fn G_InitWorldSession(ctx: &mut GameContext) {
    let s = trap::Cvar_VariableStringBuffer(ctx.engine, "session", MAX_STRING_CHARS as usize);

    let gt: c_int = atoi(&s);

    if ctx.world.cvars.g_gametype.integer != gt {
        ctx.world.level.newSession = qtrue;
        G_Printf(ctx, "Gametype changed, clearing session data.\n");
    }
}

/// Raven `G_WriteSessionData`.
///
/// Source: `oracle/codemp/game/g_session.c:312-322`
pub fn G_WriteSessionData(ctx: &mut GameContext) {
    let s = format!("{}", ctx.world.cvars.g_gametype.integer);
    trap::Cvar_Set(ctx.engine, "session", &s);

    for i in 0..ctx.world.level.maxclients {
        if ctx.world.client(i as usize).pers.connected == CON_CONNECTED {
            G_WriteClientSessionData(ctx, i as usize);
        }
    }
}
