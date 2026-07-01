use core::ffi::c_char;

use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_SENDCLIENTCOMMAND`.
///
/// Raven wrapper: `void trap_SendClientCommand( const char *s )`.
/// The MP client switch decodes the command string through `VMA(1)`.
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:115-116`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:760-762`
#[derive(Debug)]
pub struct CgSendclientcommandArgs {
    command: *const c_char,
}

impl CgSendclientcommandArgs {
    /// Construct raw `trap_SendClientCommand` syscall args.
    ///
    /// # Safety
    /// `command` must point to a valid NUL-terminated C string for the duration
    /// of the syscall.
    pub const unsafe fn new(command: *const c_char) -> Self {
        Self { command }
    }

    pub const fn command(&self) -> *const c_char {
        self.command
    }
}

/// `CG_SENDCLIENTCOMMAND` MP cgame imports syscall ABI token.
///
/// Raven wrapper: `syscall( CG_SENDCLIENTCOMMAND, s );`
/// Raven transport: `CL_AddReliableCommand( (const char *)VMA(1) );`
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:81`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:115-116`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:762`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:760-762`
pub struct CgSendclientcommand;

impl OutboundSysCall for CgSendclientcommand {
    type Import = MpCgameImport;
    type Args = CgSendclientcommandArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_SENDCLIENTCOMMAND;
}

impl EncodeSysCall for CgSendclientcommand {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.command())])
    }
}

impl DecodeSysCallReturn for CgSendclientcommand {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
