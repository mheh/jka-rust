use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_G2_LISTSURFACES` MP UI imports syscall boundary token.
///
/// Raven: Ghoul2 Insert Start
/// Source: `oracle/oracle/codemp/ui/ui_public.h:142`
pub struct UiG2Listsurfaces;

impl OutboundSysCall for UiG2Listsurfaces {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_G2_LISTSURFACES;
}
