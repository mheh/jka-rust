use crate::ffi::types::qboolean;
use super::super::MpUiExport;
use crate::abi::generic::InboundVmCall;

/// `UI_DRAW_CONNECT_SCREEN` MP UI exports vmMain ABI token.
///
/// Raven enum comment is shifted around this section; this call uses
/// `void UI_DrawConnectScreen( qboolean overlay )`.
///
/// Source (enum): `oracle/oracle/codemp/ui/ui_public.h:243`
/// Source (args): `oracle/oracle/codemp/ui/ui_main.c:615`
/// Source (output): `oracle/oracle/codemp/ui/ui_main.c:615` (return 0)
/// Source (call site/transport): `oracle/oracle/codemp/client/cl_scrn.cpp:437`
pub struct UiDrawConnectScreen;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UiDrawConnectScreenArgs {
    overlay: qboolean,
}

impl UiDrawConnectScreenArgs {
    pub const fn new(overlay: qboolean) -> Self {
        Self { overlay }
    }

    pub const fn overlay(self) -> qboolean {
        self.overlay
    }
}

impl InboundVmCall for UiDrawConnectScreen {
    type Command = MpUiExport;
    type Args = UiDrawConnectScreenArgs;
    type Output = ();

    const COMMAND: MpUiExport = MpUiExport::UI_DRAW_CONNECT_SCREEN;
}
