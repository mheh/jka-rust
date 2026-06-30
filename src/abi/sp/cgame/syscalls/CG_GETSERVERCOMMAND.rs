use core::ffi::c_int;

use super::super::SpCgameImport;
use crate::abi::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};
use crate::ffi::types::qboolean;

/// Arguments for `CG_GETSERVERCOMMAND`.
///
/// Raven wrapper: `return syscall( CG_GETSERVERCOMMAND, serverCommandNumber );`
/// Raven transport: `return CL_GetServerCommand(args[1]);`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:463-464`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:764-765`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgGetservercommandArgs {
    server_command_number: c_int,
}

impl CgGetservercommandArgs {
    pub const fn new(server_command_number: c_int) -> Self {
        Self {
            server_command_number,
        }
    }
}

/// `CG_GETSERVERCOMMAND` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:157`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:463-464`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:764-765`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:764-765`
pub struct CgGetservercommand;

impl OutboundSysCall for CgGetservercommand {
    type Import = SpCgameImport;
    type Args = CgGetservercommandArgs;
    type Output = qboolean;

    const IMPORT: SpCgameImport = SpCgameImport::CG_GETSERVERCOMMAND;
}

impl EncodeSysCall for CgGetservercommand {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.server_command_number as isize])
    }
}

impl DecodeSysCallReturn for CgGetservercommand {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
