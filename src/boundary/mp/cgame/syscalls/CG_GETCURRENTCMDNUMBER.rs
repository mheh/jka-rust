use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_GETCURRENTCMDNUMBER` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:185`
pub struct CgGetcurrentcmdnumber;

impl OutboundSysCall for CgGetcurrentcmdnumber {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_GETCURRENTCMDNUMBER;
}
