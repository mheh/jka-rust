use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_UPDATESCREEN` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:82`
pub struct CgUpdatescreen;

impl OutboundSysCall for CgUpdatescreen {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_UPDATESCREEN;
}
