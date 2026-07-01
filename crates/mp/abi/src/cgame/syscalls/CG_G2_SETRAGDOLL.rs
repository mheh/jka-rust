use core::ffi::c_void;

use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use mp_qshared::common::mp::qcommon::sharedRagDollParams_t;

/// Arguments for `CG_G2_SETRAGDOLL`.
///
/// Raven wrapper: `syscall(CG_G2_SETRAGDOLL, ghoul2, params);`
/// Raven transport: `sharedRagDollParams_t *rdParamst = (sharedRagDollParams_t *)VMA(2);`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:998-1000`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2570`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1536-1569`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgG2SetragdollArgs {
    ghoul2: *mut c_void,
    params: *mut sharedRagDollParams_t,
}

impl CgG2SetragdollArgs {
    pub const fn new(ghoul2: *mut c_void, params: *mut sharedRagDollParams_t) -> Self {
        Self { ghoul2, params }
    }
}

/// `CG_G2_SETRAGDOLL` MP cgame imports syscall ABI token.
///
/// Raven: rww - RAGDOLL_BEGIN
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:297-300`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:998-1000`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1536-1569`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1536-1569`
pub struct CgG2Setragdoll;

impl OutboundSysCall for CgG2Setragdoll {
    type Import = MpCgameImport;
    type Args = CgG2SetragdollArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_G2_SETRAGDOLL;
}

impl EncodeSysCall for CgG2Setragdoll {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.ghoul2), ptr_to_word(args.params)])
    }
}

impl DecodeSysCallReturn for CgG2Setragdoll {
    fn decode_return(_word: isize) -> Self::Output {}
}
