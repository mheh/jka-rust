//! `MenuScratch` — the framework's function-local persistent scratch.

/// `ui_shared.c`'s function-local `static` state that genuinely persists across
/// calls, owned by [`MenuSystem`](super::menu_system::MenuSystem) (§B3: no
/// `static mut`).
///
/// PORT-NOTE: the file's other function-local statics do not land here —
/// `PC_SourceError`/`PC_SourceWarning`'s `char string[4096]` and
/// `String_Alloc`'s `staticNULL` are single-call formatting/return scratch that
/// owned returns dissolve, and `PC_String_Parse`'s `static char *squiggy = "}"`
/// is a string literal, not state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(non_snake_case)]
pub struct MenuScratch {
    /// Raven `Menu_HandleKey`'s `static qboolean inHandleKey` — the reentrancy
    /// guard that suppresses a nested key handler.
    /// Source: `oracle/codemp/ui/ui_shared.c` (`Menu_HandleKey`)
    pub inHandleKey: bool,
}
