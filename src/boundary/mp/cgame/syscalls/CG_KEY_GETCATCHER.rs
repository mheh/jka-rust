use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_KEY_GETCATCHER` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:195`
pub struct CgKeyGetcatcher;

impl OutboundSysCall for CgKeyGetcatcher {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_KEY_GETCATCHER;
}
