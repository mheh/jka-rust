//! Port of `oracle/codemp/cgame/cg_newDraw.c` — the menu-framework owner-draw and feeder surface cgame exposes. Functions land via the C5
//! transcription waves.

#![allow(non_snake_case)]

use core::ffi::c_int;
use core::ptr::null_mut;

use mp_bg::bg_misc::BG_FindItemForPowerup;
use mp_bg::public::configstring::CS_LOCATIONS;
use mp_bg::public::gametype::{
    GT_CTF, GT_CTY, GT_DUEL, GT_FFA, GT_HOLOCRON, GT_JEDIMASTER, GT_POWERDUEL, GT_SIEGE,
    GT_SINGLE_PLAYER, GT_TEAM,
};
use mp_bg::public::pers_enum::persEnum_t::{PERS_RANK, PERS_SCORE, PERS_TEAM};
use mp_bg::public::pmtype::pmtype_t;
use mp_bg::public::powerup::{PW_BLUEFLAG, PW_NEUTRALFLAG, PW_NUM_POWERUPS, PW_REDFLAG};
use mp_bg::public::stat_index::statIndex_t;
use mp_bg::public::team::{TEAM_BLUE, TEAM_RED, TEAM_SPECTATOR};
use mp_bg::public::teamtask::teamtask_t;
use mp_bg::weapons::weapon_data::weaponData;

use mp_qshared::shared::{qfalse, qhandle_t, vec4_t, FLAG_TAKEN, FLAG_TAKEN_BLUE, FLAG_TAKEN_RED};

use mp_uishared::shared::display_state::DisplayState;
use mp_uishared::shared::menu_system::MenuSystem;
use mp_uishared::shared::menudef::{
    CG_ACCURACY, CG_ASSISTS, CG_BLUE_SCORE, CG_CAPTURES, CG_DEFEND, CG_EXCELLENT, CG_GAUNTLET,
    CG_IMPRESSIVE, CG_PERFECT, CG_PLAYER_AMMO_VALUE, CG_PLAYER_ARMOR_VALUE, CG_PLAYER_FORCE_VALUE,
    CG_PLAYER_HEALTH, CG_PLAYER_SCORE, CG_RED_SCORE, CG_SELECTEDPLAYER_ARMOR,
    CG_SELECTEDPLAYER_HEALTH, CG_SHOW_ANYNONTEAMGAME, CG_SHOW_ANYTEAMGAME,
    CG_SHOW_BLUE_TEAM_HAS_REDFLAG, CG_SHOW_CTF, CG_SHOW_DURINGINCOMINGVOICE,
    CG_SHOW_HEALTHCRITICAL, CG_SHOW_HEALTHOK, CG_SHOW_IF_PLAYER_HAS_FLAG, CG_SHOW_NOTEAMINFO,
    CG_SHOW_OTHERTEAMHASFLAG, CG_SHOW_RED_TEAM_HAS_BLUEFLAG, CG_SHOW_SINGLEPLAYER,
    CG_SHOW_TEAMINFO, CG_SHOW_TOURNAMENT, CG_SHOW_YOURTEAMHASENEMYFLAG, ITEM_TEXTSTYLE_NORMAL,
};
use mp_uishared::shared::rect_def_t::RectDef;
use mp_uishared::ui_shared::{
    Display_CursorType, Display_HandleKey, Display_MouseMove, Menus_CloseByName, Menus_OpenByName,
    CURSOR_ARROW, CURSOR_SIZER,
};

use native_string::{latin1_to_string, string_to_latin1, Q_stricmpBytes};

use crate::cg_draw::{CG_Text_Paint, CG_Text_Width, MenuFontToHandle};
use crate::cg_drawtools::{CG_DrawPic, CG_GetColorForHealth};
use crate::cg_event::CG_PlaceString;
use crate::cg_main::{CG_ConfigString, CG_GetLocationString, CG_GetStringEdString};
use crate::trap;
use crate::world::cg_context::CgContext;
use crate::world::cg_world::CgWorld;

/// Raven `#define PIC_WIDTH 12`.
///
/// Source: `oracle/codemp/cgame/cg_newDraw.c:324`
pub const PIC_WIDTH: c_int = 12;

// PORT-NOTE: file-local copy per cg_draw.rs's own (that one is private).
/// Source: `oracle/codemp/game/q_shared.h:1989`
const MAX_LOCATIONS: c_int = 64;

// PORT-NOTE: `q_shared.h`'s font enum is anonymous, so per the
// anonymous-enum convention this is a file-local `const`, mirroring
// `cg_draw.rs`'s own copy.
/// Source: `oracle/codemp/game/q_shared.h:3176-3182`
const FONT_MEDIUM: c_int = 2;

// PORT-NOTE: `keycodes.h`'s `A_*` enum has no `mp_qshared` home (see
// `mp_uishared::ui_shared`'s own file-local copies), so this key code gets a
// local numeric twin - the same ordinal as `fakeAscii_t::A_MOUSE2`.
/// Source: `oracle/codemp/ui/keycodes.h:156`
const A_MOUSE2: c_int = 142;

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

        let sPlaceWith = trap::SP_GetStringTextString(ctx.engine, "MP_INGAME_PLACE_WITH", 256)
            .unwrap_or_else(|| "??MP_INGAME_PLACE_WITH".to_string());
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

/// Raven `CG_Text_Paint_Limit` — paints `text` inside the `[x, *maxX]` pixel
/// budget, truncating to whatever prefix fits when the whole string is too
/// wide; feeds `*maxX` back to the caller either way (the next paint
/// position when it all fit, `0` once truncated).
///
/// Raven's truncation scratch buffer always drops the very last "letter" it
/// decoded, even when that letter would still have fit: `psOutLastGood` is
/// reset to the buffer position from BEFORE each iteration's append, and the
/// final NUL lands there once the loop exits - so whatever the exiting
/// iteration just appended never survives into the printed prefix. Kept
/// verbatim (§A2 - no speculative off-by-one fix).
///
/// Source: `oracle/codemp/cgame/cg_newDraw.c:267-320`
#[allow(clippy::too_many_arguments)]
pub fn CG_Text_Paint_Limit(
    ctx: &CgContext,
    cgDC: &DisplayState,
    maxX: &mut f32,
    x: f32,
    y: f32,
    scale: f32,
    color: vec4_t,
    text: &str,
    adjust: f32,
    limit: c_int,
    iMenuFont: c_int,
) {
    let iFontIndex = MenuFontToHandle(cgDC, iMenuFont);

    let iPixelLen = trap::R_Font_StrLenPixels(ctx.engine, text, iFontIndex, scale);
    if x + iPixelLen as f32 > *maxX {
        // whole text won't fit, so print just the amount that does - walk it
        // one engine-decoded "letter" (possibly a 2-byte code) at a time.
        let queryBytes = string_to_latin1(text);
        let mut sTemp: Vec<u8> = Vec::new();
        let mut lastGoodLen: usize = 0;
        let mut pos: usize = 0;

        while pos < queryBytes.len() {
            let soFar = latin1_to_string(&sTemp);
            let widthSoFar = trap::R_Font_StrLenPixels(ctx.engine, &soFar, iFontIndex, scale);
            if x + widthSoFar as f32 > *maxX {
                break;
            }
            // sanity: leave room for at least one more byte, mirroring
            // Raven's `char sTemp[4096]` scratch bound.
            if sTemp.len() >= 4095 {
                break;
            }

            lastGoodLen = sTemp.len();

            let (uiLetter, advanceCount, _bIsTrailingPunctuation) =
                trap::AnyLanguage_ReadCharFromString(ctx.engine, &queryBytes[pos..]);
            pos += advanceCount as usize;

            if uiLetter > 255 {
                sTemp.push((uiLetter >> 8) as u8);
                sTemp.push((uiLetter & 0xFF) as u8);
            } else {
                sTemp.push((uiLetter & 0xFF) as u8);
            }
        }
        sTemp.truncate(lastGoodLen);

        *maxX = 0.0; // feedback
        let sTempStr = latin1_to_string(&sTemp);
        CG_Text_Paint(
            ctx,
            cgDC,
            x,
            y,
            scale,
            color,
            &sTempStr,
            adjust,
            limit,
            ITEM_TEXTSTYLE_NORMAL,
            iMenuFont,
        );
    } else {
        // whole text fits fine, so print it all
        *maxX = x + iPixelLen as f32; // feedback the next position, as the caller expects
        CG_Text_Paint(
            ctx,
            cgDC,
            x,
            y,
            scale,
            color,
            text,
            adjust,
            limit,
            ITEM_TEXTSTYLE_NORMAL,
            iMenuFont,
        );
    }
}

/// Raven `CG_DrawMedal` — the scoreboard "medal" owner-draw: an accuracy/
/// award icon plus its numeric caption for `cg.scores[cg.selectedScore]`.
///
/// Raven's `vec4_t color` parameter is a C array, so it decays to a pointer
/// and the in-body writes (`color[3] = ...`) are really an in/out parameter;
/// this fn has no live caller (its one call site, `CG_OwnerDraw`'s switch, is
/// `#if 0`'d out in Raven and lands as the empty stub above), so the port
/// keeps this file's established by-value `vec4_t` shape rather than
/// threading a `&mut`.
///
/// Source: `oracle/codemp/cgame/cg_newDraw.c:496-558`
pub fn CG_DrawMedal(
    ctx: &mut CgContext,
    cgDC: &DisplayState,
    ownerDraw: c_int,
    rect: &RectDef,
    scale: f32,
    mut color: vec4_t,
    shader: qhandle_t,
) {
    let mut value: f32 = 0.0;
    let mut text: Option<String> = None;
    color[3] = 0.25;

    let score = &ctx.world.cg.scores[ctx.world.cg.selectedScore as usize];
    match ownerDraw {
        v if v == CG_ACCURACY => value = score.accuracy as f32,
        v if v == CG_ASSISTS => value = score.assistCount as f32,
        v if v == CG_DEFEND => value = score.defendCount as f32,
        v if v == CG_EXCELLENT => value = score.excellentCount as f32,
        v if v == CG_IMPRESSIVE => value = score.impressiveCount as f32,
        v if v == CG_PERFECT => value = score.perfect as f32,
        v if v == CG_GAUNTLET => value = score.guantletCount as f32,
        v if v == CG_CAPTURES => value = score.captures as f32,
        _ => {}
    }

    if value > 0.0 {
        if ownerDraw != CG_PERFECT {
            if ownerDraw == CG_ACCURACY {
                text = Some(format!("{}%", value as i32));
                if value > 50.0 {
                    color[3] = 1.0;
                }
            } else {
                text = Some(format!("{}", value as i32));
                color[3] = 1.0;
            }
        } else {
            if value != 0.0 {
                color[3] = 1.0;
            }
            text = Some("Wow".to_string());
        }
    }

    trap::R_SetColor(ctx.engine, Some(&color));
    CG_DrawPic(ctx, rect.x, rect.y, rect.w, rect.h, shader);

    if let Some(text) = text {
        color[3] = 1.0;
        let textWidth = CG_Text_Width(ctx, cgDC, &text, scale, 0);
        CG_Text_Paint(
            ctx,
            cgDC,
            rect.x + (rect.w - textWidth as f32) / 2.0,
            rect.y + rect.h + 10.0,
            scale,
            color,
            &text,
            0.0,
            0,
            0,
            FONT_MEDIUM,
        );
    }
    trap::R_SetColor(ctx.engine, None);
}

/// Raven `CG_KeyEvent` — routes a key press either to the movement
/// key-catcher (gameplay states, scoreboard closed) or into the menu
/// framework's key handling / captured-item release.
///
/// DEFERRED: the captured-item ACQUIRE branch - `cgs.capturedItem` is still
/// the raw `*mut c_void` C1 port noted on `CG_MouseEvent` above, and
/// `Display_CaptureItem` now returns `Option<MenuId>`, an arena index with no
/// address to store there. Releasing it (nulling on a non-null capture) needs
/// none of that and is ported faithfully.
/// Source: `oracle/codemp/cgame/cg_newDraw.c:851-856`
pub fn CG_KeyEvent(
    ctx: &mut CgContext,
    menus: &mut MenuSystem,
    ds: &DisplayState,
    key: c_int,
    down: bool,
) {
    if !down {
        return;
    }

    // Raven checks `PM_NORMAL` twice (a duplicate arm) - kept verbatim.
    let pm_type = ctx.world.cg.predictedPlayerState.pm_type;
    if pm_type == pmtype_t::PM_NORMAL as c_int
        || pm_type == pmtype_t::PM_JETPACK as c_int
        || pm_type == pmtype_t::PM_NORMAL as c_int
        || (pm_type == pmtype_t::PM_SPECTATOR as c_int && ctx.world.cg.showScores == qfalse)
    {
        CG_EventHandling(ctx, menus, ds, CGAME_EVENT_NONE);
        trap::Key_SetCatcher(ctx.engine, 0);
        return;
    }

    let (cx, cy) = (ctx.world.cgs.cursorX, ctx.world.cgs.cursorY);
    Display_HandleKey(menus, ds, ctx, key, down, cx, cy);

    if !ctx.world.cgs.capturedItem.is_null() {
        ctx.world.cgs.capturedItem = null_mut();
    } else if key == A_MOUSE2 && down {
        // DEFERRED: see the fn doc above.
        // Source: oracle/codemp/cgame/cg_newDraw.c:854-855
    }
}

/// Raven `CG_DrawNewTeamInfo` — the scoreboard team-overlay row: each
/// teammate's powerup icons, health/armor tint, current task icon, name, and
/// location, painted top to bottom until the rect runs out of room.
///
/// PORT-NOTE: Raven computes `pwidth` (max player-name width) and `lwidth`
/// (max location-name width) up front but never reads either afterward - the
/// draw loop below hardcodes its column split instead. The dead stores go,
/// but the measuring loops still run: each `CG_Text_Width` is a live
/// `R_Font_StrLenPixels` engine query Raven issues every frame.
/// Source: `oracle/codemp/cgame/cg_newDraw.c:338-359`
#[allow(clippy::too_many_arguments)]
pub fn CG_DrawNewTeamInfo(
    ctx: &mut CgContext,
    cgDC: &DisplayState,
    rect: &RectDef,
    _text_x: f32,
    text_y: f32,
    scale: f32,
    color: vec4_t,
    _shader: qhandle_t,
) {
    let count = if ctx.world.draw.numSortedTeamPlayers > 8 {
        8
    } else {
        ctx.world.draw.numSortedTeamPlayers
    };

    // §F19: Raven derefs `cg.snap` unguarded for every row's team check; the
    // port resolves it once up front and does nothing with no snapshot,
    // rather than crashing partway through the first row.
    let Some(snap) = ctx.world.cg.snap_ref() else {
        return;
    };
    let myTeam = snap.ps.persistant[PERS_TEAM as usize];

    // max player name width (dead store dropped, live engine queries kept -
    // see the PORT-NOTE above)
    for i in 0..count as usize {
        let idx = ctx.world.draw.sortedTeamPlayers[i] as usize;
        let (infoValid, ciTeam, nameBuf) = {
            let ci = &ctx.world.cgs.clientinfo[idx];
            (ci.infoValid, ci.team, ci.name)
        };
        if infoValid != qfalse && ciTeam == myTeam {
            let nameBytes: Vec<u8> = nameBuf
                .iter()
                .take_while(|&&c| c != 0)
                .map(|&c| c as u8)
                .collect();
            let name = latin1_to_string(&nameBytes);
            CG_Text_Width(ctx, cgDC, &name, scale, 0);
        }
    }

    // max location name width
    for i in 1..MAX_LOCATIONS {
        let cs = CG_ConfigString(ctx, CS_LOCATIONS + i);
        let p = CG_GetLocationString(ctx, &cs);
        if !p.is_empty() {
            CG_Text_Width(ctx, cgDC, &p, scale, 0);
        }
    }

    let mut y = rect.y;

    for i in 0..count as usize {
        let idx = ctx.world.draw.sortedTeamPlayers[i] as usize;
        let (infoValid, ciTeam, powerups, health, armor, teamTask, location, nameBuf) = {
            let ci = &ctx.world.cgs.clientinfo[idx];
            (
                ci.infoValid,
                ci.team,
                ci.powerups,
                ci.health,
                ci.armor,
                ci.teamTask,
                ci.location,
                ci.name,
            )
        };

        if infoValid == qfalse || ciTeam != myTeam {
            continue;
        }

        let mut xx = (rect.x + 1.0) as i32;
        for j in 0..=PW_NUM_POWERUPS {
            if powerups & (1 << j) != 0 {
                if let Some(item) = BG_FindItemForPowerup(j) {
                    // §F19: Raven hands `item->icon` straight to the trap; the
                    // ported icon is optional, and an item without one
                    // registers the empty name rather than dereferencing NULL.
                    let icon = item.item().icon.unwrap_or("");
                    let shader = trap::R_RegisterShader(ctx.engine, icon);
                    CG_DrawPic(
                        ctx,
                        xx as f32,
                        y,
                        PIC_WIDTH as f32,
                        PIC_WIDTH as f32,
                        shader,
                    );
                    xx += PIC_WIDTH;
                }
            }
        }

        // FIXME: max of 3 powerups shown properly
        xx = (rect.x + (PIC_WIDTH * 3) as f32 + 2.0) as i32;

        let hcolor = CG_GetColorForHealth(health, armor);
        trap::R_SetColor(ctx.engine, Some(&hcolor));
        let heartShader = ctx.world.cgs.media.heartShader;
        CG_DrawPic(
            ctx,
            xx as f32,
            y + 1.0,
            (PIC_WIDTH - 2) as f32,
            (PIC_WIDTH - 2) as f32,
            heartShader,
        );

        // Raven's `//Com_sprintf(st, ...)` / `//CG_Text_Paint(...)` health
        // caption is commented out in the C - dead, carried as no-op.

        // draw weapon icon
        xx += PIC_WIDTH + 1;

        // Raven's `#if 0` weapon-icon block ("weapon used is not that
        // useful, use the space for task") never compiled into retail -
        // dropped, not ported.

        trap::R_SetColor(ctx.engine, None);
        let h = CG_StatusHandle(ctx.world, teamTask);

        if h != 0 {
            CG_DrawPic(ctx, xx as f32, y, PIC_WIDTH as f32, PIC_WIDTH as f32, h);
        }

        xx += PIC_WIDTH + 1;

        let leftOver = rect.w - xx as f32;
        let mut maxx = xx as f32 + leftOver / 3.0;

        let nameBytes: Vec<u8> = nameBuf
            .iter()
            .take_while(|&&c| c != 0)
            .map(|&c| c as u8)
            .collect();
        let name = latin1_to_string(&nameBytes);
        CG_Text_Paint_Limit(
            ctx,
            cgDC,
            &mut maxx,
            xx as f32,
            y + text_y,
            scale,
            color,
            &name,
            0.0,
            0,
            FONT_MEDIUM,
        );

        let csStr = CG_ConfigString(ctx, CS_LOCATIONS + location);
        let mut p = CG_GetLocationString(ctx, &csStr);
        if p.is_empty() {
            p = "unknown".to_string();
        }

        xx = (xx as f32 + (leftOver / 3.0 + 2.0)) as i32;
        maxx = rect.w - 4.0;

        CG_Text_Paint_Limit(
            ctx,
            cgDC,
            &mut maxx,
            xx as f32,
            y + text_y,
            scale,
            color,
            &p,
            0.0,
            0,
            FONT_MEDIUM,
        );
        y += text_y + 2.0;
        if y + text_y + 2.0 > rect.y + rect.h {
            break;
        }
    }
}

/// Raven `CG_DrawTeamSpectators` — the scrolling spectator-name ticker along
/// the scoreboard's bottom edge; two overlapping paints (`spectatorPaintX`/
/// `spectatorPaintX2`) let the tail of one lap keep sliding while the next
/// lap's head enters from the right.
///
/// Source: `oracle/codemp/cgame/cg_newDraw.c:437-492`
pub fn CG_DrawTeamSpectators(
    ctx: &mut CgContext,
    cgDC: &DisplayState,
    rect: &RectDef,
    scale: f32,
    color: vec4_t,
    _shader: qhandle_t,
) {
    if ctx.world.cg.spectatorLen == 0 {
        return;
    }

    if ctx.world.cg.spectatorWidth == -1.0 {
        ctx.world.cg.spectatorWidth = 0.0;
        ctx.world.cg.spectatorPaintX = (rect.x + 1.0) as i32;
        ctx.world.cg.spectatorPaintX2 = -1;
    }

    if ctx.world.cg.spectatorOffset > ctx.world.cg.spectatorLen {
        ctx.world.cg.spectatorOffset = 0;
        ctx.world.cg.spectatorPaintX = (rect.x + 1.0) as i32;
        ctx.world.cg.spectatorPaintX2 = -1;
    }

    if ctx.world.cg.time > ctx.world.cg.spectatorTime {
        ctx.world.cg.spectatorTime = ctx.world.cg.time + 10;
        if (ctx.world.cg.spectatorPaintX as f32) <= rect.x + 2.0 {
            if ctx.world.cg.spectatorOffset < ctx.world.cg.spectatorLen {
                let offset = ctx.world.cg.spectatorOffset as usize;
                let tailBytes: Vec<u8> = ctx.world.cg.spectatorList[offset..]
                    .iter()
                    .take_while(|&&c| c != 0)
                    .map(|&c| c as u8)
                    .collect();
                let tail = latin1_to_string(&tailBytes);
                let width = CG_Text_Width(ctx, cgDC, &tail, scale, 1);
                ctx.world.cg.spectatorPaintX += width - 1;
                ctx.world.cg.spectatorOffset += 1;
            } else {
                ctx.world.cg.spectatorOffset = 0;
                if ctx.world.cg.spectatorPaintX2 >= 0 {
                    ctx.world.cg.spectatorPaintX = ctx.world.cg.spectatorPaintX2;
                } else {
                    ctx.world.cg.spectatorPaintX = (rect.x + rect.w - 2.0) as i32;
                }
                ctx.world.cg.spectatorPaintX2 = -1;
            }
        } else {
            ctx.world.cg.spectatorPaintX -= 1;
            if ctx.world.cg.spectatorPaintX2 >= 0 {
                ctx.world.cg.spectatorPaintX2 -= 1;
            }
        }
    }

    let mut maxX = rect.x + rect.w - 2.0;
    let offset = ctx.world.cg.spectatorOffset as usize;
    let tailBytes: Vec<u8> = ctx.world.cg.spectatorList[offset..]
        .iter()
        .take_while(|&&c| c != 0)
        .map(|&c| c as u8)
        .collect();
    let tail = latin1_to_string(&tailBytes);
    let spectatorPaintX = ctx.world.cg.spectatorPaintX as f32;
    CG_Text_Paint_Limit(
        ctx,
        cgDC,
        &mut maxX,
        spectatorPaintX,
        rect.y + rect.h - 3.0,
        scale,
        color,
        &tail,
        0.0,
        0,
        FONT_MEDIUM,
    );

    if ctx.world.cg.spectatorPaintX2 >= 0 {
        let mut maxX2 = rect.x + rect.w - 2.0;
        let fullBytes: Vec<u8> = ctx
            .world
            .cg
            .spectatorList
            .iter()
            .take_while(|&&c| c != 0)
            .map(|&c| c as u8)
            .collect();
        let full = latin1_to_string(&fullBytes);
        let spectatorPaintX2 = ctx.world.cg.spectatorPaintX2 as f32;
        let limit = ctx.world.cg.spectatorOffset;
        CG_Text_Paint_Limit(
            ctx,
            cgDC,
            &mut maxX2,
            spectatorPaintX2,
            rect.y + rect.h - 3.0,
            scale,
            color,
            &full,
            0.0,
            limit,
            FONT_MEDIUM,
        );
    }

    // if we have an offset ( we are skipping the first part of the string )
    // and we fit the string
    if ctx.world.cg.spectatorOffset != 0 && maxX > 0.0 {
        if ctx.world.cg.spectatorPaintX2 == -1 {
            ctx.world.cg.spectatorPaintX2 = (rect.x + rect.w - 2.0) as i32;
        }
    } else {
        ctx.world.cg.spectatorPaintX2 = -1;
    }
}
