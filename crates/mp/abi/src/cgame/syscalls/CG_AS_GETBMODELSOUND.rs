use core::ffi::{c_char, c_int};

use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_AS_GETBMODELSOUND`.
///
/// Raven wrapper: `syscall(CG_AS_GETBMODELSOUND, name, stage)`.
/// Raven transport: `AS_GetBModelSound((const char *)VMA(1), args[2])`.
///
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:257-259`
/// Args source: `oracle/codemp/cgame/cg_local.h:2243`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:857-858`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgAsGetbmodelsoundArgs {
    name: *const c_char,
    stage: c_int,
}

impl CgAsGetbmodelsoundArgs {
    pub const fn new(name: *const c_char, stage: c_int) -> Self {
        Self { name, stage }
    }
}

/// `CG_AS_GETBMODELSOUND` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/codemp/cgame/cg_public.h:114`
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:257-259`
/// Output source: `oracle/codemp/cgame/cg_local.h:2243`
/// Output source: `oracle/codemp/client/cl_cgame.cpp:857-858`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:857-858`
pub struct CgAsGetbmodelsound;

impl OutboundSysCall for CgAsGetbmodelsound {
    type Import = MpCgameImport;
    type Args = CgAsGetbmodelsoundArgs;
    type Output = c_int;

    const IMPORT: MpCgameImport = MpCgameImport::CG_AS_GETBMODELSOUND;
}

impl EncodeSysCall for CgAsGetbmodelsound {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.name), args.stage as isize])
    }
}

impl DecodeSysCallReturn for CgAsGetbmodelsound {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
