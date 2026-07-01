use core::ffi::c_void;

use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use mp_qshared::shared::qboolean;

/// Arguments for `CG_G2_ABSURDSMOOTHING`.
///
/// Raven wrapper: `syscall(CG_G2_ABSURDSMOOTHING, ghoul2, status);`
/// Raven transport: `G2API_AbsurdSmoothing(g2, (qboolean)args[2]); return 0;`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:992-994`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2568`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1527-1533`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgG2AbsurdsmoothingArgs {
    ghoul2: *mut c_void,
    status: qboolean,
}

impl CgG2AbsurdsmoothingArgs {
    pub const fn new(ghoul2: *mut c_void, status: qboolean) -> Self {
        Self { ghoul2, status }
    }
}

/// `CG_G2_ABSURDSMOOTHING` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:295`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:992-994`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1527-1533`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1527-1533`
pub struct CgG2Absurdsmoothing;

impl OutboundSysCall for CgG2Absurdsmoothing {
    type Import = MpCgameImport;
    type Args = CgG2AbsurdsmoothingArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_G2_ABSURDSMOOTHING;
}

impl EncodeSysCall for CgG2Absurdsmoothing {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.ghoul2), args.status as isize])
    }
}

impl DecodeSysCallReturn for CgG2Absurdsmoothing {
    fn decode_return(_word: isize) -> Self::Output {}
}
