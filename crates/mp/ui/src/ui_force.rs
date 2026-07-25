//! `ui_force.c` — the force-allocation screen.
//!
//! Source: `oracle/codemp/ui/ui_force.c`

#![allow(non_snake_case)]

use core::ffi::c_int;

use mp_bg::bg_misc::forceMasteryPoints;
use mp_bg::public::team::TEAM_SPECTATOR;
use mp_qshared::common::mp::qcommon::saber::saber_colors::{
    SABER_BLUE, SABER_GREEN, SABER_ORANGE, SABER_PURPLE, SABER_RED, SABER_YELLOW,
};
use mp_qshared::shared::cbuf_exec::cbufExec_t;
use mp_qshared::shared::force_powers::{
    FORCE_DARKSIDE, FORCE_LEVEL_1, FORCE_LIGHTSIDE, FP_LEVITATION, FP_SABERTHROW, FP_SABER_DEFENSE,
    FP_SABER_OFFENSE, NUM_FORCE_POWERS, NUM_FORCE_POWER_LEVELS,
};
use mp_qshared::shared::vec4_t;
use mp_qshared::shared::FS_WRITE;
use mp_uishared::shared::display_context::DisplayContext;
use mp_uishared::shared::menudef::FEEDER_FORCECFG;
use mp_uishared::shared::rect_def_t::RectDef;
use mp_uishared::ui_shared::{Menu_SetFeederSelection, Menu_ShowItemByName, Menus_FindByName};
use native_string::Q_stricmp;
use native_types::fileHandle_t;

use crate::trap;
use crate::world::ui_context::UiContext;
use crate::world::ui_world::UiWorld;

use super::ui_atoms::{Com_Printf, UI_Cvar_VariableString, UI_DrawHandlePic};
use super::ui_main::{UI_LoadForceConfig_List, UI_TeamName, UI_TrueJediEnabled};

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
pub fn UI_SaveForceTemplate(ctx: &mut UiContext, dc: &mut dyn DisplayContext) {
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
            Menu_SetFeederSelection(
                &mut ctx.world.menus,
                dc,
                None,
                FEEDER_FORCECFG,
                translated,
                None,
            );
            foundFeederItem = true;
        }
    }

    // Else, go back to 0
    if !foundFeederItem {
        Menu_SetFeederSelection(&mut ctx.world.menus, dc, None, FEEDER_FORCECFG, 0, None);
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
pub fn UpdateForceUsed(ctx: &mut UiContext, dc: &mut dyn DisplayContext) {
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

    let menu = Menus_FindByName(&ctx.world.menus, "ingame_playerforce");
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
            Menu_ShowItemByName(&mut ctx.world.menus, dc, menu, "setFP_SABER_DEFENSE", true);
            Menu_ShowItemByName(&mut ctx.world.menus, dc, menu, "setfp_saberthrow", true);
            Menu_ShowItemByName(&mut ctx.world.menus, dc, menu, "effectentry", true);
            Menu_ShowItemByName(&mut ctx.world.menus, dc, menu, "effectfield", true);
            Menu_ShowItemByName(&mut ctx.world.menus, dc, menu, "nosaber", false);
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
                Menu_ShowItemByName(&mut ctx.world.menus, dc, menu, "setfp_saberdefend", false);
                Menu_ShowItemByName(&mut ctx.world.menus, dc, menu, "setfp_saberthrow", false);
                Menu_ShowItemByName(&mut ctx.world.menus, dc, menu, "effectentry", false);
                Menu_ShowItemByName(&mut ctx.world.menus, dc, menu, "effectfield", false);
                Menu_ShowItemByName(&mut ctx.world.menus, dc, menu, "nosaber", true);
            }
        } else if let Some(menu) = menu {
            Menu_ShowItemByName(&mut ctx.world.menus, dc, menu, "setfp_saberdefend", true);
            Menu_ShowItemByName(&mut ctx.world.menus, dc, menu, "setfp_saberthrow", true);
            Menu_ShowItemByName(&mut ctx.world.menus, dc, menu, "effectentry", true);
            Menu_ShowItemByName(&mut ctx.world.menus, dc, menu, "effectfield", true);
            Menu_ShowItemByName(&mut ctx.world.menus, dc, menu, "nosaber", false);
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
