use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_TRUEMALLOC` MP cgame imports syscall boundary token.
///
/// Raven: rww - dynamic vm memory allocation!
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:250`
pub struct CgTruemalloc;

impl OutboundSysCall for CgTruemalloc {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_TRUEMALLOC;
}
