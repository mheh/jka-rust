use core::ffi::c_int;

use super::super::MpUiImport;
use crate::abi::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `UI_CIN_SETEXTENTS`.
///
/// Raven: allows you to resize the animation dynamically.
/// Raven wrapper: `syscall(UI_CIN_SETEXTENTS, handle, x, y, w, h)`.
/// Raven transport: `CIN_SetExtents(args[1], args[2], args[3], args[4], args[5]); return 0;`
///
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:428-430`
/// Args source: `oracle/oracle/codemp/ui/ui_local.h:1006`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1197-1199`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiCinSetextentsArgs {
    handle: c_int,
    x: c_int,
    y: c_int,
    w: c_int,
    h: c_int,
}

impl UiCinSetextentsArgs {
    pub const fn new(handle: c_int, x: c_int, y: c_int, w: c_int, h: c_int) -> Self {
        Self { handle, x, y, w, h }
    }

    pub const fn handle(&self) -> c_int {
        self.handle
    }

    pub const fn x(&self) -> c_int {
        self.x
    }

    pub const fn y(&self) -> c_int {
        self.y
    }

    pub const fn w(&self) -> c_int {
        self.w
    }

    pub const fn h(&self) -> c_int {
        self.h
    }
}

/// `UI_CIN_SETEXTENTS` MP UI imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/ui/ui_public.h:109`
/// Enum comment source: `oracle/oracle/codemp/ui/ui_public.h:105-109`
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:428-430`
/// Output source: `oracle/oracle/codemp/client/cl_ui.cpp:1197-1199`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1197-1199`
pub struct UiCinSetextents;

impl OutboundSysCall for UiCinSetextents {
    type Import = MpUiImport;
    type Args = UiCinSetextentsArgs;
    type Output = ();

    const IMPORT: MpUiImport = MpUiImport::UI_CIN_SETEXTENTS;
}

impl EncodeSysCall for UiCinSetextents {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            args.handle() as isize,
            args.x() as isize,
            args.y() as isize,
            args.w() as isize,
            args.h() as isize,
        ])
    }
}

impl DecodeSysCallReturn for UiCinSetextents {
    fn decode_return(_word: isize) -> Self::Output {}
}
