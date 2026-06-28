use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CGAME_MEMCPY` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:131`
pub struct CgameMemcpy;

impl OutboundSysCall for CgameMemcpy {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CGAME_MEMCPY;
}
