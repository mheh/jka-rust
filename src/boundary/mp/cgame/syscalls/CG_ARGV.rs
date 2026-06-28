use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_ARGV` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:71`
pub struct CgArgv;

impl OutboundSysCall for CgArgv {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_ARGV;
}
