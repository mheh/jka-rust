//! Port of `oracle/codemp/cgame/cg_newDraw.c` — the menu-framework owner-draw and feeder surface cgame exposes. Functions land via the C5
//! transcription waves.

#![allow(non_snake_case)]

use core::ffi::c_int;

use mp_bg::public::gametype::{
    GT_CTF, GT_CTY, GT_FFA, GT_HOLOCRON, GT_JEDIMASTER, GT_SIEGE, GT_TEAM,
};
use mp_bg::public::pers_enum::persEnum_t::PERS_TEAM;
use mp_bg::public::pmtype::pmtype_t;
use mp_bg::public::team::{TEAM_BLUE, TEAM_RED};
use mp_bg::public::teamtask::teamtask_t;

use mp_qshared::shared::{qfalse, qhandle_t, vec4_t, FLAG_TAKEN};

use mp_uishared::shared::display_state::DisplayState;
use mp_uishared::shared::menu_system::MenuSystem;
use mp_uishared::ui_shared::{
    Display_CursorType, Display_MouseMove, Menus_CloseByName, Menus_OpenByName, CURSOR_ARROW,
    CURSOR_SIZER,
};

use native_string::{string_to_latin1, Q_stricmpBytes};

use crate::trap;
use crate::world::cg_context::CgContext;
use crate::world::cg_world::CgWorld;

/// Raven `#define PIC_WIDTH 12`.
///
/// Source: `oracle/codemp/cgame/cg_newDraw.c:324`
pub const PIC_WIDTH: c_int = 12;

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
