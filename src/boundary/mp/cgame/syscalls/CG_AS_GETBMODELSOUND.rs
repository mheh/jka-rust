use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_AS_GETBMODELSOUND` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:114`
pub struct CgAsGetbmodelsound;

impl OutboundSysCall for CgAsGetbmodelsound {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_AS_GETBMODELSOUND;
}
