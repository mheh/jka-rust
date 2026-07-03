//! MP game exports enum vocabulary.
//!
//! Transcribed from Raven `oracle/oracle/codemp/game/g_public.h`.
//! These discriminants are ABI wire values; do not renumber them.

#![allow(non_camel_case_types)] // C enumerator names kept for 1:1 traceability

#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MpGameExport {
    /// ( int levelTime, int randomSeed, int restart );
    /// init and shutdown will be called every single level
    /// The game should call G_GET_ENTITY_TOKEN to parse through all the
    /// entity configuration text and spawn gentities.
    /// Source: `oracle/oracle/codemp/game/g_public.h:735`
    GAME_INIT,

    /// (void);
    /// Source: `oracle/oracle/codemp/game/g_public.h:740`
    GAME_SHUTDOWN,

    /// ( int clientNum, qboolean firstTime, qboolean isBot );
    /// return NULL if the client is allowed to connect, otherwise return
    /// a text string with the reason for denial
    /// Source: `oracle/oracle/codemp/game/g_public.h:742`
    GAME_CLIENT_CONNECT,

    /// ( int clientNum );
    /// Source: `oracle/oracle/codemp/game/g_public.h:746`
    GAME_CLIENT_BEGIN,

    /// ( int clientNum );
    /// Source: `oracle/oracle/codemp/game/g_public.h:748`
    GAME_CLIENT_USERINFO_CHANGED,

    /// ( int clientNum );
    /// Source: `oracle/oracle/codemp/game/g_public.h:750`
    GAME_CLIENT_DISCONNECT,

    /// ( int clientNum );
    /// Source: `oracle/oracle/codemp/game/g_public.h:752`
    GAME_CLIENT_COMMAND,

    /// ( int clientNum );
    /// Source: `oracle/oracle/codemp/game/g_public.h:754`
    GAME_CLIENT_THINK,

    /// ( int levelTime );
    /// Source: `oracle/oracle/codemp/game/g_public.h:756`
    GAME_RUN_FRAME,

    /// ( void );
    /// ConsoleCommand will be called when a command has been issued
    /// that is not recognized as a builtin function.
    /// The game can issue trap_argc() / trap_argv() commands to get the command
    /// and parameters.  Return qfalse if the game doesn't recognize it as a command.
    /// Source: `oracle/oracle/codemp/game/g_public.h:758`
    GAME_CONSOLE_COMMAND,

    /// ( int time );
    /// Source: `oracle/oracle/codemp/game/g_public.h:764`
    BOTAI_START_FRAME,

    /// int entnum, char *notetrack
    /// Source: `oracle/oracle/codemp/game/g_public.h:766`
    GAME_ROFF_NOTETRACK_CALLBACK,

    /// rwwRMG - added
    /// rww - icarus callbacks
    /// Source: `oracle/oracle/codemp/game/g_public.h:768`
    GAME_SPAWN_RMG_ENTITY,

    /// Source: `oracle/oracle/codemp/game/g_public.h:771`
    GAME_ICARUS_PLAYSOUND,

    /// Source: `oracle/oracle/codemp/game/g_public.h:772`
    GAME_ICARUS_SET,

    /// Source: `oracle/oracle/codemp/game/g_public.h:773`
    GAME_ICARUS_LERP2POS,

    /// Source: `oracle/oracle/codemp/game/g_public.h:774`
    GAME_ICARUS_LERP2ORIGIN,

    /// Source: `oracle/oracle/codemp/game/g_public.h:775`
    GAME_ICARUS_LERP2ANGLES,

    /// Source: `oracle/oracle/codemp/game/g_public.h:776`
    GAME_ICARUS_GETTAG,

    /// Source: `oracle/oracle/codemp/game/g_public.h:777`
    GAME_ICARUS_LERP2START,

    /// Source: `oracle/oracle/codemp/game/g_public.h:778`
    GAME_ICARUS_LERP2END,

    /// Source: `oracle/oracle/codemp/game/g_public.h:779`
    GAME_ICARUS_USE,

    /// Source: `oracle/oracle/codemp/game/g_public.h:780`
    GAME_ICARUS_KILL,

    /// Source: `oracle/oracle/codemp/game/g_public.h:781`
    GAME_ICARUS_REMOVE,

    /// Source: `oracle/oracle/codemp/game/g_public.h:782`
    GAME_ICARUS_PLAY,

    /// Source: `oracle/oracle/codemp/game/g_public.h:783`
    GAME_ICARUS_GETFLOAT,

    /// Source: `oracle/oracle/codemp/game/g_public.h:784`
    GAME_ICARUS_GETVECTOR,

    /// Source: `oracle/oracle/codemp/game/g_public.h:785`
    GAME_ICARUS_GETSTRING,

    /// Source: `oracle/oracle/codemp/game/g_public.h:786`
    GAME_ICARUS_SOUNDINDEX,

    /// Source: `oracle/oracle/codemp/game/g_public.h:787`
    GAME_ICARUS_GETSETIDFORSTRING,

    /// Source: `oracle/oracle/codemp/game/g_public.h:788`
    GAME_NAV_CLEARPATHTOPOINT,

    /// Source: `oracle/oracle/codemp/game/g_public.h:789`
    GAME_NAV_CLEARLOS,

    /// Source: `oracle/oracle/codemp/game/g_public.h:790`
    GAME_NAV_CLEARPATHBETWEENPOINTS,

    /// Source: `oracle/oracle/codemp/game/g_public.h:791`
    GAME_NAV_CHECKNODEFAILEDFORENT,

    /// Source: `oracle/oracle/codemp/game/g_public.h:792`
    GAME_NAV_ENTISUNLOCKEDDOOR,

    /// Source: `oracle/oracle/codemp/game/g_public.h:793`
    GAME_NAV_ENTISDOOR,

    /// Source: `oracle/oracle/codemp/game/g_public.h:794`
    GAME_NAV_ENTISBREAKABLE,

    /// Source: `oracle/oracle/codemp/game/g_public.h:795`
    GAME_NAV_ENTISREMOVABLEUSABLE,

    /// Source: `oracle/oracle/codemp/game/g_public.h:796`
    GAME_NAV_FINDCOMBATPOINTWAYPOINTS,

    /// Source: `oracle/oracle/codemp/game/g_public.h:798`
    GAME_GETITEMINDEXBYTAG,
}

/// The `vmMain` pre-decode half of the SEAM-D6 enum<->wire-word pair: the raw
/// `c_int` command word is converted fallibly BEFORE the exhaustive dispatch
/// match; an unrecognized command's fallback (game returns `-1`,
/// `g_main.c:695`) lives at the conversion's `Err`, not in a match arm.
///
/// Source: `docs/architecture/engine-seam.md` § inbound dual (SEAM-D6).
impl TryFrom<i32> for MpGameExport {
    type Error = i32;

    fn try_from(v: i32) -> Result<Self, i32> {
        Ok(match v {
            x if x == Self::GAME_INIT as i32 => Self::GAME_INIT,
            x if x == Self::GAME_SHUTDOWN as i32 => Self::GAME_SHUTDOWN,
            x if x == Self::GAME_CLIENT_CONNECT as i32 => Self::GAME_CLIENT_CONNECT,
            x if x == Self::GAME_CLIENT_BEGIN as i32 => Self::GAME_CLIENT_BEGIN,
            x if x == Self::GAME_CLIENT_USERINFO_CHANGED as i32 => Self::GAME_CLIENT_USERINFO_CHANGED,
            x if x == Self::GAME_CLIENT_DISCONNECT as i32 => Self::GAME_CLIENT_DISCONNECT,
            x if x == Self::GAME_CLIENT_COMMAND as i32 => Self::GAME_CLIENT_COMMAND,
            x if x == Self::GAME_CLIENT_THINK as i32 => Self::GAME_CLIENT_THINK,
            x if x == Self::GAME_RUN_FRAME as i32 => Self::GAME_RUN_FRAME,
            x if x == Self::GAME_CONSOLE_COMMAND as i32 => Self::GAME_CONSOLE_COMMAND,
            x if x == Self::BOTAI_START_FRAME as i32 => Self::BOTAI_START_FRAME,
            x if x == Self::GAME_ROFF_NOTETRACK_CALLBACK as i32 => Self::GAME_ROFF_NOTETRACK_CALLBACK,
            x if x == Self::GAME_SPAWN_RMG_ENTITY as i32 => Self::GAME_SPAWN_RMG_ENTITY,
            x if x == Self::GAME_ICARUS_PLAYSOUND as i32 => Self::GAME_ICARUS_PLAYSOUND,
            x if x == Self::GAME_ICARUS_SET as i32 => Self::GAME_ICARUS_SET,
            x if x == Self::GAME_ICARUS_LERP2POS as i32 => Self::GAME_ICARUS_LERP2POS,
            x if x == Self::GAME_ICARUS_LERP2ORIGIN as i32 => Self::GAME_ICARUS_LERP2ORIGIN,
            x if x == Self::GAME_ICARUS_LERP2ANGLES as i32 => Self::GAME_ICARUS_LERP2ANGLES,
            x if x == Self::GAME_ICARUS_GETTAG as i32 => Self::GAME_ICARUS_GETTAG,
            x if x == Self::GAME_ICARUS_LERP2START as i32 => Self::GAME_ICARUS_LERP2START,
            x if x == Self::GAME_ICARUS_LERP2END as i32 => Self::GAME_ICARUS_LERP2END,
            x if x == Self::GAME_ICARUS_USE as i32 => Self::GAME_ICARUS_USE,
            x if x == Self::GAME_ICARUS_KILL as i32 => Self::GAME_ICARUS_KILL,
            x if x == Self::GAME_ICARUS_REMOVE as i32 => Self::GAME_ICARUS_REMOVE,
            x if x == Self::GAME_ICARUS_PLAY as i32 => Self::GAME_ICARUS_PLAY,
            x if x == Self::GAME_ICARUS_GETFLOAT as i32 => Self::GAME_ICARUS_GETFLOAT,
            x if x == Self::GAME_ICARUS_GETVECTOR as i32 => Self::GAME_ICARUS_GETVECTOR,
            x if x == Self::GAME_ICARUS_GETSTRING as i32 => Self::GAME_ICARUS_GETSTRING,
            x if x == Self::GAME_ICARUS_SOUNDINDEX as i32 => Self::GAME_ICARUS_SOUNDINDEX,
            x if x == Self::GAME_ICARUS_GETSETIDFORSTRING as i32 => Self::GAME_ICARUS_GETSETIDFORSTRING,
            x if x == Self::GAME_NAV_CLEARPATHTOPOINT as i32 => Self::GAME_NAV_CLEARPATHTOPOINT,
            x if x == Self::GAME_NAV_CLEARLOS as i32 => Self::GAME_NAV_CLEARLOS,
            x if x == Self::GAME_NAV_CLEARPATHBETWEENPOINTS as i32 => Self::GAME_NAV_CLEARPATHBETWEENPOINTS,
            x if x == Self::GAME_NAV_CHECKNODEFAILEDFORENT as i32 => Self::GAME_NAV_CHECKNODEFAILEDFORENT,
            x if x == Self::GAME_NAV_ENTISUNLOCKEDDOOR as i32 => Self::GAME_NAV_ENTISUNLOCKEDDOOR,
            x if x == Self::GAME_NAV_ENTISDOOR as i32 => Self::GAME_NAV_ENTISDOOR,
            x if x == Self::GAME_NAV_ENTISBREAKABLE as i32 => Self::GAME_NAV_ENTISBREAKABLE,
            x if x == Self::GAME_NAV_ENTISREMOVABLEUSABLE as i32 => Self::GAME_NAV_ENTISREMOVABLEUSABLE,
            x if x == Self::GAME_NAV_FINDCOMBATPOINTWAYPOINTS as i32 => Self::GAME_NAV_FINDCOMBATPOINTWAYPOINTS,
            x if x == Self::GAME_GETITEMINDEXBYTAG as i32 => Self::GAME_GETITEMINDEXBYTAG,
            _ => return Err(v),
        })
    }
}
