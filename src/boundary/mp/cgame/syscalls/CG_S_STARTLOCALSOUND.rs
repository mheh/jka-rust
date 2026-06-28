use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_S_STARTLOCALSOUND` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:98`
pub struct CgSStartlocalsound;

impl OutboundSysCall for CgSStartlocalsound {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_S_STARTLOCALSOUND;
}
