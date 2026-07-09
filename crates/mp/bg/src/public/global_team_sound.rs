//! MP `bg_public.h` global team sound definitions.
//!
//! Type definition source: `oracle/codemp/game/bg_public.h:993-1005`

#![allow(non_camel_case_types)]

/// Raven `global_team_sound_t`.
///
/// Type definition source: `oracle/codemp/game/bg_public.h:993-1005`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum global_team_sound_t {
    GTS_RED_CAPTURE = 0,
    GTS_BLUE_CAPTURE = 1,
    GTS_RED_RETURN = 2,
    GTS_BLUE_RETURN = 3,
    GTS_RED_TAKEN = 4,
    GTS_BLUE_TAKEN = 5,
    GTS_REDTEAM_SCORED = 6,
    GTS_BLUETEAM_SCORED = 7,
    GTS_REDTEAM_TOOK_LEAD = 8,
    GTS_BLUETEAM_TOOK_LEAD = 9,
    GTS_TEAMS_ARE_TIED = 10,
}
