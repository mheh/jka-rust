use core::ffi::c_char;

use super::super::MpUiImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `UI_R_REMAP_SHADER`.
///
/// C ABI: `void trap_R_RemapShader(const char *oldShader, const char *newShader, const char *timeOffset)`.
/// Raven's client switch forwards all three strings through `VMA(1..3)`.
///
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:434-435`
/// Output source: `oracle/oracle/codemp/ui/ui_syscalls.c:434-435`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1201-1202`
#[derive(Debug, Clone, Copy)]
pub struct UiRRemapShaderArgs {
    pub old_shader: *const c_char,
    pub new_shader: *const c_char,
    pub time_offset: *const c_char,
}

impl UiRRemapShaderArgs {
    pub const fn new(
        old_shader: *const c_char,
        new_shader: *const c_char,
        time_offset: *const c_char,
    ) -> Self {
        Self {
            old_shader,
            new_shader,
            time_offset,
        }
    }
}

/// `UI_R_REMAP_SHADER` MP UI imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:110`
pub struct UiRRemapShader;

impl OutboundSysCall for UiRRemapShader {
    type Import = MpUiImport;
    type Args = UiRRemapShaderArgs;
    type Output = ();

    const IMPORT: MpUiImport = MpUiImport::UI_R_REMAP_SHADER;
}

impl EncodeSysCall for UiRRemapShader {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.old_shader),
            ptr_to_word(args.new_shader),
            ptr_to_word(args.time_offset),
        ])
    }
}

impl DecodeSysCallReturn for UiRRemapShader {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
