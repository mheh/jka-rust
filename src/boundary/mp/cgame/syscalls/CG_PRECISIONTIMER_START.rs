use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_PRECISIONTIMER_START` MP cgame imports syscall boundary token.
///
/// Raven: Also for profiling.. do not use for game related tasks.
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:62`
pub struct CgPrecisiontimerStart;

impl OutboundSysCall for CgPrecisiontimerStart {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_PRECISIONTIMER_START;
}
