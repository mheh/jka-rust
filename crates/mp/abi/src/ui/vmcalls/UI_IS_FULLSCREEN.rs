use super::super::MpUiExport;
use abi_transport::generic::InboundVmCall;
use mp_qshared::shared::qboolean;

/// `UI_IS_FULLSCREEN` MP UI exports vmMain ABI token.
///
/// Raven signature in this enum block is shifted; actual signature is
/// `_UI_IsFullscreen( void )`.
///
/// Source (enum): `oracle/codemp/ui/ui_public.h:234`
/// Source (args): `oracle/codemp/ui/ui_main.c:555`
/// Source (output): `oracle/codemp/ui/ui_main.c:605`
/// Source (call site/transport): `oracle/codemp/client/cl_scrn.cpp:418`
pub struct UiIsFullscreen;

impl InboundVmCall for UiIsFullscreen {
    type Command = MpUiExport;
    type Args = ();
    type Output = qboolean;

    const COMMAND: MpUiExport = MpUiExport::UI_IS_FULLSCREEN;
}
