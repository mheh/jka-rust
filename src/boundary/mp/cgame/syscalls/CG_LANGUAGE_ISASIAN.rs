use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_LANGUAGE_ISASIAN` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:126`
pub struct CgLanguageIsasian;

impl OutboundSysCall for CgLanguageIsasian {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_LANGUAGE_ISASIAN;
}
