use core::ffi::c_int;

use super::super::MpUiExport;
use crate::abi::generic::InboundVmCall;

/// `UI_GETAPIVERSION` MP UI exports vmMain ABI token.
///
/// Source (enum): `oracle/oracle/codemp/ui/ui_public.h:217`
/// Source (args): `oracle/oracle/codemp/ui/ui_main.c:585` (no args)
/// Source (output): `oracle/oracle/codemp/ui/ui_main.c:580`
/// Source (call site/transport): `oracle/oracle/codemp/client/cl_ui.cpp:1484`
/// Source: `oracle/oracle/codemp/ui/ui_public.h:217`
pub struct UiGetapiversion;

impl InboundVmCall for UiGetapiversion {
    type Command = MpUiExport;
    type Args = ();
    type Output = c_int;

    const COMMAND: MpUiExport = MpUiExport::UI_GETAPIVERSION;
}
