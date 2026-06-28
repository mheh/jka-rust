use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_R_INPVS` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:217`
pub struct CgRInpvs;

impl OutboundSysCall for CgRInpvs {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_R_INPVS;
}
