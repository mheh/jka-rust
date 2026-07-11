use core::ffi::c_int;

use super::super::MpCgameImport;
use abi_transport::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_FX_ADJUST_TIME`.
///
/// Raven wrapper: `syscall( CG_FX_ADJUST_TIME, time );`
/// Raven transport: `FX_AdjustTime(args[1]); return 0;`
///
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:689-691`
/// Args source: `oracle/codemp/cgame/cg_local.h:2408`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:1159-1161`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgFxAdjustTimeArgs {
    time: c_int,
}

impl CgFxAdjustTimeArgs {
    pub const fn new(time: c_int) -> Self {
        Self { time }
    }
}

/// `CG_FX_ADJUST_TIME` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/codemp/cgame/cg_public.h:230`
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:689-691`
/// Output source: `oracle/codemp/client/cl_cgame.cpp:1159-1161`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:1159-1161`
pub struct CgFxAdjustTime;

impl OutboundSysCall for CgFxAdjustTime {
    type Import = MpCgameImport;
    type Args = CgFxAdjustTimeArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_FX_ADJUST_TIME;
}

impl EncodeSysCall for CgFxAdjustTime {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.time as isize])
    }
}

impl DecodeSysCallReturn for CgFxAdjustTime {
    fn decode_return(_word: isize) -> Self::Output {}
}
