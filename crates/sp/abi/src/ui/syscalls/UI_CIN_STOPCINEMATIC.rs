use super::super::types::e_status;
use super::super::SpUiImport;
use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};
use core::ffi::c_int;

/// `UI_CIN_STOPCINEMATIC` SP UI imports syscall ABI token.
///
/// Raven: stops playing the cinematic and ends it. should always return `FMV_EOF`.
/// Raven wrapper: `trap_CIN_StopCinematic( handle )`.
/// Raven transport: `return CIN_StopCinematic(args[1]);`
///
/// Enum value source: `oracle/code/ui/ui_public.h:228`
/// Type definition source: `oracle/code/game/q_shared.h:2670-2679`
/// Args source: `oracle/code/ui/ui_syscalls.cpp:173-176`
/// Args source: `oracle/code/ui/ui_local.h:190`
/// Output source: `oracle/code/client/client.h:431`
/// Output source: `oracle/code/client/cl_ui.cpp:470`
/// Transport/switch source: `oracle/code/client/cl_ui.cpp:470`
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

pub struct UiCinStopcinematic;

impl OutboundSysCall for UiCinStopcinematic {
    type Import = SpUiImport;
    type Args = UiCinStopcinematicArgs;
    type Output = e_status;

    const IMPORT: SpUiImport = SpUiImport::UI_CIN_STOPCINEMATIC;
}

impl EncodeSysCall for UiCinStopcinematic {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.handle() as isize])
    }
}

impl DecodeSysCallReturn for UiCinStopcinematic {
    fn decode_return(word: isize) -> Self::Output {
        e_status::from_wire(word as c_int)
    }
}
