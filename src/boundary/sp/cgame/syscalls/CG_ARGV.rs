use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_ARGV` SP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:68`
pub struct CgArgv;

impl OutboundSysCall for CgArgv {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_ARGV;
}
