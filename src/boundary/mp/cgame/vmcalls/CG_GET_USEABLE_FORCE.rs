use super::super::MpCgameExport;
use crate::boundary::generic::InboundVmCall;

/// `CG_GET_USEABLE_FORCE` MP cgame exports vmMain boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:416`
pub struct CgGetUseableForce;

impl InboundVmCall for CgGetUseableForce {
    type Command = MpCgameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: MpCgameExport = MpCgameExport::CG_GET_USEABLE_FORCE;
}
