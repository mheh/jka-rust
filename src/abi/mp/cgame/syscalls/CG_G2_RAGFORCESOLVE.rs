use core::ffi::c_void;

use super::super::MpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::ffi::types::qboolean;

/// Arguments for `CG_G2_RAGFORCESOLVE`.
///
/// Raven wrapper: `return syscall(CG_G2_RAGFORCESOLVE, ghoul2, force);`
/// Raven transport: `return G2API_RagForceSolve(*((CGhoul2Info_v *)args[1]), (qboolean)args[2]);`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:1035-1037`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2579`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1605-1606`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgG2RagforcesolveArgs {
    ghoul2: *mut c_void,
    force: qboolean,
}

impl CgG2RagforcesolveArgs {
    pub const fn new(ghoul2: *mut c_void, force: qboolean) -> Self {
        Self { ghoul2, force }
    }
}

/// `CG_G2_RAGFORCESOLVE` MP cgame imports syscall ABI token.
///
/// Raven: make sure we are actively performing solve/settle routines, if desired
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:312`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:1035-1037`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1605-1606`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1605-1606`
pub struct CgG2Ragforcesolve;

impl OutboundSysCall for CgG2Ragforcesolve {
    type Import = MpCgameImport;
    type Args = CgG2RagforcesolveArgs;
    type Output = qboolean;

    const IMPORT: MpCgameImport = MpCgameImport::CG_G2_RAGFORCESOLVE;
}

impl EncodeSysCall for CgG2Ragforcesolve {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.ghoul2), args.force as isize])
    }
}

impl DecodeSysCallReturn for CgG2Ragforcesolve {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
