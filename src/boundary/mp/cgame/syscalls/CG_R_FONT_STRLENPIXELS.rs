use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_R_FONT_STRLENPIXELS` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:122`
pub struct CgRFontStrlenpixels;

impl OutboundSysCall for CgRFontStrlenpixels {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_R_FONT_STRLENPIXELS;
}
