use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_CM_MARKFRAGMENTS` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:94`
pub struct CgCmMarkfragments;

impl OutboundSysCall for CgCmMarkfragments {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_CM_MARKFRAGMENTS;
}
