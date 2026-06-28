use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_R_AUTOMAPELEVADJ` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:174`
pub struct CgRAutomapelevadj;

impl OutboundSysCall for CgRAutomapelevadj {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_R_AUTOMAPELEVADJ;
}
