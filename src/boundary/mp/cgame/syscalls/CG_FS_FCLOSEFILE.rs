use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_FS_FCLOSEFILE` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:76`
pub struct CgFsFclosefile;

impl OutboundSysCall for CgFsFclosefile {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_FS_FCLOSEFILE;
}
