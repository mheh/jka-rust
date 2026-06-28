use super::super::MpCgameExport;
use crate::boundary::generic::InboundVmCall;

/// `CG_GET_ORIGIN` MP cgame exports vmMain boundary token.
///
/// Raven: int entnum, vec3_t origin
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:418`
pub struct CgGetOrigin;

impl InboundVmCall for CgGetOrigin {
    type Command = MpCgameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: MpCgameExport = MpCgameExport::CG_GET_ORIGIN;
}
