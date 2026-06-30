use core::ffi::c_int;

use super::super::MpUiImport;
use crate::abi::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};
use crate::ffi::syscalls::pass_float;

/// Arguments for `UI_R_DRAWSTRETCHPIC`.
///
/// Raven wrapper packs each float with `PASSFLOAT`.
/// Raven transport reads each float with `VMF`.
///
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:194-195`
/// Args source: `oracle/oracle/codemp/ui/ui_local.h:945`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:984-986`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiRDrawstretchpicArgs {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    s1: f32,
    t1: f32,
    s2: f32,
    t2: f32,
    h_shader: c_int,
}

impl UiRDrawstretchpicArgs {
    pub const fn new(
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        s1: f32,
        t1: f32,
        s2: f32,
        t2: f32,
        h_shader: c_int,
    ) -> Self {
        Self {
            x,
            y,
            w,
            h,
            s1,
            t1,
            s2,
            t2,
            h_shader,
        }
    }
}

/// `UI_R_DRAWSTRETCHPIC` MP UI imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/ui/ui_public.h:46`
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:194-195`
/// Output source: `oracle/oracle/codemp/client/cl_ui.cpp:984-986`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:984-986`
pub struct UiRDrawstretchpic;

impl OutboundSysCall for UiRDrawstretchpic {
    type Import = MpUiImport;
    type Args = UiRDrawstretchpicArgs;
    type Output = ();

    const IMPORT: MpUiImport = MpUiImport::UI_R_DRAWSTRETCHPIC;
}

impl EncodeSysCall for UiRDrawstretchpic {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            pass_float(args.x),
            pass_float(args.y),
            pass_float(args.w),
            pass_float(args.h),
            pass_float(args.s1),
            pass_float(args.t1),
            pass_float(args.s2),
            pass_float(args.t2),
            args.h_shader as isize,
        ])
    }
}

impl DecodeSysCallReturn for UiRDrawstretchpic {
    fn decode_return(_word: isize) -> Self::Output {}
}
