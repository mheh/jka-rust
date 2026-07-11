use core::ffi::c_int;

use super::super::SpUiImport;
use abi_transport::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `UI_CIN_DRAWCINEMATIC` SP UI imports syscall ABI token.
///
/// Enum value source: `oracle/code/ui/ui_public.h:230`
/// Args source: `oracle/code/client/cl_ui.cpp:476-477`
/// Output source: `oracle/code/client/cl_ui.cpp:477`
/// Transport/switch source: `oracle/code/client/cl_ui.cpp:476-477`
pub struct UiCinDrawcinematic;

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

impl OutboundSysCall for UiCinDrawcinematic {
    type Import = SpUiImport;
    type Args = UiCinDrawcinematicArgs;
    type Output = ();

    const IMPORT: SpUiImport = SpUiImport::UI_CIN_DRAWCINEMATIC;
}

impl EncodeSysCall for UiCinDrawcinematic {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.handle() as isize])
    }
}

impl DecodeSysCallReturn for UiCinDrawcinematic {
    fn decode_return(_word: isize) -> Self::Output {}
}
