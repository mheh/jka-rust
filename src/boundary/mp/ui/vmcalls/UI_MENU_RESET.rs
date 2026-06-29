use super::super::MpUiExport;
use crate::boundary::generic::InboundVmCall;

/// `UI_MENU_RESET` MP UI exports vmMain boundary token.
///
/// Raven: if !overlay, the background will be drawn, otherwise it will be
/// Raven: overlayed over whatever the cgame has drawn.
/// Raven: a GetClientState syscall will be made to get the current strings
/// (enum comment block is shifted; this call itself is `void Menu_Reset(void)`).
///
/// Source (enum): `oracle/oracle/codemp/ui/ui_public.h:250`
/// Source (args): `oracle/oracle/codemp/ui/ui_main.c:619` (no args)
/// Source (output): `oracle/oracle/codemp/ui/ui_main.c:620` (return 0)
/// Source (call site/transport): `oracle/oracle/codemp/client/cl_ui.cpp:1451`
/// Source: `oracle/oracle/codemp/ui/ui_public.h:250`
pub struct UiMenuReset;

impl InboundVmCall for UiMenuReset {
    type Command = MpUiExport;
    type Args = ();
    type Output = ();

    const COMMAND: MpUiExport = MpUiExport::UI_MENU_RESET;
}
