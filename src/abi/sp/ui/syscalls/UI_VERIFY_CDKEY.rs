use core::ffi::{c_char, c_int};

use super::super::SpUiImport;
use crate::abi::generic::OutboundSysCall;

/// `UI_VERIFY_CDKEY` SP UI imports syscall ABI token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:233`
pub struct UiVerifyCdkey;

impl OutboundSysCall for UiVerifyCdkey {
    type Import = SpUiImport;
    /// Args source: `oracle/oracle/codemp/client/cl_ui.cpp:1206-1208`.
    type Args = (*const c_char, *const c_char);
    /// Output source: `oracle/oracle/codemp/client/cl_ui.cpp:1206-1208`.
    type Output = c_int;

    const IMPORT: SpUiImport = SpUiImport::UI_VERIFY_CDKEY;
}
