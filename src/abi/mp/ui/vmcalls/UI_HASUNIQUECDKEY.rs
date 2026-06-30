use super::super::MpUiExport;
use crate::abi::generic::InboundVmCall;
use crate::shared::qboolean;

/// `UI_HASUNIQUECDKEY` MP UI exports vmMain ABI token.
///
/// Raven enum comment is shifted; this entry is verified via module transport and
/// is a pure boolean return.
///
/// Source (enum): `oracle/oracle/codemp/ui/ui_public.h:245`
/// Source (args): `oracle/oracle/codemp/ui/ui_main.c:618` (no args)
/// Source (output): `oracle/oracle/codemp/ui/ui_main.c:619` (return qtrue/qfalse)
/// Source (call site/transport): `oracle/oracle/codemp/client/cl_ui.cpp:1500`
pub struct UiHasuniquecdkey;

impl InboundVmCall for UiHasuniquecdkey {
    type Command = MpUiExport;
    type Args = ();
    type Output = qboolean;

    const COMMAND: MpUiExport = MpUiExport::UI_HASUNIQUECDKEY;
}
