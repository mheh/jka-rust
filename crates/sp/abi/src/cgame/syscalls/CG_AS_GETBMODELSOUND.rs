use core::ffi::{c_char, c_int};

use super::super::SpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_AS_GETBMODELSOUND`.
///
/// Raven wrapper: `return syscall( CG_AS_GETBMODELSOUND, name, stage );`
/// Raven transport: `return AS_GetBModelSound((const char *) VMA(1), args[2]);`
///
/// Args source: `oracle/code/cgame/cg_syscalls.cpp:205-206`
/// Args source: `oracle/code/game/q_shared.h:186`
/// Transport/switch source: `oracle/code/client/cl_cgame.cpp:578-579`
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

/// `CG_AS_GETBMODELSOUND` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/code/cgame/cg_public.h:166`
/// Args source: `oracle/code/cgame/cg_syscalls.cpp:205-206`
/// Args source: `oracle/code/game/q_shared.h:186`
/// Output source: `oracle/code/client/cl_cgame.cpp:578-579`
/// Transport/switch source: `oracle/code/client/cl_cgame.cpp:578-579`
pub struct CgAsGetbmodelsound;

impl OutboundSysCall for CgAsGetbmodelsound {
    type Import = SpCgameImport;
    type Args = CgAsGetbmodelsoundArgs;
    type Output = c_int;

    const IMPORT: SpCgameImport = SpCgameImport::CG_AS_GETBMODELSOUND;
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
