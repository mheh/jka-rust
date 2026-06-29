use core::ffi::c_int;

use super::super::SpUiImport;
use crate::boundary::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `UI_KEY_SETCATCHER`.
///
/// Raven wrapper: `syscall( UI_KEY_SETCATCHER, catcher );`
/// Raven transport: `Key_SetCatcher( args[1] ); return 0;`
///
/// Enum source: `oracle/oracle/code/ui/ui_public.h:193`
/// Args source: `oracle/oracle/code/client/cl_ui.cpp:411-412`
/// Transport/switch source: `oracle/oracle/code/client/cl_ui.cpp:411-413`
pub struct UiKeySetcatcherArgs {
    /// Key catcher mask, read by Raven as `args[1]`.
    catcher: c_int,
}

impl UiKeySetcatcherArgs {
    pub const fn new(catcher: c_int) -> Self {
        Self { catcher }
    }

    pub const fn catcher(&self) -> c_int {
        self.catcher
    }
}

/// `UI_KEY_SETCATCHER` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:193`
pub struct UiKeySetcatcher;

impl OutboundSysCall for UiKeySetcatcher {
    type Import = SpUiImport;
    type Args = UiKeySetcatcherArgs;
    type Output = ();

    const IMPORT: SpUiImport = SpUiImport::UI_KEY_SETCATCHER;
}

impl EncodeSysCall for UiKeySetcatcher {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.catcher() as isize])
    }
}

impl DecodeSysCallReturn for UiKeySetcatcher {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
