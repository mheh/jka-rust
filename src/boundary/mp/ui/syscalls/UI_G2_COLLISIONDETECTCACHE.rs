use super::super::MpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_G2_COLLISIONDETECTCACHE` MP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:151`
pub struct UiG2Collisiondetectcache;

impl OutboundSysCall for UiG2Collisiondetectcache {
    type Import = MpUiImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpUiImport = MpUiImport::UI_G2_COLLISIONDETECTCACHE;
}
