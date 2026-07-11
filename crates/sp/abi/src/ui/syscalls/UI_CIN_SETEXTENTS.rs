use core::ffi::c_int;

use super::super::SpUiImport;
use abi_transport::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `UI_CIN_SETEXTENTS` SP UI imports syscall ABI token.
///
/// Enum value source: `oracle/code/ui/ui_public.h:231`
/// Args source: `oracle/code/client/cl_ui.cpp:488-489`
/// Output source: `oracle/code/client/cl_ui.cpp:489`
/// Transport/switch source: `oracle/code/client/cl_ui.cpp:488-489`
pub struct UiCinSetextents;

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

impl OutboundSysCall for UiCinSetextents {
    type Import = SpUiImport;
    type Args = UiCinSetextentsArgs;
    type Output = ();

    const IMPORT: SpUiImport = SpUiImport::UI_CIN_SETEXTENTS;
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
