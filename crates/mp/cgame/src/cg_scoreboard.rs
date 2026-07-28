//! Port of `oracle/codemp/cgame/cg_scoreboard.c` — the scoreboard layout and its per-client rows. Functions land via the C5
//! transcription waves.

#![allow(non_snake_case)]

use core::ffi::c_int;

use mp_bg::public::configstring::RANK_TIED_FLAG;
use mp_bg::public::duel_team::duelTeam_t;
use mp_bg::public::gametype::{GT_DUEL, GT_POWERDUEL, GT_SIEGE, GT_TEAM};
use mp_bg::public::pers_enum::persEnum_t::{PERS_RANK, PERS_SCORE, PERS_TEAM};
use mp_bg::public::pmtype::pmtype_t;
use mp_bg::public::powerup::{PW_BLUEFLAG, PW_NEUTRALFLAG, PW_REDFLAG};
use mp_bg::public::stat_index::statIndex_t::STAT_CLIENTS_READY;
use mp_bg::public::team::{team_t, TEAM_BLUE, TEAM_FREE, TEAM_RED, TEAM_SPECTATOR};
use mp_qshared::shared::q_color::colorWhite;
use mp_qshared::shared::screen::SCREEN_WIDTH;
use mp_qshared::shared::{ct_table_t, qfalse, vec4_t, BIGCHAR_HEIGHT};
use mp_uishared::shared::display_state::DisplayState;
use mp_uishared::shared::menudef::ITEM_TEXTSTYLE_OUTLINED;
use native_string::{buf_to_string, latin1_to_string};

use crate::cg_draw::{
    colorTable, CG_DrawFlagModel, CG_DrawTeamBackground, CG_Text_Paint, CG_Text_Width,
};
use crate::cg_drawtools::{CG_DrawPic, CG_FadeColor, CG_FillRect, UI_DrawProportionalString};
use crate::cg_event::CG_PlaceString;
use crate::cg_main::{CG_GetStringEdString, Com_Printf};
use crate::cg_players::CG_LoadDeferredPlayers;
use crate::local::score_t::score_t;
use crate::trap;
use crate::world::cg_context::CgContext;
use crate::world::cg_world::CgWorld;

/// Raven `#define UI_CENTER 0x00000001` — each TU keeps its own copy of the
/// `displayContextDef_t` style flags (`cg_drawtools.rs`'s precedent).
/// Source: `oracle/codemp/cgame/cg_local.h`
const UI_CENTER: c_int = 0x0000_0001;

/// Raven `#define UI_DROPSHADOW 0x00000800`.
/// Source: `oracle/codemp/cgame/cg_local.h`
const UI_DROPSHADOW: c_int = 0x0000_0800;

/// Raven `#define FADE_TIME 200` — file-local copy (`cg_drawtools.rs`'s
/// precedent - each TU keeps its own).
/// Source: `oracle/codemp/cgame/cg_drawtools.c:25`
const FADE_TIME: c_int = 200;

// PORT-NOTE: `q_shared.h`'s font enum is anonymous, so per the anonymous-enum
// convention these are file-local `const`s, same story as `cg_draw.rs`'s own
// copies (each TU keeps its own - they're never exported).
/// Source: `oracle/codemp/game/q_shared.h:3176-3182`
const FONT_SMALL: c_int = 1;
/// Source: `oracle/codemp/game/q_shared.h:3176-3182`
const FONT_MEDIUM: c_int = 2;

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

/// Raven `CG_DrawClientScore` — one scoreboard row: the class/flag icon, the
/// local-client highlight bar, name, and either score/ping/time or dashes
/// when the player has never reported in.
///
/// Source: `oracle/codemp/cgame/cg_scoreboard.c:62-225`
#[allow(clippy::too_many_arguments)]
pub fn CG_DrawClientScore(
    ctx: &mut CgContext,
    ds: &DisplayState,
    y: c_int,
    score: &score_t,
    // Raven's `color` param is never read in this function's body.
    _color: &vec4_t,
    fade: f32,
    largeFormat: bool,
) {
    // vec3_t headAngles;

    let scale: f32 = if largeFormat { 1.0 } else { 0.75 };

    if score.client < 0 || score.client >= ctx.world.cgs.maxclients {
        Com_Printf(ctx, &format!("Bad score->client: {}\n", score.client));
        return;
    }

    let ci = &ctx.world.cgs.clientinfo[score.client as usize];
    let powerups = ci.powerups;
    let duelTeam = ci.duelTeam;
    let siegeIndex = ci.siegeIndex;
    let team = ci.team;
    let wins = ci.wins;
    let losses = ci.losses;
    let name = buf_to_string(&ci.name.map(|c| c as u8));

    let gametype = ctx.world.cgs.gametype;

    let iconx = SB_BOTICON_X + (SB_RATING_WIDTH / 2);
    // Raven computes `headx` too, but never reads it back in this function.
    let _headx = SB_HEAD_X + (SB_RATING_WIDTH / 2);

    // draw the handicap or bot skill marker (unless player has flag)
    if (powerups & (1 << PW_NEUTRALFLAG)) != 0 {
        if largeFormat {
            CG_DrawFlagModel(
                ctx,
                iconx as f32,
                (y - (32 - BIGCHAR_HEIGHT) / 2) as f32,
                32.0,
                32.0,
                TEAM_FREE,
                false,
            );
        } else {
            CG_DrawFlagModel(ctx, iconx as f32, y as f32, 16.0, 16.0, TEAM_FREE, false);
        }
    } else if (powerups & (1 << PW_REDFLAG)) != 0 {
        // PORT-NOTE: Raven's `largeFormat` if/else draws the byte-identical
        // call in both arms (`cg_scoreboard.c:102-109`) - collapsed since
        // there is no behavioral difference (porting-rules §C10).
        let screenXScale = ctx.world.cgs.screenXScale;
        let screenYScale = ctx.world.cgs.screenYScale;
        CG_DrawFlagModel(
            ctx,
            iconx as f32 * screenXScale,
            y as f32 * screenYScale,
            32.0 * screenXScale,
            32.0 * screenYScale,
            TEAM_RED,
            false,
        );
    } else if (powerups & (1 << PW_BLUEFLAG)) != 0 {
        // PORT-NOTE: same Raven redundant if/else as the red-flag arm above
        // (`cg_scoreboard.c:113-120`).
        let screenXScale = ctx.world.cgs.screenXScale;
        let screenYScale = ctx.world.cgs.screenYScale;
        CG_DrawFlagModel(
            ctx,
            iconx as f32 * screenXScale,
            y as f32 * screenYScale,
            32.0 * screenXScale,
            32.0 * screenYScale,
            TEAM_BLUE,
            false,
        );
    } else if gametype == GT_POWERDUEL
        && (duelTeam == duelTeam_t::DUELTEAM_LONE as c_int
            || duelTeam == duelTeam_t::DUELTEAM_DOUBLE as c_int)
    {
        if duelTeam == duelTeam_t::DUELTEAM_LONE as c_int {
            let shader = trap::R_RegisterShaderNoMip(ctx.engine, "gfx/mp/pduel_icon_lone");
            CG_DrawPic(ctx, iconx as f32, y as f32, 32.0, 32.0, shader);
        } else {
            let shader = trap::R_RegisterShaderNoMip(ctx.engine, "gfx/mp/pduel_icon_double");
            CG_DrawPic(ctx, iconx as f32, y as f32, 32.0, 32.0, shader);
        }
    } else if gametype == GT_SIEGE {
        // try to draw the shader for this class on the scoreboard
        if siegeIndex != -1 {
            let classShader = ctx.world.bg_state.bgSiegeClasses[siegeIndex as usize].classShader;

            if classShader != 0 {
                let wh = if largeFormat { 24.0 } else { 12.0 };
                CG_DrawPic(ctx, iconx as f32, y as f32, wh, wh, classShader);
            }
        }
    }
    // else: draw the wins/losses - Raven leaves this dead/commented out
    // ("rww - in duel, we now show wins/losses in place of 'frags'. This is
    // because duel now defaults to 1 kill per round."), `cg_scoreboard.c:146-156`.

    // highlight your position
    // §F19: Raven derefs `cg.snap` unguarded for both reads below; with no
    // snapshot yet, this row is never the local client's and there is no
    // ready marker to draw.
    let snap_local = ctx
        .world
        .cg
        .snap_ref()
        .map(|snap| (snap.ps.clientNum, snap.ps.persistant));

    if let Some((snapClientNum, persistant)) = snap_local {
        if score.client == snapClientNum {
            ctx.world.scoreboard.localClient = true;

            let rank = if persistant[PERS_TEAM as usize] == TEAM_SPECTATOR || gametype >= GT_TEAM {
                -1
            } else {
                persistant[PERS_RANK as usize] & !RANK_TIED_FLAG
            };

            let mut hcolor: vec4_t = if rank == 0 {
                [0.0, 0.0, 0.7, 0.0]
            } else if rank == 1 {
                [0.7, 0.0, 0.0, 0.0]
            } else if rank == 2 {
                [0.7, 0.7, 0.0, 0.0]
            } else {
                [0.7, 0.7, 0.7, 0.0]
            };

            hcolor[3] = fade * 0.7;
            CG_FillRect(
                ctx,
                (SB_SCORELINE_X - 5) as f32,
                (y + 2) as f32,
                (640 - SB_SCORELINE_X * 2 + 10) as f32,
                (if largeFormat {
                    SB_NORMAL_HEIGHT
                } else {
                    SB_INTER_HEIGHT
                }) as f32,
                &hcolor,
            );
        }
    }

    CG_Text_Paint(
        ctx,
        ds,
        SB_NAME_X as f32,
        y as f32,
        0.9 * scale,
        colorWhite,
        &name,
        0.0,
        0,
        ITEM_TEXTSTYLE_OUTLINED,
        FONT_MEDIUM,
    );

    if score.ping != -1 {
        if team != TEAM_SPECTATOR || gametype == GT_DUEL || gametype == GT_POWERDUEL {
            if gametype == GT_DUEL || gametype == GT_POWERDUEL {
                CG_Text_Paint(
                    ctx,
                    ds,
                    SB_SCORE_X,
                    y as f32,
                    1.0 * scale,
                    colorWhite,
                    &format!("{wins}/{losses}"),
                    0.0,
                    0,
                    ITEM_TEXTSTYLE_OUTLINED,
                    FONT_SMALL,
                );
            } else {
                CG_Text_Paint(
                    ctx,
                    ds,
                    SB_SCORE_X,
                    y as f32,
                    1.0 * scale,
                    colorWhite,
                    &format!("{}", score.score),
                    0.0,
                    0,
                    ITEM_TEXTSTYLE_OUTLINED,
                    FONT_SMALL,
                );
            }
        }

        CG_Text_Paint(
            ctx,
            ds,
            SB_PING_X,
            y as f32,
            1.0 * scale,
            colorWhite,
            &format!("{}", score.ping),
            0.0,
            0,
            ITEM_TEXTSTYLE_OUTLINED,
            FONT_SMALL,
        );
        CG_Text_Paint(
            ctx,
            ds,
            SB_TIME_X,
            y as f32,
            1.0 * scale,
            colorWhite,
            &format!("{}", score.time),
            0.0,
            0,
            ITEM_TEXTSTYLE_OUTLINED,
            FONT_SMALL,
        );
    } else {
        CG_Text_Paint(
            ctx,
            ds,
            SB_SCORE_X,
            y as f32,
            1.0 * scale,
            colorWhite,
            "-",
            0.0,
            0,
            ITEM_TEXTSTYLE_OUTLINED,
            FONT_SMALL,
        );
        CG_Text_Paint(
            ctx,
            ds,
            SB_PING_X,
            y as f32,
            1.0 * scale,
            colorWhite,
            "-",
            0.0,
            0,
            ITEM_TEXTSTYLE_OUTLINED,
            FONT_SMALL,
        );
        CG_Text_Paint(
            ctx,
            ds,
            SB_TIME_X,
            y as f32,
            1.0 * scale,
            colorWhite,
            "-",
            0.0,
            0,
            ITEM_TEXTSTYLE_OUTLINED,
            FONT_SMALL,
        );
    }

    // add the "ready" marker for intermission exiting
    let stats_ready = ctx
        .world
        .cg
        .snap_ref()
        .map(|snap| snap.ps.stats[STAT_CLIENTS_READY as usize]);

    if let Some(stats) = stats_ready {
        if (stats & (1 << score.client)) != 0 {
            let ready = CG_GetStringEdString(ctx, "MP_INGAME", "READY");
            CG_Text_Paint(
                ctx,
                ds,
                (SB_NAME_X - 64) as f32,
                (y + 2) as f32,
                0.7 * scale,
                colorWhite,
                &ready,
                0.0,
                0,
                ITEM_TEXTSTYLE_OUTLINED,
                FONT_MEDIUM,
            );
        }
    }
}

/// Raven `CG_TeamScoreboard` — draws (or, with `countOnly`, just tallies) the
/// rows of `cg.scores` whose client is on `team`, stopping once `maxClients`
/// rows have been counted.
///
/// Source: `oracle/codemp/cgame/cg_scoreboard.c:232-261`
#[allow(clippy::too_many_arguments)]
pub fn CG_TeamScoreboard(
    ctx: &mut CgContext,
    ds: &DisplayState,
    y: c_int,
    team: team_t,
    fade: f32,
    maxClients: c_int,
    lineHeight: c_int,
    countOnly: bool,
) -> c_int {
    let color: vec4_t = [1.0, 1.0, 1.0, fade];

    let mut count: c_int = 0;
    let mut i: c_int = 0;

    while i < ctx.world.cg.numScores && count < maxClients {
        let score_ref = &ctx.world.cg.scores[i as usize];
        let ci = &ctx.world.cgs.clientinfo[score_ref.client as usize];

        if team != ci.team {
            i += 1;
            continue;
        }

        if !countOnly {
            // `CG_DrawClientScore` takes `ctx: &mut CgContext`, which would
            // alias the `&score_t` borrowed straight out of `ctx.world.cg.scores`
            // above - copy the row out first (same fix `zeroed_score`'s sibling
            // fn in `cg_servercmds.rs` uses; `score_t` has no `Clone`).
            let score_ref = &ctx.world.cg.scores[i as usize];
            let score = score_t {
                client: score_ref.client,
                score: score_ref.score,
                ping: score_ref.ping,
                time: score_ref.time,
                scoreFlags: score_ref.scoreFlags,
                powerUps: score_ref.powerUps,
                accuracy: score_ref.accuracy,
                impressiveCount: score_ref.impressiveCount,
                excellentCount: score_ref.excellentCount,
                guantletCount: score_ref.guantletCount,
                defendCount: score_ref.defendCount,
                assistCount: score_ref.assistCount,
                captures: score_ref.captures,
                perfect: score_ref.perfect,
                team: score_ref.team,
            };

            CG_DrawClientScore(
                ctx,
                ds,
                y + lineHeight * count,
                &score,
                &color,
                fade,
                lineHeight == SB_NORMAL_HEIGHT,
            );
        }

        count += 1;
        i += 1;
    }

    count
}

/// Raven `CG_DrawOldScoreboard` — the legacy (non-`ui_shared`) scoreboard:
/// the duel-winner/killed-by banner, the current-rank or team-lead line, the
/// header row, the per-team score columns, and the local-client row pinned
/// to the bottom when it didn't already print in its team's block.
///
/// Source: `oracle/codemp/cgame/cg_scoreboard.c:345-636`
pub fn CG_DrawOldScoreboard(ctx: &mut CgContext, ds: &DisplayState) -> bool {
    // don't draw amuthing if the menu or console is up
    if ctx.world.cvars.cg_paused.integer != 0 {
        ctx.world.cg.deferredPlayerLoading = 0;
        return false;
    }

    // don't draw scoreboard during death while warmup up
    if ctx.world.cg.warmup != 0 && ctx.world.cg.showScores == qfalse {
        return false;
    }

    let fade: f32;
    let fadeColor: vec4_t;

    if ctx.world.cg.showScores != qfalse
        || ctx.world.cg.predictedPlayerState.pm_type == pmtype_t::PM_DEAD as c_int
        || ctx.world.cg.predictedPlayerState.pm_type == pmtype_t::PM_INTERMISSION as c_int
    {
        fade = 1.0;
        fadeColor = colorWhite;
    } else {
        match CG_FadeColor(ctx.world, ctx.world.cg.scoreFadeTime, FADE_TIME) {
            Some(c) => {
                fade = c[0];
                fadeColor = c;
            }
            None => {
                // next time scoreboard comes up, don't print killer
                ctx.world.cg.deferredPlayerLoading = 0;
                ctx.world.cg.killerName[0] = 0;
                return false;
            }
        }
    }

    // fragged by ... line
    // or if in intermission and duel, prints the winner of the duel round
    if (ctx.world.cgs.gametype == GT_DUEL || ctx.world.cgs.gametype == GT_POWERDUEL)
        && ctx.world.cgs.duelWinner != -1
        && ctx.world.cg.predictedPlayerState.pm_type == pmtype_t::PM_INTERMISSION as c_int
    {
        // §F19: `duelWinner` is a server-supplied configstring index; skip
        // the banner instead of indexing OOB if it's ever out of range.
        let name = ctx
            .world
            .cgs
            .clientinfo
            .get(ctx.world.cgs.duelWinner as usize)
            .map(|ci| buf_to_string(&ci.name.map(|c| c as u8)));

        if let Some(name) = name {
            let duelWins = CG_GetStringEdString(ctx, "MP_INGAME", "DUEL_WINS");
            let s = format!("{name}^7 {duelWins}");
            let x = SCREEN_WIDTH / 2;
            let y = 40;
            let width = CG_Text_Width(ctx, ds, &s, 1.0, FONT_MEDIUM);
            CG_Text_Paint(
                ctx,
                ds,
                (x - width / 2) as f32,
                y as f32,
                1.0,
                colorWhite,
                &s,
                0.0,
                0,
                ITEM_TEXTSTYLE_OUTLINED,
                FONT_MEDIUM,
            );
        }
    } else if (ctx.world.cgs.gametype == GT_DUEL || ctx.world.cgs.gametype == GT_POWERDUEL)
        && ctx.world.cgs.duelist1 != -1
        && ctx.world.cgs.duelist2 != -1
        && ctx.world.cg.predictedPlayerState.pm_type == pmtype_t::PM_INTERMISSION as c_int
    {
        // §F19: same OOB-index caution as `duelWinner` above.
        let name1 = ctx
            .world
            .cgs
            .clientinfo
            .get(ctx.world.cgs.duelist1 as usize)
            .map(|ci| buf_to_string(&ci.name.map(|c| c as u8)));
        let name2 = ctx
            .world
            .cgs
            .clientinfo
            .get(ctx.world.cgs.duelist2 as usize)
            .map(|ci| buf_to_string(&ci.name.map(|c| c as u8)));

        if let (Some(name1), Some(name2)) = (name1, name2) {
            let s = if ctx.world.cgs.gametype == GT_POWERDUEL && ctx.world.cgs.duelist3 != -1 {
                let name3 = ctx
                    .world
                    .cgs
                    .clientinfo
                    .get(ctx.world.cgs.duelist3 as usize)
                    .map(|ci| buf_to_string(&ci.name.map(|c| c as u8)));

                name3.map(|name3| {
                    let versus = CG_GetStringEdString(ctx, "MP_INGAME", "SPECHUD_VERSUS");
                    let and = CG_GetStringEdString(ctx, "MP_INGAME", "AND");
                    format!("{name1}^7 {versus} {name2}^7 {and} {name3}")
                })
            } else {
                let versus = CG_GetStringEdString(ctx, "MP_INGAME", "SPECHUD_VERSUS");
                Some(format!("{name1}^7 {versus} {name2}"))
            };

            if let Some(s) = s {
                let x = SCREEN_WIDTH / 2;
                let y = 40;
                let width = CG_Text_Width(ctx, ds, &s, 1.0, FONT_MEDIUM);
                CG_Text_Paint(
                    ctx,
                    ds,
                    (x - width / 2) as f32,
                    y as f32,
                    1.0,
                    colorWhite,
                    &s,
                    0.0,
                    0,
                    ITEM_TEXTSTYLE_OUTLINED,
                    FONT_MEDIUM,
                );
            }
        }
    } else if ctx.world.cg.killerName[0] != 0 {
        let killer: Vec<u8> = ctx
            .world
            .cg
            .killerName
            .iter()
            .take_while(|&&c| c != 0)
            .map(|&c| c as u8)
            .collect();
        let killerName = latin1_to_string(&killer);
        let killedBy = CG_GetStringEdString(ctx, "MP_INGAME", "KILLEDBY");
        let s = format!("{killedBy} {killerName}");
        let x = SCREEN_WIDTH / 2;
        let y = 40;
        let width = CG_Text_Width(ctx, ds, &s, 1.0, FONT_MEDIUM);
        CG_Text_Paint(
            ctx,
            ds,
            (x - width / 2) as f32,
            y as f32,
            1.0,
            colorWhite,
            &s,
            0.0,
            0,
            ITEM_TEXTSTYLE_OUTLINED,
            FONT_MEDIUM,
        );
    }

    // current rank
    if ctx.world.cgs.gametype == GT_POWERDUEL {
        // do nothing?
    } else if ctx.world.cgs.gametype < GT_TEAM {
        // §F19: Raven derefs `cg.snap` unguarded here; with no snapshot yet
        // there's no rank line to draw.
        let snap_data = ctx.world.cg.snap_ref().map(|snap| {
            (
                snap.ps.persistant[PERS_TEAM as usize],
                snap.ps.persistant[PERS_RANK as usize],
                snap.ps.persistant[PERS_SCORE as usize],
            )
        });

        if let Some((team, rank, score)) = snap_data {
            if team != TEAM_SPECTATOR {
                let sPlace = trap::SP_GetStringTextString(ctx.engine, "MP_INGAME_PLACE", 256)
                    .unwrap_or_else(|| "??MP_INGAME_PLACE".to_string());
                let sOf = trap::SP_GetStringTextString(ctx.engine, "MP_INGAME_OF", 256)
                    .unwrap_or_else(|| "??MP_INGAME_OF".to_string());
                let sWith = trap::SP_GetStringTextString(ctx.engine, "MP_INGAME_WITH", 256)
                    .unwrap_or_else(|| "??MP_INGAME_WITH".to_string());

                let place = CG_PlaceString(ctx, rank + 1);
                let numScores = ctx.world.cg.numScores;

                // Raven also computes `w = CG_DrawStrlen(s) * BIGCHAR_WIDTH`
                // here, but the big-string draw it fed is commented out
                // (`cg_scoreboard.c:453`) - the pure dead store is dropped.
                let s = format!("{place} {sPlace} ({sOf} {numScores}) {sWith} {score}");
                let x = SCREEN_WIDTH / 2;
                let y = 60;
                UI_DrawProportionalString(
                    ctx,
                    ds,
                    x,
                    y,
                    &s,
                    UI_CENTER | UI_DROPSHADOW,
                    colorTable[ct_table_t::CT_WHITE as usize],
                );
            }
        }
    } else if ctx.world.cgs.gametype != GT_SIEGE {
        let teamScores0 = ctx.world.cg.teamScores[0];
        let teamScores1 = ctx.world.cg.teamScores[1];

        let s = if teamScores0 == teamScores1 {
            let tiedAt = CG_GetStringEdString(ctx, "MP_INGAME", "TIEDAT");
            format!("{tiedAt} {teamScores0}")
        } else if teamScores0 >= teamScores1 {
            let redLeads = CG_GetStringEdString(ctx, "MP_INGAME", "RED_LEADS");
            format!("{redLeads}, {teamScores0} / {teamScores1}")
        } else {
            let blueLeads = CG_GetStringEdString(ctx, "MP_INGAME", "BLUE_LEADS");
            format!("{blueLeads}, {teamScores1} / {teamScores0}")
        };

        let x = SCREEN_WIDTH / 2;
        let y = 60;
        CG_Text_Paint(
            ctx,
            ds,
            x as f32,
            y as f32,
            1.0,
            colorWhite,
            &s,
            0.0,
            0,
            ITEM_TEXTSTYLE_OUTLINED,
            FONT_MEDIUM,
        );
    } else if ctx.world.cgs.gametype == GT_SIEGE
        && (ctx.world.scoreboard.cg_siegeWinTeam == 1 || ctx.world.scoreboard.cg_siegeWinTeam == 2)
    {
        let s = if ctx.world.scoreboard.cg_siegeWinTeam == 1 {
            CG_GetStringEdString(ctx, "MP_INGAME", "SIEGETEAM1WIN")
        } else {
            CG_GetStringEdString(ctx, "MP_INGAME", "SIEGETEAM2WIN")
        };

        let x = SCREEN_WIDTH / 2;
        let y = 60;
        CG_Text_Paint(
            ctx,
            ds,
            x as f32,
            y as f32,
            1.0,
            colorWhite,
            &s,
            0.0,
            0,
            ITEM_TEXTSTYLE_OUTLINED,
            FONT_MEDIUM,
        );
    }

    // scoreboard
    let mut y = SB_HEADER;

    let backShader = trap::R_RegisterShaderNoMip(ctx.engine, "gfx/menus/menu_buttonback.tga");
    CG_DrawPic(
        ctx,
        (SB_SCORELINE_X - 40) as f32,
        (y - 5) as f32,
        (SB_SCORELINE_WIDTH + 80) as f32,
        40.0,
        backShader,
    );

    let name_hdr = CG_GetStringEdString(ctx, "MP_INGAME", "NAME");
    CG_Text_Paint(
        ctx,
        ds,
        SB_NAME_X as f32,
        y as f32,
        1.0,
        colorWhite,
        &name_hdr,
        0.0,
        0,
        ITEM_TEXTSTYLE_OUTLINED,
        FONT_MEDIUM,
    );

    if ctx.world.cgs.gametype == GT_DUEL || ctx.world.cgs.gametype == GT_POWERDUEL {
        let sWL = trap::SP_GetStringTextString(ctx.engine, "MP_INGAME_W_L", 100)
            .unwrap_or_else(|| "??MP_INGAME_W_L".to_string());

        CG_Text_Paint(
            ctx,
            ds,
            SB_SCORE_X,
            y as f32,
            1.0,
            colorWhite,
            &sWL,
            0.0,
            0,
            ITEM_TEXTSTYLE_OUTLINED,
            FONT_MEDIUM,
        );
    } else {
        let score_hdr = CG_GetStringEdString(ctx, "MP_INGAME", "SCORE");
        CG_Text_Paint(
            ctx,
            ds,
            SB_SCORE_X,
            y as f32,
            1.0,
            colorWhite,
            &score_hdr,
            0.0,
            0,
            ITEM_TEXTSTYLE_OUTLINED,
            FONT_MEDIUM,
        );
    }

    let ping_hdr = CG_GetStringEdString(ctx, "MP_INGAME", "PING");
    CG_Text_Paint(
        ctx,
        ds,
        SB_PING_X,
        y as f32,
        1.0,
        colorWhite,
        &ping_hdr,
        0.0,
        0,
        ITEM_TEXTSTYLE_OUTLINED,
        FONT_MEDIUM,
    );
    let time_hdr = CG_GetStringEdString(ctx, "MP_INGAME", "TIME");
    CG_Text_Paint(
        ctx,
        ds,
        SB_TIME_X,
        y as f32,
        1.0,
        colorWhite,
        &time_hdr,
        0.0,
        0,
        ITEM_TEXTSTYLE_OUTLINED,
        FONT_MEDIUM,
    );

    y = SB_TOP;

    // If there are more than SB_MAXCLIENTS_NORMAL, use the interleaved scores
    let (mut maxClients, lineHeight, topBorderSize, bottomBorderSize) =
        if ctx.world.cg.numScores > SB_MAXCLIENTS_NORMAL {
            (SB_MAXCLIENTS_INTER, SB_INTER_HEIGHT, 8, 16)
        } else {
            (SB_MAXCLIENTS_NORMAL, SB_NORMAL_HEIGHT, 8, 8)
        };

    ctx.world.scoreboard.localClient = false;

    // I guess this should end up being able to display 19 clients at once.
    // In a team game, if there are 9 or more clients on the team not in the lead,
    // we only want to show 10 of the clients on the team in the lead, so that we
    // have room to display the clients in the lead on the losing team.

    // I guess this can be accomplished simply by printing the first teams score with a maxClients
    // value passed in related to how many players are on both teams.
    if ctx.world.cgs.gametype >= GT_TEAM {
        // teamplay scoreboard
        y += lineHeight / 2;

        let (leadTeam, trailTeam) = if ctx.world.cg.teamScores[0] >= ctx.world.cg.teamScores[1] {
            (TEAM_RED, TEAM_BLUE)
        } else {
            (TEAM_BLUE, TEAM_RED)
        };

        let mut team1MaxCl = CG_GetTeamCount(ctx.world, leadTeam, maxClients);
        let mut team2MaxCl = CG_GetTeamCount(ctx.world, trailTeam, maxClients);

        if team1MaxCl > 10 && (team1MaxCl + team2MaxCl) > maxClients {
            team1MaxCl -= team2MaxCl;
            // subtract as many as you have to down to 10, once we get there
            // we just set it to 10
            if team1MaxCl < 10 {
                team1MaxCl = 10;
            }
        }

        // team2 can display however many is left over after team1's display
        team2MaxCl = maxClients - team1MaxCl;

        let n1 = CG_TeamScoreboard(ctx, ds, y, leadTeam, fade, team1MaxCl, lineHeight, true);
        CG_DrawTeamBackground(
            ctx,
            SB_SCORELINE_X - 5,
            y - topBorderSize,
            640 - SB_SCORELINE_X * 2 + 10,
            n1 * lineHeight + bottomBorderSize,
            0.33,
            leadTeam,
        );
        CG_TeamScoreboard(ctx, ds, y, leadTeam, fade, team1MaxCl, lineHeight, false);
        y += (n1 * lineHeight) + BIGCHAR_HEIGHT;

        let n2 = CG_TeamScoreboard(ctx, ds, y, trailTeam, fade, team2MaxCl, lineHeight, true);
        CG_DrawTeamBackground(
            ctx,
            SB_SCORELINE_X - 5,
            y - topBorderSize,
            640 - SB_SCORELINE_X * 2 + 10,
            n2 * lineHeight + bottomBorderSize,
            0.33,
            trailTeam,
        );
        CG_TeamScoreboard(ctx, ds, y, trailTeam, fade, team2MaxCl, lineHeight, false);
        y += (n2 * lineHeight) + BIGCHAR_HEIGHT;

        maxClients -= team1MaxCl + team2MaxCl;

        let n1 = CG_TeamScoreboard(
            ctx,
            ds,
            y,
            TEAM_SPECTATOR,
            fade,
            maxClients,
            lineHeight,
            false,
        );
        y += (n1 * lineHeight) + BIGCHAR_HEIGHT;
    } else {
        // free for all scoreboard
        let n1 = CG_TeamScoreboard(ctx, ds, y, TEAM_FREE, fade, maxClients, lineHeight, false);
        y += (n1 * lineHeight) + BIGCHAR_HEIGHT;
        let n2 = CG_TeamScoreboard(
            ctx,
            ds,
            y,
            TEAM_SPECTATOR,
            fade,
            maxClients - n1,
            lineHeight,
            false,
        );
        y += (n2 * lineHeight) + BIGCHAR_HEIGHT;
    }

    if !ctx.world.scoreboard.localClient {
        // draw local client at the bottom
        // §F19: Raven derefs `cg.snap` unguarded for the clientNum compare;
        // with no snapshot yet, no row matches.
        let localClientNum = ctx.world.cg.snap_ref().map(|snap| snap.ps.clientNum);

        if let Some(localClientNum) = localClientNum {
            for i in 0..ctx.world.cg.numScores as usize {
                if ctx.world.cg.scores[i].client == localClientNum {
                    // `CG_DrawClientScore` takes `ctx: &mut CgContext`, which
                    // would alias the `&score_t` borrowed straight out of
                    // `ctx.world.cg.scores` above - copy the row out first
                    // (`CG_TeamScoreboard`'s own precedent above).
                    let score_ref = &ctx.world.cg.scores[i];
                    let score = score_t {
                        client: score_ref.client,
                        score: score_ref.score,
                        ping: score_ref.ping,
                        time: score_ref.time,
                        scoreFlags: score_ref.scoreFlags,
                        powerUps: score_ref.powerUps,
                        accuracy: score_ref.accuracy,
                        impressiveCount: score_ref.impressiveCount,
                        excellentCount: score_ref.excellentCount,
                        guantletCount: score_ref.guantletCount,
                        defendCount: score_ref.defendCount,
                        assistCount: score_ref.assistCount,
                        captures: score_ref.captures,
                        perfect: score_ref.perfect,
                        team: score_ref.team,
                    };

                    CG_DrawClientScore(
                        ctx,
                        ds,
                        y,
                        &score,
                        &fadeColor,
                        fade,
                        lineHeight == SB_NORMAL_HEIGHT,
                    );
                    break;
                }
            }
        }
    }

    // load any models that have been deferred
    ctx.world.cg.deferredPlayerLoading += 1;
    if ctx.world.cg.deferredPlayerLoading > 10 {
        CG_LoadDeferredPlayers(ctx.world);
    }

    true
}
