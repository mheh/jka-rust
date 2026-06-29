use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_GET_CDKEY` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:205`
pub struct UiGetCdkey;

impl OutboundSysCall for UiGetCdkey {
    type Import = SpUiImport;
    /// TODO: Port args/output.
    /// SP Oracle sources in this assignment only expose enum (`ui_public.h:205`) and do not include
    /// a UI->engine signature for `UI_GET_CDKEY` (no matching `UI_GET_CDKEY` case/wrapper found in
    /// `oracle/oracle/code/ui/ui_syscalls.cpp` or `oracle/oracle/code/client/cl_ui.cpp`).
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_GET_CDKEY;
}
