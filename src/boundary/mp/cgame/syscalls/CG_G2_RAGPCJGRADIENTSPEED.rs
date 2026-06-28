use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_G2_RAGPCJGRADIENTSPEED` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:308`
pub struct CgG2Ragpcjgradientspeed;

impl OutboundSysCall for CgG2Ragpcjgradientspeed {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_G2_RAGPCJGRADIENTSPEED;
}
