use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_GETSNAPSHOT` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:182`
pub struct CgGetsnapshot;

impl OutboundSysCall for CgGetsnapshot {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_GETSNAPSHOT;
}
