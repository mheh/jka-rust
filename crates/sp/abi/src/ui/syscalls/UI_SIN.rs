use super::super::SpUiImport;
use abi_transport::generic::OutboundSysCall;

/// `UI_SIN` SP UI imports syscall ABI token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:242`
pub struct UiSin;

impl OutboundSysCall for UiSin {
    type Import = SpUiImport;
    /// Args source: no direct `UI_SIN` case in `oracle/oracle/code/client/cl_ui.cpp`.
    /// Fallback source (same Raven ABI profile): `oracle/oracle/codemp/client/cl_ui.cpp:826-828`.
    /// TODO: SP UI transport evidence for this call is currently missing in
    /// `oracle/oracle/code/client/cl_ui.cpp`; validate against that switch path.
    type Args = f32;
    /// Output source: `oracle/oracle/codemp/client/cl_ui.cpp:827`
    /// SP engine transport evidence not present yet; this follows MP float-return convention.
    type Output = f32;

    const IMPORT: SpUiImport = SpUiImport::UI_SIN;
}
