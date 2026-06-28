use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_G2_ADDBOLT` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:283`
pub struct CgG2Addbolt;

impl OutboundSysCall for CgG2Addbolt {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_G2_ADDBOLT;
}
