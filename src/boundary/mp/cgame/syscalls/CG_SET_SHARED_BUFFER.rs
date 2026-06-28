use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_SET_SHARED_BUFFER` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:328`
pub struct CgSetSharedBuffer;

impl OutboundSysCall for CgSetSharedBuffer {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_SET_SHARED_BUFFER;
}
