use core::ffi::c_char;

use super::super::MpUiImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::codemp::game::q_shared_h::qhandle_t;

/// Arguments for `UI_R_REGISTERSHADERNOMIP`.
///
/// C ABI: `qhandle_t trap_R_RegisterShaderNoMip(const char *name)`.
/// Raven's client switch forwards the name through `VMA(1)` and returns the
/// renderer handle word.
///
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:151-160`
/// Output source: `oracle/oracle/codemp/ui/ui_syscalls.c:151-160`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:938-939`
#[derive(Debug, Clone, Copy)]
pub struct UiRRegistershadernomipArgs {
    pub name: *const c_char,
}

impl UiRRegistershadernomipArgs {
    pub const fn new(name: *const c_char) -> Self {
        Self { name }
    }
}

/// `UI_R_REGISTERSHADERNOMIP` MP UI imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:38`
pub struct UiRRegistershadernomip;

impl OutboundSysCall for UiRRegistershadernomip {
    type Import = MpUiImport;
    type Args = UiRRegistershadernomipArgs;
    type Output = qhandle_t;

    const IMPORT: MpUiImport = MpUiImport::UI_R_REGISTERSHADERNOMIP;
}

impl EncodeSysCall for UiRRegistershadernomip {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.name)])
    }
}

impl DecodeSysCallReturn for UiRRegistershadernomip {
    fn decode_return(word: isize) -> Self::Output {
        word as qhandle_t
    }
}
