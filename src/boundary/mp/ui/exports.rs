//! MP UI exports enum vocabulary.
//!
//! Transcribed from Raven `oracle/oracle/codemp/ui/ui_public.h`.
//! These discriminants are ABI wire values; do not renumber them.

#![allow(non_camel_case_types)]

#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MpUiExport {
    /// system reserved
    /// Source: `oracle/oracle/codemp/ui/ui_public.h:217`
    UI_GETAPIVERSION = 0,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:219`
    UI_INIT,

    /// void	UI_Init( void );
    /// Source: `oracle/oracle/codemp/ui/ui_public.h:222`
    UI_SHUTDOWN,

    /// void	UI_Shutdown( void );
    /// Source: `oracle/oracle/codemp/ui/ui_public.h:225`
    UI_KEY_EVENT,

    /// void	UI_KeyEvent( int key );
    /// Source: `oracle/oracle/codemp/ui/ui_public.h:228`
    UI_MOUSE_EVENT,

    /// void	UI_MouseEvent( int dx, int dy );
    /// Source: `oracle/oracle/codemp/ui/ui_public.h:231`
    UI_REFRESH,

    /// void	UI_Refresh( int time );
    /// Source: `oracle/oracle/codemp/ui/ui_public.h:234`
    UI_IS_FULLSCREEN,

    /// qboolean UI_IsFullscreen( void );
    /// Source: `oracle/oracle/codemp/ui/ui_public.h:237`
    UI_SET_ACTIVE_MENU,

    /// void	UI_SetActiveMenu( uiMenuCommand_t menu );
    /// Source: `oracle/oracle/codemp/ui/ui_public.h:240`
    UI_CONSOLE_COMMAND,

    /// qboolean UI_ConsoleCommand( int realTime );
    /// Source: `oracle/oracle/codemp/ui/ui_public.h:243`
    UI_DRAW_CONNECT_SCREEN,

    /// void	UI_DrawConnectScreen( qboolean overlay );
    /// Source: `oracle/oracle/codemp/ui/ui_public.h:245`
    UI_HASUNIQUECDKEY,

    /// if !overlay, the background will be drawn, otherwise it will be
    /// overlayed over whatever the cgame has drawn.
    /// a GetClientState syscall will be made to get the current strings
    /// Source: `oracle/oracle/codemp/ui/ui_public.h:250`
    UI_MENU_RESET,

}
