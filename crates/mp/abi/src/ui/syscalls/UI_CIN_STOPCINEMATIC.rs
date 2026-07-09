use core::ffi::c_int;

use super::super::MpUiImport;
use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// Arguments for `UI_CIN_STOPCINEMATIC`.
///
/// Raven: stops playing the cinematic and ends it. should always return
/// FMV_EOF. cinematics must be stopped in reverse order of when they are
/// started.
/// Raven wrapper: `syscall(UI_CIN_STOPCINEMATIC, handle)`.
/// Raven transport: `return CIN_StopCinematic(args[1]);`
///
/// Args source: `oracle/codemp/ui/ui_syscalls.c:409-412`
/// Args source: `oracle/codemp/ui/ui_local.h:1003`
/// Transport/switch source: `oracle/codemp/client/cl_ui.cpp:1187-1188`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiCinStopcinematicArgs {
    handle: c_int,
}

impl UiCinStopcinematicArgs {
    pub const fn new(handle: c_int) -> Self {
        Self { handle }
    }

    pub const fn handle(&self) -> c_int {
        self.handle
    }
}

/// `UI_CIN_STOPCINEMATIC` MP UI imports syscall ABI token.
///
/// Raven `e_status` is an integer transport value.
///
/// Enum value source: `oracle/codemp/ui/ui_public.h:106`
/// Enum comment source: `oracle/codemp/ui/ui_public.h:105-109`
/// Args source: `oracle/codemp/ui/ui_syscalls.c:409-412`
/// Output source: `oracle/codemp/ui/ui_local.h:1003`
/// Output source: `oracle/codemp/client/cl_ui.cpp:1187-1188`
/// Transport/switch source: `oracle/codemp/client/cl_ui.cpp:1187-1188`
pub struct UiCinStopcinematic;

impl OutboundSysCall for UiCinStopcinematic {
    type Import = MpUiImport;
    type Args = UiCinStopcinematicArgs;
    type Output = c_int;

    const IMPORT: MpUiImport = MpUiImport::UI_CIN_STOPCINEMATIC;
}

impl EncodeSysCall for UiCinStopcinematic {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.handle() as isize])
    }
}

impl DecodeSysCallReturn for UiCinStopcinematic {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
