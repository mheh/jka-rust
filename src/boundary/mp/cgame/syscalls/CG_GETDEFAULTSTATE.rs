use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_GETDEFAULTSTATE` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:183`
pub struct CgGetdefaultstate;

impl OutboundSysCall for CgGetdefaultstate {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_GETDEFAULTSTATE;
}
