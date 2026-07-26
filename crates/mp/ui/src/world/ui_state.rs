//! `UiState` — the ui module's one owned island, split into the three borrows
//! the `vmMain` shell hands out (DEC-38 ruling 1, revised).

#![allow(non_snake_case)]

use mp_uishared::shared::display_state::DisplayState;
use mp_uishared::shared::menu_system::MenuSystem;

use super::ui_world::UiWorld;

/// Everything the ui module owns across `vmMain` calls, held by the shell's
/// `WORLD` cell (STATE-D6) and split into three disjoint borrows on every
/// dispatch.
///
/// Raven kept all three as file-scope state in one link unit — `uiInfo_t
/// uiInfo` (`ui_local.h:729-843`), its `displayContextDef_t uiDC` member
/// (`ui_local.h:730`), and `ui_shared.c`'s `Menus[]`/`menuStack[]` pool
/// globals (`ui_shared.c:111-115`). DEC-36 D1/D2 folded the latter two INTO
/// [`UiWorld`]; DEC-38 ruling 1 hoists them back OUT, because the ported ui
/// fns must hold `menus`/`uiDC` beside a live [`UiContext`](super::ui_context::UiContext)
/// that itself implements
/// [`DisplayContext`](mp_uishared::shared::display_context::DisplayContext) —
/// three disjoint fields, three disjoint borrows, no aliasing (§B4).
///
/// Source: `docs/decisions.md` DEC-38 (ruling 1, revised)
pub struct UiState {
    /// Raven `uiInfo_t uiInfo` minus the two hoisted members.
    /// Source: `oracle/codemp/ui/ui_local.h:729-843`
    pub world: UiWorld,

    /// The menu framework. Raven: `ui_shared.c`'s file-scope arrays.
    /// Source: `oracle/codemp/ui/ui_shared.c:111-115`
    pub menus: MenuSystem,

    /// Raven `displayContextDef_t uiDC`'s data tail (DEC-36 D3).
    /// Source: `oracle/codemp/ui/ui_local.h:730`
    pub uiDC: DisplayState,
}

impl Default for UiState {
    /// Raven's `uiInfo` is a zeroed file-scope struct `_UI_Init` fills; the
    /// framework arrays and `uiDC` start zeroed beside it.
    ///
    /// Source: `oracle/codemp/ui/ui_main.c:875`
    fn default() -> Self {
        UiState {
            world: UiWorld::default(),
            menus: MenuSystem::default(),
            uiDC: DisplayState::default(),
        }
    }
}
