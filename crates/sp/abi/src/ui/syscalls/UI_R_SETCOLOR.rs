use super::super::SpUiImport;
use abi_transport::generic::OutboundSysCall;

/// `UI_R_SETCOLOR` SP UI imports syscall ABI token.
///
/// Source: `oracle/code/ui/ui_public.h:178`
pub struct UiRSetcolor;

impl OutboundSysCall for UiRSetcolor {
    type Import = SpUiImport;
    /// Args source: `oracle/code/ui/ui_public.h:84` and `oracle/code/client/cl_ui.cpp:419`.
    type Args = *const f32;
    /// Output source: `oracle/code/ui/ui_public.h:84` and `oracle/code/client/cl_ui.cpp:419`.
    type Output = ();

    const IMPORT: SpUiImport = SpUiImport::UI_R_SETCOLOR;
}
