use core::ffi::c_void;

use super::super::MpUiImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `UI_R_RENDERSCENE`.
///
/// C ABI: `void trap_R_RenderScene(const refdef_t *fd)`.
/// Raven's client switch forwards the scene description through `VMA(1)`.
///
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:186-187`
/// Output source: `oracle/oracle/codemp/ui/ui_syscalls.c:186-187`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:976-977`
#[derive(Debug, Clone, Copy)]
pub struct UiRRendersceneArgs {
    pub refdef: *const c_void,
}

impl UiRRendersceneArgs {
    pub const fn new(refdef: *const c_void) -> Self {
        Self { refdef }
    }
}

/// `UI_R_RENDERSCENE` MP UI imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:44`
pub struct UiRRenderscene;

impl OutboundSysCall for UiRRenderscene {
    type Import = MpUiImport;
    type Args = UiRRendersceneArgs;
    type Output = ();

    const IMPORT: MpUiImport = MpUiImport::UI_R_RENDERSCENE;
}

impl EncodeSysCall for UiRRenderscene {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.refdef)])
    }
}

impl DecodeSysCallReturn for UiRRenderscene {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
