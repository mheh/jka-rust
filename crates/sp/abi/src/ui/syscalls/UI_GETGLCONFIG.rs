use super::super::SpUiImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use core::ffi::c_void;

/// `UI_GETGLCONFIG` SP UI imports syscall ABI token.
///
/// Source: `oracle/code/ui/ui_public.h:195`
pub struct UiGetglconfig;

#[derive(Debug)]
pub struct UiGetglconfigArgs {
    glconfig: *mut c_void,
}

impl UiGetglconfigArgs {
    pub const fn new(glconfig: *mut c_void) -> Self {
        Self { glconfig }
    }

    pub const fn glconfig(&self) -> *mut c_void {
        self.glconfig
    }
}

impl OutboundSysCall for UiGetglconfig {
    type Import = SpUiImport;
    /// Raven wrapper: `syscall( UI_GETGLCONFIG, glconfig );`
    ///
    /// Args source: `oracle/code/ui/ui_syscalls.cpp:158-161`
    /// Output source: `oracle/code/ui/ui_syscalls.cpp:158-161`
    /// Transport/switch source: `oracle/code/client/cl_ui.cpp:397-399`
    type Args = UiGetglconfigArgs;
    /// Fills out an engine `glconfig_t` and returns no value.
    type Output = ();

    const IMPORT: SpUiImport = SpUiImport::UI_GETGLCONFIG;
}

impl EncodeSysCall for UiGetglconfig {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.glconfig())])
    }
}

impl DecodeSysCallReturn for UiGetglconfig {
    fn decode_return(_word: isize) -> Self::Output {}
}
