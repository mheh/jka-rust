//! `UiScratch` — ui's function-local persistent scratch.

#![allow(non_snake_case)]

use core::ffi::c_int;

/// The `ui_*.c` function-local `static`s that genuinely persist across calls,
/// owned by `UiWorld` (§B3: no `static mut`). Each field cites the function
/// whose `static` it replaces.
///
/// PORT-NOTE: ui's other function-local statics do not land here. `UI_Argv`'s
/// and `UI_Cvar_VariableString`'s `char buffer[1024]`, `GetMenuBuffer`'s `char
/// buf[MAX_MENUFILE]`, `GetCRDelineatedString`'s `char sTemp[256]`,
/// `UI_GetStringEdString`'s `char text[1024]`, `UI_FeederItemText`'s
/// `hostname`/`clientBuff`/`needPass` and the `char info[1024]` staging buffers
/// in `UI_CheckPassword`/`UI_FeederCount`/`UI_FeederItemImage`/
/// `UI_FeederSelection` were all `static` only so a `const char *` could be
/// returned or so a 1 KB buffer stayed off the stack; owned returns (§C7)
/// dissolve every one of them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct UiScratch {
    /// Raven `UI_BuildServerDisplayList`'s `static int numinvisible` — how many
    /// servers the last pass filtered out, compared against the current pass to
    /// decide whether the display list changed.
    /// Source: `oracle/codemp/ui/ui_main.c` (`UI_BuildServerDisplayList`)
    pub UI_BuildServerDisplayList_numinvisible: c_int,

    /// Raven `UI_BuildFindPlayerList`'s `static int numFound` — players matched
    /// so far across the multi-frame search.
    /// Source: `oracle/codemp/ui/ui_main.c` (`UI_BuildFindPlayerList`)
    pub UI_BuildFindPlayerList_numFound: c_int,
    /// Raven `UI_BuildFindPlayerList`'s `static int numTimeOuts`.
    /// Source: `oracle/codemp/ui/ui_main.c` (`UI_BuildFindPlayerList`)
    pub UI_BuildFindPlayerList_numTimeOuts: c_int,

    /// Raven `UI_FeederItemText`'s `static int lastColumn` — the column the
    /// previous call formatted, memoized with `lastTime` so a repeated request
    /// reuses the cached server-info parse.
    /// Source: `oracle/codemp/ui/ui_main.c:8780` (`UI_FeederItemText`)
    pub UI_FeederItemText_lastColumn: c_int,
    /// Raven `UI_FeederItemText`'s `static int lastTime`.
    /// Source: `oracle/codemp/ui/ui_main.c:8780` (`UI_FeederItemText`)
    pub UI_FeederItemText_lastTime: c_int,

    /// Raven `_UI_Refresh`'s `static int index` — cursor into the FPS ring.
    /// Source: `oracle/codemp/ui/ui_main.c` (`_UI_Refresh`)
    pub UI_Refresh_index: c_int,
    /// Raven `_UI_Refresh`'s `static int previousTimes[4]` — the frame-time
    /// ring the displayed FPS averages.
    /// Source: `oracle/codemp/ui/ui_main.c` (`_UI_Refresh`)
    pub UI_Refresh_previousTimes: [c_int; 4],
}
