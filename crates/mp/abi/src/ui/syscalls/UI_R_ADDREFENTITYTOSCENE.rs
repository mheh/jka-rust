use core::ffi::c_void;

use super::super::MpUiImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `UI_R_ADDREFENTITYTOSCENE`.
///
/// C ABI: `void trap_R_AddRefEntityToScene(const refEntity_t *re)`.
/// Raven's client switch forwards the raw `refEntity_t` block through `VMA(1)`.
///
/// Args source: `oracle/codemp/ui/ui_syscalls.c:174-175`
/// Output source: `oracle/codemp/ui/ui_syscalls.c:174-175`
/// Transport/switch source: `oracle/codemp/client/cl_ui.cpp:960-961`
#[derive(Debug, Clone, Copy)]
pub struct UiRAddrefentitytosceneArgs {
    pub ref_entity: *const c_void,
}

impl UiRAddrefentitytosceneArgs {
    pub const fn new(ref_entity: *const c_void) -> Self {
        Self { ref_entity }
    }
}

/// `UI_R_ADDREFENTITYTOSCENE` MP UI imports syscall ABI token.
///
/// Source: `oracle/codemp/ui/ui_public.h:41`
pub struct UiRAddrefentitytoscene;

impl OutboundSysCall for UiRAddrefentitytoscene {
    type Import = MpUiImport;
    type Args = UiRAddrefentitytosceneArgs;
    type Output = ();

    const IMPORT: MpUiImport = MpUiImport::UI_R_ADDREFENTITYTOSCENE;
}

impl EncodeSysCall for UiRAddrefentitytoscene {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.ref_entity)])
    }
}

impl DecodeSysCallReturn for UiRAddrefentitytoscene {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
