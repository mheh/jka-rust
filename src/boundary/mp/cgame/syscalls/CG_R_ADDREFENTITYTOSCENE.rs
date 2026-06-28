use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_R_ADDREFENTITYTOSCENE` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:151`
pub struct CgRAddrefentitytoscene;

impl OutboundSysCall for CgRAddrefentitytoscene {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_R_ADDREFENTITYTOSCENE;
}
