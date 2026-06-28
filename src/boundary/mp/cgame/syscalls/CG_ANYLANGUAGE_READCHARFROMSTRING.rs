use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_ANYLANGUAGE_READCHARFROMSTRING` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:128`
pub struct CgAnylanguageReadcharfromstring;

impl OutboundSysCall for CgAnylanguageReadcharfromstring {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_ANYLANGUAGE_READCHARFROMSTRING;
}
