use core::ffi::c_void;

use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_TRUEFREE`.
///
/// Raven: dynamic vm memory allocation.
/// Raven wrapper: `syscall(CG_TRUEFREE, ptr);`
/// Raven transport: `VM_Shifted_Free((void **)VMA(1)); return 0;`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:762-764`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2436-2438`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1288-1290`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgTruefreeArgs {
    ptr: *mut *mut c_void,
}

impl CgTruefreeArgs {
    pub const fn new(ptr: *mut *mut c_void) -> Self {
        Self { ptr }
    }
}

/// `CG_TRUEFREE` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:251`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:762-764`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1288-1290`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1288-1290`
pub struct CgTruefree;

impl OutboundSysCall for CgTruefree {
    type Import = MpCgameImport;
    type Args = CgTruefreeArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_TRUEFREE;
}

impl EncodeSysCall for CgTruefree {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.ptr)])
    }
}

impl DecodeSysCallReturn for CgTruefree {
    fn decode_return(_word: isize) -> Self::Output {}
}
