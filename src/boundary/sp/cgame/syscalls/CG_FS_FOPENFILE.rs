use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_FS_FOPENFILE` SP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:70`
pub struct CgFsFopenfile;

impl OutboundSysCall for CgFsFopenfile {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_FS_FOPENFILE;
}
