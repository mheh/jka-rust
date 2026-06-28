use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_R_REGISTERSKIN` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:118`
pub struct CgRRegisterskin;

impl OutboundSysCall for CgRRegisterskin {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_R_REGISTERSKIN;
}
