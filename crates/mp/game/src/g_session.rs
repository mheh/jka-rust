// PORT-COMPLETE: g_session.c 0/5
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
use mp_abi::game::syscalls::G_CVAR_SET::GCvarSetArgs;
use mp_abi::game::syscalls::G_CVAR_VARIABLE_STRING_BUFFER::GCvarVariableStringBufferArgs;
use mp_bg::public::duel_team::duelTeam_t::*;

/// Raven `G_WriteClientSessionData`.
///
/// Source: `oracle/codemp/game/g_session.c:23-96`
pub fn G_WriteClientSessionData(ctx: &mut GameContext, client: usize) {
    let mut siege_class: [c_char; 64] = [0; 64];
    let mut saber_type: [c_char; 64] = [0; 64];
    let mut saber2_type: [c_char; 64] = [0; 64];

    {
        let c = ctx.world.client(client);
        siege_class.copy_from_slice(&c.sess.siegeClass);
        saber_type.copy_from_slice(&c.sess.saberType);
        saber2_type.copy_from_slice(&c.sess.saber2Type);
    }

    let mut i = 0;
    while i < 64 && siege_class[i] != 0 {
        if siege_class[i] == b' ' as c_char {
            siege_class[i] = 1;
        }
        i += 1;
    }

    if siege_class[0] == 0 {
        siege_class[0] = b'n' as c_char;
        siege_class[1] = b'o' as c_char;
        siege_class[2] = b'n' as c_char;
        siege_class[3] = b'e' as c_char;
        siege_class[4] = 0;
    }

    i = 0;
    while i < 64 && saber_type[i] != 0 {
        if saber_type[i] == b' ' as c_char {
            saber_type[i] = 1;
        }
        i += 1;
    }

    i = 0;
    while i < 64 && saber2_type[i] != 0 {
        if saber2_type[i] == b' ' as c_char {
            saber2_type[i] = 1;
        }
        i += 1;
    }

    let siege_class_str = unsafe { cstr_to_str(siege_class.as_ptr()) };
    let saber_type_str = unsafe { cstr_to_str(saber_type.as_ptr()) };
    let saber2_type_str = unsafe { cstr_to_str(saber2_type.as_ptr()) };

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
    trap::Cvar_Set(ctx.engine, GCvarSetArgs::new(cstr(&var), cstr(&s)));
}

/// Raven `G_ReadSessionData`.
///
/// Source: `oracle/codemp/game/g_session.c:105-177`
pub fn G_ReadSessionData(ctx: &mut GameContext, client: usize) {
    // `client - level.clients` recomputes to the client index `client`.
    let var = format!("session{}", client);
    let mut s: [c_char; MAX_STRING_CHARS as usize] = [0; MAX_STRING_CHARS as usize];

    trap::Cvar_VariableStringBuffer(
        ctx.engine,
        GCvarVariableStringBufferArgs::new(cstr(&var), s.as_mut_ptr(), MAX_STRING_CHARS as i32),
    );

    let s_str = unsafe { cstr_to_str(s.as_ptr()) };
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
        write_cstr_field(&mut c.sess.siegeClass, parts[idx]);
        idx += 1;
    }
    if idx < parts.len() {
        write_cstr_field(&mut c.sess.saberType, parts[idx]);
        idx += 1;
    }
    if idx < parts.len() {
        write_cstr_field(&mut c.sess.saber2Type, parts[idx]);
    }

    let mut i = 0;
    while i < c.sess.siegeClass.len() && c.sess.siegeClass[i] != 0 {
        if c.sess.siegeClass[i] == 1 {
            c.sess.siegeClass[i] = b' ' as c_char;
        }
        i += 1;
    }

    i = 0;
    while i < c.sess.saberType.len() && c.sess.saberType[i] != 0 {
        if c.sess.saberType[i] == 1 {
            c.sess.saberType[i] = b' ' as c_char;
        }
        i += 1;
    }

    i = 0;
    while i < c.sess.saber2Type.len() && c.sess.saber2Type[i] != 0 {
        if c.sess.saber2Type[i] == 1 {
            c.sess.saber2Type[i] = b' ' as c_char;
        }
        i += 1;
    }

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
pub fn G_InitSessionData(
    ctx: &mut GameContext,
    client: usize,
    userinfo: *mut c_char,
    isBot: qboolean,
) {
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
                let value =
                    Info_ValueForKey(&mut ctx.world.bg_state.qs, userinfo, cstr("team").as_ptr());
                // `value` is the info-string return (seam), deref stays raw.
                let value_char = if value.is_null() {
                    0 as c_char
                } else {
                    unsafe { *value }
                };
                if value_char == b'r' as c_char || value_char == b'R' as c_char {
                    ctx.world.client_mut(client).sess.sessionTeam = TEAM_RED;
                } else if value_char == b'b' as c_char || value_char == b'B' as c_char {
                    ctx.world.client_mut(client).sess.sessionTeam = TEAM_BLUE;
                } else {
                    let team = PickTeam(ctx, -1);
                    ctx.world.client_mut(client).sess.sessionTeam = team;
                }
                BroadcastTeamChange(ctx, client_id, -1);
            }
        }
    } else {
        let value = Info_ValueForKey(&mut ctx.world.bg_state.qs, userinfo, cstr("team").as_ptr());
        // `value` is the info-string return (seam), deref stays raw.
        let value_char = if value.is_null() {
            0 as c_char
        } else {
            unsafe { *value }
        };
        if value_char == b's' as c_char {
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
        let c = ctx.world.client_mut(client);
        c.sess.siegeClass[0] = 0;
        c.sess.saberType[0] = 0;
        c.sess.saber2Type[0] = 0;
    }

    G_WriteClientSessionData(ctx, client);
}

/// Raven `G_InitWorldSession`.
///
/// Source: `oracle/codemp/game/g_session.c:291-304`
pub fn G_InitWorldSession(ctx: &mut GameContext) {
    let mut s: [c_char; MAX_STRING_CHARS as usize] = [0; MAX_STRING_CHARS as usize];

    trap::Cvar_VariableStringBuffer(
        ctx.engine,
        GCvarVariableStringBufferArgs::new(
            cstr("session"),
            s.as_mut_ptr(),
            MAX_STRING_CHARS as c_int,
        ),
    );

    let gt: c_int = atoi(s.as_ptr());

    if ctx.world.cvars.g_gametype.integer != gt {
        ctx.world.level.newSession = qtrue;
        G_Printf(
            ctx,
            cstr("Gametype changed, clearing session data.\n").as_ptr(),
        );
    }
}

/// Raven `G_WriteSessionData`.
///
/// Source: `oracle/codemp/game/g_session.c:312-322`
pub fn G_WriteSessionData(ctx: &mut GameContext) {
    let s = format!("{}", ctx.world.cvars.g_gametype.integer);
    trap::Cvar_Set(ctx.engine, GCvarSetArgs::new(cstr("session"), cstr(&s)));

    for i in 0..ctx.world.level.maxclients {
        if ctx.world.client(i as usize).pers.connected == CON_CONNECTED {
            G_WriteClientSessionData(ctx, i as usize);
        }
    }
}
