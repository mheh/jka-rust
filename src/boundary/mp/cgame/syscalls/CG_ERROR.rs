use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_ERROR` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:58`
pub struct CgError;

impl OutboundSysCall for CgError {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_ERROR;
}
