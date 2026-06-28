use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_TESTPRINTINT` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:191`
pub struct CgTestprintint;

impl OutboundSysCall for CgTestprintint {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_TESTPRINTINT;
}
