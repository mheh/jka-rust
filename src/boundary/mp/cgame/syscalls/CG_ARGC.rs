use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_ARGC` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:70`
pub struct CgArgc;

impl OutboundSysCall for CgArgc {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_ARGC;
}
