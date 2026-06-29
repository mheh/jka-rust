use core::ffi::c_void;

use super::super::MpCgameImport;
use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::ffi::types::qboolean;

/// Arguments for `CG_G2_OVERRIDESERVER`.
///
/// Raven wrapper: `return syscall(CG_G2_OVERRIDESERVER, serverInstance);`
/// Raven transport: `return G2API_OverrideServerWithClientData(&g2[0]);`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:1075-1077`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2590`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1631-1635`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgG2OverrideserverArgs {
    server_instance: *mut c_void,
}

impl CgG2OverrideserverArgs {
    pub const fn new(server_instance: *mut c_void) -> Self {
        Self { server_instance }
    }
}

/// `CG_G2_OVERRIDESERVER` MP cgame imports syscall boundary token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:324`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:1075-1077`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1631-1635`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1631-1635`
pub struct CgG2Overrideserver;

impl OutboundSysCall for CgG2Overrideserver {
    type Import = MpCgameImport;
    type Args = CgG2OverrideserverArgs;
    type Output = qboolean;

    const IMPORT: MpCgameImport = MpCgameImport::CG_G2_OVERRIDESERVER;
}

impl EncodeSysCall for CgG2Overrideserver {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.server_instance)])
    }
}

impl DecodeSysCallReturn for CgG2Overrideserver {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
