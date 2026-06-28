//! `mp game export surface` — the commands the engine sends to the module via `vmMain`.
//!
//! Transcribed verbatim (order = value) from the **original Raven JKA**
//! `refs/raven-jediacademy/codemp/game/g_public.h`. The numbering is sequential from 0. The
//! per-variant comments are carried over from g_public.h.

#![allow(non_camel_case_types)] // C enumerator names kept for 1:1 traceability

use core::ffi::c_int;

#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MpGameExport {
    /// `( int levelTime, int randomSeed, int restart )`
    ///
    /// init and shutdown will be called every single level.
    /// The game should call G_GET_ENTITY_TOKEN to parse through all the
    /// entity configuration text and spawn gentities.
    GAME_INIT,

    /// `(void)`
    GAME_SHUTDOWN,

    /// `( int clientNum, qboolean firstTime, qboolean isBot )`
    ///
    /// return NULL if the client is allowed to connect, otherwise return
    /// a text string with the reason for denial.
    GAME_CLIENT_CONNECT,

    /// `( int clientNum )`
    GAME_CLIENT_BEGIN,

    /// `( int clientNum )`
    GAME_CLIENT_USERINFO_CHANGED,

    /// `( int clientNum )`
    GAME_CLIENT_DISCONNECT,

    /// `( int clientNum )`
    GAME_CLIENT_COMMAND,

    /// `( int clientNum )`
    GAME_CLIENT_THINK,

    /// `( int levelTime )`
    GAME_RUN_FRAME,

    /// `( void )`
    ///
    /// ConsoleCommand will be called when a command has been issued
    /// that is not recognized as a builtin function.
    /// The game can issue trap_argc() / trap_argv() commands to get the command
    /// and parameters.  Return qfalse if the game doesn't recognize it as a command.
    GAME_CONSOLE_COMMAND,

    /// `( int time )`
    BOTAI_START_FRAME,

    /// `( int entnum, char *notetrack )`
    GAME_ROFF_NOTETRACK_CALLBACK,

    GAME_SPAWN_RMG_ENTITY, // rwwRMG - added

    // rww - icarus callbacks
    GAME_ICARUS_PLAYSOUND,
    GAME_ICARUS_SET,
    GAME_ICARUS_LERP2POS,
    GAME_ICARUS_LERP2ORIGIN,
    GAME_ICARUS_LERP2ANGLES,
    GAME_ICARUS_GETTAG,
    GAME_ICARUS_LERP2START,
    GAME_ICARUS_LERP2END,
    GAME_ICARUS_USE,
    GAME_ICARUS_KILL,
    GAME_ICARUS_REMOVE,
    GAME_ICARUS_PLAY,
    GAME_ICARUS_GETFLOAT,
    GAME_ICARUS_GETVECTOR,
    GAME_ICARUS_GETSTRING,
    GAME_ICARUS_SOUNDINDEX,
    GAME_ICARUS_GETSETIDFORSTRING,
    GAME_NAV_CLEARPATHTOPOINT,
    GAME_NAV_CLEARLOS,
    GAME_NAV_CLEARPATHBETWEENPOINTS,
    GAME_NAV_CHECKNODEFAILEDFORENT,
    GAME_NAV_ENTISUNLOCKEDDOOR,
    GAME_NAV_ENTISDOOR,
    GAME_NAV_ENTISBREAKABLE,
    GAME_NAV_ENTISREMOVABLEUSABLE,
    GAME_NAV_FINDCOMBATPOINTWAYPOINTS,
    GAME_GETITEMINDEXBYTAG,
}

impl MpGameExport {
    /// Convert a raw `vmMain` command integer into a known export, or `None` if
    /// the engine sent a value outside the original-JKA range.
    pub fn from_raw(v: c_int) -> Option<MpGameExport> {
        if (MpGameExport::GAME_INIT as c_int..=MpGameExport::GAME_GETITEMINDEXBYTAG as c_int)
            .contains(&v)
        {
            // SAFETY: `MpGameExport` is `repr(i32)` and contiguous from 0, and `v`
            // was just bounds-checked against its first and last discriminants.
            Some(unsafe { core::mem::transmute::<i32, MpGameExport>(v) })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::MpGameExport::*;
    use core::ffi::c_int;

    #[test]
    fn discriminants_match_original_jka() {
        assert_eq!(GAME_INIT as c_int, 0);
        assert_eq!(GAME_RUN_FRAME as c_int, 8);
        assert_eq!(GAME_CONSOLE_COMMAND as c_int, 9);
        assert_eq!(GAME_ICARUS_PLAYSOUND as c_int, 13);
        assert_eq!(GAME_GETITEMINDEXBYTAG as c_int, 39);
    }

    #[test]
    fn from_raw_roundtrips_and_rejects_out_of_range() {
        assert_eq!(super::MpGameExport::from_raw(8), Some(GAME_RUN_FRAME));
        assert_eq!(super::MpGameExport::from_raw(-1), None);
        assert_eq!(super::MpGameExport::from_raw(40), None);
    }
}
