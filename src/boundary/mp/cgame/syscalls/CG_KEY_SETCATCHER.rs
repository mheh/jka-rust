use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_KEY_SETCATCHER` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:196`
pub struct CgKeySetcatcher;

impl OutboundSysCall for CgKeySetcatcher {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_KEY_SETCATCHER;
}
