#![allow(non_camel_case_types, non_snake_case)]

use std::os::raw::c_int;

/// Raven `uiMenuCommand_t` — menu command type.
///
/// Raven: Typedef for int menu command enumeration.
/// Type definition source: `oracle/codemp/ui/ui_public.h:194-208`
pub type uiMenuCommand_t = c_int;

pub const UIMENU_NONE: uiMenuCommand_t = 0;
pub const UIMENU_MAIN: uiMenuCommand_t = 1;
pub const UIMENU_INGAME: uiMenuCommand_t = 2;
pub const UIMENU_PLAYERCONFIG: uiMenuCommand_t = 3;
pub const UIMENU_TEAM: uiMenuCommand_t = 4;
pub const UIMENU_POSTGAME: uiMenuCommand_t = 5;
pub const UIMENU_PLAYERFORCE: uiMenuCommand_t = 6;
pub const UIMENU_SIEGEMESSAGE: uiMenuCommand_t = 7;
pub const UIMENU_SIEGEOBJECTIVES: uiMenuCommand_t = 8;
pub const UIMENU_VOICECHAT: uiMenuCommand_t = 9;
pub const UIMENU_CLOSEALL: uiMenuCommand_t = 10;
pub const UIMENU_CLASSSEL: uiMenuCommand_t = 11;
