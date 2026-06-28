use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_R_ADDPOLYSTOSCENE` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:153`
pub struct CgRAddpolystoscene;

impl OutboundSysCall for CgRAddpolystoscene {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_R_ADDPOLYSTOSCENE;
}
