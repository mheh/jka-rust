//! `ui_force.c` — the force-allocation screen.
//!
//! Source: `oracle/codemp/ui/ui_force.c`

#![allow(non_snake_case)]

use core::ffi::c_int;

use mp_bg::bg_misc::bgForcePowerCost;
use mp_qshared::common::mp::qcommon::saber::saber_colors::{
    SABER_BLUE, SABER_GREEN, SABER_ORANGE, SABER_PURPLE, SABER_RED, SABER_YELLOW,
};
use mp_qshared::shared::cbuf_exec::cbufExec_t;
use mp_qshared::shared::force_powers::{FORCE_LEVEL_1, FORCE_LIGHTSIDE};
use mp_qshared::shared::vec4_t;
use mp_uishared::shared::rect_def_t::RectDef;

use crate::trap;
use crate::world::ui_context::UiContext;
use crate::world::ui_world::UiWorld;

use super::ui_atoms::UI_DrawHandlePic;

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
        let star_color = bgForcePowerCost[forceindex as usize][i as usize];

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
