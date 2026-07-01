use core::ffi::c_char;

use super::super::SpUiImport;
use abi_transport::generic::OutboundSysCall;
use sp_qshared::shared::qhandle_t;

/// `UI_R_REGISTERSKIN` SP UI imports syscall ABI token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:171`
pub struct UiRRegisterskin;

impl OutboundSysCall for UiRRegisterskin {
    type Import = SpUiImport;
    /// Args source: `oracle/oracle/code/ui/ui_public.h:47` and MP fallback `oracle/oracle/codemp/ui/ui_syscalls.c:108`.
    type Args = *const c_char;
    /// Output source: `oracle/oracle/code/ui/ui_public.h:47` and MP fallback `oracle/oracle/codemp/ui/ui_syscalls.c:108`.
    type Output = qhandle_t;

    const IMPORT: SpUiImport = SpUiImport::UI_R_REGISTERSKIN;
}
