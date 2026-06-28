use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_PRINT` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:57`
pub struct CgPrint;

impl OutboundSysCall for CgPrint {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_PRINT;
}
