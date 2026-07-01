use core::ffi::{c_char, c_int};

use super::super::MpUiImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `UI_R_SHADERNAMEFROMINDEX`.
///
/// C ABI: `void trap_R_ShaderNameFromIndex(char *name, int index)`.
/// Raven's client switch forwards the writable buffer through `VMA(1)` and
/// reads the index from `args[2]`.
///
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:165-167`
/// Output source: `oracle/oracle/codemp/ui/ui_syscalls.c:165-167`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:941-944`
#[derive(Debug, Clone, Copy)]
pub struct UiRShadernamefromindexArgs {
    pub name: *mut c_char,
    pub index: c_int,
}

impl UiRShadernamefromindexArgs {
    pub const fn new(name: *mut c_char, index: c_int) -> Self {
        Self { name, index }
    }
}

/// `UI_R_SHADERNAMEFROMINDEX` MP UI imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:39`
pub struct UiRShadernamefromindex;

impl OutboundSysCall for UiRShadernamefromindex {
    type Import = MpUiImport;
    type Args = UiRShadernamefromindexArgs;
    type Output = ();

    const IMPORT: MpUiImport = MpUiImport::UI_R_SHADERNAMEFROMINDEX;
}

impl EncodeSysCall for UiRShadernamefromindex {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.name), args.index as isize])
    }
}

impl DecodeSysCallReturn for UiRShadernamefromindex {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
