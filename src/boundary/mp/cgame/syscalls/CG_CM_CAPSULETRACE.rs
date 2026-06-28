use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_CM_CAPSULETRACE` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:91`
pub struct CgCmCapsuletrace;

impl OutboundSysCall for CgCmCapsuletrace {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_CM_CAPSULETRACE;
}
