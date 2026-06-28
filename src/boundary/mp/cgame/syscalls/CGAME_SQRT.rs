use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CGAME_SQRT` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:136`
pub struct CgameSqrt;

impl OutboundSysCall for CgameSqrt {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CGAME_SQRT;
}
