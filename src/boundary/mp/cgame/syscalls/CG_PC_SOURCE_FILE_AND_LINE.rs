use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_PC_SOURCE_FILE_AND_LINE` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:203`
pub struct CgPcSourceFileAndLine;

impl OutboundSysCall for CgPcSourceFileAndLine {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_PC_SOURCE_FILE_AND_LINE;
}
