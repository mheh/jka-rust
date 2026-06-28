use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_GETCURRENTSNAPSHOTNUMBER` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:181`
pub struct CgGetcurrentsnapshotnumber;

impl OutboundSysCall for CgGetcurrentsnapshotnumber {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_GETCURRENTSNAPSHOTNUMBER;
}
