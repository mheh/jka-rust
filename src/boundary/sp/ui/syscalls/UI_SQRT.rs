use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_SQRT` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:245`
pub struct UiSqrt;

impl OutboundSysCall for UiSqrt {
    type Import = SpUiImport;
    /// Args source: no direct `UI_SQRT` case in `oracle/oracle/code/client/cl_ui.cpp`.
    /// Fallback source (same Raven ABI profile): `oracle/oracle/codemp/client/cl_ui.cpp:832-833`.
    /// TODO: SP UI transport evidence for this call is currently missing in
    /// `oracle/oracle/code/client/cl_ui.cpp`; validate against that switch path.
    type Args = f32;
    /// Output source: `oracle/oracle/codemp/client/cl_ui.cpp:833`
    /// SP engine transport evidence not present; this follows MP float-return convention.
    type Output = f32;

    const IMPORT: SpUiImport = SpUiImport::UI_SQRT;
}
