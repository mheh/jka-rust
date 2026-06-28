use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_R_RENDERSCENE` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:158`
pub struct CgRRenderscene;

impl OutboundSysCall for CgRRenderscene {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_R_RENDERSCENE;
}
