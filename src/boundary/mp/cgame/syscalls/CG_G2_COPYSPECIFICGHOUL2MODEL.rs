use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_G2_COPYSPECIFICGHOUL2MODEL` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:274`
pub struct CgG2Copyspecificghoul2model;

impl OutboundSysCall for CgG2Copyspecificghoul2model {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_G2_COPYSPECIFICGHOUL2MODEL;
}
