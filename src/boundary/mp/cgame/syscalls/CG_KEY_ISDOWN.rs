use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_KEY_ISDOWN` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:194`
pub struct CgKeyIsdown;

impl OutboundSysCall for CgKeyIsdown {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_KEY_ISDOWN;
}
