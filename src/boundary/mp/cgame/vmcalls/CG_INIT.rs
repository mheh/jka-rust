use super::super::MpCgameExport;
use crate::boundary::generic::InboundVmCall;

/// `CG_INIT` MP cgame exports vmMain boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:353`
pub struct CgInit;

impl InboundVmCall for CgInit {
    type Command = MpCgameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: MpCgameExport = MpCgameExport::CG_INIT;
}
