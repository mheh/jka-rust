use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_FS_FCLOSEFILE` SP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:73`
pub struct CgFsFclosefile;

impl OutboundSysCall for CgFsFclosefile {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_FS_FCLOSEFILE;
}
