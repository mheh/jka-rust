use core::ffi::c_char;

use super::super::SpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_AS_ADDENTRY`.
///
/// Raven wrapper: `syscall( CG_AS_ADDENTRY, name );`
/// Raven transport: `AS_AddPrecacheEntry((const char *) VMA(1));`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:193-194`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:575-577`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgAsAddentryArgs {
    name: *const c_char,
}

impl CgAsAddentryArgs {
    pub const fn new(name: *const c_char) -> Self {
        Self { name }
    }
}

/// `CG_AS_ADDENTRY` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:165`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:193-194`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:575-577`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:575-577`
pub struct CgAsAddentry;

impl OutboundSysCall for CgAsAddentry {
    type Import = SpCgameImport;
    type Args = CgAsAddentryArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_AS_ADDENTRY;
}

impl EncodeSysCall for CgAsAddentry {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.name)])
    }
}

impl DecodeSysCallReturn for CgAsAddentry {
    fn decode_return(_word: isize) -> Self::Output {}
}
