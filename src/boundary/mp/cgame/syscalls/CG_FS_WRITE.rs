use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_FS_WRITE` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:75`
pub struct CgFsWrite;

impl OutboundSysCall for CgFsWrite {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_FS_WRITE;
}
