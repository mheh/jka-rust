use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_ANYLANGUAGE_READFROMSTRING` SP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:129`
pub struct CgAnylanguageReadfromstring;

impl OutboundSysCall for CgAnylanguageReadfromstring {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_ANYLANGUAGE_READFROMSTRING;
}
