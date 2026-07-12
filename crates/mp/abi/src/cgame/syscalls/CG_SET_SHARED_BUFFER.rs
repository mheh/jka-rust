use core::ffi::c_char;

use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_SET_SHARED_BUFFER`.
///
/// Raven wrapper: `syscall(CG_SET_SHARED_BUFFER, memory);`
/// Raven transport: `cl.mSharedMemory = ((char *)VMA(1)); return 0;`
///
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:1085-1087`
/// Args source: `oracle/codemp/cgame/cg_main.c:3713`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:1682-1684`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgSetSharedBufferArgs {
    memory: *mut c_char,
}

impl CgSetSharedBufferArgs {
    pub const fn new(memory: *mut c_char) -> Self {
        Self { memory }
    }
}

/// `CG_SET_SHARED_BUFFER` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/codemp/cgame/cg_public.h:328`
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:1085-1087`
/// Output source: `oracle/codemp/client/cl_cgame.cpp:1682-1684`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:1682-1684`
pub struct CgSetSharedBuffer;

impl OutboundSysCall for CgSetSharedBuffer {
    type Import = MpCgameImport;
    type Args = CgSetSharedBufferArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_SET_SHARED_BUFFER;
}

impl EncodeSysCall for CgSetSharedBuffer {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.memory)])
    }
}

impl DecodeSysCallReturn for CgSetSharedBuffer {
    fn decode_return(_word: isize) -> Self::Output {}
}
