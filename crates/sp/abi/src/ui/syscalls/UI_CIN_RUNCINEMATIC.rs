use super::super::types::e_status;
use super::super::SpUiImport;
use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};
use core::ffi::c_int;

/// `UI_CIN_RUNCINEMATIC` SP UI imports syscall ABI token.
///
/// Raven: will run a frame of the cinematic but will not draw it. Returns `e_status`.
///
/// Enum value source: `oracle/code/ui/ui_public.h:229`
/// Type definition source: `oracle/code/game/q_shared.h:2670-2679`
/// Args source: `oracle/code/client/cl_ui.cpp:473-474`
/// Output source: `oracle/code/client/client.h:432`
/// Output source: `oracle/code/client/cl_ui.cpp:473-474`
/// Transport/switch source: `oracle/code/client/cl_ui.cpp:473-474`
pub struct UiCinRuncinematic;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiCinRuncinematicArgs {
    handle: c_int,
}

impl UiCinRuncinematicArgs {
    pub const fn new(handle: c_int) -> Self {
        Self { handle }
    }

    pub const fn handle(&self) -> c_int {
        self.handle
    }
}

impl OutboundSysCall for UiCinRuncinematic {
    type Import = SpUiImport;
    type Args = UiCinRuncinematicArgs;
    type Output = e_status;

    const IMPORT: SpUiImport = SpUiImport::UI_CIN_RUNCINEMATIC;
}

impl EncodeSysCall for UiCinRuncinematic {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.handle() as isize])
    }
}

impl DecodeSysCallReturn for UiCinRuncinematic {
    fn decode_return(word: isize) -> Self::Output {
        e_status::from_wire(word as c_int)
    }
}
