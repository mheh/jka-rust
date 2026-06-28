use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_TESTPRINTFLOAT` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:192`
pub struct CgTestprintfloat;

impl OutboundSysCall for CgTestprintfloat {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_TESTPRINTFLOAT;
}
