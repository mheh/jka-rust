use core::ffi::c_int;

use super::super::MpUiImport;
use abi_transport::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `UI_CIN_RUNCINEMATIC`.
///
/// Raven: will run a frame of the cinematic but will not draw it. Will return
/// FMV_EOF if the end of the cinematic has been reached.
/// Raven wrapper: `syscall(UI_CIN_RUNCINEMATIC, handle)`.
/// Raven transport: `return CIN_RunCinematic(args[1]);`
///
/// Args source: `oracle/codemp/ui/ui_syscalls.c:416-418`
/// Args source: `oracle/codemp/ui/ui_local.h:1004`
/// Transport/switch source: `oracle/codemp/client/cl_ui.cpp:1190-1191`
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

/// `UI_CIN_RUNCINEMATIC` MP UI imports syscall ABI token.
///
/// Raven `e_status` is an integer transport value.
///
/// Enum value source: `oracle/codemp/ui/ui_public.h:107`
/// Enum comment source: `oracle/codemp/ui/ui_public.h:105-109`
/// Args source: `oracle/codemp/ui/ui_syscalls.c:416-418`
/// Output source: `oracle/codemp/ui/ui_local.h:1004`
/// Output source: `oracle/codemp/client/cl_ui.cpp:1190-1191`
/// Transport/switch source: `oracle/codemp/client/cl_ui.cpp:1190-1191`
pub struct UiCinRuncinematic;

impl OutboundSysCall for UiCinRuncinematic {
    type Import = MpUiImport;
    type Args = UiCinRuncinematicArgs;
    type Output = c_int;

    const IMPORT: MpUiImport = MpUiImport::UI_CIN_RUNCINEMATIC;
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
