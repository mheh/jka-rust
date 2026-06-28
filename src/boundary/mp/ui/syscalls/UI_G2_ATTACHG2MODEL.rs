use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_G2_ATTACHG2MODEL` MP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:188`
pub struct UiG2Attachg2model;

impl OutboundSysCall for UiG2Attachg2model {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_G2_ATTACHG2MODEL;
}
