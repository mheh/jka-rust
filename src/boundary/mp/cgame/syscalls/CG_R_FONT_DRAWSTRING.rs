use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_R_FONT_DRAWSTRING` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:125`
pub struct CgRFontDrawstring;

impl OutboundSysCall for CgRFontDrawstring {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_R_FONT_DRAWSTRING;
}
