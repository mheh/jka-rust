use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_CM_TEMPCAPSULEMODEL` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:87`
pub struct CgCmTempcapsulemodel;

impl OutboundSysCall for CgCmTempcapsulemodel {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_CM_TEMPCAPSULEMODEL;
}
