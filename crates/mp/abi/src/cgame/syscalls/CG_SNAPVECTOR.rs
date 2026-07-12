use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use mp_qshared::shared::vec3_t;

/// Arguments for `CG_SNAPVECTOR`.
///
/// Raven wrapper: `syscall( CG_SNAPVECTOR, v );`
/// Raven transport: `Sys_SnapVector( (float *)VMA(1) ); return 0;`
///
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:579-580`
/// Args source: `oracle/codemp/cgame/cg_local.h:2387`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:1021-1023`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgSnapvectorArgs {
    v: *mut vec3_t,
}

impl CgSnapvectorArgs {
    pub const fn new(v: *mut vec3_t) -> Self {
        Self { v }
    }
}

/// `CG_SNAPVECTOR` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/codemp/cgame/cg_public.h:209`
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:579-580`
/// Output source: `oracle/codemp/client/cl_cgame.cpp:1021-1023`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:1021-1023`
pub struct CgSnapvector;

impl OutboundSysCall for CgSnapvector {
    type Import = MpCgameImport;
    type Args = CgSnapvectorArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_SNAPVECTOR;
}

impl EncodeSysCall for CgSnapvector {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.v)])
    }
}

impl DecodeSysCallReturn for CgSnapvector {
    fn decode_return(_word: isize) -> Self::Output {}
}
