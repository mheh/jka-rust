use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_MILLISECONDS` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:59`
pub struct CgMilliseconds;

impl OutboundSysCall for CgMilliseconds {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_MILLISECONDS;
}
