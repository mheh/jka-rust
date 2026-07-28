//! Port of `oracle/codemp/cgame/cg_newDraw.c` — the menu-framework owner-draw and feeder surface cgame exposes. Functions land via the C5
//! transcription waves.

#![allow(non_snake_case)]

use core::ffi::c_int;

use mp_bg::public::gametype::{
    GT_CTF, GT_CTY, GT_DUEL, GT_FFA, GT_HOLOCRON, GT_JEDIMASTER, GT_POWERDUEL, GT_SIEGE,
    GT_SINGLE_PLAYER, GT_TEAM,
};
use mp_bg::public::pers_enum::persEnum_t::{PERS_RANK, PERS_SCORE, PERS_TEAM};
use mp_bg::public::pmtype::pmtype_t;
use mp_bg::public::powerup::{PW_BLUEFLAG, PW_NEUTRALFLAG, PW_REDFLAG};
use mp_bg::public::stat_index::statIndex_t;
use mp_bg::public::team::{TEAM_BLUE, TEAM_RED, TEAM_SPECTATOR};
use mp_bg::public::teamtask::teamtask_t;
use mp_bg::weapons::weapon_data::weaponData;

use mp_qshared::shared::{qfalse, qhandle_t, vec4_t, FLAG_TAKEN, FLAG_TAKEN_BLUE, FLAG_TAKEN_RED};

use mp_uishared::shared::display_state::DisplayState;
use mp_uishared::shared::menu_system::MenuSystem;
use mp_uishared::shared::menudef::{
    CG_BLUE_SCORE, CG_PLAYER_AMMO_VALUE, CG_PLAYER_ARMOR_VALUE, CG_PLAYER_FORCE_VALUE,
    CG_PLAYER_HEALTH, CG_PLAYER_SCORE, CG_RED_SCORE, CG_SELECTEDPLAYER_ARMOR,
    CG_SELECTEDPLAYER_HEALTH, CG_SHOW_ANYNONTEAMGAME, CG_SHOW_ANYTEAMGAME,
    CG_SHOW_BLUE_TEAM_HAS_REDFLAG, CG_SHOW_CTF, CG_SHOW_DURINGINCOMINGVOICE,
    CG_SHOW_HEALTHCRITICAL, CG_SHOW_HEALTHOK, CG_SHOW_IF_PLAYER_HAS_FLAG, CG_SHOW_NOTEAMINFO,
    CG_SHOW_OTHERTEAMHASFLAG, CG_SHOW_RED_TEAM_HAS_BLUEFLAG, CG_SHOW_SINGLEPLAYER,
    CG_SHOW_TEAMINFO, CG_SHOW_TOURNAMENT, CG_SHOW_YOURTEAMHASENEMYFLAG,
};
use mp_uishared::ui_shared::{
    Display_CursorType, Display_MouseMove, Menus_CloseByName, Menus_OpenByName, CURSOR_ARROW,
    CURSOR_SIZER,
};

use native_string::{latin1_to_string, string_to_latin1, Q_stricmpBytes};

use crate::cg_event::CG_PlaceString;
use crate::cg_main::CG_GetStringEdString;
use crate::trap;
use crate::world::cg_context::CgContext;
use crate::world::cg_world::CgWorld;

/// Raven `#define PIC_WIDTH 12`.
///
/// Source: `oracle/codemp/cgame/cg_newDraw.c:324`
pub const PIC_WIDTH: c_int = 12;

/// Raven's anonymous `enum { CGAME_EVENT_NONE, CGAME_EVENT_TEAMMENU,
/// CGAME_EVENT_SCOREBOARD, CGAME_EVENT_EDITHUD }` - no typedef name, so these
/// land as plain `c_int` constants; only `CG_EventHandling` in this file
/// consumes them.
///
/// Source: `oracle/codemp/cgame/cg_public.h:37-42`
pub const CGAME_EVENT_NONE: c_int = 0;
pub const CGAME_EVENT_TEAMMENU: c_int = 1;
pub const CGAME_EVENT_SCOREBOARD: c_int = 2;
pub const CGAME_EVENT_EDITHUD: c_int = 3;

/// Raven `CG_GetSelectedPlayer` — the scoreboard-selected player index, reset
/// to 0 whenever it falls outside the current team roster.
///
/// Source: `oracle/codemp/cgame/cg_newDraw.c:7-12`
pub fn CG_GetSelectedPlayer(world: &mut CgWorld) -> c_int {
    if world.cvars.cg_currentSelectedPlayer.integer < 0
        || world.cvars.cg_currentSelectedPlayer.integer >= world.draw.numSortedTeamPlayers
    {
        world.cvars.cg_currentSelectedPlayer.integer = 0;
    }
    world.cvars.cg_currentSelectedPlayer.integer
}

/// Raven `CG_StatusHandle` — the team-task icon shader for a `teamtask_t`
/// value; unrecognized tasks (and `TEAMTASK_OFFENSE` itself) fall through to
/// the assault shader, matching Raven's `h` pre-init plus `default` arm.
///
/// Source: `oracle/codemp/cgame/cg_newDraw.c:14-43`
pub fn CG_StatusHandle(world: &CgWorld, task: c_int) -> qhandle_t {
    match task {
        t if t == teamtask_t::TEAMTASK_OFFENSE as c_int => world.cgs.media.assaultShader,
        t if t == teamtask_t::TEAMTASK_DEFENSE as c_int => world.cgs.media.defendShader,
        t if t == teamtask_t::TEAMTASK_PATROL as c_int => world.cgs.media.patrolShader,
        t if t == teamtask_t::TEAMTASK_FOLLOW as c_int => world.cgs.media.followShader,
        t if t == teamtask_t::TEAMTASK_CAMP as c_int => world.cgs.media.campShader,
        t if t == teamtask_t::TEAMTASK_RETRIEVE as c_int => world.cgs.media.retrieveShader,
        t if t == teamtask_t::TEAMTASK_ESCORT as c_int => world.cgs.media.escortShader,
        _ => world.cgs.media.assaultShader,
    }
}

/// Raven `CG_OtherTeamHasFlag` — has the opposing team taken our flag (CTF/CTY
/// only)?
///
/// Source: `oracle/codemp/cgame/cg_newDraw.c:93-105`
pub fn CG_OtherTeamHasFlag(world: &CgWorld) -> bool {
    if world.cgs.gametype == GT_CTF || world.cgs.gametype == GT_CTY {
        // §F19: Raven derefs `cg.snap` unguarded - with no snapshot the port
        // answers `qfalse`, the same as this fn's own fall-through.
        let Some(snap) = world.cg.snap_ref() else {
            return false;
        };
        let team = snap.ps.persistant[PERS_TEAM as usize];
        if team == TEAM_RED && world.cgs.redflag == FLAG_TAKEN {
            return true;
        } else if team == TEAM_BLUE && world.cgs.blueflag == FLAG_TAKEN {
            return true;
        } else {
            return false;
        }
    }
    false
}

/// Raven `CG_YourTeamHasFlag` — has our team taken the opposing team's flag
/// (CTF/CTY only)?
///
/// Source: `oracle/codemp/cgame/cg_newDraw.c:107-119`
pub fn CG_YourTeamHasFlag(world: &CgWorld) -> bool {
    if world.cgs.gametype == GT_CTF || world.cgs.gametype == GT_CTY {
        // §F19: same null-snap early-out as `CG_OtherTeamHasFlag`.
        let Some(snap) = world.cg.snap_ref() else {
            return false;
        };
        let team = snap.ps.persistant[PERS_TEAM as usize];
        if team == TEAM_RED && world.cgs.blueflag == FLAG_TAKEN {
            return true;
        } else if team == TEAM_BLUE && world.cgs.redflag == FLAG_TAKEN {
            return true;
        } else {
            return false;
        }
    }
    false
}

/// Raven `CG_GameTypeString` — the HUD/menu display name for the current
/// `cgs.gametype`; unrecognized values fall through to Raven's empty string.
///
/// Source: `oracle/codemp/cgame/cg_newDraw.c:242-259`
pub fn CG_GameTypeString(world: &CgWorld) -> &'static str {
    if world.cgs.gametype == GT_FFA {
        "Free For All"
    } else if world.cgs.gametype == GT_HOLOCRON {
        "Holocron FFA"
    } else if world.cgs.gametype == GT_JEDIMASTER {
        "Jedi Master"
    } else if world.cgs.gametype == GT_TEAM {
        "Team FFA"
    } else if world.cgs.gametype == GT_SIEGE {
        "Siege"
    } else if world.cgs.gametype == GT_CTF {
        "Capture the Flag"
    } else if world.cgs.gametype == GT_CTY {
        "Capture the Ysalamiri"
    } else {
        ""
    }
}

/// Raven `CG_OwnerDraw` — the menu-framework's stat owner-draw dispatch.
///
/// Raven's whole switch lives inside `#if 0` ("Ignore all this, at least for
/// now. May put some stat stuff back in menu files later."), so the retail
/// build never draws through here; the port carries the same dead signature.
///
/// Source: `oracle/codemp/cgame/cg_newDraw.c:562-745`
#[allow(clippy::too_many_arguments)]
pub fn CG_OwnerDraw(
    _x: f32,
    _y: f32,
    _w: f32,
    _h: f32,
    _text_x: f32,
    _text_y: f32,
    _ownerDraw: c_int,
    _ownerDrawFlags: c_int,
    _align: c_int,
    _special: f32,
    _scale: f32,
    _color: vec4_t,
    _shader: qhandle_t,
    _textStyle: c_int,
    _font: c_int,
) {
}

/// Raven `CG_MouseEvent` — routes a raw mouse delta either to the movement
/// key-catcher (gameplay states, scoreboard closed) or into the menu
/// framework's captured-item drag / hover-move.
///
/// DEFERRED: the captured-item branch — `cgs.capturedItem` is still the raw
/// `*mut c_void` C1 port of what Raven's `Display_CaptureItem` actually hands
/// back (`&Menus[i]`, a `menuDef_t*`, despite the "Item" name); the ported
/// `Display_MouseMove` wants `Option<MenuId>` (an arena index), and there is
/// no safe way to recover that index from the stored pointer. DEC-46 doesn't
/// cover `cgs_t.capturedItem`'s retype. The guard clause and cursor clamp
/// above it need none of this and are ported faithfully.
/// Source: `oracle/codemp/cgame/cg_newDraw.c:775-776`
pub fn CG_MouseEvent(
    ctx: &mut CgContext,
    menus: &mut MenuSystem,
    ds: &DisplayState,
    x: c_int,
    y: c_int,
) {
    let pm_type = ctx.world.cg.predictedPlayerState.pm_type;
    if (pm_type == pmtype_t::PM_NORMAL as c_int
        || pm_type == pmtype_t::PM_JETPACK as c_int
        || pm_type == pmtype_t::PM_FLOAT as c_int
        || pm_type == pmtype_t::PM_SPECTATOR as c_int)
        && ctx.world.cg.showScores == qfalse
    {
        trap::Key_SetCatcher(ctx.engine, 0);
        return;
    }

    ctx.world.cgs.cursorX += x;
    if ctx.world.cgs.cursorX < 0 {
        ctx.world.cgs.cursorX = 0;
    } else if ctx.world.cgs.cursorX > 640 {
        ctx.world.cgs.cursorX = 640;
    }

    ctx.world.cgs.cursorY += y;
    if ctx.world.cgs.cursorY < 0 {
        ctx.world.cgs.cursorY = 0;
    } else if ctx.world.cgs.cursorY > 480 {
        ctx.world.cgs.cursorY = 480;
    }

    let n = Display_CursorType(menus, ctx.world.cgs.cursorX, ctx.world.cgs.cursorY);
    ctx.world.cgs.activeCursor = 0;
    if n == CURSOR_ARROW {
        ctx.world.cgs.activeCursor = ctx.world.cgs.media.selectCursor;
    } else if n == CURSOR_SIZER {
        ctx.world.cgs.activeCursor = ctx.world.cgs.media.sizeCursor;
    }

    if !ctx.world.cgs.capturedItem.is_null() {
        // DEFERRED: see the fn doc above — `cgs.capturedItem` can't resolve
        // to `Option<MenuId>` yet.
        // Source: oracle/codemp/cgame/cg_newDraw.c:775-776
    } else {
        let (cx, cy) = (ctx.world.cgs.cursorX, ctx.world.cgs.cursorY);
        Display_MouseMove(menus, ds, ctx, None, cx, cy);
    }
}

/// Raven `CG_HideTeamMenu` — closes the team-join and "get more players"
/// popups.
///
/// Source: `oracle/codemp/cgame/cg_newDraw.c:789-792`
pub fn CG_HideTeamMenu(ctx: &mut CgContext, menus: &mut MenuSystem, ds: &DisplayState) {
    Menus_CloseByName(menus, ds, ctx, "teamMenu");
    Menus_CloseByName(menus, ds, ctx, "getMenu");
}

/// Raven `CG_ShowTeamMenu` — opens the team-join popup.
///
/// Source: `oracle/codemp/cgame/cg_newDraw.c:800-802`
pub fn CG_ShowTeamMenu(ctx: &mut CgContext, menus: &mut MenuSystem, ds: &DisplayState) {
    Menus_OpenByName(menus, ds, ctx, "teamMenu");
}

/// Raven `CG_ClientNumFromName` — looks up a client number by exact
/// (case-insensitive) name match against every client with valid info; `-1`
/// on no match, matching Raven's fall-through.
///
/// Source: `oracle/codemp/cgame/cg_newDraw.c:860-868`
pub fn CG_ClientNumFromName(world: &CgWorld, p: &str) -> c_int {
    let query = string_to_latin1(p);
    for i in 0..world.cgs.maxclients as usize {
        let ci = &world.cgs.clientinfo[i];
        if ci.infoValid != qfalse {
            // Raven's fixed `char name[MAX_QPATH]` compared byte-for-byte in
            // Latin-1 space, not the decoded `&str` — no `from_utf8_lossy`.
            let name: Vec<u8> = ci.name.iter().map(|&c| c as u8).collect();
            if Q_stricmpBytes(&name, &query) == 0 {
                return i as c_int;
            }
        }
    }
    -1
}

/// Raven `CG_ShowResponseHead` — opens the voice-chat response menu, nudges
/// the console text over to make room for it, and starts its 2.5-second
/// display timer.
///
/// Source: `oracle/codemp/cgame/cg_newDraw.c:870-874`
pub fn CG_ShowResponseHead(ctx: &mut CgContext, menus: &mut MenuSystem, ds: &DisplayState) {
    Menus_OpenByName(menus, ds, ctx, "voiceMenu");
    trap::Cvar_Set(ctx.engine, "cl_conXOffset", "72");
    ctx.world.cg.voiceTime = ctx.world.cg.time;
}

/// Raven `CG_RunMenuScript` — cgame's menu-item script hook does nothing;
/// item scripts run entirely inside `ui_shared.c`'s own dispatch.
///
/// Source: `oracle/codemp/cgame/cg_newDraw.c:876-877`
pub fn CG_RunMenuScript(_args: &mut &str) {}

/// Raven `CG_DeferMenuScript` — cgame never defers a menu script; always
/// `qfalse`.
///
/// Source: `oracle/codemp/cgame/cg_newDraw.c:879-882`
pub fn CG_DeferMenuScript(_args: &mut &str) -> bool {
    false
}

/// Raven `CG_GetTeamColor` — the translucent HUD tint for the local player's
/// team (red/blue/free-for-all).
///
/// Raven filled a caller-owned `vec4_t *color`; every arm writes all four
/// components, so the port returns the color instead.
///
/// Source: `oracle/codemp/cgame/cg_newDraw.c:884-898`
pub fn CG_GetTeamColor(world: &CgWorld) -> vec4_t {
    // §F19: Raven derefs `cg.snap` unguarded - with no snapshot we take the
    // non-team tint, Raven's own else arm.
    let team = world
        .cg
        .snap_ref()
        .map(|snap| snap.ps.persistant[PERS_TEAM as usize]);

    let mut color: vec4_t = [0.0; 4];
    if team == Some(TEAM_RED) {
        color[0] = 1.0;
        color[3] = 0.25;
        color[1] = 0.0;
        color[2] = 0.0;
    } else if team == Some(TEAM_BLUE) {
        color[0] = 0.0;
        color[1] = 0.0;
        color[2] = 1.0;
        color[3] = 0.25;
    } else {
        color[0] = 0.0;
        color[2] = 0.0;
        color[1] = 0.17;
        color[3] = 0.25;
    }
    color
}

/// Raven `CG_GetValue` — the menu-framework owner-draw numeric-value
/// dispatch backing player/team stat, ammo, score, and force meters; `-1.0`
/// on an unrecognized `ownerDraw` (and on the ammo arm's "no weapon" branch,
/// which Raven leaves without a `return`).
///
/// Source: `oracle/codemp/cgame/cg_newDraw.c:46-91`
pub fn CG_GetValue(world: &mut CgWorld, ownerDraw: c_int) -> f32 {
    // §F19: Raven derefs `cg.snap->ps.clientNum` unconditionally before the
    // switch ever runs, so every arm - even the ones that never touch `ps` -
    // crashes alike with no snapshot. The port takes this fn's own default
    // return uniformly to match that crash-everywhere shape.
    let Some(snap) = world.cg.snap_ref() else {
        return -1.0;
    };
    let ps = snap.ps;
    let clientNum = ps.clientNum as usize;

    match ownerDraw {
        v if v == CG_SELECTEDPLAYER_ARMOR => {
            let sel = CG_GetSelectedPlayer(world) as usize;
            let idx = world.draw.sortedTeamPlayers[sel] as usize;
            world.cgs.clientinfo[idx].armor as f32
        }
        v if v == CG_SELECTEDPLAYER_HEALTH => {
            let sel = CG_GetSelectedPlayer(world) as usize;
            let idx = world.draw.sortedTeamPlayers[sel] as usize;
            world.cgs.clientinfo[idx].health as f32
        }
        v if v == CG_PLAYER_ARMOR_VALUE => ps.stats[statIndex_t::STAT_ARMOR as usize] as f32,
        v if v == CG_PLAYER_AMMO_VALUE => {
            let weapon = world.entities[clientNum].currentState.weapon;
            if weapon != 0 {
                let ammoIndex = weaponData[weapon as usize].ammoIndex as usize;
                ps.ammo[ammoIndex] as f32
            } else {
                -1.0
            }
        }
        v if v == CG_PLAYER_SCORE => ps.persistant[PERS_SCORE as usize] as f32,
        v if v == CG_PLAYER_HEALTH => ps.stats[statIndex_t::STAT_HEALTH as usize] as f32,
        v if v == CG_RED_SCORE => world.cgs.scores1 as f32,
        v if v == CG_BLUE_SCORE => world.cgs.scores2 as f32,
        v if v == CG_PLAYER_FORCE_VALUE => ps.fd.forcePower as f32,
        _ => -1.0,
    }
}

/// Raven `CG_OwnerDrawVisible` — the menu-framework's show/hide test for a
/// `CG_SHOW_*` flag combination.
///
/// Source: `oracle/codemp/cgame/cg_newDraw.c:123-201`
pub fn CG_OwnerDrawVisible(world: &CgWorld, flags: c_int) -> bool {
    if flags & CG_SHOW_TEAMINFO != 0 {
        return world.cvars.cg_currentSelectedPlayer.integer == world.draw.numSortedTeamPlayers;
    }

    if flags & CG_SHOW_NOTEAMINFO != 0 {
        return world.cvars.cg_currentSelectedPlayer.integer != world.draw.numSortedTeamPlayers;
    }

    if flags & CG_SHOW_OTHERTEAMHASFLAG != 0 {
        return CG_OtherTeamHasFlag(world);
    }

    if flags & CG_SHOW_YOURTEAMHASENEMYFLAG != 0 {
        return CG_YourTeamHasFlag(world);
    }

    if flags & (CG_SHOW_BLUE_TEAM_HAS_REDFLAG | CG_SHOW_RED_TEAM_HAS_BLUEFLAG) != 0 {
        if flags & CG_SHOW_BLUE_TEAM_HAS_REDFLAG != 0
            && (world.cgs.redflag == FLAG_TAKEN || world.cgs.flagStatus == FLAG_TAKEN_RED)
        {
            return true;
        } else if flags & CG_SHOW_RED_TEAM_HAS_BLUEFLAG != 0
            && (world.cgs.blueflag == FLAG_TAKEN || world.cgs.flagStatus == FLAG_TAKEN_BLUE)
        {
            return true;
        }
        return false;
    }

    if flags & CG_SHOW_ANYTEAMGAME != 0 && world.cgs.gametype >= GT_TEAM {
        return true;
    }

    if flags & CG_SHOW_ANYNONTEAMGAME != 0 && world.cgs.gametype < GT_TEAM {
        return true;
    }

    if flags & CG_SHOW_CTF != 0 && (world.cgs.gametype == GT_CTF || world.cgs.gametype == GT_CTY) {
        return true;
    }

    // §F19: Raven derefs `cg.snap` unguarded on these two health arms - with
    // no snapshot the port falls through, same as an out-of-range health
    // reading would.
    if flags & CG_SHOW_HEALTHCRITICAL != 0 {
        if let Some(snap) = world.cg.snap_ref() {
            if snap.ps.stats[statIndex_t::STAT_HEALTH as usize] < 25 {
                return true;
            }
        }
    }

    if flags & CG_SHOW_HEALTHOK != 0 {
        if let Some(snap) = world.cg.snap_ref() {
            if snap.ps.stats[statIndex_t::STAT_HEALTH as usize] >= 25 {
                return true;
            }
        }
    }

    if flags & CG_SHOW_SINGLEPLAYER != 0 && world.cgs.gametype == GT_SINGLE_PLAYER {
        return true;
    }

    if flags & CG_SHOW_TOURNAMENT != 0
        && (world.cgs.gametype == GT_DUEL || world.cgs.gametype == GT_POWERDUEL)
    {
        return true;
    }

    // CG_SHOW_DURINGINCOMINGVOICE: Raven's arm has an empty body - carried as
    // a no-op, matching the C.
    if flags & CG_SHOW_DURINGINCOMINGVOICE != 0 {}

    if flags & CG_SHOW_IF_PLAYER_HAS_FLAG != 0 {
        if let Some(snap) = world.cg.snap_ref() {
            if snap.ps.powerups[PW_REDFLAG as usize] != 0
                || snap.ps.powerups[PW_BLUEFLAG as usize] != 0
                || snap.ps.powerups[PW_NEUTRALFLAG as usize] != 0
            {
                return true;
            }
        }
    }

    false
}

/// Raven `CG_GetKillerText` — the "killed by \<name\>" banner text; empty
/// once no kill has happened yet.
///
/// Raven's function-local `static const char *s` is stateful across calls:
/// when `cg.killerName` is empty (it IS cleared, `cg_draw.c:6762` /
/// `cg_scoreboard.c:375`) Raven returns the PREVIOUS call's pointer — which by
/// then aims at `va()`'s rotating scratch buffer, i.e. stale memory. §F19: the
/// port answers empty on that path instead of replaying garbage.
///
/// Source: `oracle/codemp/cgame/cg_newDraw.c:204-210`
pub fn CG_GetKillerText(ctx: &mut CgContext) -> String {
    if ctx.world.cg.killerName[0] != 0 {
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
        return format!("{killedBy} {killerName}");
    }
    String::new()
}

/// Raven `CG_GetGameStatusText` — the scoreboard-header banner: place/score
/// while free-for-all, blank in power duel, or the team lead/tie line.
///
/// Raven cached this behind a function-local `static const char *s`; the
/// port returns an owned `String` each call, same rationale as
/// `CG_GetKillerText`.
///
/// Source: `oracle/codemp/cgame/cg_newDraw.c:213-240`
pub fn CG_GetGameStatusText(ctx: &mut CgContext) -> String {
    if ctx.world.cgs.gametype == GT_POWERDUEL {
        return String::new();
    }

    if ctx.world.cgs.gametype < GT_TEAM {
        // §F19: Raven derefs `cg.snap` unguarded here too - with no snapshot
        // the port answers empty, matching the fn's own untouched-`s`
        // default.
        let Some(snap) = ctx.world.cg.snap_ref() else {
            return String::new();
        };
        if snap.ps.persistant[PERS_TEAM as usize] == TEAM_SPECTATOR {
            return String::new();
        }
        let rank = snap.ps.persistant[PERS_RANK as usize] + 1;
        let score = snap.ps.persistant[PERS_SCORE as usize];

        let sPlaceWith = trap::SP_GetStringTextString(ctx.engine, "MP_INGAME_PLACE_WITH", 256);
        let place = CG_PlaceString(ctx, rank);
        return format!("{place} {sPlaceWith} {score}");
    }

    let redScore = ctx.world.cg.teamScores[0];
    let blueScore = ctx.world.cg.teamScores[1];
    if redScore == blueScore {
        let tiedAt = CG_GetStringEdString(ctx, "MP_INGAME", "TIEDAT");
        format!("{tiedAt} {redScore}")
    } else if redScore >= blueScore {
        let redLeads = CG_GetStringEdString(ctx, "MP_INGAME", "RED_LEADS");
        format!("{redLeads}, {redScore} / {blueScore}")
    } else {
        let blueLeads = CG_GetStringEdString(ctx, "MP_INGAME", "BLUE_LEADS");
        format!("{blueLeads}, {blueScore} / {redScore}")
    }
}

/// Raven `CG_EventHandling` — switches the active in-menu event mode; the
/// team-menu arm's `CG_ShowTeamMenu` call is commented out in Raven and the
/// scoreboard arm has no body, so both land as no-ops.
///
/// Source: `oracle/codemp/cgame/cg_newDraw.c:816-825`
pub fn CG_EventHandling(
    ctx: &mut CgContext,
    menus: &mut MenuSystem,
    ds: &DisplayState,
    eventType: c_int,
) {
    ctx.world.cgs.eventHandling = eventType;
    if eventType == CGAME_EVENT_NONE {
        CG_HideTeamMenu(ctx, menus, ds);
    } else if eventType == CGAME_EVENT_TEAMMENU {
        // CG_ShowTeamMenu(); - Raven left this call commented out.
    } else if eventType == CGAME_EVENT_SCOREBOARD {
    }
}
