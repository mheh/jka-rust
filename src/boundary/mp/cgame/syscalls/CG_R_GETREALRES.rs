use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_R_GETREALRES` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:173`
pub struct CgRGetrealres;

impl OutboundSysCall for CgRGetrealres {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_R_GETREALRES;
}
