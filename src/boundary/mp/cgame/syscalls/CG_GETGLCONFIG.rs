use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_GETGLCONFIG` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:179`
pub struct CgGetglconfig;

impl OutboundSysCall for CgGetglconfig {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_GETGLCONFIG;
}
