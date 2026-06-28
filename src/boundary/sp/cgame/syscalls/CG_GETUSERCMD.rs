use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_GETUSERCMD` SP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:159`
pub struct CgGetusercmd;

impl OutboundSysCall for CgGetusercmd {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_GETUSERCMD;
}
