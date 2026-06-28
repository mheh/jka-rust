use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_PRECISIONTIMER_END` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:63`
pub struct CgPrecisiontimerEnd;

impl OutboundSysCall for CgPrecisiontimerEnd {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_PRECISIONTIMER_END;
}
