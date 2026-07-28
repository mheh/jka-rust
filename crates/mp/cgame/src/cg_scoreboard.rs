//! Port of `oracle/codemp/cgame/cg_scoreboard.c` — the scoreboard layout and its per-client rows. Functions land via the C5
//! transcription waves.

#![allow(non_snake_case)]

use core::ffi::c_int;

use mp_bg::public::team::team_t;
use mp_qshared::shared::qfalse;

use crate::world::cg_world::CgWorld;

/// Raven `#define SCOREBOARD_X (0)`.
/// Source: `oracle/codemp/cgame/cg_scoreboard.c:8`
pub const SCOREBOARD_X: c_int = 0;

/// Raven `#define SB_HEADER 86`.
/// Source: `oracle/codemp/cgame/cg_scoreboard.c:10`
pub const SB_HEADER: c_int = 86;

/// Raven `#define SB_TOP (SB_HEADER+32)`.
/// Source: `oracle/codemp/cgame/cg_scoreboard.c:11`
pub const SB_TOP: c_int = SB_HEADER + 32;

/// Raven `#define SB_STATUSBAR 420`.
/// Source: `oracle/codemp/cgame/cg_scoreboard.c:14`
pub const SB_STATUSBAR: c_int = 420;

/// Raven `#define SB_NORMAL_HEIGHT 25`.
/// Source: `oracle/codemp/cgame/cg_scoreboard.c:16`
pub const SB_NORMAL_HEIGHT: c_int = 25;

/// Raven `#define SB_INTER_HEIGHT 15 // interleaved height`.
/// Source: `oracle/codemp/cgame/cg_scoreboard.c:17`
pub const SB_INTER_HEIGHT: c_int = 15;

/// Raven `#define SB_MAXCLIENTS_NORMAL ((SB_STATUSBAR - SB_TOP) / SB_NORMAL_HEIGHT)`.
/// Source: `oracle/codemp/cgame/cg_scoreboard.c:19`
pub const SB_MAXCLIENTS_NORMAL: c_int = (SB_STATUSBAR - SB_TOP) / SB_NORMAL_HEIGHT;

/// Raven `#define SB_MAXCLIENTS_INTER ((SB_STATUSBAR - SB_TOP) / SB_INTER_HEIGHT - 1)`.
/// Source: `oracle/codemp/cgame/cg_scoreboard.c:20`
pub const SB_MAXCLIENTS_INTER: c_int = (SB_STATUSBAR - SB_TOP) / SB_INTER_HEIGHT - 1;

/// Raven `#define SB_LEFT_BOTICON_X (SCOREBOARD_X+0)`.
/// Source: `oracle/codemp/cgame/cg_scoreboard.c:26`
pub const SB_LEFT_BOTICON_X: c_int = SCOREBOARD_X + 0;

/// Raven `#define SB_LEFT_HEAD_X (SCOREBOARD_X+32)`.
/// Source: `oracle/codemp/cgame/cg_scoreboard.c:27`
pub const SB_LEFT_HEAD_X: c_int = SCOREBOARD_X + 32;

/// Raven `#define SB_RIGHT_BOTICON_X (SCOREBOARD_X+64)`.
/// Source: `oracle/codemp/cgame/cg_scoreboard.c:28`
pub const SB_RIGHT_BOTICON_X: c_int = SCOREBOARD_X + 64;

/// Raven `#define SB_RIGHT_HEAD_X (SCOREBOARD_X+96)`.
/// Source: `oracle/codemp/cgame/cg_scoreboard.c:29`
pub const SB_RIGHT_HEAD_X: c_int = SCOREBOARD_X + 96;

/// Raven `#define SB_BOTICON_X (SCOREBOARD_X+32)`.
/// Source: `oracle/codemp/cgame/cg_scoreboard.c:31`
pub const SB_BOTICON_X: c_int = SCOREBOARD_X + 32;

/// Raven `#define SB_HEAD_X (SCOREBOARD_X+64)`.
/// Source: `oracle/codemp/cgame/cg_scoreboard.c:32`
pub const SB_HEAD_X: c_int = SCOREBOARD_X + 64;

/// Raven `#define SB_SCORELINE_X 100`.
/// Source: `oracle/codemp/cgame/cg_scoreboard.c:34`
pub const SB_SCORELINE_X: c_int = 100;

/// Raven `#define SB_SCORELINE_WIDTH (640 - SB_SCORELINE_X * 2)`.
/// Source: `oracle/codemp/cgame/cg_scoreboard.c:35`
pub const SB_SCORELINE_WIDTH: c_int = 640 - SB_SCORELINE_X * 2;

/// Raven `#define SB_RATING_WIDTH 0 // (6 * BIGCHAR_WIDTH)`.
/// Source: `oracle/codemp/cgame/cg_scoreboard.c:37`
pub const SB_RATING_WIDTH: c_int = 0;

/// Raven `#define SB_NAME_X (SB_SCORELINE_X)`.
/// Source: `oracle/codemp/cgame/cg_scoreboard.c:38`
pub const SB_NAME_X: c_int = SB_SCORELINE_X;

/// Raven `#define SB_SCORE_X (SB_SCORELINE_X + .55 * SB_SCORELINE_WIDTH)` — float
/// math in the macro, so it lands as `f32` (`cg_draw.rs`'s precedent for
/// float-literal `#define`s).
/// Source: `oracle/codemp/cgame/cg_scoreboard.c:39`
pub const SB_SCORE_X: f32 = SB_SCORELINE_X as f32 + 0.55 * SB_SCORELINE_WIDTH as f32;

/// Raven `#define SB_PING_X (SB_SCORELINE_X + .70 * SB_SCORELINE_WIDTH)`.
/// Source: `oracle/codemp/cgame/cg_scoreboard.c:40`
pub const SB_PING_X: f32 = SB_SCORELINE_X as f32 + 0.70 * SB_SCORELINE_WIDTH as f32;

/// Raven `#define SB_TIME_X (SB_SCORELINE_X + .85 * SB_SCORELINE_WIDTH)`.
/// Source: `oracle/codemp/cgame/cg_scoreboard.c:41`
pub const SB_TIME_X: f32 = SB_SCORELINE_X as f32 + 0.85 * SB_SCORELINE_WIDTH as f32;

/// Raven `CG_GetClassCount` — counts connected, valid, `team`-matching clients
/// whose siege class shader (looked up through `bgSiegeClasses` by
/// `clientInfo_t.siegeIndex`) equals `siegeClass`.
///
/// Source: `oracle/codemp/cgame/cg_scoreboard.c:263-292`
pub fn CG_GetClassCount(world: &CgWorld, team: team_t, siegeClass: c_int) -> c_int {
    let mut count: c_int = 0;

    for i in 0..world.cgs.maxclients as usize {
        let ci = &world.cgs.clientinfo[i];

        if ci.infoValid == qfalse || team != ci.team {
            continue;
        }

        // Raven indexes `bgSiegeClasses[ci->siegeIndex]` unguarded here
        // (`cg_scoreboard.c:279`) - unlike the scoreboard icon draw elsewhere
        // in this file, which checks `siegeIndex != -1` first
        // (`cg_scoreboard.c:136`). A disconnected/non-siege client's -1 reads
        // memory before the array in Raven (UB); the port's `as usize` wraps
        // it huge and the array index panics instead (§F19).
        let scl = &world.bg_state.bgSiegeClasses[ci.siegeIndex as usize];

        // Correct class?
        if siegeClass != scl.classShader {
            continue;
        }

        count += 1;
    }

    count
}

/// Raven `CG_GetTeamNonScoreCount` — counts valid clients on `team`, also
/// counting anyone whose siege desired-team (pre-spawn team pick) is `team`
/// even if their live team hasn't switched yet.
///
/// Source: `oracle/codemp/cgame/cg_scoreboard.c:294-312`
pub fn CG_GetTeamNonScoreCount(world: &CgWorld, team: team_t) -> c_int {
    let mut count: c_int = 0;

    for i in 0..world.cgs.maxclients as usize {
        let ci = &world.cgs.clientinfo[i];

        if ci.infoValid == qfalse || (team != ci.team && team != ci.siegeDesiredTeam) {
            continue;
        }

        count += 1;
    }

    count
}

/// Raven `CG_GetTeamCount` — counts `team`-matching clients among the first
/// `cg.numScores` scoreboard entries, stopping early once `count` reaches
/// `maxClients`.
///
/// Source: `oracle/codemp/cgame/cg_scoreboard.c:314-336`
pub fn CG_GetTeamCount(world: &CgWorld, team: team_t, maxClients: c_int) -> c_int {
    let mut count: c_int = 0;
    let mut i: c_int = 0;

    while i < world.cg.numScores && count < maxClients {
        let score = &world.cg.scores[i as usize];
        let ci = &world.cgs.clientinfo[score.client as usize];

        if team == ci.team {
            count += 1;
        }

        i += 1;
    }

    count
}
