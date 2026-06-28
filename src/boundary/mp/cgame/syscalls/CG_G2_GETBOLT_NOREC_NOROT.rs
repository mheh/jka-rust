use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_G2_GETBOLT_NOREC_NOROT` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:262`
pub struct CgG2GetboltNorecNorot;

impl OutboundSysCall for CgG2GetboltNorecNorot {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_G2_GETBOLT_NOREC_NOROT;
}
