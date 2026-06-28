use super::super::MpCgameExport;
use crate::boundary::generic::InboundVmCall;

/// `CG_DRAW_ACTIVE_FRAME` MP cgame exports vmMain boundary token.
///
/// Raven: qboolean (*CG_ConsoleCommand)( void );
/// Raven: a console command has been issued locally that is not recognized by the
/// Raven: main game system.
/// Raven: use Cmd_Argc() / Cmd_Argv() to read the command, return qfalse if the
/// Raven: command is not known to the game
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:373`
pub struct CgDrawActiveFrame;

impl InboundVmCall for CgDrawActiveFrame {
    type Command = MpCgameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: MpCgameExport = MpCgameExport::CG_DRAW_ACTIVE_FRAME;
}
