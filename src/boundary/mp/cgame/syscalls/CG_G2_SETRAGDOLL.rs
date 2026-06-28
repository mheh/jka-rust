use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_G2_SETRAGDOLL` MP cgame imports syscall boundary token.
///
/// Raven: rww - RAGDOLL_BEGIN
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:300`
pub struct CgG2Setragdoll;

impl OutboundSysCall for CgG2Setragdoll {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_G2_SETRAGDOLL;
}
