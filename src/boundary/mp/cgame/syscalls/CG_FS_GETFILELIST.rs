use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_FS_GETFILELIST` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:77`
pub struct CgFsGetfilelist;

impl OutboundSysCall for CgFsGetfilelist {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_FS_GETFILELIST;
}
