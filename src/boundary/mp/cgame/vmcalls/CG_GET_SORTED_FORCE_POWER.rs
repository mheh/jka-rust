use super::super::MpCgameExport;
use crate::boundary::generic::InboundVmCall;

/// `CG_GET_SORTED_FORCE_POWER` MP cgame exports vmMain boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:437`
pub struct CgGetSortedForcePower;

impl InboundVmCall for CgGetSortedForcePower {
    type Command = MpCgameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: MpCgameExport = MpCgameExport::CG_GET_SORTED_FORCE_POWER;
}
