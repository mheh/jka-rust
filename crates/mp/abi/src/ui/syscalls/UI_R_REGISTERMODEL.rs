use core::ffi::c_char;

use super::super::MpUiImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use mp_qshared::shared::qhandle_t;

/// Arguments for `UI_R_REGISTERMODEL`.
///
/// C ABI: `qhandle_t trap_R_RegisterModel(const char *name)`.
/// Raven's client switch forwards the shader name through `VMA(1)` and returns
/// the renderer handle word.
///
/// Args source: `oracle/codemp/ui/ui_syscalls.c:103-104`
/// Output source: `oracle/codemp/ui/ui_syscalls.c:103-104`
/// Transport/switch source: `oracle/codemp/client/cl_ui.cpp:932-933`
#[derive(Debug, Clone, Copy)]
pub struct UiRRegistermodelArgs {
    pub name: *const c_char,
}

impl UiRRegistermodelArgs {
    pub const fn new(name: *const c_char) -> Self {
        Self { name }
    }
}

/// `UI_R_REGISTERMODEL` MP UI imports syscall ABI token.
///
/// Source: `oracle/codemp/ui/ui_public.h:36`
pub struct UiRRegistermodel;

impl OutboundSysCall for UiRRegistermodel {
    type Import = MpUiImport;
    type Args = UiRRegistermodelArgs;
    type Output = qhandle_t;

    const IMPORT: MpUiImport = MpUiImport::UI_R_REGISTERMODEL;
}

impl EncodeSysCall for UiRRegistermodel {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.name)])
    }
}

impl DecodeSysCallReturn for UiRRegistermodel {
    fn decode_return(word: isize) -> Self::Output {
        word as qhandle_t
    }
}
