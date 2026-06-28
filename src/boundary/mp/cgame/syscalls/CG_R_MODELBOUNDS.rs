use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_R_MODELBOUNDS` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:161`
pub struct CgRModelbounds;

impl OutboundSysCall for CgRModelbounds {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_R_MODELBOUNDS;
}
