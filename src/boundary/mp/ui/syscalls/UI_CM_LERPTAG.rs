use core::ffi::{c_char, c_int};

use super::super::MpUiImport;
use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::ffi::syscalls::pass_float;

/// Arguments for `UI_CM_LERPTAG`.
///
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:206-207`
/// Args source: `oracle/oracle/codemp/ui/ui_local.h:948`
/// Output source: `oracle/oracle/codemp/client/cl_ui.cpp:996-998`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:996-998`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiCmLerptagArgs {
    tag: *mut core::ffi::c_void,
    mod_: c_int,
    start_frame: c_int,
    end_frame: c_int,
    frac: f32,
    tag_name: *const c_char,
}

impl UiCmLerptagArgs {
    pub const fn new(
        tag: *mut core::ffi::c_void,
        mod_: c_int,
        start_frame: c_int,
        end_frame: c_int,
        frac: f32,
        tag_name: *const c_char,
    ) -> Self {
        Self {
            tag,
            mod_,
            start_frame,
            end_frame,
            frac,
            tag_name,
        }
    }
}

/// `UI_CM_LERPTAG` MP UI imports syscall boundary token.
///
/// Enum value source: `oracle/oracle/codemp/ui/ui_public.h:48`
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:206-207`
/// Output source: `oracle/oracle/codemp/client/cl_ui.cpp:996-998`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:996-998`
pub struct UiCmLerptag;

impl OutboundSysCall for UiCmLerptag {
    type Import = MpUiImport;
    type Args = UiCmLerptagArgs;
    type Output = ();

    const IMPORT: MpUiImport = MpUiImport::UI_CM_LERPTAG;
}

impl EncodeSysCall for UiCmLerptag {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.tag),
            args.mod_ as isize,
            args.start_frame as isize,
            args.end_frame as isize,
            pass_float(args.frac),
            ptr_to_word(args.tag_name),
        ])
    }
}

impl DecodeSysCallReturn for UiCmLerptag {
    fn decode_return(_word: isize) -> Self::Output {}
}
