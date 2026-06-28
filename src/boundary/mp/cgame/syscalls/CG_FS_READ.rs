use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_FS_READ` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:74`
pub struct CgFsRead;

impl OutboundSysCall for CgFsRead {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_FS_READ;
}
