use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_LANGUAGE_USESSPACES` SP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:128`
pub struct CgLanguageUsesspaces;

impl OutboundSysCall for CgLanguageUsesspaces {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_LANGUAGE_USESSPACES;
}
