use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_CM_LERPTAG` SP UI imports syscall boundary token.
///
/// Enum value source: `oracle/oracle/code/ui/ui_public.h:181`
/// Args source: not present in SP `oracle/oracle/code/ui/ui_syscalls.cpp` or
/// `oracle/oracle/code/client/cl_ui.cpp`; MP fallback reference:
/// `oracle/oracle/codemp/ui/ui_syscalls.c:206-207`.
/// Output source: not present in SP call table; MP fallback reference:
/// `oracle/oracle/codemp/client/cl_ui.cpp:996-998`.
/// Transport/switch source: not present in SP `cl_ui.cpp` call table; MP fallback reference
/// `oracle/oracle/codemp/client/cl_ui.cpp:996-998`.
/// TODO: SP `UI_CM_LERPTAG` transport payload remains ambiguous because no SP engine switch
/// case is present in `oracle/oracle/code/client/cl_ui.cpp`.
pub struct UiCmLerptag;

impl OutboundSysCall for UiCmLerptag {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_CM_LERPTAG;
}
