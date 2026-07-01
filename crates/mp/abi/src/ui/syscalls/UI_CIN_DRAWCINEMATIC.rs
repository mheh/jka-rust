use core::ffi::c_int;

use super::super::MpUiImport;
use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// Arguments for `UI_CIN_DRAWCINEMATIC`.
///
/// Raven: draws the current frame.
/// Raven wrapper: `syscall(UI_CIN_DRAWCINEMATIC, handle)`.
/// Raven transport: `CIN_DrawCinematic(args[1]); return 0;`
///
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:422-424`
/// Args source: `oracle/oracle/codemp/ui/ui_local.h:1005`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1193-1195`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiCinDrawcinematicArgs {
    handle: c_int,
}

impl UiCinDrawcinematicArgs {
    pub const fn new(handle: c_int) -> Self {
        Self { handle }
    }

    pub const fn handle(&self) -> c_int {
        self.handle
    }
}

/// `UI_CIN_DRAWCINEMATIC` MP UI imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/ui/ui_public.h:108`
/// Enum comment source: `oracle/oracle/codemp/ui/ui_public.h:105-109`
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:422-424`
/// Output source: `oracle/oracle/codemp/client/cl_ui.cpp:1193-1195`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1193-1195`
pub struct UiCinDrawcinematic;

impl OutboundSysCall for UiCinDrawcinematic {
    type Import = MpUiImport;
    type Args = UiCinDrawcinematicArgs;
    type Output = ();

    const IMPORT: MpUiImport = MpUiImport::UI_CIN_DRAWCINEMATIC;
}

impl EncodeSysCall for UiCinDrawcinematic {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.handle() as isize])
    }
}

impl DecodeSysCallReturn for UiCinDrawcinematic {
    fn decode_return(_word: isize) -> Self::Output {}
}
