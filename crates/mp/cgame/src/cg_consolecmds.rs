//! Port of `oracle/codemp/cgame/cg_consolecmds.c` — the cgame console-command table and its handlers. Functions land via the C5
//! transcription waves.

#![allow(non_snake_case)]

use core::ffi::c_int;

use native_string::{atoi, buf_to_string, Q_stricmp};

use mp_bg::public::gametype::GT_SIEGE;
use mp_bg::public::pers_enum::persEnum_t::PERS_TEAM;
use mp_bg::saga::siege_team_t::{SIEGETEAM_TEAM1, SIEGETEAM_TEAM2};

use mp_qshared::shared::limits::MAX_TOKEN_CHARS;
use mp_qshared::shared::q_math::YAW;
use mp_qshared::shared::SCREEN_HEIGHT;
use mp_qshared::shared::{qfalse, qtrue};

use mp_uishared::shared::display_context::DisplayContext;
use mp_uishared::shared::display_state::DisplayState;
use mp_uishared::shared::menu_id::MenuId;
use mp_uishared::shared::menu_system::MenuSystem;
use mp_uishared::shared::menudef::{FEEDER_BLUETEAM_LIST, FEEDER_REDTEAM_LIST, FEEDER_SCOREBOARD};
use mp_uishared::ui_shared::Menu_ScrollFeeder;

use crate::cg_draw::CG_CenterPrint;
use crate::cg_main::{
    CG_Argv, CG_BuildSpectatorString, CG_CrosshairPlayer, CG_GetStringEdString, CG_LastAttacker,
    CG_NextForcePower_f, CG_NextInventory_f, CG_PrevForcePower_f, CG_PrevInventory_f, CG_Printf,
};
use crate::cg_players::CG_LoadDeferredPlayers;
use crate::cg_saga::CG_SiegeBriefingDisplay;
use crate::cg_view::{
    CG_AddBufferedSound, CG_TestGun_f, CG_TestModelNextFrame_f, CG_TestModelNextSkin_f,
    CG_TestModelPrevFrame_f, CG_TestModelPrevSkin_f, CG_TestModel_f,
};
use crate::cg_weapons::{CG_NextWeapon_f, CG_PrevWeapon_f, CG_WeaponClean_f, CG_Weapon_f};
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

/// Raven `CG_TargetCommand_f` — sends the `gc <targetNum> <parm>` server
/// command for whichever crosshair-target console command (bound key) invoked
/// it; a no-op with nothing under the crosshair.
///
/// Source: `oracle/codemp/cgame/cg_consolecmds.c:13-24`
pub fn CG_TargetCommand_f(ctx: &mut CgContext) {
    let targetNum = CG_CrosshairPlayer(ctx.world);
    if targetNum == 0 {
        return;
    }

    let test = trap::Argv(ctx.engine, 1, 4);
    trap::SendConsoleCommand(ctx.engine, &format!("gc {} {}", targetNum, atoi(&test)));
}

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

/// Raven `CG_Viewpos_f` — prints the map name plus the current view origin/yaw
/// to the console, x86 float-to-int truncation on each component.
///
/// Source: `oracle/codemp/cgame/cg_consolecmds.c:59-63`
pub fn CG_Viewpos_f(ctx: &mut CgContext) {
    let mapname = buf_to_string(&ctx.world.cgs.mapname.map(|c| c as u8));
    let vieworg = ctx.world.cg.refdef.vieworg;
    let viewangles = ctx.world.cg.refdef.viewangles;
    let msg = format!(
        "{} ({} {} {}) : {}\n",
        mapname,
        vieworg[0] as c_int,
        vieworg[1] as c_int,
        vieworg[2] as c_int,
        viewangles[YAW] as c_int,
    );
    CG_Printf(ctx, &msg);
}

/// Raven `CG_ScoresDown_f` — requests a fresh scoreboard from the server if
/// the cached one is more than two seconds stale, else just shows the cached
/// contents.
///
/// Source: `oracle/codemp/cgame/cg_consolecmds.c:66-86`
pub fn CG_ScoresDown_f(ctx: &mut CgContext) {
    CG_BuildSpectatorString(ctx);
    if ctx.world.cg.scoresRequestTime + 2000 < ctx.world.cg.time {
        // the scores are more than two seconds out of data,
        // so request new ones
        ctx.world.cg.scoresRequestTime = ctx.world.cg.time;
        trap::SendClientCommand(ctx.engine, "score");

        // leave the current scores up if they were already
        // displayed, but if this is the first hit, clear them out
        if ctx.world.cg.showScores == qfalse {
            ctx.world.cg.showScores = qtrue;
            ctx.world.cg.numScores = 0;
        }
    } else {
        // show the cached contents even if they just pressed if it
        // is within two seconds
        ctx.world.cg.showScores = qtrue;
    }
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

/// Raven `CG_spWin_f` — the SP mission-win screen: parks the camera in orbit,
/// plays the winner stinger, and centerprints the "you win" string.
///
/// Raven's `trap_S_StartLocalSound` line is commented out in the oracle;
/// `CG_AddBufferedSound` is the only sound call left live. Kept as written.
///
/// Source: `oracle/codemp/cgame/cg_consolecmds.c:116-125`
pub fn CG_spWin_f(ctx: &mut CgContext) {
    trap::Cvar_Set(ctx.engine, "cg_cameraOrbit", "2");
    trap::Cvar_Set(ctx.engine, "cg_cameraOrbitDelay", "35");
    trap::Cvar_Set(ctx.engine, "cg_thirdPerson", "1");
    trap::Cvar_Set(ctx.engine, "cg_thirdPersonAngle", "0");
    trap::Cvar_Set(ctx.engine, "cg_thirdPersonRange", "100");

    let winnerSound = ctx.world.cgs.media.winnerSound;
    CG_AddBufferedSound(ctx.world, winnerSound);
    // trap_S_StartLocalSound(cgs.media.winnerSound, CHAN_ANNOUNCER); - commented out in Raven

    let msg = CG_GetStringEdString(ctx, "MP_INGAME", "YOU_WIN");
    CG_CenterPrint(ctx.world, &msg, (SCREEN_HEIGHT as f64 * 0.30) as c_int, 0);
}

/// Raven `CG_spLose_f` — the SP mission-loss screen: parks the camera in
/// orbit, plays the loser stinger, and centerprints the "you lose" string.
///
/// Raven's `trap_S_StartLocalSound` line is commented out in the oracle;
/// `CG_AddBufferedSound` is the only sound call left live. Kept as written.
///
/// Source: `oracle/codemp/cgame/cg_consolecmds.c:127-136`
pub fn CG_spLose_f(ctx: &mut CgContext) {
    trap::Cvar_Set(ctx.engine, "cg_cameraOrbit", "2");
    trap::Cvar_Set(ctx.engine, "cg_cameraOrbitDelay", "35");
    trap::Cvar_Set(ctx.engine, "cg_thirdPerson", "1");
    trap::Cvar_Set(ctx.engine, "cg_thirdPersonAngle", "0");
    trap::Cvar_Set(ctx.engine, "cg_thirdPersonRange", "100");

    let loserSound = ctx.world.cgs.media.loserSound;
    CG_AddBufferedSound(ctx.world, loserSound);
    // trap_S_StartLocalSound(cgs.media.loserSound, CHAN_ANNOUNCER); - commented out in Raven

    let msg = CG_GetStringEdString(ctx, "MP_INGAME", "YOU_LOSE");
    CG_CenterPrint(ctx.world, &msg, (SCREEN_HEIGHT as f64 * 0.30) as c_int, 0);
}

/// Raven `CG_TellTarget_f` — sends a `tell <clientNum> <message>` server
/// command to whoever is under the crosshair; a no-op with nothing under it.
///
/// Source: `oracle/codemp/cgame/cg_consolecmds.c:139-152`
pub fn CG_TellTarget_f(ctx: &mut CgContext) {
    let clientNum = CG_CrosshairPlayer(ctx.world);
    if clientNum == -1 {
        return;
    }

    let message = trap::Args(ctx.engine, 128);
    // Com_sprintf into `command[128]` - one Latin-1 char is one C byte, so 127
    // of them plus the NUL is everything that survives
    let command: String = format!("tell {} {}", clientNum, message)
        .chars()
        .take(127)
        .collect();
    trap::SendClientCommand(ctx.engine, &command);
}

/// Raven `CG_TellAttacker_f` — sends a `tell <clientNum> <message>` server
/// command to the local player's last attacker; a no-op with none recorded.
///
/// Source: `oracle/codemp/cgame/cg_consolecmds.c:154-167`
pub fn CG_TellAttacker_f(ctx: &mut CgContext) {
    let clientNum = CG_LastAttacker(ctx.world);
    if clientNum == -1 {
        return;
    }

    let message = trap::Args(ctx.engine, 128);
    // Com_sprintf into `command[128]` - one Latin-1 char is one C byte, so 127
    // of them plus the NUL is everything that survives
    let command: String = format!("tell {} {}", clientNum, message)
        .chars()
        .take(127)
        .collect();
    trap::SendClientCommand(ctx.engine, &command);
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

/// Raven `CG_SiegeBriefing_f` — pops the siege briefing display for the
/// local player's own team; a no-op outside siege or off a valid siege team.
///
/// Source: `oracle/codemp/cgame/cg_consolecmds.c:195-213`
pub fn CG_SiegeBriefing_f(ctx: &mut CgContext) {
    if ctx.world.cgs.gametype != GT_SIEGE {
        // cannot be displayed unless in this gametype
        return;
    }

    let team = ctx.world.cg.predictedPlayerState.persistant[PERS_TEAM as usize];
    if team != SIEGETEAM_TEAM1 && team != SIEGETEAM_TEAM2 {
        // cannot be displayed if not on a valid team
        return;
    }

    CG_SiegeBriefingDisplay(ctx, team, 0);
}

/// Raven `CG_SiegeCvarUpdate_f` — refreshes the siege briefing cvars for the
/// local player's own team without popping the display; a no-op outside
/// siege or off a valid siege team.
///
/// Source: `oracle/codemp/cgame/cg_consolecmds.c:215-233`
pub fn CG_SiegeCvarUpdate_f(ctx: &mut CgContext) {
    if ctx.world.cgs.gametype != GT_SIEGE {
        // cannot be displayed unless in this gametype
        return;
    }

    let team = ctx.world.cg.predictedPlayerState.persistant[PERS_TEAM as usize];
    if team != SIEGETEAM_TEAM1 && team != SIEGETEAM_TEAM2 {
        // cannot be displayed if not on a valid team
        return;
    }

    CG_SiegeBriefingDisplay(ctx, team, 1);
}

/// Raven `CG_SiegeCompleteCvarUpdate_f` — refreshes the siege briefing cvars
/// for both teams at once (siege load); a no-op outside siege.
///
/// Source: `oracle/codemp/cgame/cg_consolecmds.c:234-245`
pub fn CG_SiegeCompleteCvarUpdate_f(ctx: &mut CgContext) {
    if ctx.world.cgs.gametype != GT_SIEGE {
        // cannot be displayed unless in this gametype
        return;
    }

    // Set up cvars for both teams
    CG_SiegeBriefingDisplay(ctx, SIEGETEAM_TEAM1, 1);
    CG_SiegeBriefingDisplay(ctx, SIEGETEAM_TEAM2, 1);
}

/// Raven `CG_ConsoleCommand` — the vmMain console-command dispatch: walks
/// `commands[]` for a case-insensitive name match against `argv(0)` and calls
/// the matching handler, `qtrue`; `qfalse` if nothing matched (the engine then
/// tries the command elsewhere).
///
/// DEC-46.4 turns Raven's `commands[]` fn-pointer column into this `match`; the
/// name loop over `COMMANDS` keeps Raven's `Q_stricmp` case-insensitive
/// matching instead of a case-sensitive Rust `match` on the string itself.
/// `menus`/`ds`/`dc`/`menuScoreboard` thread in beside `ctx` because two arms
/// (`scoresDown`/`scoresUp`) reach [`CG_scrollScoresDown_f`]/
/// [`CG_scrollScoresUp_f`], which need the shared menu framework the same way
/// their own doc comments explain.
///
/// Source: `oracle/codemp/cgame/cg_consolecmds.c:309-323`
pub fn CG_ConsoleCommand(
    ctx: &mut CgContext,
    menus: &mut MenuSystem,
    ds: &DisplayState,
    dc: &mut dyn DisplayContext,
    menuScoreboard: Option<MenuId>,
) -> bool {
    let cmd = CG_Argv(ctx, 0);

    for name in COMMANDS.iter().copied() {
        if Q_stricmp(&cmd, name) != 0 {
            continue;
        }

        match name {
            "testgun" => CG_TestGun_f(ctx),
            "testmodel" => CG_TestModel_f(ctx),
            "nextframe" => CG_TestModelNextFrame_f(ctx),
            "prevframe" => CG_TestModelPrevFrame_f(ctx),
            "nextskin" => CG_TestModelNextSkin_f(ctx),
            "prevskin" => CG_TestModelPrevSkin_f(ctx),
            "viewpos" => CG_Viewpos_f(ctx),
            "+scores" => CG_ScoresDown_f(ctx),
            "-scores" => CG_ScoresUp_f(ctx.world),
            "sizeup" => CG_SizeUp_f(ctx),
            "sizedown" => CG_SizeDown_f(ctx),
            "weapnext" => CG_NextWeapon_f(ctx),
            "weapprev" => CG_PrevWeapon_f(ctx),
            "weapon" => CG_Weapon_f(ctx),
            "weaponclean" => CG_WeaponClean_f(ctx),
            "tell_target" => CG_TellTarget_f(ctx),
            "tell_attacker" => CG_TellAttacker_f(ctx),
            "tcmd" => CG_TargetCommand_f(ctx),
            "spWin" => CG_spWin_f(ctx),
            "spLose" => CG_spLose_f(ctx),
            "scoresDown" => CG_scrollScoresDown_f(ctx.world, menus, ds, dc, menuScoreboard),
            "scoresUp" => CG_scrollScoresUp_f(ctx.world, menus, ds, dc, menuScoreboard),
            "startOrbit" => CG_StartOrbit_f(ctx),
            "loaddeferred" => CG_LoadDeferredPlayers(ctx.world),
            "invnext" => CG_NextInventory_f(ctx.world),
            "invprev" => CG_PrevInventory_f(ctx.world),
            "forcenext" => CG_NextForcePower_f(ctx),
            "forceprev" => CG_PrevForcePower_f(ctx),
            "briefing" => CG_SiegeBriefing_f(ctx),
            "siegeCvarUpdate" => CG_SiegeCvarUpdate_f(ctx),
            "siegeCompleteCvarUpdate" => CG_SiegeCompleteCvarUpdate_f(ctx),
            _ => unreachable!("COMMANDS lists {name:?} with no CG_ConsoleCommand dispatch arm"),
        }

        return true;
    }

    false
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
