use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_LANGUAGE_ISASIAN` SP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:127`
pub struct CgLanguageIsasian;

impl OutboundSysCall for CgLanguageIsasian {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_LANGUAGE_ISASIAN;
}
