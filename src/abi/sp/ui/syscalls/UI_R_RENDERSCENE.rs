use core::ffi::c_void;

use super::super::SpUiImport;
use crate::abi::generic::OutboundSysCall;

/// `UI_R_RENDERSCENE` SP UI imports syscall ABI token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:177`
pub struct UiRRenderscene;

impl OutboundSysCall for UiRRenderscene {
    type Import = SpUiImport;
    /// Args source: `oracle/oracle/code/ui/ui_public.h:80`; FIXME: create type `refdef_t`.
    type Args = *const c_void;
    /// Output source: `oracle/oracle/code/ui/ui_public.h:80`.
    type Output = ();

    const IMPORT: SpUiImport = SpUiImport::UI_R_RENDERSCENE;
}
