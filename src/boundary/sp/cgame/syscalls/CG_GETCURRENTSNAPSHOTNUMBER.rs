use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_GETCURRENTSNAPSHOTNUMBER` SP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:152`
pub struct CgGetcurrentsnapshotnumber;

impl OutboundSysCall for CgGetcurrentsnapshotnumber {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_GETCURRENTSNAPSHOTNUMBER;
}
