use core::ffi::c_int;

use super::super::SpUiImport;
use crate::boundary::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `UI_CIN_RUNCINEMATIC` SP UI imports syscall boundary token.
///
/// Raven: will run a frame of the cinematic but will not draw it. Returns `e_status`.
/// FIXME: create type `e_status` in Rust (Raven source: `oracle/oracle/code/game/q_shared.h:2670-2679`).
///
/// Enum value source: `oracle/oracle/code/ui/ui_public.h:229`
/// Args source: `oracle/oracle/code/client/cl_ui.cpp:473-474`
/// Output source: `oracle/oracle/code/client/client.h:432`
/// Transport/switch source: `oracle/oracle/code/client/cl_ui.cpp:473-474`
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
    /// Representing `e_status` as `c_int` for ABI compatibility.
    type Output = c_int;

    const IMPORT: SpUiImport = SpUiImport::UI_CIN_RUNCINEMATIC;
}

impl EncodeSysCall for UiCinRuncinematic {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.handle() as isize])
    }
}

impl DecodeSysCallReturn for UiCinRuncinematic {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
