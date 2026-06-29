use core::ffi::c_char;

use super::super::SpCgameImport;
use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_SENDCONSOLECOMMAND`.
///
/// Raven wrapper: `syscall( CG_SENDCONSOLECOMMAND, text );`
/// Raven transport: `Cbuf_AddText( (const char *) VMA(1) );`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:98-100`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:473-475`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgSendconsolecommandArgs {
    text: *const c_char,
}

impl CgSendconsolecommandArgs {
    /// # Safety
    /// `text` must point to a valid NUL-terminated C string.
    pub const unsafe fn new(text: *const c_char) -> Self {
        Self { text }
    }
}

/// `CG_SENDCONSOLECOMMAND` SP cgame imports syscall boundary token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:74`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:98-100`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:473-475`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:473-475`
pub struct CgSendconsolecommand;

impl OutboundSysCall for CgSendconsolecommand {
    type Import = SpCgameImport;
    type Args = CgSendconsolecommandArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_SENDCONSOLECOMMAND;
}

impl EncodeSysCall for CgSendconsolecommand {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.text)])
    }
}

impl DecodeSysCallReturn for CgSendconsolecommand {
    fn decode_return(_word: isize) -> Self::Output {}
}
