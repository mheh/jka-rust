use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_G2_GETSURFACENAME` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:326`
pub struct CgG2Getsurfacename;

impl OutboundSysCall for CgG2Getsurfacename {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_G2_GETSURFACENAME;
}
