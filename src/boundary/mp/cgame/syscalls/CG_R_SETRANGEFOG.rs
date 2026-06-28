use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_R_SETRANGEFOG` MP cgame imports syscall boundary token.
///
/// Raven: linear fogging, with settable range -rww
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:165`
pub struct CgRSetrangefog;

impl OutboundSysCall for CgRSetrangefog {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_R_SETRANGEFOG;
}
