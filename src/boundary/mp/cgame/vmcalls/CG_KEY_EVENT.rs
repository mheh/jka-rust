use super::super::MpCgameExport;
use crate::boundary::generic::InboundVmCall;

/// `CG_KEY_EVENT` MP cgame exports vmMain boundary token.
///
/// Raven: int (*CG_LastAttacker)( void );
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:384`
pub struct CgKeyEvent;

impl InboundVmCall for CgKeyEvent {
    type Command = MpCgameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: MpCgameExport = MpCgameExport::CG_KEY_EVENT;
}
