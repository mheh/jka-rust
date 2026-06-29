use core::ffi::c_char;

use super::super::SpCgameImport;
use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_ADDCOMMAND`.
///
/// Raven wrapper: `syscall( CG_ADDCOMMAND, cmdName );`
/// Raven transport: `CL_AddCgameCommand( (const char *) VMA(1) );`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:102-104`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:476-478`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgAddcommandArgs {
    cmd_name: *const c_char,
}

impl CgAddcommandArgs {
    pub const fn new(cmd_name: *const c_char) -> Self {
        Self { cmd_name }
    }
}

/// `CG_ADDCOMMAND` SP cgame imports syscall boundary token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:75`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:102-104`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:476-478`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:476-478`
pub struct CgAddcommand;

impl OutboundSysCall for CgAddcommand {
    type Import = SpCgameImport;
    type Args = CgAddcommandArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_ADDCOMMAND;
}

impl EncodeSysCall for CgAddcommand {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.cmd_name)])
    }
}

impl DecodeSysCallReturn for CgAddcommand {
    fn decode_return(_word: isize) -> Self::Output {}
}
