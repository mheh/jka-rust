use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_R_LIGHTFORPOINT` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:155`
pub struct CgRLightforpoint;

impl OutboundSysCall for CgRLightforpoint {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_R_LIGHTFORPOINT;
}
