use core::ffi::{c_int, c_void};

use super::super::MpCgameImport;
use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_TRUEMALLOC`.
///
/// Raven: dynamic vm memory allocation.
/// Raven wrapper: `syscall(CG_TRUEMALLOC, ptr, size);`
/// Raven transport: `VM_Shifted_Alloc((void **)VMA(1), args[2]); return 0;`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:756-759`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2436-2437`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1284-1287`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgTruemallocArgs {
    ptr: *mut *mut c_void,
    size: c_int,
}

impl CgTruemallocArgs {
    pub const fn new(ptr: *mut *mut c_void, size: c_int) -> Self {
        Self { ptr, size }
    }
}

/// `CG_TRUEMALLOC` MP cgame imports syscall boundary token.
///
/// Raven: rww - dynamic vm memory allocation!
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:249-250`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:756-759`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1284-1287`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1284-1287`
pub struct CgTruemalloc;

impl OutboundSysCall for CgTruemalloc {
    type Import = MpCgameImport;
    type Args = CgTruemallocArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_TRUEMALLOC;
}

impl EncodeSysCall for CgTruemalloc {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.ptr), args.size as isize])
    }
}

impl DecodeSysCallReturn for CgTruemalloc {
    fn decode_return(_word: isize) -> Self::Output {}
}
