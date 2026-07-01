use core::ffi::c_char;

use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_REMOVECOMMAND`.
///
/// Raven wrapper: `syscall( CG_REMOVECOMMAND, cmdName );`
/// Raven transport: `Cmd_RemoveCommand( (const char *)VMA(1) ); return 0;`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:111-112`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:757-759`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgRemovecommandArgs {
    cmd_name: *const c_char,
}

impl CgRemovecommandArgs {
    pub const fn new(cmd_name: *const c_char) -> Self {
        Self { cmd_name }
    }
}

/// `CG_REMOVECOMMAND` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:80`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:111-112`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:757-759`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:757-759`
pub struct CgRemovecommand;

impl OutboundSysCall for CgRemovecommand {
    type Import = MpCgameImport;
    type Args = CgRemovecommandArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_REMOVECOMMAND;
}

impl EncodeSysCall for CgRemovecommand {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.cmd_name)])
    }
}

impl DecodeSysCallReturn for CgRemovecommand {
    fn decode_return(_word: isize) -> Self::Output {}
}
