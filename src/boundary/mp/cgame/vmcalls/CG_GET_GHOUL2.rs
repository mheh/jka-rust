use super::super::MpCgameExport;
use crate::boundary::generic::InboundVmCall;

/// `CG_GET_GHOUL2` MP cgame exports vmMain boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:399`
pub struct CgGetGhoul2;

impl InboundVmCall for CgGetGhoul2 {
    type Command = MpCgameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: MpCgameExport = MpCgameExport::CG_GET_GHOUL2;
}
