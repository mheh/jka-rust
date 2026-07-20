// PORT-COMPLETE: g_arenas.c

//! FAITHFUL port of `oracle/codemp/game/g_arenas.c`.
//!
//! Filled by the jampgame mega-pass; functions reach file-scope game state
//! (`level`, `g_entities`, cvars) and engine traps through the threaded
//! `GameContext`/`GameWorld` handle.
#![allow(non_snake_case, unused, clippy::all)]
#![deny(unsafe_code)]

use crate::g_main::CalculateRanks;
use crate::prelude::*;
use crate::trap;

/// Raven `UpdateTournamentInfo`.
///
/// Source: `oracle/codemp/game/g_arenas.c:20-101`
pub fn UpdateTournamentInfo(ctx: &mut GameContext) {
    let mut i: c_int;
    let mut playerClientNum: c_int;
    let mut n: c_int;
    let mut accuracy: c_int;
    let mut perfect: c_int;
    let msglen: c_int;
    let mut buflen: c_int;
    let mut score1: c_int;
    let mut score2: c_int;
    let mut won: bool;
    let mut buf = [0 as c_char; 32];
    let mut msg = [0 as c_char; MAX_STRING_CHARS as usize];

    // find the real player
    let mut player_valid = false;
    i = 0;
    while i < ctx.world.level.maxclients {
        let player = ctx.world.entity(EntityId(i as u32));
        if player.inuse == 0 {
            i += 1;
            continue;
        }
        if player.r.svFlags & SVF_BOT == 0 {
            player_valid = true;
            break;
        }
        i += 1;
    }
    // this should never happen!
    if !player_valid || i == ctx.world.level.maxclients {
        return;
    }
    playerClientNum = i;

    CalculateRanks(ctx);

    if ctx.world.client(playerClientNum as usize).sess.sessionTeam == TEAM_SPECTATOR as c_int {
        let formatted = format!(
            "postgame {} {} 0 0 0 0 0 0 0 0 0 0 0",
            ctx.world.level.numNonSpectatorClients, playerClientNum
        );
        write_cstr_field(&mut msg, &formatted);
    } else {
        // Raven's `player->client` aliases `&level.clients[playerClientNum]`: a
        // client entity's `client` pointer is set to its own slot in the owned
        // arena at spawn (§B5), so the index is the entity number `playerClientNum`.
        let client = ctx.world.client(playerClientNum as usize);
        if client.accuracy_shots != 0 {
            accuracy = client.accuracy_hits * 100 / client.accuracy_shots;
        } else {
            accuracy = 0;
        }
        won = false;
        if ctx.world.cvars.g_gametype.integer >= GT_CTF as c_int {
            score1 = ctx.world.level.teamScores[TEAM_RED as usize];
            score2 = ctx.world.level.teamScores[TEAM_BLUE as usize];
            if ctx.world.client(playerClientNum as usize).sess.sessionTeam == TEAM_RED as c_int {
                won = ctx.world.level.teamScores[TEAM_RED as usize]
                    > ctx.world.level.teamScores[TEAM_BLUE as usize];
            } else {
                won = ctx.world.level.teamScores[TEAM_BLUE as usize]
                    > ctx.world.level.teamScores[TEAM_RED as usize];
            }
        } else {
            // `&level.clients[playerClientNum] == &level.clients[sortedClients[0]]`
            // is an identity test over the contiguous client arena — equivalent to
            // comparing the two indices (g_arenas.c:70).
            if playerClientNum == ctx.world.level.sortedClients[0] {
                won = true;
                score1 = ctx
                    .world
                    .client(ctx.world.level.sortedClients[0] as usize)
                    .ps
                    .persistant[PERS_SCORE as usize];
                score2 = ctx
                    .world
                    .client(ctx.world.level.sortedClients[1] as usize)
                    .ps
                    .persistant[PERS_SCORE as usize];
            } else {
                score2 = ctx
                    .world
                    .client(ctx.world.level.sortedClients[0] as usize)
                    .ps
                    .persistant[PERS_SCORE as usize];
                score1 = ctx
                    .world
                    .client(ctx.world.level.sortedClients[1] as usize)
                    .ps
                    .persistant[PERS_SCORE as usize];
            }
        }
        if won && client.ps.persistant[PERS_KILLED as usize] == 0 {
            perfect = 1;
        } else {
            perfect = 0;
        }
        let formatted = format!(
            "postgame {} {} {} {} {} {} {} {} {} {} {} {} {} {}",
            ctx.world.level.numNonSpectatorClients,
            playerClientNum,
            accuracy,
            client.ps.persistant[PERS_IMPRESSIVE_COUNT as usize],
            client.ps.persistant[PERS_EXCELLENT_COUNT as usize],
            client.ps.persistant[PERS_DEFEND_COUNT as usize],
            client.ps.persistant[PERS_ASSIST_COUNT as usize],
            client.ps.persistant[PERS_GAUNTLET_FRAG_COUNT as usize],
            client.ps.persistant[PERS_SCORE as usize],
            perfect,
            score1,
            score2,
            ctx.world.level.time,
            client.ps.persistant[PERS_CAPTURES as usize]
        );
        write_cstr_field(&mut msg, &formatted);
    }

    msglen = msg.iter().position(|&c| c == 0).unwrap_or(0) as c_int;
    i = 0;
    while i < ctx.world.level.numNonSpectatorClients {
        n = ctx.world.level.sortedClients[i as usize];
        let buf_str = format!(
            " {} {} {}",
            n,
            ctx.world.client(n as usize).ps.persistant[PERS_RANK as usize],
            ctx.world.client(n as usize).ps.persistant[PERS_SCORE as usize]
        );
        write_cstr_field(&mut buf, &buf_str);
        buflen = buf.iter().position(|&c| c == 0).unwrap_or(0) as c_int;
        if (msglen + buflen + 1) as usize >= MAX_STRING_CHARS {
            break;
        }
        // strcat(msg, buf)
        let msg_len = msg.iter().position(|&c| c == 0).unwrap_or(0);
        let buf_len = buf.iter().position(|&c| c == 0).unwrap_or(0);
        for j in 0..buf_len {
            msg[msg_len + j] = buf[j];
        }
        msg[msg_len + buf_len] = 0;
        // Oracle never updates `msglen` in this loop (g_arenas.c:90-99): the guard
        // above keeps comparing the original length every iteration. Preserved.
        i += 1;
    }

    let msg_str = cstr_from_chars(&msg).to_string_lossy().into_owned();
    trap::SendConsoleCommand(ctx.engine, EXEC_APPEND as c_int, &msg_str);
}
