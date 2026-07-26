//! MP UI exports enum vocabulary.
//!
//! Transcribed from Raven `oracle/codemp/ui/ui_public.h`.
//! These discriminants are ABI wire values; do not renumber them.

#![allow(non_camel_case_types)]

#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MpUiExport {
    /// system reserved
    /// Source: `oracle/codemp/ui/ui_public.h:217`
    UI_GETAPIVERSION = 0,

    /// void	UI_Init( void );
    /// Source: `oracle/codemp/ui/ui_public.h:219`
    UI_INIT,

    /// void	UI_Shutdown( void );
    /// Source: `oracle/codemp/ui/ui_public.h:222`
    UI_SHUTDOWN,

    /// void	UI_KeyEvent( int key );
    /// Source: `oracle/codemp/ui/ui_public.h:225`
    UI_KEY_EVENT,

    /// void	UI_MouseEvent( int dx, int dy );
    /// Source: `oracle/codemp/ui/ui_public.h:228`
    UI_MOUSE_EVENT,

    /// void	UI_Refresh( int time );
    /// Source: `oracle/codemp/ui/ui_public.h:231`
    UI_REFRESH,

    /// qboolean UI_IsFullscreen( void );
    /// Source: `oracle/codemp/ui/ui_public.h:234`
    UI_IS_FULLSCREEN,

    /// void	UI_SetActiveMenu( uiMenuCommand_t menu );
    /// Source: `oracle/codemp/ui/ui_public.h:237`
    UI_SET_ACTIVE_MENU,

    /// qboolean UI_ConsoleCommand( int realTime );
    /// Source: `oracle/codemp/ui/ui_public.h:240`
    UI_CONSOLE_COMMAND,

    /// void	UI_DrawConnectScreen( qboolean overlay );
    ///
    /// Raven: if !overlay, the background will be drawn, otherwise it will be
    /// overlayed over whatever the cgame has drawn.
    /// a GetClientState syscall will be made to get the current strings
    /// Source: `oracle/codemp/ui/ui_public.h:243`
    UI_DRAW_CONNECT_SCREEN,

    /// Source: `oracle/codemp/ui/ui_public.h:245`
    UI_HASUNIQUECDKEY,

    /// Source: `oracle/codemp/ui/ui_public.h:250`
    UI_MENU_RESET,
}

/// The `vmMain` pre-decode half of the SEAM-D6 enum<->wire-word pair: the raw
/// `c_int` command word is converted fallibly BEFORE the exhaustive dispatch
/// match; an unrecognized command's fallback (ui returns `-1`, Raven's
/// `ui_main.c:624` post-switch fall-through) lives at the conversion's `Err`, not in a
/// match arm. Mirrors `MpGameExport`'s `TryFrom` (`../game/exports.rs`).
///
/// Source: `docs/architecture/engine-seam.md` § inbound dual (SEAM-D6).
impl TryFrom<i32> for MpUiExport {
    type Error = i32;

    fn try_from(v: i32) -> Result<Self, i32> {
        Ok(match v {
            x if x == Self::UI_GETAPIVERSION as i32 => Self::UI_GETAPIVERSION,
            x if x == Self::UI_INIT as i32 => Self::UI_INIT,
            x if x == Self::UI_SHUTDOWN as i32 => Self::UI_SHUTDOWN,
            x if x == Self::UI_KEY_EVENT as i32 => Self::UI_KEY_EVENT,
            x if x == Self::UI_MOUSE_EVENT as i32 => Self::UI_MOUSE_EVENT,
            x if x == Self::UI_REFRESH as i32 => Self::UI_REFRESH,
            x if x == Self::UI_IS_FULLSCREEN as i32 => Self::UI_IS_FULLSCREEN,
            x if x == Self::UI_SET_ACTIVE_MENU as i32 => Self::UI_SET_ACTIVE_MENU,
            x if x == Self::UI_CONSOLE_COMMAND as i32 => Self::UI_CONSOLE_COMMAND,
            x if x == Self::UI_DRAW_CONNECT_SCREEN as i32 => Self::UI_DRAW_CONNECT_SCREEN,
            x if x == Self::UI_HASUNIQUECDKEY as i32 => Self::UI_HASUNIQUECDKEY,
            x if x == Self::UI_MENU_RESET as i32 => Self::UI_MENU_RESET,
            _ => return Err(v),
        })
    }
}
