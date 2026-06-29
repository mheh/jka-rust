use core::ffi::{c_char, c_int};

use super::super::SpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_S_REGISTERSOUND`.
///
/// Raven wrapper: `return syscall( CG_S_REGISTERSOUND, sample );`
/// Raven transport: `return S_RegisterSound( (const char *) VMA(1) );`
///
/// Raven comment: returns buzz if not found.
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:229-230`
/// Output type source: `oracle/oracle/code/game/q_shared.h:186`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:605-606`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgSRegistersoundArgs {
    sample: *const c_char,
}

impl CgSRegistersoundArgs {
    pub const fn new(sample: *const c_char) -> Self {
        Self { sample }
    }

    pub const fn sample(&self) -> *const c_char {
        self.sample
    }
}

/// `CG_S_REGISTERSOUND` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:98`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:229-230`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:605-606`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:605-606`
pub struct CgSRegistersound;

impl OutboundSysCall for CgSRegistersound {
    type Import = SpCgameImport;
    type Args = CgSRegistersoundArgs;
    type Output = c_int;

    const IMPORT: SpCgameImport = SpCgameImport::CG_S_REGISTERSOUND;
}

impl EncodeSysCall for CgSRegistersound {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.sample())])
    }
}

impl DecodeSysCallReturn for CgSRegistersound {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
