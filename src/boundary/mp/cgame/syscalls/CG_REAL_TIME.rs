use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_REAL_TIME` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:208`
pub struct CgRealTime;

impl OutboundSysCall for CgRealTime {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_REAL_TIME;
}
