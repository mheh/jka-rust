use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_G2_SETBONEIKSTATE` MP cgame imports syscall boundary token.
///
/// Raven: rww - ik move method, allows you to specify a bone and move it to a world point (within joint constraints)
/// Raven: by using the majority of gil's existing bone angling stuff from the ragdoll code.
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:316`
pub struct CgG2Setboneikstate;

impl OutboundSysCall for CgG2Setboneikstate {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_G2_SETBONEIKSTATE;
}
