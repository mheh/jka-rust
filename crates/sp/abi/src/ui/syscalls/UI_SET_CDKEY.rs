use core::ffi::c_char;

use super::super::SpUiImport;
use abi_transport::generic::OutboundSysCall;

/// `UI_SET_CDKEY` SP UI imports syscall ABI token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:206`
pub struct UiSetCdkey;

impl OutboundSysCall for UiSetCdkey {
    type Import = SpUiImport;
    /// Args source: `oracle/oracle/codemp/client/cl_ui.cpp:1127-1129`.
    type Args = *mut c_char;
    /// Output source: `oracle/oracle/codemp/client/cl_ui.cpp:1127-1129`.
    type Output = ();

    const IMPORT: SpUiImport = SpUiImport::UI_SET_CDKEY;
}
