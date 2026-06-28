use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_R_LOADWORLDMAP` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:116`
pub struct CgRLoadworldmap;

impl OutboundSysCall for CgRLoadworldmap {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_R_LOADWORLDMAP;
}
