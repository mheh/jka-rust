use core::ffi::c_char;

use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_AS_ADDPRECACHEENTRY`.
///
/// Raven wrapper: `syscall(CG_AS_ADDPRECACHEENTRY, name);`
/// Raven transport: `AS_AddPrecacheEntry((const char *)VMA(1)); return 0;`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:247-249`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2241`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:852-854`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgAsAddprecacheentryArgs {
    name: *const c_char,
}

impl CgAsAddprecacheentryArgs {
    pub const fn new(name: *const c_char) -> Self {
        Self { name }
    }
}

/// `CG_AS_ADDPRECACHEENTRY` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:112`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:247-249`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:852-854`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:852-854`
pub struct CgAsAddprecacheentry;

impl OutboundSysCall for CgAsAddprecacheentry {
    type Import = MpCgameImport;
    type Args = CgAsAddprecacheentryArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_AS_ADDPRECACHEENTRY;
}

impl EncodeSysCall for CgAsAddprecacheentry {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.name)])
    }
}

impl DecodeSysCallReturn for CgAsAddprecacheentry {
    fn decode_return(_word: isize) -> Self::Output {}
}
