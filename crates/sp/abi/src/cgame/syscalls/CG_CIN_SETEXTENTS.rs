use core::ffi::c_int;

use super::super::SpCgameImport;
use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// Arguments for `CG_CIN_SETEXTENTS`.
///
/// Raven: allows you to resize the animation dynamically.
/// Raven wrapper: `syscall(CG_CIN_SETEXTENTS, handle, x, y, w, h);`
/// Raven transport: `CIN_SetExtents(args[1], args[2], args[3], args[4], args[5]); return 0;`
///
/// Args source: `oracle/code/cgame/cg_syscalls.cpp:543-545`
/// Output source: `oracle/code/client/cl_cgame.cpp:830-832`
/// Transport/switch source: `oracle/code/client/cl_cgame.cpp:830-832`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgCinSetextentsArgs {
    handle: c_int,
    x: c_int,
    y: c_int,
    w: c_int,
    h: c_int,
}

impl CgCinSetextentsArgs {
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

/// `CG_CIN_SETEXTENTS` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/code/cgame/cg_public.h:189`
/// Args source: `oracle/code/cgame/cg_syscalls.cpp:543-545`
/// Output source: `oracle/code/client/cl_cgame.cpp:830-832`
/// Transport/switch source: `oracle/code/client/cl_cgame.cpp:830-832`
pub struct CgCinSetextents;

impl OutboundSysCall for CgCinSetextents {
    type Import = SpCgameImport;
    type Args = CgCinSetextentsArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_CIN_SETEXTENTS;
}

impl EncodeSysCall for CgCinSetextents {
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

impl DecodeSysCallReturn for CgCinSetextents {
    fn decode_return(_word: isize) -> Self::Output {}
}
