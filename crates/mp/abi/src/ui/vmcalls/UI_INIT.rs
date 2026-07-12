use super::super::MpUiExport;
use abi_transport::generic::InboundVmCall;
use mp_qshared::shared::qboolean;

/// `UI_INIT` MP UI exports vmMain ABI token.
///
/// Raven comment alignment for this enum block is shifted in this snapshot (`ui_public.h` around UI_INIT),
/// so signature is validated from `ui_main.c` switch/function call paths.
///
/// Source (enum): `oracle/codemp/ui/ui_public.h:219`
/// Source (args): `oracle/codemp/ui/ui_main.c:550`
/// Source (output): `oracle/codemp/ui/ui_main.c:589`
/// Source (call site/transport): `oracle/codemp/client/cl_ui.cpp:1494`
pub struct UiInit;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UiInitArgs {
    in_game_load: qboolean,
}

impl UiInitArgs {
    pub const fn new(in_game_load: qboolean) -> Self {
        Self { in_game_load }
    }

    pub const fn in_game_load(self) -> qboolean {
        self.in_game_load
    }
}

impl InboundVmCall for UiInit {
    type Command = MpUiExport;
    type Args = UiInitArgs;
    type Output = ();

    const COMMAND: MpUiExport = MpUiExport::UI_INIT;
}
