use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_S_UPDATEENTITYPOSITION` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:101`
pub struct CgSUpdateentityposition;

impl OutboundSysCall for CgSUpdateentityposition {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_S_UPDATEENTITYPOSITION;
}
