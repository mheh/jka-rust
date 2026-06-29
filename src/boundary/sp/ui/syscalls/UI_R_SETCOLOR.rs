use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_R_SETCOLOR` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:178`
pub struct UiRSetcolor;

impl OutboundSysCall for UiRSetcolor {
    type Import = SpUiImport;
    /// Args source: `oracle/oracle/code/ui/ui_public.h:84` and `oracle/oracle/code/client/cl_ui.cpp:419`.
    type Args = *const f32;
    /// Output source: `oracle/oracle/code/ui/ui_public.h:84` and `oracle/oracle/code/client/cl_ui.cpp:419`.
    type Output = ();

    const IMPORT: SpUiImport = SpUiImport::UI_R_SETCOLOR;
}
