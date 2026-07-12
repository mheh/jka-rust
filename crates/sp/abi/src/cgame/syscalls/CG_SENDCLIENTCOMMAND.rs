use core::ffi::c_char;

use super::super::SpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_SENDCLIENTCOMMAND`.
///
/// Raven wrapper: `syscall( CG_SENDCLIENTCOMMAND, s );`
/// Raven transport: `CL_AddReliableCommand( (const char *) VMA(1) );`
///
/// Args source: `oracle/code/cgame/cg_syscalls.cpp:106-108`
/// Transport/switch source: `oracle/code/client/cl_cgame.cpp:479-481`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgSendclientcommandArgs {
    text: *const c_char,
}

impl CgSendclientcommandArgs {
    /// # Safety
    /// `text` must point to a valid NUL-terminated C string.
    pub const unsafe fn new(text: *const c_char) -> Self {
        Self { text }
    }
}

/// `CG_SENDCLIENTCOMMAND` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/code/cgame/cg_public.h:76`
/// Args source: `oracle/code/cgame/cg_syscalls.cpp:106-108`
/// Output source: `oracle/code/client/cl_cgame.cpp:479-481`
/// Transport/switch source: `oracle/code/client/cl_cgame.cpp:479-481`
pub struct CgSendclientcommand;

impl OutboundSysCall for CgSendclientcommand {
    type Import = SpCgameImport;
    type Args = CgSendclientcommandArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_SENDCLIENTCOMMAND;
}

impl EncodeSysCall for CgSendclientcommand {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.text)])
    }
}

impl DecodeSysCallReturn for CgSendclientcommand {
    fn decode_return(_word: isize) -> Self::Output {}
}
