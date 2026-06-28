use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_R_REGISTERFONT` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:121`
pub struct CgRRegisterfont;

impl OutboundSysCall for CgRRegisterfont {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_R_REGISTERFONT;
}
