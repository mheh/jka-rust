use core::ffi::c_char;

use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_ADDCOMMAND`.
///
/// Raven: register a command name so the console can perform command
/// completion.
/// Raven wrapper: `syscall( CG_ADDCOMMAND, cmdName );`
/// Raven transport: `CL_AddCgameCommand( (const char *)VMA(1) ); return 0;`
///
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:107-108`
/// Args source: `oracle/codemp/cgame/cg_local.h:2186-2188`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:754-756`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgAddcommandArgs {
    cmd_name: *const c_char,
}

impl CgAddcommandArgs {
    pub const fn new(cmd_name: *const c_char) -> Self {
        Self { cmd_name }
    }
}

/// `CG_ADDCOMMAND` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/codemp/cgame/cg_public.h:79`
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:107-108`
/// Output source: `oracle/codemp/client/cl_cgame.cpp:754-756`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:754-756`
pub struct CgAddcommand;

impl OutboundSysCall for CgAddcommand {
    type Import = MpCgameImport;
    type Args = CgAddcommandArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_ADDCOMMAND;
}

impl EncodeSysCall for CgAddcommand {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.cmd_name)])
    }
}

impl DecodeSysCallReturn for CgAddcommand {
    fn decode_return(_word: isize) -> Self::Output {}
}
