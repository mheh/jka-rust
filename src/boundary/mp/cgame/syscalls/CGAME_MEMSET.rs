use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CGAME_MEMSET` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:130`
pub struct CgameMemset;

impl OutboundSysCall for CgameMemset {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CGAME_MEMSET;
}
