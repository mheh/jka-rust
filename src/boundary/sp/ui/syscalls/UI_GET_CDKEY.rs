use core::ffi::{c_char, c_int};

use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_GET_CDKEY` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:205`
pub struct UiGetCdkey;

impl OutboundSysCall for UiGetCdkey {
    type Import = SpUiImport;
    /// Args source: `oracle/oracle/codemp/client/cl_ui.cpp:1123-1125`.
    type Args = (*mut c_char, c_int);
    /// Output source: `oracle/oracle/codemp/client/cl_ui.cpp:1123-1125`.
    type Output = ();

    const IMPORT: SpUiImport = SpUiImport::UI_GET_CDKEY;
}
