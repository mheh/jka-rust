//! `UiGameinfoState` — `ui_gameinfo.c`'s file-scope globals as one `UiWorld`
//! sub-struct.

#![allow(non_snake_case)]

/// Raven `#define MAX_ARENAS 1024`.
///
/// Source: `oracle/codemp/game/bg_public.h:1673`
pub const MAX_ARENAS: usize = 1024;

/// Raven `#define MAX_BOTS 1024`.
///
/// Source: `oracle/codemp/game/bg_public.h:1676`
pub const MAX_BOTS: usize = 1024;

/// The parsed arena and bot info-string caches (`ui_gameinfo.c` file-scope
/// globals folded onto `UiWorld`, §B3).
///
/// PORT-NOTE: Raven's `char *ui_botInfos[MAX_BOTS]` / `char
/// *ui_arenaInfos[MAX_ARENAS]` held `String_Alloc`-style pointers into a parse
/// buffer alongside `ui_numBots`/`ui_numArenas`; owned `Vec<String>`s carry the
/// entries and each count is the matching `len()`.
///
/// Source: `oracle/codemp/ui/ui_gameinfo.c:15-19`
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UiGameinfoState {
    /// Raven `static char *ui_botInfos[MAX_BOTS]` + `int ui_numBots`.
    /// Source: `oracle/codemp/ui/ui_gameinfo.c:15-16`
    pub ui_botInfos: Vec<String>,
    /// Raven `static char *ui_arenaInfos[MAX_ARENAS]` + `static int ui_numArenas`.
    /// Source: `oracle/codemp/ui/ui_gameinfo.c:18-19`
    pub ui_arenaInfos: Vec<String>,
}
