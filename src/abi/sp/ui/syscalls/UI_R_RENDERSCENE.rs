use super::super::types::refdef_t;
use super::super::SpUiImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `UI_R_RENDERSCENE` SP UI imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/code/ui/ui_public.h:177`
/// Type definition source: `oracle/oracle/code/renderer/tr_types.h:103-176`
/// Args source: `oracle/oracle/code/ui/ui_public.h:80`
/// Args source: `oracle/oracle/code/ui/ui_local.h:205`
/// Output source: `oracle/oracle/code/ui/ui_syscalls.cpp:54-58`
/// Transport/function-table source: `oracle/oracle/code/ui/ui_syscalls.cpp:54-58`
/// SP caveat: Raven's SP wrapper calls `ui.R_RenderScene(fd)` directly; no active
/// `UI_R_RENDERSCENE` case exists in `oracle/oracle/code/client/cl_ui.cpp`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiRRendersceneArgs {
    refdef: *const refdef_t,
}

impl UiRRendersceneArgs {
    pub const fn new(refdef: *const refdef_t) -> Self {
        Self { refdef }
    }

    pub const fn refdef(&self) -> *const refdef_t {
        self.refdef
    }
}

pub struct UiRRenderscene;

impl OutboundSysCall for UiRRenderscene {
    type Import = SpUiImport;
    type Args = UiRRendersceneArgs;
    type Output = ();

    const IMPORT: SpUiImport = SpUiImport::UI_R_RENDERSCENE;
}

impl EncodeSysCall for UiRRenderscene {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.refdef())])
    }
}

impl DecodeSysCallReturn for UiRRenderscene {
    fn decode_return(_word: isize) -> Self::Output {}
}
