use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_R_REMAP_SHADER` SP UI imports syscall boundary token.
///
/// Raven: 80
/// Source: `oracle/oracle/code/ui/ui_public.h:232`
pub struct UiRRemapShader;

impl OutboundSysCall for UiRRemapShader {
    type Import = SpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpUiImport = SpUiImport::UI_R_REMAP_SHADER;
}
