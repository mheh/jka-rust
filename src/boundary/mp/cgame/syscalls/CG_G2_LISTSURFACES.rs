use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_G2_LISTSURFACES` MP cgame imports syscall boundary token.
///
/// Raven: Ghoul2 Insert Start
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:256`
pub struct CgG2Listsurfaces;

impl OutboundSysCall for CgG2Listsurfaces {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_G2_LISTSURFACES;
}
