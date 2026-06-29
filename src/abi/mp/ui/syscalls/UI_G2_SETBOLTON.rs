use core::ffi::{c_int, c_void};

use super::super::MpUiImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `UI_G2_SETBOLTON`.
///
/// Raven wrapper: `void trap_G2API_SetBoltInfo(void *ghoul2, int modelIndex, int boltInfo)`.
///
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:604-606`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1357-1359`
#[derive(Debug)]
pub struct UiG2SetboltonArgs {
    /// Ghoul2 instance pointer transported as raw `args[1]`.
    ghoul2: *mut c_void,
    /// Model index read directly from `args[2]`.
    model_index: c_int,
    /// Bolt info read directly from `args[3]`.
    bolt_info: c_int,
}

impl UiG2SetboltonArgs {
    pub fn new(ghoul2: *mut c_void, model_index: c_int, bolt_info: c_int) -> Self {
        Self {
            ghoul2,
            model_index,
            bolt_info,
        }
    }

    pub fn ghoul2(&self) -> *mut c_void {
        self.ghoul2
    }
    pub fn model_index(&self) -> c_int {
        self.model_index
    }
    pub fn bolt_info(&self) -> c_int {
        self.bolt_info
    }
}

/// `UI_G2_SETBOLTON` MP UI imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/ui/ui_public.h:164`
/// Enum comment source: `oracle/oracle/codemp/ui/ui_public.h:164`
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:604-606`
/// Output source: `oracle/oracle/codemp/client/cl_ui.cpp:1357-1359`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1357-1359`
pub struct UiG2Setbolton;

impl OutboundSysCall for UiG2Setbolton {
    type Import = MpUiImport;
    type Args = UiG2SetboltonArgs;
    type Output = ();

    const IMPORT: MpUiImport = MpUiImport::UI_G2_SETBOLTON;
}

impl EncodeSysCall for UiG2Setbolton {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.ghoul2),
            a.model_index as isize,
            a.bolt_info as isize,
        ])
    }
}

impl DecodeSysCallReturn for UiG2Setbolton {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
