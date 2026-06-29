use core::ffi::{c_int, c_void};

use super::super::SpCgameImport;
use crate::abi::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_Z_MALLOC`.
///
/// Raven wrapper: `return (void *)syscall(CG_Z_MALLOC,size,tag);`
/// Raven transport: `return (int)Z_Malloc(args[1], (memtag_t) args[2], qfalse);`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:548-550`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:834-835`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:834-835`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgZMallocArgs {
    size: c_int,
    // FIXME: create type `memtag_t` (Raven typedef: `oracle/oracle/code/game/q_shared.h:2688`).
    tag: c_int,
}

impl CgZMallocArgs {
    pub const fn new(size: c_int, tag: c_int) -> Self {
        Self { size, tag }
    }

    pub const fn size(&self) -> c_int {
        self.size
    }

    pub const fn tag(&self) -> c_int {
        self.tag
    }
}

/// `CG_Z_MALLOC` SP cgame imports syscall ABI token.
///
/// FIXME: create type `memtag_t` for the second argument; using `c_int` preserves
/// the existing syscall word transport until the shared Raven type exists.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:190`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:548-550`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:834-835`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:834-835`
pub struct CgZMalloc;

impl OutboundSysCall for CgZMalloc {
    type Import = SpCgameImport;
    type Args = CgZMallocArgs;
    type Output = *mut c_void;

    const IMPORT: SpCgameImport = SpCgameImport::CG_Z_MALLOC;
}

impl EncodeSysCall for CgZMalloc {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.size() as isize, args.tag() as isize])
    }
}

impl DecodeSysCallReturn for CgZMalloc {
    fn decode_return(word: isize) -> Self::Output {
        word as *mut c_void
    }
}
