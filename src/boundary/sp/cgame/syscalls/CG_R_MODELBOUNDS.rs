use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_R_MODELBOUNDS` SP cgame imports syscall boundary token.
///
/// Raven: CG_R_DRAWSCREENSHOT,
/// Source: `oracle/oracle/code/cgame/cg_public.h:143`
pub struct CgRModelbounds;

impl OutboundSysCall for CgRModelbounds {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_R_MODELBOUNDS;
}
