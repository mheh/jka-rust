//! Port of `oracle/codemp/cgame/cg_info.c` — the loading screen and connection info display. Functions land via the C5
//! transcription waves.

#![allow(non_snake_case)]

use native_string::Q_strncpyz;

use mp_qshared::shared::MAX_STRING_CHARS;

use crate::trap;
use crate::world::cg_context::CgContext;

/// Raven `#define MAX_LOADING_PLAYER_ICONS 16`.
/// Source: `oracle/codemp/cgame/cg_info.c:7`
pub const MAX_LOADING_PLAYER_ICONS: usize = 16;

/// Raven `#define MAX_LOADING_ITEM_ICONS 26`.
/// Source: `oracle/codemp/cgame/cg_info.c:8`
pub const MAX_LOADING_ITEM_ICONS: usize = 26;

// DEFERRED: UI_INFOFONT — oracle/codemp/cgame/cg_info.c:109
// `#define UI_INFOFONT (UI_BIGFONT)` resolves to q_shared.h's `UI_BIGFONT`,
// whose numeric value is not in this packet's FILE-SCOPE CONSTANTS and has no
// existing Rust binding in mp_qshared/mp_uishared to alias; not needed by
// CG_LoadingString (the only fn this wave opens), so left unported rather
// than guessed.

/// Raven `CG_LoadingString` — copies the loading-screen status text into
/// `cg.infoScreenText` and forces the frame to draw immediately so the
/// player sees load progress instead of a frozen screen.
///
/// Source: `oracle/codemp/cgame/cg_info.c:21-25`
pub fn CG_LoadingString(ctx: &mut CgContext, s: &str) {
    Q_strncpyz(&mut ctx.world.cg.infoScreenText, s, MAX_STRING_CHARS);

    trap::UpdateScreen(ctx.engine);
}
