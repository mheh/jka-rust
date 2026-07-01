use core::ffi::{c_char, c_int, c_void};

use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_G2_GETGLANAME`.
///
/// Raven wrapper: `syscall(CG_G2_GETGLANAME, ghoul2, modelIndex, fillBuf);`
/// Raven transport copies `G2API_GetGLAName` into the caller-provided buffer.
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:885-887`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2552`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1406-1418`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgG2GetglanameArgs {
    ghoul2: *mut c_void,
    model_index: c_int,
    fill_buf: *mut c_char,
}

impl CgG2GetglanameArgs {
    pub const fn new(ghoul2: *mut c_void, model_index: c_int, fill_buf: *mut c_char) -> Self {
        Self {
            ghoul2,
            model_index,
            fill_buf,
        }
    }
}

/// `CG_G2_GETGLANAME` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:272`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:885-887`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1406-1418`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1406-1418`
pub struct CgG2Getglaname;

impl OutboundSysCall for CgG2Getglaname {
    type Import = MpCgameImport;
    type Args = CgG2GetglanameArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_G2_GETGLANAME;
}

impl EncodeSysCall for CgG2Getglaname {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.ghoul2),
            args.model_index as isize,
            ptr_to_word(args.fill_buf),
        ])
    }
}

impl DecodeSysCallReturn for CgG2Getglaname {
    fn decode_return(_word: isize) -> Self::Output {}
}
