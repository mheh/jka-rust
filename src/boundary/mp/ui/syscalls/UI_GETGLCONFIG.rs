use core::ffi::c_void;

use super::super::MpUiImport;
use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `UI_GETGLCONFIG`.
///
/// Raven wrapper: `syscall( UI_GETGLCONFIG, glconfig );`
/// Raven transport: `CL_GetGlconfig( (glconfig_t *)VMA(1) );`
///
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:262-263`
#[derive(Debug)]
pub struct UiGetglconfigArgs {
    glconfig: *mut c_void,
}

impl UiGetglconfigArgs {
    pub fn new(glconfig: *mut c_void) -> Self {
        Self { glconfig }
    }

    pub fn glconfig(&self) -> *mut c_void {
        self.glconfig
    }
}

/// `UI_GETGLCONFIG` MP UI imports syscall boundary token.
///
/// Raven wrapper: `syscall( UI_GETGLCONFIG, glconfig );`
/// Raven transport: `CL_GetGlconfig( (glconfig_t *)VMA(1) );`
///
/// Enum value source: `oracle/oracle/codemp/ui/ui_public.h:62`
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:262-263`
/// Output source: `oracle/oracle/codemp/ui/ui_local.h:962`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1048-1050`
pub struct UiGetglconfig;

impl OutboundSysCall for UiGetglconfig {
    type Import = MpUiImport;
    type Args = UiGetglconfigArgs;
    type Output = ();

    const IMPORT: MpUiImport = MpUiImport::UI_GETGLCONFIG;
}

impl EncodeSysCall for UiGetglconfig {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.glconfig())])
    }
}

impl DecodeSysCallReturn for UiGetglconfig {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
