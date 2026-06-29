use core::ffi::c_int;

use crate::ffi::types::qboolean;
use super::super::MpUiExport;
use crate::boundary::generic::InboundVmCall;

/// `UI_CONSOLE_COMMAND` MP UI exports vmMain boundary token.
///
/// Raven enum comment is currently shifted in this snapshot; this call uses
/// `qboolean UI_ConsoleCommand( int realTime )`.
///
/// Source (enum): `oracle/oracle/codemp/ui/ui_public.h:240`
/// Source (args): `oracle/oracle/codemp/ui/ui_main.c:612`
/// Source (output): `oracle/oracle/codemp/ui/ui_main.c:612`
/// Source (call site/transport): `oracle/oracle/codemp/client/cl_ui.cpp:1518`
pub struct UiConsoleCommand;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UiConsoleCommandArgs {
    real_time: c_int,
}

impl UiConsoleCommandArgs {
    pub const fn new(real_time: c_int) -> Self {
        Self { real_time }
    }

    pub const fn real_time(self) -> c_int {
        self.real_time
    }
}

impl InboundVmCall for UiConsoleCommand {
    type Command = MpUiExport;
    type Args = UiConsoleCommandArgs;
    type Output = qboolean;

    const COMMAND: MpUiExport = MpUiExport::UI_CONSOLE_COMMAND;
}
