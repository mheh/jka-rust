use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_G2_RAGFORCESOLVE` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:312`
pub struct CgG2Ragforcesolve;

impl OutboundSysCall for CgG2Ragforcesolve {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_G2_RAGFORCESOLVE;
}
