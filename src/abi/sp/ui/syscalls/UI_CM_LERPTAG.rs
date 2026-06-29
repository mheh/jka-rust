use core::ffi::{c_char, c_int, c_void};

use super::super::SpUiImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::ffi::syscalls::pass_float;

/// `UI_CM_LERPTAG` SP UI imports syscall ABI token.
///
/// Enum source: `oracle/oracle/code/ui/ui_public.h:181`
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:206-207` and `oracle/oracle/codemp/ui/ui_local.h:948`
/// Output source: `oracle/oracle/codemp/client/cl_ui.cpp:996-998`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:996-998`
/// TODO: SP `UI_CM_LERPTAG` has no explicit engine switch case in
/// `oracle/oracle/code/client/cl_ui.cpp`; payload shape is inherited from MP implementation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiCmLerptagArgs {
    tag: *mut c_void,
    mod_: c_int,
    start_frame: c_int,
    end_frame: c_int,
    frac: f32,
    tag_name: *const c_char,
}

impl UiCmLerptagArgs {
    pub const fn new(
        tag: *mut c_void,
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

    pub const fn tag(&self) -> *mut c_void {
        self.tag
    }

    pub const fn mod_(&self) -> c_int {
        self.mod_
    }

    pub const fn start_frame(&self) -> c_int {
        self.start_frame
    }

    pub const fn end_frame(&self) -> c_int {
        self.end_frame
    }

    pub const fn frac(&self) -> f32 {
        self.frac
    }

    pub const fn tag_name(&self) -> *const c_char {
        self.tag_name
    }
}

pub struct UiCmLerptag;

impl OutboundSysCall for UiCmLerptag {
    type Import = SpUiImport;
    type Args = UiCmLerptagArgs;
    type Output = ();

    const IMPORT: SpUiImport = SpUiImport::UI_CM_LERPTAG;
}

impl EncodeSysCall for UiCmLerptag {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.tag()),
            args.mod_() as isize,
            args.start_frame() as isize,
            args.end_frame() as isize,
            pass_float(args.frac()),
            ptr_to_word(args.tag_name()),
        ])
    }
}

impl DecodeSysCallReturn for UiCmLerptag {
    fn decode_return(_word: isize) -> Self::Output {}
}
