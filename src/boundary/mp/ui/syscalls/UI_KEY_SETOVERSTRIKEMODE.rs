use super::super::MpUiImport;
use crate::boundary::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::ffi::types::qboolean;

/// Arguments for `UI_KEY_SETOVERSTRIKEMODE`.
///
/// Raven wrapper: `syscall( UI_KEY_SETOVERSTRIKEMODE, state );`
/// Raven transport: `Key_SetOverstrikeMode((qboolean)args[1]); return 0;`
///
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:238-239`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1025-1027`
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct UiKeySetoverstrikemodeArgs {
    state: qboolean,
}

impl UiKeySetoverstrikemodeArgs {
    pub const fn new(state: qboolean) -> Self {
        Self { state }
    }

    pub const fn state(&self) -> qboolean {
        self.state
    }
}

/// `UI_KEY_SETOVERSTRIKEMODE` MP UI imports syscall boundary token.
///
/// Enum value source: `oracle/oracle/codemp/ui/ui_public.h:57`
/// Enum comment source: `oracle/oracle/codemp/ui/ui_public.h:52-62`
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:238-239`
/// Output source: `oracle/oracle/codemp/client/cl_ui.cpp:1025-1027`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1025-1027`
pub struct UiKeySetoverstrikemode;

impl OutboundSysCall for UiKeySetoverstrikemode {
    type Import = MpUiImport;
    type Args = UiKeySetoverstrikemodeArgs;
    type Output = ();

    const IMPORT: MpUiImport = MpUiImport::UI_KEY_SETOVERSTRIKEMODE;
}

impl EncodeSysCall for UiKeySetoverstrikemode {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.state() as isize])
    }
}

impl DecodeSysCallReturn for UiKeySetoverstrikemode {
    fn decode_return(_word: isize) -> Self::Output {}
}
