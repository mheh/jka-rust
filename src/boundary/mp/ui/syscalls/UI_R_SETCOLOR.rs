use core::ffi::c_void;

use super::super::MpUiImport;
use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `UI_R_SETCOLOR`.
///
/// C ABI: `void trap_R_SetColor(const float *rgba)`.
/// Raven's client switch forwards the color pointer through `VMA(1)`; `NULL`
/// clears the current color.
///
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:190-191`
/// Output source: `oracle/oracle/codemp/ui/ui_syscalls.c:190-191`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:980-981`
#[derive(Debug, Clone, Copy)]
pub struct UiRSetcolorArgs {
    pub rgba: *const f32,
}

impl UiRSetcolorArgs {
    pub const fn new(rgba: *const f32) -> Self {
        Self { rgba }
    }
}

/// `UI_R_SETCOLOR` MP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:45`
pub struct UiRSetcolor;

impl OutboundSysCall for UiRSetcolor {
    type Import = MpUiImport;
    type Args = UiRSetcolorArgs;
    type Output = ();

    const IMPORT: MpUiImport = MpUiImport::UI_R_SETCOLOR;
}

impl EncodeSysCall for UiRSetcolor {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.rgba as *const c_void)])
    }
}

impl DecodeSysCallReturn for UiRSetcolor {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
