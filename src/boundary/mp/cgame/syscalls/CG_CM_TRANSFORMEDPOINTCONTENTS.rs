use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_CM_TRANSFORMEDPOINTCONTENTS` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:89`
pub struct CgCmTransformedpointcontents;

impl OutboundSysCall for CgCmTransformedpointcontents {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_CM_TRANSFORMEDPOINTCONTENTS;
}
