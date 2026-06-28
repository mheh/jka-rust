use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_R_REGISTERMODEL` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:117`
pub struct CgRRegistermodel;

impl OutboundSysCall for CgRRegistermodel {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_R_REGISTERMODEL;
}
