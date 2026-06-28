use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_R_FONT_STRHEIGHTPIXELS` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:124`
pub struct CgRFontStrheightpixels;

impl OutboundSysCall for CgRFontStrheightpixels {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_R_FONT_STRHEIGHTPIXELS;
}
