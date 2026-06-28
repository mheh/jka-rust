use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_S_GETSAMPLELENGTH` SP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:167`
pub struct CgSGetsamplelength;

impl OutboundSysCall for CgSGetsamplelength {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_S_GETSAMPLELENGTH;
}
