use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CGAME_STRNCPY` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:132`
pub struct CgameStrncpy;

impl OutboundSysCall for CgameStrncpy {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CGAME_STRNCPY;
}
