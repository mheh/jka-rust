use core::ffi::c_int;

use super::super::MpUiExport;
use crate::abi::generic::InboundVmCall;

/// `UI_SET_ACTIVE_MENU` MP UI exports vmMain ABI token.
///
/// Raven comment in this enum block is shifted; actual signature is
/// `_UI_SetActiveMenu( uiMenuCommand_t menu )`.
///
/// Source (enum): `oracle/oracle/codemp/ui/ui_public.h:237`
/// Source (args): `oracle/oracle/codemp/ui/ui_main.c:608`
/// Source (output): `oracle/oracle/codemp/ui/ui_main.c:608` (return 0)
/// Source (call site/transport): `oracle/oracle/codemp/client/cl_scrn.cpp:429`
pub struct UiSetActiveMenu;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UiSetActiveMenuArgs {
    menu: c_int,
}

impl UiSetActiveMenuArgs {
    pub const fn new(menu: c_int) -> Self {
        Self { menu }
    }

    pub const fn menu(self) -> c_int {
        self.menu
    }
}

impl InboundVmCall for UiSetActiveMenu {
    type Command = MpUiExport;
    type Args = UiSetActiveMenuArgs;
    type Output = ();

    const COMMAND: MpUiExport = MpUiExport::UI_SET_ACTIVE_MENU;
}
