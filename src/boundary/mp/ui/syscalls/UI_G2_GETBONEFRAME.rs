use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_G2_GETBONEFRAME` MP UI imports syscall boundary token.
///
/// Raven: trimmed down version of GBA, so I don't have to pass all those unused args across the VM-exe border
/// Source: `oracle/oracle/codemp/ui/ui_public.h:156`
pub struct UiG2Getboneframe;

impl OutboundSysCall for UiG2Getboneframe {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_G2_GETBONEFRAME;
}
