use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_PC_ADD_GLOBAL_DEFINE` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:199`
pub struct CgPcAddGlobalDefine;

impl OutboundSysCall for CgPcAddGlobalDefine {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_PC_ADD_GLOBAL_DEFINE;
}
