use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_R_REMAP_SHADER` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:167`
pub struct CgRRemapShader;

impl OutboundSysCall for CgRRemapShader {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_R_REMAP_SHADER;
}
