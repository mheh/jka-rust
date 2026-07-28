//! Port of `oracle/codemp/cgame/cg_consolecmds.c` — the cgame console-command table and its handlers. Functions land via the C5
//! transcription waves.

#![allow(non_snake_case)]

use native_string::atoi;

use mp_qshared::shared::limits::MAX_TOKEN_CHARS;
use mp_qshared::shared::qfalse;

use mp_uishared::shared::display_context::DisplayContext;
use mp_uishared::shared::display_state::DisplayState;
use mp_uishared::shared::menu_id::MenuId;
use mp_uishared::shared::menu_system::MenuSystem;
use mp_uishared::shared::menudef::{FEEDER_BLUETEAM_LIST, FEEDER_REDTEAM_LIST, FEEDER_SCOREBOARD};
use mp_uishared::ui_shared::Menu_ScrollFeeder;

use crate::trap;
use crate::world::cg_context::CgContext;
use crate::world::cg_world::CgWorld;

/// Raven `consoleCommand_t commands[]` — the console-command registration
/// table. `CG_InitConsoleCommands` (below) only reads the `cmd` name column;
/// the `function` dispatch column is `CG_ConsoleCommand`'s concern, a fn this
/// wave does not open. The commented-out `"camera"` entry (`cg_consolecmds.c:289`)
/// never compiled into Raven's array, so it is omitted here too.
///
/// Source: `oracle/codemp/cgame/cg_consolecmds.c:265-298`
const COMMANDS: &[&str] = &[
    "testgun",
    "testmodel",
    "nextframe",
    "prevframe",
    "nextskin",
    "prevskin",
    "viewpos",
    "+scores",
    "-scores",
    "sizeup",
    "sizedown",
    "weapnext",
    "weapprev",
    "weapon",
    "weaponclean",
    "tell_target",
    "tell_attacker",
    "tcmd",
    "spWin",
    "spLose",
    "scoresDown",
    "scoresUp",
    "startOrbit",
    "loaddeferred",
    "invnext",
    "invprev",
    "forcenext",
    "forceprev",
    "briefing",
    "siegeCvarUpdate",
    "siegeCompleteCvarUpdate",
];

/// Raven `CG_SizeUp_f` — bumps `cg_viewsize` up by 10.
///
/// Source: `oracle/codemp/cgame/cg_consolecmds.c:35-37`
pub fn CG_SizeUp_f(ctx: &mut CgContext) {
    let size = ctx.world.cvars.cg_viewsize.integer + 10;
    trap::Cvar_Set(ctx.engine, "cg_viewsize", &format!("{}", size));
}

/// Raven `CG_SizeDown_f` — drops `cg_viewsize` down by 10.
///
/// Source: `oracle/codemp/cgame/cg_consolecmds.c:47-49`
pub fn CG_SizeDown_f(ctx: &mut CgContext) {
    let size = ctx.world.cvars.cg_viewsize.integer - 10;
    trap::Cvar_Set(ctx.engine, "cg_viewsize", &format!("{}", size));
}

/// Raven `CG_ScoresUp_f` — dismisses the scoreboard if it's showing, latching
/// the fade-out start time to now.
///
/// Source: `oracle/codemp/cgame/cg_consolecmds.c:88-93`
pub fn CG_ScoresUp_f(world: &mut CgWorld) {
    if world.cg.showScores != qfalse {
        world.cg.showScores = qfalse;
        world.cg.scoreFadeTime = world.cg.time;
    }
}

/// Raven `CG_scrollScoresDown_f` — simulates a cursor-down keypress on the
/// scoreboard's three list feeders while the scoreboard is up.
///
/// `menuScoreboard` is `cg_draw.c`'s cached scoreboard menu handle
/// (`Menus_FindByName` result); it is not yet a `CgWorld` field (that TU's
/// wave hasn't folded its statics in), so it threads in as a parameter here —
/// the caller supplies it the same way `menus`/`ds`/`dc` already thread beside
/// `CgContext` (DEC-46 `CgState` split).
///
/// Source: `oracle/codemp/cgame/cg_consolecmds.c:98-104`
pub fn CG_scrollScoresDown_f(
    world: &CgWorld,
    menus: &mut MenuSystem,
    ds: &DisplayState,
    dc: &mut dyn DisplayContext,
    menuScoreboard: Option<MenuId>,
) {
    if menuScoreboard.is_some() && world.cg.scoreBoardShowing != qfalse {
        Menu_ScrollFeeder(menus, ds, dc, menuScoreboard, FEEDER_SCOREBOARD, true);
        Menu_ScrollFeeder(menus, ds, dc, menuScoreboard, FEEDER_REDTEAM_LIST, true);
        Menu_ScrollFeeder(menus, ds, dc, menuScoreboard, FEEDER_BLUETEAM_LIST, true);
    }
}

/// Raven `CG_scrollScoresUp_f` — simulates a cursor-up keypress on the
/// scoreboard's three list feeders while the scoreboard is up.
///
/// See [`CG_scrollScoresDown_f`] for why `menuScoreboard` threads as a
/// parameter.
///
/// Source: `oracle/codemp/cgame/cg_consolecmds.c:107-113`
pub fn CG_scrollScoresUp_f(
    world: &CgWorld,
    menus: &mut MenuSystem,
    ds: &DisplayState,
    dc: &mut dyn DisplayContext,
    menuScoreboard: Option<MenuId>,
) {
    if menuScoreboard.is_some() && world.cg.scoreBoardShowing != qfalse {
        Menu_ScrollFeeder(menus, ds, dc, menuScoreboard, FEEDER_SCOREBOARD, false);
        Menu_ScrollFeeder(menus, ds, dc, menuScoreboard, FEEDER_REDTEAM_LIST, false);
        Menu_ScrollFeeder(menus, ds, dc, menuScoreboard, FEEDER_BLUETEAM_LIST, false);
    }
}

/// Raven `CG_StartOrbit_f` — toggles the developer-only orbit camera; a no-op
/// unless `developer` is set (guards a cheat-adjacent debug feature outside
/// dev builds).
///
/// Source: `oracle/codemp/cgame/cg_consolecmds.c:176-192`
pub fn CG_StartOrbit_f(ctx: &mut CgContext) {
    let var = trap::Cvar_VariableStringBuffer(ctx.engine, "developer", MAX_TOKEN_CHARS);
    if atoi(&var) == 0 {
        return;
    }

    if ctx.world.cvars.cg_cameraOrbit.value != 0.0 {
        trap::Cvar_Set(ctx.engine, "cg_cameraOrbit", "0");
        trap::Cvar_Set(ctx.engine, "cg_thirdPerson", "0");
    } else {
        trap::Cvar_Set(ctx.engine, "cg_cameraOrbit", "5");
        trap::Cvar_Set(ctx.engine, "cg_thirdPerson", "1");
        trap::Cvar_Set(ctx.engine, "cg_thirdPersonAngle", "0");
        trap::Cvar_Set(ctx.engine, "cg_thirdPersonRange", "100");
    }
}

/// Raven `CG_InitConsoleCommands` — registers every cgame console command with
/// the engine, plus the block of server-side commands cgame only forwards
/// (unrecognized locally, so the engine relays them to the game server).
///
/// Source: `oracle/codemp/cgame/cg_consolecmds.c:334-391`
pub fn CG_InitConsoleCommands(ctx: &mut CgContext) {
    for cmd in COMMANDS.iter().copied() {
        trap::AddCommand(ctx.engine, cmd);
    }

    // the game server will interpret these commands, which will be automatically
    // forwarded to the server after they are not recognized locally
    trap::AddCommand(ctx.engine, "forcechanged");
    trap::AddCommand(ctx.engine, "sv_invnext");
    trap::AddCommand(ctx.engine, "sv_invprev");
    trap::AddCommand(ctx.engine, "sv_forcenext");
    trap::AddCommand(ctx.engine, "sv_forceprev");
    trap::AddCommand(ctx.engine, "sv_saberswitch");
    trap::AddCommand(ctx.engine, "engage_duel");
    trap::AddCommand(ctx.engine, "force_heal");
    trap::AddCommand(ctx.engine, "force_speed");
    trap::AddCommand(ctx.engine, "force_throw");
    trap::AddCommand(ctx.engine, "force_pull");
    trap::AddCommand(ctx.engine, "force_distract");
    trap::AddCommand(ctx.engine, "force_rage");
    trap::AddCommand(ctx.engine, "force_protect");
    trap::AddCommand(ctx.engine, "force_absorb");
    trap::AddCommand(ctx.engine, "force_healother");
    trap::AddCommand(ctx.engine, "force_forcepowerother");
    trap::AddCommand(ctx.engine, "force_seeing");
    trap::AddCommand(ctx.engine, "use_seeker");
    trap::AddCommand(ctx.engine, "use_field");
    trap::AddCommand(ctx.engine, "use_bacta");
    trap::AddCommand(ctx.engine, "use_electrobinoculars");
    trap::AddCommand(ctx.engine, "zoom");
    trap::AddCommand(ctx.engine, "use_sentry");
    trap::AddCommand(ctx.engine, "bot_order");
    trap::AddCommand(ctx.engine, "saberAttackCycle");
    trap::AddCommand(ctx.engine, "kill");
    trap::AddCommand(ctx.engine, "say");
    trap::AddCommand(ctx.engine, "say_team");
    trap::AddCommand(ctx.engine, "tell");
    trap::AddCommand(ctx.engine, "give");
    trap::AddCommand(ctx.engine, "god");
    trap::AddCommand(ctx.engine, "notarget");
    trap::AddCommand(ctx.engine, "noclip");
    trap::AddCommand(ctx.engine, "team");
    trap::AddCommand(ctx.engine, "follow");
    trap::AddCommand(ctx.engine, "levelshot");
    trap::AddCommand(ctx.engine, "addbot");
    trap::AddCommand(ctx.engine, "setviewpos");
    trap::AddCommand(ctx.engine, "callvote");
    trap::AddCommand(ctx.engine, "vote");
    trap::AddCommand(ctx.engine, "callteamvote");
    trap::AddCommand(ctx.engine, "teamvote");
    trap::AddCommand(ctx.engine, "stats");
    trap::AddCommand(ctx.engine, "teamtask");
    // spelled wrong, but not changing for demo
    trap::AddCommand(ctx.engine, "loaddefered");
}
