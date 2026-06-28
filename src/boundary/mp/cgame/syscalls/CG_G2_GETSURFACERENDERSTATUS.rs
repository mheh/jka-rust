use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_G2_GETSURFACERENDERSTATUS` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:290`
pub struct CgG2Getsurfacerenderstatus;

impl OutboundSysCall for CgG2Getsurfacerenderstatus {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_G2_GETSURFACERENDERSTATUS;
}
