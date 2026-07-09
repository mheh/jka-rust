use core::ffi::c_int;

use super::super::MpUiImport;
use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// Arguments for `UI_ARGC`.
///
/// Raven's `trap_Argc` forwards only the syscall token, so this call has no
/// transport payload.
///
/// Args source: `oracle/codemp/ui/ui_syscalls.c:71`
/// Transport source: `oracle/codemp/ui/ui_syscalls.c:72`
/// Transport/switch source: `oracle/codemp/client/cl_ui.cpp:728`
#[derive(Debug, Default)]
pub struct CgArgcArgs;

impl CgArgcArgs {
    pub const fn new() -> Self {
        Self
    }
}

/// `UI_ARGC` MP cgame imports syscall ABI token.
///
/// Returns the number of tokens in the current command string.
/// C signature: `int trap_Argc( void )`
///
/// Raven: ( void );
/// Raven: ClientCommand and ServerCommand parameter access
///
/// Enum value source: `oracle/codemp/ui/ui_public.h:70`
/// Output source: `oracle/codemp/client/cl_ui.cpp:729`
/// Transport/switch source: `oracle/codemp/client/cl_ui.cpp:728`
pub struct CgArgc;

impl OutboundSysCall for CgArgc {
    type Import = MpUiImport;
    type Args = CgArgcArgs;
    type Output = c_int;

    const IMPORT: MpUiImport = MpUiImport::UI_ARGC;
}

impl EncodeSysCall for CgArgc {
    fn encode_syscall(_args: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for CgArgc {
    // `Cmd_Argc` returns an `int`; the engine's return word is that value.
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
