//! `ui_gameinfo.c` — arena/bot info loading.
//!
//! Source: `oracle/codemp/ui/ui_gameinfo.c`

#![allow(non_snake_case)]

use core::ffi::c_int;

use native_string::info::Info_ValueForKey;
use native_string::q_string::Q_stricmp;

use crate::trap;
use crate::world::ui_context::UiContext;
use crate::world::ui_world::UiWorld;

/// Raven `UI_GetBotInfoByNumber` — retrieve bot info string by numeric index.
///
/// Source: `oracle/codemp/ui/ui_gameinfo.c:297-303`
pub fn UI_GetBotInfoByNumber<'a>(ctx: &'a mut UiContext, num: c_int) -> Option<&'a str> {
    let num_bots = ctx.world.gameinfo.ui_botInfos.len() as c_int;
    if num < 0 || num >= num_bots {
        trap::Print(ctx.engine, &format!("^1Invalid bot number: {}\n", num));
        return None;
    }
    Some(ctx.world.gameinfo.ui_botInfos[num as usize].as_str())
}

/// Raven `UI_GetBotInfoByName` — retrieve bot info string by name.
///
/// Source: `oracle/codemp/ui/ui_gameinfo.c:311-323`
pub fn UI_GetBotInfoByName<'a>(world: &'a UiWorld, name: &str) -> Option<&'a str> {
    for bot_info in &world.gameinfo.ui_botInfos {
        let value = Info_ValueForKey(bot_info, "name");
        if Q_stricmp(&value, name) == 0 {
            return Some(bot_info.as_str());
        }
    }
    None
}

/// Raven `UI_GetNumBots` — return count of loaded bot info strings.
///
/// Source: `oracle/codemp/ui/ui_gameinfo.c:325-327`
pub fn UI_GetNumBots(world: &UiWorld) -> c_int {
    world.gameinfo.ui_botInfos.len() as c_int
}
