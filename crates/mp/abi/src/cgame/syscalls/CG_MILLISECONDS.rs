use core::ffi::c_int;

use super::super::MpCgameImport;
use abi_transport::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_MILLISECONDS`.
///
/// `trap_Milliseconds` takes no arguments; the transport carries no payload
/// words after the import token.
///
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:29`
/// Transport source: `oracle/codemp/cgame/cg_syscalls.c:30`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:694`
#[derive(Debug, Default)]
pub struct CgMillisecondsArgs;

impl CgMillisecondsArgs {
    pub const fn new() -> Self {
        Self
    }
}

/// `CG_MILLISECONDS` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/codemp/cgame/cg_public.h:59`
/// Output source: `oracle/codemp/cgame/cg_syscalls.c:29`
/// Output source: `oracle/codemp/cgame/cg_syscalls.c:30`
/// Output source: `oracle/codemp/client/cl_cgame.cpp:695`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:694`
pub struct CgMilliseconds;

impl OutboundSysCall for CgMilliseconds {
    type Import = MpCgameImport;
    type Args = CgMillisecondsArgs;
    type Output = c_int;

    const IMPORT: MpCgameImport = MpCgameImport::CG_MILLISECONDS;
}

impl EncodeSysCall for CgMilliseconds {
    fn encode_syscall(_args: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for CgMilliseconds {
    // `trap_Milliseconds` returns `int`; the engine's return word is that value.
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
