use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_G2_RAGPCJCONSTRAINT` MP cgame imports syscall boundary token.
///
/// Raven: rww - RAGDOLL_END
/// Raven: additional ragdoll options -rww
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:307`
pub struct CgG2Ragpcjconstraint;

impl OutboundSysCall for CgG2Ragpcjconstraint {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_G2_RAGPCJCONSTRAINT;
}
