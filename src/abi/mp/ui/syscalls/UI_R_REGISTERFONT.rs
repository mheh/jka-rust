use core::ffi::c_char;

use super::super::MpUiImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::codemp::game::q_shared_h::qhandle_t;

/// Arguments for `UI_R_REGISTERFONT`.
///
/// C ABI: `qhandle_t trap_R_RegisterFont(const char *fontName)`.
/// Raven's client switch forwards the name through `VMA(1)` and returns the
/// renderer handle word.
///
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:111-113`
/// Output source: `oracle/oracle/codemp/ui/ui_syscalls.c:111-113`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1132-1133`
#[derive(Debug, Clone, Copy)]
pub struct UiRRegisterfontArgs {
    pub font_name: *const c_char,
}

impl UiRRegisterfontArgs {
    pub const fn new(font_name: *const c_char) -> Self {
        Self { font_name }
    }
}

/// `UI_R_REGISTERFONT` MP UI imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:75`
pub struct UiRRegisterfont;

impl OutboundSysCall for UiRRegisterfont {
    type Import = MpUiImport;
    type Args = UiRRegisterfontArgs;
    type Output = qhandle_t;

    const IMPORT: MpUiImport = MpUiImport::UI_R_REGISTERFONT;
}

impl EncodeSysCall for UiRRegisterfont {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.font_name)])
    }
}

impl DecodeSysCallReturn for UiRRegisterfont {
    fn decode_return(word: isize) -> Self::Output {
        word as qhandle_t
    }
}
