use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CGAME_SIN` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:133`
pub struct CgameSin;

impl OutboundSysCall for CgameSin {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CGAME_SIN;
}
