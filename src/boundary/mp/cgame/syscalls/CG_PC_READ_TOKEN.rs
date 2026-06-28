use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_PC_READ_TOKEN` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:202`
pub struct CgPcReadToken;

impl OutboundSysCall for CgPcReadToken {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_PC_READ_TOKEN;
}
