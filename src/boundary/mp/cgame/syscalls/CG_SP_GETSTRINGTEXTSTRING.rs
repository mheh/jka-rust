use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_SP_GETSTRINGTEXTSTRING` MP cgame imports syscall boundary token.
///
/// Raven: CG_SP_PRINT,
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:240`
pub struct CgSpGetstringtextstring;

impl OutboundSysCall for CgSpGetstringtextstring {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_SP_GETSTRINGTEXTSTRING;
}
