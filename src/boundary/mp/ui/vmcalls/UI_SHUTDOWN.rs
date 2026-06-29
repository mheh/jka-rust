use super::super::MpUiExport;
use crate::boundary::generic::InboundVmCall;

/// `UI_SHUTDOWN` MP UI exports vmMain boundary token.
///
/// Raven: void	UI_Init( void );
/// Raven comment for this enum slot is shifted; actual signature in `ui_main.c` is
/// `void _UI_Shutdown( void )`.
///
/// Source (enum): `oracle/oracle/codemp/ui/ui_public.h:222`
/// Source (args): `oracle/oracle/codemp/ui/ui_main.c:551`
/// Source (output): `oracle/oracle/codemp/ui/ui_main.c:589-599` (return 0)
/// Source (call site/transport): `oracle/oracle/codemp/client/cl_ui.cpp:1450`
/// Source: `oracle/oracle/codemp/ui/ui_public.h:222`
pub struct UiShutdown;

impl InboundVmCall for UiShutdown {
    type Command = MpUiExport;
    type Args = ();
    type Output = ();

    const COMMAND: MpUiExport = MpUiExport::UI_SHUTDOWN;
}
