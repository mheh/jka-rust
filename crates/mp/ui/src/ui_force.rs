//! `ui_force.c` — the force-allocation screen.
//!
//! Source: `oracle/codemp/ui/ui_force.c`

#![allow(non_snake_case)]

use core::ffi::c_int;

use mp_bg::bg_misc::forceMasteryPoints;
use mp_bg::bg_misc::BG_LegalizedForcePowers;
use mp_bg::public::configstring::CS_SERVERINFO;
use mp_bg::public::team::{TEAM_BLUE, TEAM_RED, TEAM_SPECTATOR};
use mp_qshared::common::mp::qcommon::saber::saber_colors::{
    SABER_BLUE, SABER_GREEN, SABER_ORANGE, SABER_PURPLE, SABER_RED, SABER_YELLOW,
};
use mp_qshared::shared::cbuf_exec::cbufExec_t;
use mp_qshared::shared::force_powers::{
    FORCE_DARKSIDE, FORCE_LEVEL_1, FORCE_LEVEL_3, FORCE_LIGHTSIDE, FP_HEAL, FP_LEVITATION,
    FP_SABERTHROW, FP_SABER_DEFENSE, FP_SABER_OFFENSE, NUM_FORCE_POWERS, NUM_FORCE_POWER_LEVELS,
};
use mp_qshared::shared::vec4_t;
use mp_qshared::shared::{FS_READ, FS_WRITE, MAX_INFO_VALUE};
use mp_uishared::shared::display_state::DisplayState;
use mp_uishared::shared::menu_system::MenuSystem;
use mp_uishared::shared::menudef::{
    FEEDER_FORCECFG, FEEDER_Q3HEADS, UI_FORCE_RANK, UI_FORCE_RANK_LEVITATION,
    UI_FORCE_RANK_SABERATTACK, UI_FORCE_RANK_SABERDEFEND,
};
use mp_uishared::shared::rect_def_t::RectDef;
use mp_uishared::ui_shared::{Menu_SetFeederSelection, Menu_ShowItemByName, Menus_FindByName};
use native_string::{atoi, latin1_to_string, Info_ValueForKey, Q_stricmp};
use native_types::fileHandle_t;

use crate::keycodes::fake_ascii_t::fakeAscii_t;
use crate::trap;
use crate::world::ui_context::UiContext;
use crate::world::ui_world::UiWorld;

use super::ui_atoms::{Com_Printf, UI_Cvar_VariableString, UI_DrawHandlePic};
use super::ui_main::{
    UI_FeederSelection, UI_LoadForceConfig_List, UI_TeamName, UI_TrueJediEnabled,
};

/// Raven `#define FORCE_NONJEDI 0`.
///
/// Source: `oracle/codemp/ui/ui_force.h:4`
const FORCE_NONJEDI: c_int = 0;

/// Raven `#define FORCE_JEDI 1`.
///
/// Source: `oracle/codemp/ui/ui_force.h:5`
const FORCE_JEDI: c_int = 1;

/// Raven `UI_InitForceShaders` — registers the force-star and saber-color
/// shaders used on the force-allocation screen.
///
/// Source: `oracle/codemp/ui/ui_force.c:99-126`
pub fn UI_InitForceShaders(ctx: &mut UiContext) {
    let force = &mut ctx.world.force;

    force.uiForceStarShaders[0][0] = trap::R_RegisterShaderNoMip(ctx.engine, "forcestar0");
    force.uiForceStarShaders[0][1] = trap::R_RegisterShaderNoMip(ctx.engine, "forcestar0");
    force.uiForceStarShaders[1][0] = trap::R_RegisterShaderNoMip(ctx.engine, "forcecircle1");
    force.uiForceStarShaders[1][1] = trap::R_RegisterShaderNoMip(ctx.engine, "forcestar1");
    force.uiForceStarShaders[2][0] = trap::R_RegisterShaderNoMip(ctx.engine, "forcecircle2");
    force.uiForceStarShaders[2][1] = trap::R_RegisterShaderNoMip(ctx.engine, "forcestar2");
    force.uiForceStarShaders[3][0] = trap::R_RegisterShaderNoMip(ctx.engine, "forcecircle3");
    force.uiForceStarShaders[3][1] = trap::R_RegisterShaderNoMip(ctx.engine, "forcestar3");
    force.uiForceStarShaders[4][0] = trap::R_RegisterShaderNoMip(ctx.engine, "forcecircle4");
    force.uiForceStarShaders[4][1] = trap::R_RegisterShaderNoMip(ctx.engine, "forcestar4");
    force.uiForceStarShaders[5][0] = trap::R_RegisterShaderNoMip(ctx.engine, "forcecircle5");
    force.uiForceStarShaders[5][1] = trap::R_RegisterShaderNoMip(ctx.engine, "forcestar5");
    force.uiForceStarShaders[6][0] = trap::R_RegisterShaderNoMip(ctx.engine, "forcecircle6");
    force.uiForceStarShaders[6][1] = trap::R_RegisterShaderNoMip(ctx.engine, "forcestar6");
    force.uiForceStarShaders[7][0] = trap::R_RegisterShaderNoMip(ctx.engine, "forcecircle7");
    force.uiForceStarShaders[7][1] = trap::R_RegisterShaderNoMip(ctx.engine, "forcestar7");
    force.uiForceStarShaders[8][0] = trap::R_RegisterShaderNoMip(ctx.engine, "forcecircle8");
    force.uiForceStarShaders[8][1] = trap::R_RegisterShaderNoMip(ctx.engine, "forcestar8");

    force.uiSaberColorShaders[SABER_RED as usize] =
        trap::R_RegisterShaderNoMip(ctx.engine, "menu/art/saber_red");
    force.uiSaberColorShaders[SABER_ORANGE as usize] =
        trap::R_RegisterShaderNoMip(ctx.engine, "menu/art/saber_orange");
    force.uiSaberColorShaders[SABER_YELLOW as usize] =
        trap::R_RegisterShaderNoMip(ctx.engine, "menu/art/saber_yellow");
    force.uiSaberColorShaders[SABER_GREEN as usize] =
        trap::R_RegisterShaderNoMip(ctx.engine, "menu/art/saber_green");
    force.uiSaberColorShaders[SABER_BLUE as usize] =
        trap::R_RegisterShaderNoMip(ctx.engine, "menu/art/saber_blue");
    force.uiSaberColorShaders[SABER_PURPLE as usize] =
        trap::R_RegisterShaderNoMip(ctx.engine, "menu/art/saber_purple");
}

/// Raven `UI_UpdateClientForcePowers` — writes the `forcepowers` cvar from the
/// current allocation, and if the player touched anything, appends the
/// `forcechanged` client command (optionally with `teamArg`).
///
/// Source: `oracle/codemp/ui/ui_force.c:174-198`
pub fn UI_UpdateClientForcePowers(ctx: &mut UiContext, teamArg: &str) {
    let r = ctx.world.force.uiForcePowersRank;
    let value = format!(
        "{}-{}-{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}",
        ctx.world.force.uiForceRank,
        ctx.world.force.uiForceSide,
        r[0],
        r[1],
        r[2],
        r[3],
        r[4],
        r[5],
        r[6],
        r[7],
        r[8],
        r[9],
        r[10],
        r[11],
        r[12],
        r[13],
        r[14],
        r[15],
        r[16],
        r[17],
    );
    trap::Cvar_Set(ctx.engine, "forcepowers", &value);

    if ctx.world.force.gTouchedForce {
        if !teamArg.is_empty() {
            trap::Cmd_ExecuteText(
                ctx.engine,
                cbufExec_t::EXEC_APPEND as c_int,
                &format!("forcechanged \"{}\"\n", teamArg),
            );
        } else {
            trap::Cmd_ExecuteText(
                ctx.engine,
                cbufExec_t::EXEC_APPEND as c_int,
                "forcechanged\n",
            );
        }
    }

    ctx.world.force.gTouchedForce = false;
}

/// Raven `UI_TranslateFCFIndex` — translates a raw force-config-list index
/// into the index relative to its light/dark sub-range.
///
/// Source: `oracle/codemp/ui/ui_force.c:200-208`
pub fn UI_TranslateFCFIndex(world: &UiWorld, index: c_int) -> c_int {
    if world.force.uiForceSide == FORCE_LIGHTSIDE {
        return index - world.forceConfigLightIndexBegin;
    }

    index - world.forceConfigDarkIndexBegin
}

/// Raven `UI_DrawForceStars` — draws a row of force-power stars on a force
/// UI screen, with shading for disabled powers.
///
/// Source: `oracle/codemp/ui/ui_force.c:129-171`
pub fn UI_DrawForceStars(
    ctx: &mut UiContext,
    rect: &RectDef,
    _scale: f32,
    _color: &vec4_t,
    _textStyle: c_int,
    forceindex: c_int,
    val: c_int,
    min: c_int,
    max: c_int,
) {
    // Raven `int xPos = rect->x` — the origin truncates to int and advances by
    // ints, so every draw lands on the truncated position.
    let mut xPos: c_int = rect.x as c_int;
    let width: c_int = 16;
    let pad: c_int = 4;

    let mut v = val;
    if v < min || v > max {
        v = min;
    }

    for i in FORCE_LEVEL_1..=max {
        let star_color = ctx.world.bg_state.bgForcePowerCost[forceindex as usize][i as usize];

        if ctx.world.force.uiForcePowersDisabled[forceindex as usize] {
            let gr_color: vec4_t = [0.2, 0.2, 0.2, 1.0];
            trap::R_SetColor(ctx.engine, Some(&gr_color));
        }

        if v >= i {
            // Draw a star.
            UI_DrawHandlePic(
                ctx,
                xPos as f32,
                rect.y + 6.0,
                width as f32,
                width as f32,
                ctx.world.force.uiForceStarShaders[star_color as usize][1],
            );
        } else {
            // Draw a circle.
            UI_DrawHandlePic(
                ctx,
                xPos as f32,
                rect.y + 6.0,
                width as f32,
                width as f32,
                ctx.world.force.uiForceStarShaders[star_color as usize][0],
            );
        }

        if ctx.world.force.uiForcePowersDisabled[forceindex as usize] {
            trap::R_SetColor(ctx.engine, None);
        }

        xPos += width + pad;
    }
}

/// Raven `UI_SaveForceTemplate` — writes the current force-power allocation
/// to a `.fcf` template file under `forcecfg/light/` or `forcecfg/dark/`,
/// then re-scans the force-config feeder list and selects the newly-saved
/// entry (falling back to index 0 if the saved name doesn't match anything
/// in the current-side range).
///
/// Source: `oracle/codemp/ui/ui_force.c:210-285`
pub fn UI_SaveForceTemplate(ctx: &mut UiContext, menus: &mut MenuSystem, ds: &DisplayState) {
    let selectedName = UI_Cvar_VariableString(ctx, "ui_SaveFCF");

    if selectedName.is_empty() {
        Com_Printf(ctx, "You did not provide a name for the template.\n");
        return;
    }

    let mut f: fileHandle_t = 0;
    if ctx.world.force.uiForceSide == FORCE_LIGHTSIDE {
        // write it into the light side folder
        trap::FS_FOpenFile(
            ctx.engine,
            &format!("forcecfg/light/{}.fcf", selectedName),
            &mut f,
            FS_WRITE,
        );
    } else {
        // if it isn't light it must be dark
        trap::FS_FOpenFile(
            ctx.engine,
            &format!("forcecfg/dark/{}.fcf", selectedName),
            &mut f,
            FS_WRITE,
        );
    }

    if f == 0 {
        Com_Printf(
            ctx,
            "There was an error writing the template file (read-only?).\n",
        );
        return;
    }

    let mut fcfString = format!(
        "{}-{}-",
        ctx.world.force.uiForceRank, ctx.world.force.uiForceSide
    );
    // PORT-NOTE: Raven takes only the first character of the formatted rank
    // digit ("Just use the force digit even if multiple digits. Shouldn't be
    // longer than 1.") — mirror that literally rather than writing the whole
    // formatted number.
    for rank in ctx.world.force.uiForcePowersRank {
        if let Some(digit) = format!("{}", rank).chars().next() {
            fcfString.push(digit);
        }
    }
    fcfString.push('\n');

    trap::FS_Write(ctx.engine, fcfString.as_bytes(), f);
    trap::FS_FCloseFile(ctx.engine, f);

    Com_Printf(ctx, &format!("Template saved as \"{}\".\n", selectedName));

    // Now, update the FCF list
    UI_LoadForceConfig_List(ctx);

    // Then, scroll through and select the template for the file we just saved
    let mut foundFeederItem = false;
    let count = ctx.world.forceConfigNames.len();
    for i in 0..count {
        if Q_stricmp(&ctx.world.forceConfigNames[i], &selectedName) == 0
            && ((ctx.world.force.uiForceSide == FORCE_LIGHTSIDE && ctx.world.forceConfigSide[i])
                || (ctx.world.force.uiForceSide == FORCE_DARKSIDE && !ctx.world.forceConfigSide[i]))
        {
            let translated = UI_TranslateFCFIndex(ctx.world, i as c_int);
            Menu_SetFeederSelection(menus, ds, ctx, None, FEEDER_FORCECFG, translated, None);
            foundFeederItem = true;
        }
    }

    // Else, go back to 0
    if !foundFeederItem {
        Menu_SetFeederSelection(menus, ds, ctx, None, FEEDER_FORCECFG, 0, None);
    }
}

/// Raven `UpdateForceUsed` — recomputes the force-power budget: locks in the
/// max rank, ensures the level-1 jump/saber freebies, resolves jedi/non-jedi
/// status in true-jedi mode (clearing or seeding saber offense and notifying
/// the server via `forcechanged`), toggles saber-related menu items for the
/// free-saber cvar, and clamps + re-prices every force power rank against the
/// available point budget.
///
/// Source: `oracle/codemp/ui/ui_force.c:290-460`
pub fn UpdateForceUsed(ctx: &mut UiContext, menus: &mut MenuSystem) {
    // Currently we don't make a distinction between those that wish to play Jedi of lower than maximum skill.
    ctx.world.force.uiForceRank = ctx.world.force.uiMaxRank;

    ctx.world.force.uiForceUsed = 0;
    ctx.world.force.uiForceAvailable = forceMasteryPoints[ctx.world.force.uiForceRank as usize];

    // Make sure that we have one freebie in jump.
    if ctx.world.force.uiForcePowersRank[FP_LEVITATION as usize] < 1 {
        ctx.world.force.uiForcePowersRank[FP_LEVITATION as usize] = 1;
    }

    if UI_TrueJediEnabled(ctx) {
        // true jedi mode is set
        if ctx.world.force.uiJediNonJedi == -1 {
            let mut x: c_int = 0;
            let mut clear = false;
            let mut update = false;
            ctx.world.force.uiJediNonJedi = FORCE_NONJEDI;
            while x < NUM_FORCE_POWERS {
                // if any force power is set, we must be a jedi
                if x == FP_LEVITATION || x == FP_SABER_OFFENSE {
                    if ctx.world.force.uiForcePowersRank[x as usize] > 1 {
                        ctx.world.force.uiJediNonJedi = FORCE_JEDI;
                        break;
                    } else if ctx.world.force.uiForcePowersRank[x as usize] > 0 {
                        clear = true;
                    }
                } else if ctx.world.force.uiForcePowersRank[x as usize] > 0 {
                    ctx.world.force.uiJediNonJedi = FORCE_JEDI;
                    break;
                }
                x += 1;
            }
            if ctx.world.force.uiJediNonJedi == FORCE_JEDI {
                if ctx.world.force.uiForcePowersRank[FP_SABER_OFFENSE as usize] < 1 {
                    ctx.world.force.uiForcePowersRank[FP_SABER_OFFENSE as usize] = 1;
                    update = true;
                }
            } else if clear {
                let mut x: c_int = 0;
                while x < NUM_FORCE_POWERS {
                    // clear all force
                    ctx.world.force.uiForcePowersRank[x as usize] = 0;
                    x += 1;
                }
                update = true;
            }
            if update {
                let myTeam = trap::Cvar_VariableValue(ctx.engine, "ui_myteam") as c_int;
                if myTeam != TEAM_SPECTATOR {
                    let team_name = UI_TeamName(myTeam).to_string();
                    // will cause him to respawn, if it's been 5 seconds since last one
                    UI_UpdateClientForcePowers(ctx, &team_name);
                } else {
                    // just update powers
                    UI_UpdateClientForcePowers(ctx, "");
                }
            }
        }
    }

    let menu = Menus_FindByName(menus, "ingame_playerforce");
    // Set the cost of the saberattack according to whether its free.
    if ctx.world.cvars.ui_freeSaber.integer != 0 {
        // Make saber free
        // PORT-NOTE: Raven's `bgForcePowerCost` is a per-module mutable global;
        // the runtime copy lives on `BgState` (DEC-36 addendum 11).
        ctx.world.bg_state.bgForcePowerCost[FP_SABER_OFFENSE as usize][FORCE_LEVEL_1 as usize] = 0;
        ctx.world.bg_state.bgForcePowerCost[FP_SABER_DEFENSE as usize][FORCE_LEVEL_1 as usize] = 0;
        // Make sure that we have one freebie in saber if applicable.
        if ctx.world.force.uiForcePowersRank[FP_SABER_OFFENSE as usize] < 1 {
            ctx.world.force.uiForcePowersRank[FP_SABER_OFFENSE as usize] = 1;
        }
        if ctx.world.force.uiForcePowersRank[FP_SABER_DEFENSE as usize] < 1 {
            ctx.world.force.uiForcePowersRank[FP_SABER_DEFENSE as usize] = 1;
        }
        if let Some(menu) = menu {
            Menu_ShowItemByName(menus, ctx, menu, "setFP_SABER_DEFENSE", true);
            Menu_ShowItemByName(menus, ctx, menu, "setfp_saberthrow", true);
            Menu_ShowItemByName(menus, ctx, menu, "effectentry", true);
            Menu_ShowItemByName(menus, ctx, menu, "effectfield", true);
            Menu_ShowItemByName(menus, ctx, menu, "nosaber", false);
        }
    } else {
        // Make saber normal cost
        ctx.world.bg_state.bgForcePowerCost[FP_SABER_OFFENSE as usize][FORCE_LEVEL_1 as usize] = 1;
        ctx.world.bg_state.bgForcePowerCost[FP_SABER_DEFENSE as usize][FORCE_LEVEL_1 as usize] = 1;
        // Also, check if there is no saberattack.  If there isn't, there had better not be any defense or throw!
        if ctx.world.force.uiForcePowersRank[FP_SABER_OFFENSE as usize] < 1 {
            ctx.world.force.uiForcePowersRank[FP_SABER_DEFENSE as usize] = 0;
            ctx.world.force.uiForcePowersRank[FP_SABERTHROW as usize] = 0;
            if let Some(menu) = menu {
                Menu_ShowItemByName(menus, ctx, menu, "setfp_saberdefend", false);
                Menu_ShowItemByName(menus, ctx, menu, "setfp_saberthrow", false);
                Menu_ShowItemByName(menus, ctx, menu, "effectentry", false);
                Menu_ShowItemByName(menus, ctx, menu, "effectfield", false);
                Menu_ShowItemByName(menus, ctx, menu, "nosaber", true);
            }
        } else if let Some(menu) = menu {
            Menu_ShowItemByName(menus, ctx, menu, "setfp_saberdefend", true);
            Menu_ShowItemByName(menus, ctx, menu, "setfp_saberthrow", true);
            Menu_ShowItemByName(menus, ctx, menu, "effectentry", true);
            Menu_ShowItemByName(menus, ctx, menu, "effectfield", true);
            Menu_ShowItemByName(menus, ctx, menu, "nosaber", false);
        }
    }

    // Make sure that we're still legal.
    for curpower in 0..NUM_FORCE_POWERS {
        // Make sure that our ranks are within legal limits.
        if ctx.world.force.uiForcePowersRank[curpower as usize] < 0 {
            ctx.world.force.uiForcePowersRank[curpower as usize] = 0;
        } else if ctx.world.force.uiForcePowersRank[curpower as usize] >= NUM_FORCE_POWER_LEVELS {
            ctx.world.force.uiForcePowersRank[curpower as usize] = NUM_FORCE_POWER_LEVELS - 1;
        }

        let mut currank = FORCE_LEVEL_1;
        while currank <= ctx.world.force.uiForcePowersRank[curpower as usize] {
            // Check on this force power
            if ctx.world.force.uiForcePowersRank[curpower as usize] > 0 {
                // Do not charge the player for the one freebie in jump, or if there is one in saber.
                if (curpower == FP_LEVITATION && currank == FORCE_LEVEL_1)
                    || (curpower == FP_SABER_OFFENSE
                        && currank == FORCE_LEVEL_1
                        && ctx.world.cvars.ui_freeSaber.integer != 0)
                    || (curpower == FP_SABER_DEFENSE
                        && currank == FORCE_LEVEL_1
                        && ctx.world.cvars.ui_freeSaber.integer != 0)
                {
                    // Do nothing (written this way for clarity)
                } else {
                    // Check if we can accrue the cost of this power.
                    let cost =
                        ctx.world.bg_state.bgForcePowerCost[curpower as usize][currank as usize];
                    if cost > ctx.world.force.uiForceAvailable {
                        // We can't afford this power.  Break to the next one.
                        // Remove this power from the player's roster.
                        ctx.world.force.uiForcePowersRank[curpower as usize] = currank - 1;
                        break;
                    } else {
                        // Sure we can afford it.
                        ctx.world.force.uiForceUsed += cost;
                        ctx.world.force.uiForceAvailable -= cost;
                    }
                }
            }
            currank += 1;
        }
    }
}

/// Raven `UI_ReadLegalForce` — packs the current force allocation into the
/// `"rank-side-<per-power digits>"` wire format, legalizes it through
/// `BG_LegalizedForcePowers`, then unpacks the (possibly-corrected) string
/// back into the UI's force state.
///
/// Source: `oracle/codemp/ui/ui_force.c:465-629`
pub fn UI_ReadLegalForce(ctx: &mut UiContext, menus: &mut MenuSystem) {
    // First, stick them into a string.
    let mut fcfString = format!(
        "{}-{}-",
        ctx.world.force.uiForceRank, ctx.world.force.uiForceSide
    );
    // PORT-NOTE: Raven takes only the first character of the formatted rank
    // digit ("Just use the force digit even if multiple digits. Shouldn't be
    // longer than 1.") — mirror that literally rather than writing the whole
    // formatted number.
    for rank in ctx.world.force.uiForcePowersRank {
        if let Some(digit) = format!("{}", rank).chars().next() {
            fcfString.push(digit);
        }
    }
    fcfString.push('\n');

    let info = trap::GetConfigString(ctx.engine, CS_SERVERINFO, MAX_INFO_VALUE).unwrap_or_default();

    let mut forceTeam: c_int = 0;
    if atoi(&Info_ValueForKey(&info, "g_forceBasedTeams")) != 0 {
        let myTeam = trap::Cvar_VariableValue(ctx.engine, "ui_myteam") as c_int;
        if myTeam == TEAM_RED {
            forceTeam = FORCE_DARKSIDE;
        } else if myTeam == TEAM_BLUE {
            forceTeam = FORCE_LIGHTSIDE;
        }
    }

    let gametype = atoi(&Info_ValueForKey(&info, "g_gametype"));

    // Second, legalize them.
    let mut updateForceLater = false;
    let legal = BG_LegalizedForcePowers(
        &ctx.world.bg_state,
        &mut fcfString,
        ctx.world.force.uiMaxRank,
        ctx.world.cvars.ui_freeSaber.integer,
        forceTeam,
        gametype,
        0,
    );
    if legal == 0 {
        // if they were illegal, we should refresh them.
        updateForceLater = true;
    }

    // Lastly, put them back into the UI storage from the legalized string.
    let chars: Vec<char> = fcfString.chars().collect();
    let mut i = 0usize;
    let mut single = String::new();

    while i < chars.len() && chars[i] != '-' {
        single.push(chars[i]);
        i += 1;
    }
    let iBuf = atoi(&single);
    single.clear();
    i += 1;

    // PORT-NOTE: Raven's over-rank check here is a dead `return` commented
    // out ("FIXME: Print a message indicating this to the user") — the
    // assignment below runs unconditionally regardless of the check.
    let _ = iBuf > ctx.world.force.uiMaxRank || iBuf < 0;

    ctx.world.force.uiForceRank = iBuf;

    while i < chars.len() && chars[i] != '-' {
        single.push(chars[i]);
        i += 1;
    }
    ctx.world.force.uiForceSide = atoi(&single);
    single.clear();
    i += 1;

    if ctx.world.force.uiForceSide != FORCE_LIGHTSIDE
        && ctx.world.force.uiForceSide != FORCE_DARKSIDE
    {
        ctx.world.force.uiForceSide = FORCE_LIGHTSIDE;
        return;
    }

    // clear out the existing powers
    for c in 0..NUM_FORCE_POWERS as usize {
        ctx.world.force.uiForcePowersRank[c] = 0;
    }
    ctx.world.force.uiForceUsed = 0;
    ctx.world.force.uiForceAvailable = forceMasteryPoints[ctx.world.force.uiForceRank as usize];
    ctx.world.force.gTouchedForce = true;

    let mut c = 0usize;
    while i < chars.len() && c < NUM_FORCE_POWERS as usize {
        let mut iBuf = atoi(&chars[i].to_string());

        if iBuf < 0 {
            iBuf = 0;
        }

        let forcePowerRank = iBuf;

        if forcePowerRank > FORCE_LEVEL_3 || forcePowerRank < 0 {
            // err.. not correct
            c += 1;
            i += 1;
            continue; // skip this power
        }

        if ctx.world.force.uiForcePowerDarkLight[c] != 0
            && ctx.world.force.uiForcePowerDarkLight[c] != ctx.world.force.uiForceSide
        {
            // Apparently the user has crafted a force config that has powers
            // that don't fit with the config's side.
            c += 1;
            i += 1;
            continue; // skip this power
        }

        // Accrue cost for each assigned rank for this power.
        let mut currank = FORCE_LEVEL_1;
        while currank <= forcePowerRank {
            let cost = ctx.world.bg_state.bgForcePowerCost[c][currank as usize];
            if cost > ctx.world.force.uiForceAvailable {
                // Break out, we can't afford any more power.
                break;
            }
            // Pay for this rank of this power.
            ctx.world.force.uiForceUsed += cost;
            ctx.world.force.uiForceAvailable -= cost;
            ctx.world.force.uiForcePowersRank[c] += 1;
            currank += 1;
        }

        c += 1;
        i += 1;
    }

    if ctx.world.force.uiForcePowersRank[FP_LEVITATION as usize] < 1 {
        ctx.world.force.uiForcePowersRank[FP_LEVITATION as usize] = 1;
    }
    if ctx.world.force.uiForcePowersRank[FP_SABER_OFFENSE as usize] < 1
        && ctx.world.cvars.ui_freeSaber.integer != 0
    {
        ctx.world.force.uiForcePowersRank[FP_SABER_OFFENSE as usize] = 1;
    }
    if ctx.world.force.uiForcePowersRank[FP_SABER_DEFENSE as usize] < 1
        && ctx.world.cvars.ui_freeSaber.integer != 0
    {
        ctx.world.force.uiForcePowersRank[FP_SABER_DEFENSE as usize] = 1;
    }

    UpdateForceUsed(ctx, menus);

    if updateForceLater {
        ctx.world.force.gTouchedForce = true;
        UI_UpdateClientForcePowers(ctx, "");
    }
}

/// Raven `UI_UpdateForcePowers` — parses the `"forcepowers"` cvar
/// (`"rank-side-<per-power digits>"`) back into the UI's force state,
/// falling back to a default all-zero (level-1 jump/saber) allocation if the
/// cvar is empty or malformed.
///
/// Source: `oracle/codemp/ui/ui_force.c:631-758`
pub fn UI_UpdateForcePowers(ctx: &mut UiContext, menus: &mut MenuSystem) {
    let forcePowers = UI_Cvar_VariableString(ctx, "forcepowers");
    let chars: Vec<char> = forcePowers.chars().collect();
    let mut i = 0usize;
    let mut i_f: usize;

    ctx.world.force.uiForceSide = 0;

    if !forcePowers.is_empty() {
        'outer: while i < chars.len() {
            let mut readBuf = String::new();
            let mut i_r: usize = 0;

            while i < chars.len() && chars[i] != '-' && i_r < 255 {
                readBuf.push(chars[i]);
                i_r += 1;
                i += 1;
            }
            if i_r >= 255 || i >= chars.len() || chars[i] != '-' {
                ctx.world.force.uiForceSide = 0;
                break 'outer;
            }
            ctx.world.force.uiForceRank = atoi(&readBuf);

            if ctx.world.force.uiForceRank > ctx.world.force.uiMaxRank {
                ctx.world.force.uiForceRank = ctx.world.force.uiMaxRank;
            }

            i += 1;

            readBuf.clear();
            i_r = 0;
            while i < chars.len() && chars[i] != '-' && i_r < 255 {
                readBuf.push(chars[i]);
                i_r += 1;
                i += 1;
            }
            if i_r >= 255 || i >= chars.len() || chars[i] != '-' {
                ctx.world.force.uiForceSide = 0;
                break 'outer;
            }
            ctx.world.force.uiForceSide = atoi(&readBuf);

            i += 1;

            i_f = FP_HEAL as usize;

            while i < chars.len() && i_f < NUM_FORCE_POWERS as usize {
                let digit = chars[i].to_string();
                ctx.world.force.uiForcePowersRank[i_f] = atoi(&digit);

                if i_f == FP_LEVITATION as usize && ctx.world.force.uiForcePowersRank[i_f] < 1 {
                    ctx.world.force.uiForcePowersRank[i_f] = 1;
                }

                if i_f == FP_SABER_OFFENSE as usize
                    && ctx.world.force.uiForcePowersRank[i_f] < 1
                    && ctx.world.cvars.ui_freeSaber.integer != 0
                {
                    ctx.world.force.uiForcePowersRank[i_f] = 1;
                }

                if i_f == FP_SABER_DEFENSE as usize
                    && ctx.world.force.uiForcePowersRank[i_f] < 1
                    && ctx.world.cvars.ui_freeSaber.integer != 0
                {
                    ctx.world.force.uiForcePowersRank[i_f] = 1;
                }

                i_f += 1;
                i += 1;
            }

            if i_f < NUM_FORCE_POWERS as usize {
                // info for all the powers wasn't there..
                ctx.world.force.uiForceSide = 0;
                break 'outer;
            }
            i += 1;
        }
    }

    // validitycheck:
    if ctx.world.force.uiForceSide == 0 {
        ctx.world.force.uiForceSide = 1;
        ctx.world.force.uiForceRank = 1;
        for i in 0..NUM_FORCE_POWERS as usize {
            if i == FP_LEVITATION as usize {
                ctx.world.force.uiForcePowersRank[i] = 1;
            } else if i == FP_SABER_OFFENSE as usize && ctx.world.cvars.ui_freeSaber.integer != 0 {
                ctx.world.force.uiForcePowersRank[i] = 1;
            } else if i == FP_SABER_DEFENSE as usize && ctx.world.cvars.ui_freeSaber.integer != 0 {
                ctx.world.force.uiForcePowersRank[i] = 1;
            } else {
                ctx.world.force.uiForcePowersRank[i] = 0;
            }
        }

        UI_UpdateClientForcePowers(ctx, "");
    }

    UpdateForceUsed(ctx, menus);
}

/// Raven `UI_ForceSide_HandleKey` — cycles the light/dark side selector
/// (blocked when force-based teams put the player on a fixed side), resets
/// any power ranks that don't fit the newly-chosen side.
///
/// Source: `oracle/codemp/ui/ui_force.c:802-868`
#[allow(clippy::too_many_arguments)]
pub fn UI_ForceSide_HandleKey(
    ctx: &mut UiContext,
    menus: &mut MenuSystem,
    ds: &DisplayState,
    _flags: c_int,
    _special: Option<&mut f32>,
    key: c_int,
    num: c_int,
    min: c_int,
    max: c_int,
    _type_: c_int,
) -> bool {
    let info = trap::GetConfigString(ctx.engine, CS_SERVERINFO, MAX_INFO_VALUE).unwrap_or_default();

    if atoi(&Info_ValueForKey(&info, "g_forceBasedTeams")) != 0 {
        let myTeam = trap::Cvar_VariableValue(ctx.engine, "ui_myteam") as c_int;
        if myTeam == TEAM_RED {
            return false;
        }
        if myTeam == TEAM_BLUE {
            return false;
        }
    }

    if key == fakeAscii_t::A_MOUSE1 as c_int
        || key == fakeAscii_t::A_MOUSE2 as c_int
        || key == fakeAscii_t::A_ENTER as c_int
        || key == fakeAscii_t::A_KP_ENTER as c_int
    {
        let mut i = num;

        // update the feeder item selection, it might be different depending
        // on side
        Menu_SetFeederSelection(menus, ds, ctx, None, FEEDER_FORCECFG, 0, None);

        if key == fakeAscii_t::A_MOUSE2 as c_int {
            i -= 1;
        } else {
            i += 1;
        }

        if i < min {
            i = max;
        } else if i > max {
            i = min;
        }

        let num = i;

        ctx.world.force.uiForceSide = num;

        // Resetting power ranks based on if light or dark side is chosen
        let mut x = 0usize;
        while x < NUM_FORCE_POWERS as usize {
            if ctx.world.force.uiForcePowerDarkLight[x] != 0
                && ctx.world.force.uiForceSide != ctx.world.force.uiForcePowerDarkLight[x]
            {
                ctx.world.force.uiForcePowersRank[x] = 0;
            }
            x += 1;
        }

        UpdateForceUsed(ctx, menus);

        ctx.world.force.gTouchedForce = true;
        return true;
    }
    false
}

/// Raven `UI_JediNonJedi_HandleKey` — cycles the jedi/non-jedi selector
/// (true-jedi mode only): clears all force powers and notifies the server
/// when switching to non-jedi, or seeds the level-1 jump/saber-offense
/// minimums when switching to jedi.
///
/// Source: `oracle/codemp/ui/ui_force.c:870-945`
#[allow(clippy::too_many_arguments)]
pub fn UI_JediNonJedi_HandleKey(
    ctx: &mut UiContext,
    menus: &mut MenuSystem,
    _flags: c_int,
    _special: Option<&mut f32>,
    key: c_int,
    num: c_int,
    min: c_int,
    max: c_int,
    _type_: c_int,
) -> bool {
    // Raven reads `info` via `trap_GetConfigString` here but never consults
    // it in this function.
    let _info = trap::GetConfigString(ctx.engine, CS_SERVERINFO, MAX_INFO_VALUE);

    if !UI_TrueJediEnabled(ctx) {
        // true jedi mode is not set
        return false;
    }

    if key == fakeAscii_t::A_MOUSE1 as c_int
        || key == fakeAscii_t::A_MOUSE2 as c_int
        || key == fakeAscii_t::A_ENTER as c_int
        || key == fakeAscii_t::A_KP_ENTER as c_int
    {
        let mut i = num;

        if key == fakeAscii_t::A_MOUSE2 as c_int {
            i -= 1;
        } else {
            i += 1;
        }

        if i < min {
            i = max;
        } else if i > max {
            i = min;
        }

        let num = i;

        ctx.world.force.uiJediNonJedi = num;

        // Resetting power ranks based on if light or dark side is chosen
        if num == 0 {
            // not a jedi?
            let myTeam = trap::Cvar_VariableValue(ctx.engine, "ui_myteam") as c_int;
            let mut x = 0usize;
            while x < NUM_FORCE_POWERS as usize {
                // clear all force powers
                ctx.world.force.uiForcePowersRank[x] = 0;
                x += 1;
            }
            if myTeam != TEAM_SPECTATOR {
                // will cause him to respawn, if it's been 5 seconds since
                // last one
                let team_name = UI_TeamName(myTeam).to_string();
                UI_UpdateClientForcePowers(ctx, &team_name);
            } else {
                // just update powers
                UI_UpdateClientForcePowers(ctx, "");
            }
        } else {
            // a jedi, set the minimums, hopefuly they know to set the rest!
            if ctx.world.force.uiForcePowersRank[FP_LEVITATION as usize] < FORCE_LEVEL_1 {
                // force jump 1 minimum
                ctx.world.force.uiForcePowersRank[FP_LEVITATION as usize] = FORCE_LEVEL_1;
            }
            if ctx.world.force.uiForcePowersRank[FP_SABER_OFFENSE as usize] < FORCE_LEVEL_1 {
                // saber attack 1, minimum
                ctx.world.force.uiForcePowersRank[FP_SABER_OFFENSE as usize] = FORCE_LEVEL_1;
            }
        }

        UpdateForceUsed(ctx, menus);

        ctx.world.force.gTouchedForce = true;
        return true;
    }
    false
}

/// Raven `UI_ForceMaxRank_HandleKey` — cycles the max-rank selector and
/// pushes the new value to the `"g_maxForceRank"` cvar.
///
/// Source: `oracle/codemp/ui/ui_force.c:947-985`
#[allow(clippy::too_many_arguments)]
pub fn UI_ForceMaxRank_HandleKey(
    ctx: &mut UiContext,
    menus: &mut MenuSystem,
    _flags: c_int,
    _special: Option<&mut f32>,
    key: c_int,
    num: c_int,
    min: c_int,
    max: c_int,
    _type_: c_int,
) -> bool {
    if key == fakeAscii_t::A_MOUSE1 as c_int
        || key == fakeAscii_t::A_MOUSE2 as c_int
        || key == fakeAscii_t::A_ENTER as c_int
        || key == fakeAscii_t::A_KP_ENTER as c_int
    {
        let mut i = num;

        if key == fakeAscii_t::A_MOUSE2 as c_int {
            i -= 1;
        } else {
            i += 1;
        }

        if i < min {
            i = max;
        } else if i > max {
            i = min;
        }

        let num = i;

        ctx.world.force.uiMaxRank = num;

        trap::Cvar_Set(ctx.engine, "g_maxForceRank", &format!("{}", num));

        // The update force used will remove overallocated powers automatically.
        UpdateForceUsed(ctx, menus);

        ctx.world.force.gTouchedForce = true;

        return true;
    }
    false
}

/// Raven `UI_ForcePowerRank_HandleKey` — raises or lowers one force power's
/// rank by a point, gated on server-side disable, side mismatch, the
/// saber-offense prerequisite for defend/throw, and the current point
/// budget.
///
/// Source: `oracle/codemp/ui/ui_force.c:989-1078`
#[allow(clippy::too_many_arguments)]
pub fn UI_ForcePowerRank_HandleKey(
    ctx: &mut UiContext,
    menus: &mut MenuSystem,
    _flags: c_int,
    _special: Option<&mut f32>,
    key: c_int,
    _num: c_int,
    min: c_int,
    max: c_int,
    type_: c_int,
) -> bool {
    if key == fakeAscii_t::A_MOUSE1 as c_int
        || key == fakeAscii_t::A_MOUSE2 as c_int
        || key == fakeAscii_t::A_ENTER as c_int
        || key == fakeAscii_t::A_KP_ENTER as c_int
        || key == fakeAscii_t::A_BACKSPACE as c_int
    {
        // this will give us the index as long as UI_FORCE_RANK is always one
        // below the first force rank index
        let forcepower = ((type_ - UI_FORCE_RANK) - 1) as usize;

        // the power is disabled on the server
        if ctx.world.force.uiForcePowersDisabled[forcepower] {
            return true;
        }

        let mut min = min;

        // If we are not on the same side as a power, or if we are not of
        // any rank at all.
        if ctx.world.force.uiForcePowerDarkLight[forcepower] != 0
            && ctx.world.force.uiForceSide != ctx.world.force.uiForcePowerDarkLight[forcepower]
        {
            return true;
        } else if forcepower == FP_SABER_DEFENSE as usize || forcepower == FP_SABERTHROW as usize {
            // Saberdefend and saberthrow can't be bought if there is no
            // saberattack
            if ctx.world.force.uiForcePowersRank[FP_SABER_OFFENSE as usize] < 1 {
                return true;
            }
        }

        if type_ == UI_FORCE_RANK_LEVITATION {
            min += 1;
        }
        if type_ == UI_FORCE_RANK_SABERATTACK && ctx.world.cvars.ui_freeSaber.integer != 0 {
            min += 1;
        }
        if type_ == UI_FORCE_RANK_SABERDEFEND && ctx.world.cvars.ui_freeSaber.integer != 0 {
            min += 1;
        }

        let raising;
        if key == fakeAscii_t::A_MOUSE2 as c_int || key == fakeAscii_t::A_BACKSPACE as c_int {
            // Lower a point.
            if ctx.world.force.uiForcePowersRank[forcepower] <= min {
                return true;
            }
            raising = false;
        } else {
            // Raise a point.
            if ctx.world.force.uiForcePowersRank[forcepower] >= max {
                return true;
            }
            raising = true;
        }

        if raising {
            // Check if we can accrue the cost of this power.
            let rank = ctx.world.force.uiForcePowersRank[forcepower] + 1;
            let cost = ctx.world.bg_state.bgForcePowerCost[forcepower][rank as usize];
            if cost > ctx.world.force.uiForceAvailable {
                // We can't afford this power. Abandon ship.
                return true;
            } else {
                // Sure we can afford it.
                ctx.world.force.uiForceUsed += cost;
                ctx.world.force.uiForceAvailable -= cost;
                ctx.world.force.uiForcePowersRank[forcepower] = rank;
            }
        } else {
            // Lower the point.
            let rank = ctx.world.force.uiForcePowersRank[forcepower];
            let cost = ctx.world.bg_state.bgForcePowerCost[forcepower][rank as usize];
            ctx.world.force.uiForceUsed -= cost;
            ctx.world.force.uiForceAvailable += cost;
            ctx.world.force.uiForcePowersRank[forcepower] -= 1;
        }

        UpdateForceUsed(ctx, menus);

        ctx.world.force.gTouchedForce = true;

        return true;
    }
    false
}

/// Raven `UI_ForceConfigHandle` — swaps the active force-config selection:
/// stashes/restores the custom slot at index 0, or loads a `.fcf` template
/// (light/dark, with a same-side-first-then-opposite fallback) and unpacks
/// it into the UI's force state via the same legalize-then-parse pipeline as
/// `UI_ReadLegalForce`.
///
/// Source: `oracle/codemp/ui/ui_force.c:1110-1345`
pub fn UI_ForceConfigHandle(
    ctx: &mut UiContext,
    menus: &mut MenuSystem,
    oldindex: c_int,
    newindex: c_int,
) {
    if oldindex == 0 {
        // switching out from custom config, so first shove the current
        // values into the custom storage
        for i in 0..NUM_FORCE_POWERS as usize {
            ctx.world.force.gCustPowersRank[i] = ctx.world.force.uiForcePowersRank[i];
        }
        ctx.world.force.gCustRank = ctx.world.force.uiForceRank;
        ctx.world.force.gCustSide = ctx.world.force.uiForceSide;
    }

    if newindex == 0 {
        // switching back to custom, shove the values back in from the
        // custom storage
        ctx.world.force.uiForceUsed = 0;
        ctx.world.force.gTouchedForce = true;

        for i in 0..NUM_FORCE_POWERS as usize {
            ctx.world.force.uiForcePowersRank[i] = ctx.world.force.gCustPowersRank[i];
            ctx.world.force.uiForceUsed += ctx.world.force.uiForcePowersRank[i];
        }
        ctx.world.force.uiForceRank = ctx.world.force.gCustRank;
        ctx.world.force.uiForceSide = ctx.world.force.gCustSide;

        UpdateForceUsed(ctx, menus);
        return;
    }

    // If we made it here, we want to load in a new config
    let mut newindex = newindex;
    let mut f: fileHandle_t = 0;
    let mut len: c_int;

    if ctx.world.force.uiForceSide == FORCE_LIGHTSIDE {
        // we should only be displaying lightside configs, so.. look in the
        // light folder
        newindex += ctx.world.forceConfigLightIndexBegin;
        if newindex as usize >= ctx.world.forceConfigNames.len() {
            return;
        }
        len = trap::FS_FOpenFile(
            ctx.engine,
            &format!(
                "forcecfg/light/{}.fcf",
                ctx.world.forceConfigNames[newindex as usize]
            ),
            &mut f,
            FS_READ,
        );
    } else {
        // else dark
        newindex += ctx.world.forceConfigDarkIndexBegin;
        if newindex as usize >= ctx.world.forceConfigNames.len()
            || newindex > ctx.world.forceConfigLightIndexBegin
        {
            // dark gets read in before light
            return;
        }
        len = trap::FS_FOpenFile(
            ctx.engine,
            &format!(
                "forcecfg/dark/{}.fcf",
                ctx.world.forceConfigNames[newindex as usize]
            ),
            &mut f,
            FS_READ,
        );
    }

    if len <= 0 {
        // This should not have happened. But, before we quit out, attempt
        // searching the other light/dark folder for the file.
        if ctx.world.force.uiForceSide == FORCE_LIGHTSIDE {
            len = trap::FS_FOpenFile(
                ctx.engine,
                &format!(
                    "forcecfg/dark/{}.fcf",
                    ctx.world.forceConfigNames[newindex as usize]
                ),
                &mut f,
                FS_READ,
            );
        } else {
            len = trap::FS_FOpenFile(
                ctx.engine,
                &format!(
                    "forcecfg/light/{}.fcf",
                    ctx.world.forceConfigNames[newindex as usize]
                ),
                &mut f,
                FS_READ,
            );
        }

        if len <= 0 {
            // still failure? Oh well.
            return;
        }
    }

    if len >= 8192 {
        return;
    }

    let mut raw = vec![0u8; len as usize];
    trap::FS_Read(ctx.engine, &mut raw, f);
    trap::FS_FCloseFile(ctx.engine, f);
    let mut fcfBuffer = latin1_to_string(&raw);
    // Raven's `fcfBuffer[len] = 0` leaves a C string: the walks below stop at
    // the first embedded NUL. Source: `oracle/codemp/ui/ui_force.c:1195-1197`
    if let Some(p) = fcfBuffer.find('\0') {
        fcfBuffer.truncate(p);
    }

    let info = trap::GetConfigString(ctx.engine, CS_SERVERINFO, MAX_INFO_VALUE).unwrap_or_default();

    let mut forceTeam: c_int = 0;
    if atoi(&Info_ValueForKey(&info, "g_forceBasedTeams")) != 0 {
        let myTeam = trap::Cvar_VariableValue(ctx.engine, "ui_myteam") as c_int;
        if myTeam == TEAM_RED {
            forceTeam = FORCE_DARKSIDE;
        } else if myTeam == TEAM_BLUE {
            forceTeam = FORCE_LIGHTSIDE;
        }
    }

    let gametype = atoi(&Info_ValueForKey(&info, "g_gametype"));

    // legalize the config based on the max rank
    BG_LegalizedForcePowers(
        &ctx.world.bg_state,
        &mut fcfBuffer,
        ctx.world.force.uiMaxRank,
        ctx.world.cvars.ui_freeSaber.integer,
        forceTeam,
        gametype,
        0,
    );

    // now that we're done with the handle, it's time to parse our force
    // data out of the string. we store strings in rank-side-xxxxxxxxx
    // format (where the x's are individual force power levels)
    let chars: Vec<char> = fcfBuffer.chars().collect();
    let mut i = 0usize;
    let mut single = String::new();

    while i < chars.len() && chars[i] != '-' {
        single.push(chars[i]);
        i += 1;
    }
    let iBuf = atoi(&single);
    single.clear();
    i += 1;

    if iBuf > ctx.world.force.uiMaxRank || iBuf < 0 {
        // this force config uses a rank level higher than our currently
        // restricted level.. so we can't use it
        // FIXME: Print a message indicating this to the user
        return;
    }

    ctx.world.force.uiForceRank = iBuf;

    while i < chars.len() && chars[i] != '-' {
        single.push(chars[i]);
        i += 1;
    }
    ctx.world.force.uiForceSide = atoi(&single);
    single.clear();
    i += 1;

    if ctx.world.force.uiForceSide != FORCE_LIGHTSIDE
        && ctx.world.force.uiForceSide != FORCE_DARKSIDE
    {
        ctx.world.force.uiForceSide = FORCE_LIGHTSIDE;
        return;
    }

    // clear out the existing powers
    // rww - don't need to do the light/dark freebie checks here. Just trust
    // whatever the saber config says.
    for c in 0..NUM_FORCE_POWERS as usize {
        ctx.world.force.uiForcePowersRank[c] = 0;
    }
    ctx.world.force.uiForceUsed = 0;
    ctx.world.force.uiForceAvailable = forceMasteryPoints[ctx.world.force.uiForceRank as usize];
    ctx.world.force.gTouchedForce = true;

    let mut c = 0usize;
    while i < chars.len() && c < NUM_FORCE_POWERS as usize {
        let mut iBuf = atoi(&chars[i].to_string());

        if iBuf < 0 {
            iBuf = 0;
        }

        let forcePowerRank = iBuf;

        if forcePowerRank > FORCE_LEVEL_3 || forcePowerRank < 0 {
            // err.. not correct
            c += 1;
            i += 1;
            continue; // skip this power
        }

        if ctx.world.force.uiForcePowerDarkLight[c] != 0
            && ctx.world.force.uiForcePowerDarkLight[c] != ctx.world.force.uiForceSide
        {
            // Apparently the user has crafted a force config that has
            // powers that don't fit with the config's side.
            c += 1;
            i += 1;
            continue; // skip this power
        }

        // Accrue cost for each assigned rank for this power.
        let mut currank = FORCE_LEVEL_1;
        while currank <= forcePowerRank {
            let cost = ctx.world.bg_state.bgForcePowerCost[c][currank as usize];
            if cost > ctx.world.force.uiForceAvailable {
                // Break out, we can't afford any more power.
                break;
            }
            // Pay for this rank of this power.
            ctx.world.force.uiForceUsed += cost;
            ctx.world.force.uiForceAvailable -= cost;
            ctx.world.force.uiForcePowersRank[c] += 1;
            currank += 1;
        }

        c += 1;
        i += 1;
    }

    if ctx.world.force.uiForcePowersRank[FP_LEVITATION as usize] < 1 {
        ctx.world.force.uiForcePowersRank[FP_LEVITATION as usize] = 1;
    }
    if ctx.world.force.uiForcePowersRank[FP_SABER_OFFENSE as usize] < 1
        && ctx.world.cvars.ui_freeSaber.integer != 0
    {
        ctx.world.force.uiForcePowersRank[FP_SABER_OFFENSE as usize] = 1;
    }
    if ctx.world.force.uiForcePowersRank[FP_SABER_DEFENSE as usize] < 1
        && ctx.world.cvars.ui_freeSaber.integer != 0
    {
        ctx.world.force.uiForcePowersRank[FP_SABER_DEFENSE as usize] = 1;
    }

    // PORT-NOTE: `UpdateForceUsed` calls `DC->` menu-item toggles, so this fn
    // threads `menus` even though the packet's C signature
    // (`void UI_ForceConfigHandle(int, int)`) shows no such param — the
    // resolved-signature index does not carry ui_force.c's own fns, only its
    // in-module callees, so the thread is derived from `UpdateForceUsed`'s
    // already-ported shape. `ctx` doubles as the `dc` (DEC-38 ruling 1).
    UpdateForceUsed(ctx, menus);
}

/// Raven `UI_SkinColor_HandleKey` — cycles the skin-color selector on
/// confirm keys, then re-selects the feeder entry for the current head.
///
/// Source: `oracle/codemp/ui/ui_force.c:762-797`
#[allow(clippy::too_many_arguments)]
pub fn UI_SkinColor_HandleKey(
    ctx: &mut UiContext,
    menus: &mut MenuSystem,
    ds: &DisplayState,
    _flags: c_int,
    _special: Option<&mut f32>,
    key: c_int,
    num: c_int,
    min: c_int,
    max: c_int,
    _type_: c_int,
) -> bool {
    if key == fakeAscii_t::A_MOUSE1 as c_int
        || key == fakeAscii_t::A_MOUSE2 as c_int
        || key == fakeAscii_t::A_ENTER as c_int
        || key == fakeAscii_t::A_KP_ENTER as c_int
    {
        let mut i = num;

        if key == fakeAscii_t::A_MOUSE2 as c_int {
            i -= 1;
        } else {
            i += 1;
        }

        if i < min {
            i = max;
        } else if i > max {
            i = min;
        }

        let num = i;

        ctx.world.main.uiSkinColor = num;

        ctx.world.main.uiHoldSkinColor = ctx.world.main.uiSkinColor;

        let q3SelectedHead = ctx.world.q3SelectedHead;
        UI_FeederSelection(ctx, menus, ds, FEEDER_Q3HEADS as f32, q3SelectedHead, None);

        return true;
    }
    false
}
