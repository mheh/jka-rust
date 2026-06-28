use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_R_SETREFRACTIONPROP` MP cgame imports syscall boundary token.
///
/// Raven: set some properties for the draw layer for my refractive effect (here primarily for mod authors) -rww
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:166`
pub struct CgRSetrefractionprop;

impl OutboundSysCall for CgRSetrefractionprop {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_R_SETREFRACTIONPROP;
}
