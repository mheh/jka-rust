#![allow(non_camel_case_types, non_snake_case)]

/// Raven `uiExport_t` — engine export table to UI module.
///
/// Raven: UI module entry point indices for syscalls from the engine.
/// Type definition source: `oracle/codemp/ui/ui_public.h:216-251`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum uiExport_t {
    UI_GETAPIVERSION = 0,
    UI_INIT,
    UI_SHUTDOWN,
    UI_KEY_EVENT,
    UI_MOUSE_EVENT,
    UI_REFRESH,
    UI_IS_FULLSCREEN,
    UI_SET_ACTIVE_MENU,
    UI_CONSOLE_COMMAND,
    UI_DRAW_CONNECT_SCREEN,
    UI_HASUNIQUECDKEY,
    UI_MENU_RESET,
}
