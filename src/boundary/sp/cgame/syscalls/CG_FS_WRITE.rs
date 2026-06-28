use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_FS_WRITE` SP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:72`
pub struct CgFsWrite;

impl OutboundSysCall for CgFsWrite {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_FS_WRITE;
}
