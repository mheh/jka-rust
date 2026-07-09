use core::ffi::c_void;

use super::super::SpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_Z_FREE`.
///
/// Raven wrapper: `syscall(CG_Z_FREE,ptr);`
/// Raven transport: `Z_Free((void *) VMA(1)); return 0;`
///
/// Args source: `oracle/code/cgame/cg_syscalls.cpp:553-555`
/// Output source: `oracle/code/client/cl_cgame.cpp:837-839`
/// Transport/switch source: `oracle/code/client/cl_cgame.cpp:837-839`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgZFreeArgs {
    ptr: *mut c_void,
}

impl CgZFreeArgs {
    pub const fn new(ptr: *mut c_void) -> Self {
        Self { ptr }
    }

    pub const fn ptr(&self) -> *mut c_void {
        self.ptr
    }
}

/// `CG_Z_FREE` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/code/cgame/cg_public.h:191`
/// Args source: `oracle/code/cgame/cg_syscalls.cpp:553-555`
/// Output source: `oracle/code/client/cl_cgame.cpp:837-839`
/// Transport/switch source: `oracle/code/client/cl_cgame.cpp:837-839`
pub struct CgZFree;

impl OutboundSysCall for CgZFree {
    type Import = SpCgameImport;
    type Args = CgZFreeArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_Z_FREE;
}

impl EncodeSysCall for CgZFree {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.ptr())])
    }
}

impl DecodeSysCallReturn for CgZFree {
    fn decode_return(_word: isize) -> Self::Output {}
}
