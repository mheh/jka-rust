use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_RMG_INIT` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:331`
pub struct CgRmgInit;

impl OutboundSysCall for CgRmgInit {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_RMG_INIT;
}
