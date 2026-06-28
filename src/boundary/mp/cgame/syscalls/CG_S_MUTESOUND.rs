use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_S_MUTESOUND` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:96`
pub struct CgSMutesound;

impl OutboundSysCall for CgSMutesound {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_S_MUTESOUND;
}
