use super::super::MpCgameExport;
use crate::boundary::generic::InboundVmCall;

/// `CG_G2TRACE` MP cgame exports vmMain boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:406`
pub struct CgG2trace;

impl InboundVmCall for CgG2trace {
    type Command = MpCgameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: MpCgameExport = MpCgameExport::CG_G2TRACE;
}
