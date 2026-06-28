use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_G2_GETBONEFRAME` MP cgame imports syscall boundary token.
///
/// Raven: trimmed down version of GBA, so I don't have to pass all those unused args across the VM-exe border
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:271`
pub struct CgG2Getboneframe;

impl OutboundSysCall for CgG2Getboneframe {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_G2_GETBONEFRAME;
}
