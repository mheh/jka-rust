use core::ffi::c_int;

use super::super::MpUiImport;
use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// Arguments for `UI_PC_FREE_SOURCE`.
///
/// Raven wrapper: `syscall( UI_PC_FREE_SOURCE, handle );`
/// Raven transport: `return botlib_export->PC_FreeSourceHandle( args[1] );`
///
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:370-371`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1161-1162`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiPcFreeSourceArgs {
    handle: c_int,
}

impl UiPcFreeSourceArgs {
    pub const fn new(handle: c_int) -> Self {
        Self { handle }
    }

    pub const fn handle(&self) -> c_int {
        self.handle
    }
}

/// `UI_PC_FREE_SOURCE` MP UI imports syscall ABI token.
///
/// Raven wrapper: `int trap_PC_FreeSource( int handle ) { return syscall( UI_PC_FREE_SOURCE, handle ); }`
/// Raven transport: `return botlib_export->PC_FreeSourceHandle( args[1] );`
///
/// Enum value source: `oracle/oracle/codemp/ui/ui_public.h:86`
/// Enum comment source: `oracle/oracle/codemp/ui/ui_public.h:82-90`
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:370-371`
/// Output source: `oracle/oracle/codemp/client/cl_ui.cpp:1161-1162`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1161-1162`
pub struct UiPcFreeSource;

impl OutboundSysCall for UiPcFreeSource {
    type Import = MpUiImport;
    type Args = UiPcFreeSourceArgs;
    type Output = c_int;

    const IMPORT: MpUiImport = MpUiImport::UI_PC_FREE_SOURCE;
}

impl EncodeSysCall for UiPcFreeSource {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.handle() as isize])
    }
}

impl DecodeSysCallReturn for UiPcFreeSource {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
