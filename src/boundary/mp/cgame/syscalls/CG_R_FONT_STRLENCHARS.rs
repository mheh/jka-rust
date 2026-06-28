use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_R_FONT_STRLENCHARS` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:123`
pub struct CgRFontStrlenchars;

impl OutboundSysCall for CgRFontStrlenchars {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_R_FONT_STRLENCHARS;
}
