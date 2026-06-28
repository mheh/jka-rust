use super::super::MpCgameExport;
use crate::boundary::generic::InboundVmCall;

/// `CG_AUTOMAP_INPUT` MP cgame exports vmMain boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:433`
pub struct CgAutomapInput;

impl InboundVmCall for CgAutomapInput {
    type Command = MpCgameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: MpCgameExport = MpCgameExport::CG_AUTOMAP_INPUT;
}
