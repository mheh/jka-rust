use core::ffi::c_char;

use super::super::MpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_SENDCONSOLECOMMAND`.
///
/// Raven wrapper: `syscall( CG_SENDCONSOLECOMMAND, text );`
/// Raven transport: `Cbuf_AddText( (const char *)VMA(1) ); return 0;`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:103-104`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2184`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:751-753`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgSendconsolecommandArgs {
    text: *const c_char,
}

impl CgSendconsolecommandArgs {
    pub const fn new(text: *const c_char) -> Self {
        Self { text }
    }
}

/// `CG_SENDCONSOLECOMMAND` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:78`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:103-104`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:751-753`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:751-753`
pub struct CgSendconsolecommand;

impl OutboundSysCall for CgSendconsolecommand {
    type Import = MpCgameImport;
    type Args = CgSendconsolecommandArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_SENDCONSOLECOMMAND;
}

impl EncodeSysCall for CgSendconsolecommand {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.text)])
    }
}

impl DecodeSysCallReturn for CgSendconsolecommand {
    fn decode_return(_word: isize) -> Self::Output {}
}
