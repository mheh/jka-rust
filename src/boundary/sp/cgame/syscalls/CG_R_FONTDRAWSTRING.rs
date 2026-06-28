use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_R_FONTDRAWSTRING` SP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:126`
pub struct CgRFontdrawstring;

impl OutboundSysCall for CgRFontdrawstring {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_R_FONTDRAWSTRING;
}
