use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CGAME_TESTPRINTFLOAT` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:144`
pub struct CgameTestprintfloat;

impl OutboundSysCall for CgameTestprintfloat {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CGAME_TESTPRINTFLOAT;
}
