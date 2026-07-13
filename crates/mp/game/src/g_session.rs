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
pub fn G_WriteClientSessionData(ctx: GameContext<'_>, client: usize) {
    // STAGE-1: client-index param; re-derive the raw pointer the verbatim body
    // still expects (Stage-2 debt). `client_mut(i)` shares `level.clients`'s base,
    // so the body's `client - level.clients` index recomputes to `client`.
    let client: *mut gclient_t = ctx.world().client_mut(client);
    let mut siege_class: [c_char; 64] = [0; 64];
    let mut saber_type: [c_char; 64] = [0; 64];
    let mut saber2_type: [c_char; 64] = [0; 64];

    unsafe {
        core::ptr::copy_nonoverlapping(
            (*client).sess.siegeClass.as_ptr(),
            siege_class.as_mut_ptr(),
            64,
        );
        core::ptr::copy_nonoverlapping(
            (*client).sess.saberType.as_ptr(),
            saber_type.as_mut_ptr(),
            64,
        );
        core::ptr::copy_nonoverlapping(
            (*client).sess.saber2Type.as_ptr(),
            saber2_type.as_mut_ptr(),
            64,
        );
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

    let client_idx = unsafe {
        let base = (*ctx.world).level.clients;
        if client >= base {
            (client as usize - base as usize) / std::mem::size_of::<gclient_t>()
        } else {
            0
        }
    };

    let s = unsafe {
        format!(
            "{} {} {} {} {} {} {} {} {} {} {} {} {} {} {}",
            (*client).sess.sessionTeam as i32,
            (*client).sess.spectatorTime,
            (*client).sess.spectatorState as i32,
            (*client).sess.spectatorClient,
            (*client).sess.wins,
            (*client).sess.losses,
            (*client).sess.teamLeader as i32,
            (*client).sess.setForce,
            (*client).sess.saberLevel,
            (*client).sess.selectedFP,
            (*client).sess.duelTeam,
            (*client).sess.siegeDesiredTeam,
            siege_class_str,
            saber_type_str,
            saber2_type_str
        )
    };

    let var = format!("session{}", client_idx);
    trap::Cvar_Set(ctx.engine, GCvarSetArgs::new(cstr(&var), cstr(&s)));
}

/// Raven `G_ReadSessionData`.
///
/// Source: `oracle/codemp/game/g_session.c:105-177`
pub fn G_ReadSessionData(ctx: GameContext<'_>, client: usize) {
    // STAGE-1: client-index param; re-derive the raw pointer the verbatim body
    // still expects (Stage-2 debt).
    let client: *mut gclient_t = ctx.world().client_mut(client);
    unsafe {
        let client_idx = {
            let base = (*ctx.world).level.clients;
            if client >= base {
                (client as usize - base as usize) / std::mem::size_of::<gclient_t>()
            } else {
                0
            }
        };

        let var = format!("session{}", client_idx);
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

        if idx < parts.len() {
            session_team = parts[idx].parse().unwrap_or(0);
            idx += 1;
        }
        if idx < parts.len() {
            (*client).sess.spectatorTime = parts[idx].parse().unwrap_or(0);
            idx += 1;
        }
        if idx < parts.len() {
            spectator_state = parts[idx].parse().unwrap_or(0);
            idx += 1;
        }
        if idx < parts.len() {
            (*client).sess.spectatorClient = parts[idx].parse().unwrap_or(0);
            idx += 1;
        }
        if idx < parts.len() {
            (*client).sess.wins = parts[idx].parse().unwrap_or(0);
            idx += 1;
        }
        if idx < parts.len() {
            (*client).sess.losses = parts[idx].parse().unwrap_or(0);
            idx += 1;
        }
        if idx < parts.len() {
            team_leader = parts[idx].parse().unwrap_or(0);
            idx += 1;
        }
        if idx < parts.len() {
            (*client).sess.setForce = parts[idx].parse().unwrap_or(0);
            idx += 1;
        }
        if idx < parts.len() {
            (*client).sess.saberLevel = parts[idx].parse().unwrap_or(0);
            idx += 1;
        }
        if idx < parts.len() {
            (*client).sess.selectedFP = parts[idx].parse().unwrap_or(0);
            idx += 1;
        }
        if idx < parts.len() {
            (*client).sess.duelTeam = parts[idx].parse().unwrap_or(0);
            idx += 1;
        }
        if idx < parts.len() {
            (*client).sess.siegeDesiredTeam = parts[idx].parse().unwrap_or(0);
            idx += 1;
        }
        if idx < parts.len() {
            write_cstr_field(&mut (*client).sess.siegeClass, parts[idx]);
            idx += 1;
        }
        if idx < parts.len() {
            write_cstr_field(&mut (*client).sess.saberType, parts[idx]);
            idx += 1;
        }
        if idx < parts.len() {
            write_cstr_field(&mut (*client).sess.saber2Type, parts[idx]);
        }

        let mut i = 0;
        while i < (*client).sess.siegeClass.len() && (*client).sess.siegeClass[i] != 0 {
            if (*client).sess.siegeClass[i] == 1 {
                (*client).sess.siegeClass[i] = b' ' as c_char;
            }
            i += 1;
        }

        i = 0;
        while i < (*client).sess.saberType.len() && (*client).sess.saberType[i] != 0 {
            if (*client).sess.saberType[i] == 1 {
                (*client).sess.saberType[i] = b' ' as c_char;
            }
            i += 1;
        }

        i = 0;
        while i < (*client).sess.saber2Type.len() && (*client).sess.saber2Type[i] != 0 {
            if (*client).sess.saber2Type[i] == 1 {
                (*client).sess.saber2Type[i] = b' ' as c_char;
            }
            i += 1;
        }

        (*client).sess.sessionTeam = session_team as team_t;
        (*client).sess.spectatorState =
            core::mem::transmute::<i32, spectatorState_t>(spectator_state);
        (*client).sess.teamLeader = if team_leader != 0 { qtrue } else { qfalse };

        (*client).ps.fd.saberAnimLevel = (*client).sess.saberLevel;
        (*client).ps.fd.saberDrawAnimLevel = (*client).sess.saberLevel;
        (*client).ps.fd.forcePowerSelected = (*client).sess.selectedFP;
    }
}

/// Raven `G_InitSessionData`.
///
/// Source: `oracle/codemp/game/g_session.c:187-282`
pub fn G_InitSessionData(
    ctx: GameContext<'_>,
    client: usize,
    userinfo: *mut c_char,
    isBot: qboolean,
) {
    // STAGE-1: client-index param; re-derive the raw pointer the verbatim body
    // still expects (Stage-2 debt), keeping the index for the write-back call.
    let client_index: usize = client;
    let client: *mut gclient_t = ctx.world().client_mut(client);
    unsafe {
        (*client).sess.siegeDesiredTeam = TEAM_FREE;

        if (*ctx.world).cvars.g_gametype.integer >= GT_TEAM as i32 {
            if (*ctx.world).cvars.g_teamAutoJoin.integer != 0 {
                (*client).sess.sessionTeam = PickTeam(ctx, -1);
                BroadcastTeamChange(
                    ctx,
                    EntityId(client.offset_from((*ctx.world).level.clients) as u32),
                    -1,
                );
            } else {
                if isBot == qfalse {
                    (*client).sess.sessionTeam = TEAM_SPECTATOR;
                } else {
                    let value = Info_ValueForKey(userinfo, cstr("team").as_ptr());
                    let value_char = if value.is_null() { 0 as c_char } else { *value };
                    if value_char == b'r' as c_char || value_char == b'R' as c_char {
                        (*client).sess.sessionTeam = TEAM_RED;
                    } else if value_char == b'b' as c_char || value_char == b'B' as c_char {
                        (*client).sess.sessionTeam = TEAM_BLUE;
                    } else {
                        (*client).sess.sessionTeam = PickTeam(ctx, -1);
                    }
                    BroadcastTeamChange(
                        ctx,
                        EntityId(client.offset_from((*ctx.world).level.clients) as u32),
                        -1,
                    );
                }
            }
        } else {
            let value = Info_ValueForKey(userinfo, cstr("team").as_ptr());
            let value_char = if value.is_null() { 0 as c_char } else { *value };
            if value_char == b's' as c_char {
                (*client).sess.sessionTeam = TEAM_SPECTATOR;
            } else {
                match (*ctx.world).cvars.g_gametype.integer {
                    x if x == GT_DUEL as i32 => {
                        if (*ctx.world).level.numNonSpectatorClients >= 2 {
                            (*client).sess.sessionTeam = TEAM_SPECTATOR;
                        } else {
                            (*client).sess.sessionTeam = TEAM_FREE;
                        }
                    }
                    x if x == GT_POWERDUEL as i32 => {
                        let mut loners: c_int = 0;
                        let mut doubles: c_int = 0;
                        G_PowerDuelCount(ctx, &mut loners, &mut doubles, qtrue);
                        if doubles == 0 || loners > (doubles / 2) {
                            (*client).sess.duelTeam = DUELTEAM_DOUBLE as c_int;
                        } else {
                            (*client).sess.duelTeam = DUELTEAM_LONE as c_int;
                        }
                        (*client).sess.sessionTeam = TEAM_SPECTATOR;
                    }
                    _ => {
                        if (*ctx.world).cvars.g_maxGameClients.integer > 0
                            && (*ctx.world).level.numNonSpectatorClients
                                >= (*ctx.world).cvars.g_maxGameClients.integer
                        {
                            (*client).sess.sessionTeam = TEAM_SPECTATOR;
                        } else {
                            (*client).sess.sessionTeam = TEAM_FREE;
                        }
                    }
                }
            }
        }

        (*client).sess.spectatorState = SPECTATOR_FREE;
        (*client).sess.spectatorTime = (*ctx.world).level.time;

        (*client).sess.siegeClass[0] = 0;
        (*client).sess.saberType[0] = 0;
        (*client).sess.saber2Type[0] = 0;

        G_WriteClientSessionData(ctx, client_index);
    }
}

/// Raven `G_InitWorldSession`.
///
/// Source: `oracle/codemp/game/g_session.c:291-304`
pub fn G_InitWorldSession(ctx: GameContext<'_>) {
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

    unsafe {
        if (*ctx.world).cvars.g_gametype.integer != gt {
            (*ctx.world).level.newSession = qtrue;
            G_Printf(
                ctx,
                cstr("Gametype changed, clearing session data.\n").as_ptr(),
            );
        }
    }
}

/// Raven `G_WriteSessionData`.
///
/// Source: `oracle/codemp/game/g_session.c:312-322`
pub fn G_WriteSessionData(ctx: GameContext<'_>) {
    unsafe {
        let s = format!("{}", (*ctx.world).cvars.g_gametype.integer);
        trap::Cvar_Set(ctx.engine, GCvarSetArgs::new(cstr("session"), cstr(&s)));

        for i in 0..(*ctx.world).level.maxclients {
            let client = (*ctx.world).level.clients.add(i as usize);
            if (*client).pers.connected == CON_CONNECTED {
                G_WriteClientSessionData(ctx, i as usize);
            }
        }
    }
}
