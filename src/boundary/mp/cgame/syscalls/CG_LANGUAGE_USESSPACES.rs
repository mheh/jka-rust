use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_LANGUAGE_USESSPACES` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:127`
pub struct CgLanguageUsesspaces;

impl OutboundSysCall for CgLanguageUsesspaces {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_LANGUAGE_USESSPACES;
}
