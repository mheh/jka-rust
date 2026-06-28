use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_MEMCPY` MP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:117`
pub struct UiMemcpy;

impl OutboundSysCall for UiMemcpy {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_MEMCPY;
}
