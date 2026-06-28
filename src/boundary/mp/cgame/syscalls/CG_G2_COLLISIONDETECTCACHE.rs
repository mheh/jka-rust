use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_G2_COLLISIONDETECTCACHE` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:266`
pub struct CgG2Collisiondetectcache;

impl OutboundSysCall for CgG2Collisiondetectcache {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_G2_COLLISIONDETECTCACHE;
}
