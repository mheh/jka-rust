use core::ffi::{c_int, c_void};

use super::super::SpCgameImport;
use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};
use crate::cgame::types::memtag_t;

/// Arguments for `CG_Z_MALLOC`.
///
/// Raven wrapper: `return (void *)syscall(CG_Z_MALLOC,size,tag);`
/// Raven transport: `return (int)Z_Malloc(args[1], (memtag_t) args[2], qfalse);`
///
/// Args source: `oracle/code/cgame/cg_syscalls.cpp:548-550`
/// Output source: `oracle/code/client/cl_cgame.cpp:834-835`
/// Transport/switch source: `oracle/code/client/cl_cgame.cpp:834-835`
/// Type definition source: `oracle/code/game/q_shared.h:2688`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgZMallocArgs {
    size: c_int,
    tag: memtag_t,
}

impl CgZMallocArgs {
    pub const fn new(size: c_int, tag: memtag_t) -> Self {
        Self { size, tag }
    }

    pub const fn size(&self) -> c_int {
        self.size
    }

    pub const fn tag(&self) -> memtag_t {
        self.tag
    }
}

/// `CG_Z_MALLOC` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/code/cgame/cg_public.h:190`
/// Args source: `oracle/code/cgame/cg_syscalls.cpp:548-550`
/// Output source: `oracle/code/client/cl_cgame.cpp:834-835`
/// Transport/switch source: `oracle/code/client/cl_cgame.cpp:834-835`
/// Type definition source: `oracle/code/game/q_shared.h:2688`
pub struct CgZMalloc;

impl OutboundSysCall for CgZMalloc {
    type Import = SpCgameImport;
    type Args = CgZMallocArgs;
    type Output = *mut c_void;

    const IMPORT: SpCgameImport = SpCgameImport::CG_Z_MALLOC;
}

impl EncodeSysCall for CgZMalloc {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.size() as isize, args.tag().as_wire() as isize])
    }
}

impl DecodeSysCallReturn for CgZMalloc {
    fn decode_return(word: isize) -> Self::Output {
        word as *mut c_void
    }
}
