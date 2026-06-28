use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_SETCLIENTTURNEXTENT` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:189`
pub struct CgSetclientturnextent;

impl OutboundSysCall for CgSetclientturnextent {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_SETCLIENTTURNEXTENT;
}
