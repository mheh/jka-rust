use core::ffi::c_int;

use super::super::MpCgameImport;
use crate::boundary::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_GETSERVERCOMMAND`.
///
/// C ABI: `qboolean trap_GetServerCommand(int serverCommandNumber)`.
/// Raven's wrapper forwards the server command number as the only payload word,
/// and the client switch reads it from `args[1]`.
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:482-483`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:967-968`
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CgGetservercommandArgs {
    /// Server command sequence number, read by Raven as `args[1]`.
    server_command_number: c_int,
}

impl CgGetservercommandArgs {
    pub const fn new(server_command_number: c_int) -> Self {
        Self {
            server_command_number,
        }
    }

    pub const fn server_command_number(&self) -> c_int {
        self.server_command_number
    }
}

/// `CG_GETSERVERCOMMAND` MP cgame imports syscall boundary token.
///
/// Raven wrapper: `return syscall( CG_GETSERVERCOMMAND, serverCommandNumber );`
/// Raven transport: `return CL_GetServerCommand( args[1] );`
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:184`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:482-483`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:967-968`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:967-968`
pub struct CgGetservercommand;

impl OutboundSysCall for CgGetservercommand {
    type Import = MpCgameImport;
    type Args = CgGetservercommandArgs;
    type Output = c_int;

    const IMPORT: MpCgameImport = MpCgameImport::CG_GETSERVERCOMMAND;
}

impl EncodeSysCall for CgGetservercommand {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.server_command_number() as isize])
    }
}

impl DecodeSysCallReturn for CgGetservercommand {
    // `qboolean` is an int-compatible Raven return value in the syscall word.
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
