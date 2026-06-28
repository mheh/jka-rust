use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_G2_HAVEWEGHOULMODELS` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:259`
pub struct CgG2Haveweghoulmodels;

impl OutboundSysCall for CgG2Haveweghoulmodels {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_G2_HAVEWEGHOULMODELS;
}
