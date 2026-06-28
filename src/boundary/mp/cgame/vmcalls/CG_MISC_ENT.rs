use super::super::MpCgameExport;
use crate::boundary::generic::InboundVmCall;

/// `CG_MISC_ENT` MP cgame exports vmMain boundary token.
///
/// Raven: rwwRMG - added
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:435`
pub struct CgMiscEnt;

impl InboundVmCall for CgMiscEnt {
    type Command = MpCgameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: MpCgameExport = MpCgameExport::CG_MISC_ENT;
}
