use super::super::MpCgameExport;
use crate::boundary::generic::InboundVmCall;

/// `CG_IMPACT_MARK` MP cgame exports vmMain boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:426`
pub struct CgImpactMark;

impl InboundVmCall for CgImpactMark {
    type Command = MpCgameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: MpCgameExport = MpCgameExport::CG_IMPACT_MARK;
}
