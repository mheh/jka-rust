//! SP `class_t` — NPC class enumeration.
//!
//! Type definition source: `oracle/oracle/code/game/teams.h:18-88`

#![allow(non_camel_case_types)]

/// Raven SP `class_t`.
///
/// Raven: made up from the model directories; MUST be in the same order as the
/// `ClassNames` array in `NPC_stats.cpp`. SP's roster differs substantially from
/// MP's (64 variants vs 56; SP is story-NPC oriented).
/// Type definition source: `oracle/oracle/code/game/teams.h:18-88`
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum class_t {
    CLASS_NONE = 0, // hopefully this will never be used by an npc, just covering all bases
    CLASS_ATST,     // technically droid...
    CLASS_BARTENDER,
    CLASS_BESPIN_COP,
    CLASS_CLAW,
    CLASS_COMMANDO,
    CLASS_DESANN,
    CLASS_FISH,
    CLASS_FLIER2,
    CLASS_GALAK,
    CLASS_GLIDER,
    CLASS_GONK, // droid
    CLASS_GRAN,
    CLASS_HOWLER,
    CLASS_RANCOR,
    CLASS_SAND_CREATURE,
    CLASS_WAMPA,
    CLASS_IMPERIAL,
    CLASS_IMPWORKER,
    CLASS_INTERROGATOR, // droid
    CLASS_JAN,
    CLASS_JEDI,
    CLASS_KYLE,
    CLASS_LANDO,
    CLASS_LIZARD,
    CLASS_LUKE,
    CLASS_MARK1,     // droid
    CLASS_MARK2,     // droid
    CLASS_GALAKMECH, // droid
    CLASS_MINEMONSTER,
    CLASS_MONMOTHA,
    CLASS_MORGANKATARN,
    CLASS_MOUSE, // droid
    CLASS_MURJJ,
    CLASS_PRISONER,
    CLASS_PROBE,    // droid
    CLASS_PROTOCOL, // droid
    CLASS_R2D2,     // droid
    CLASS_R5D2,     // droid
    CLASS_REBEL,
    CLASS_REBORN,
    CLASS_REELO,
    CLASS_REMOTE,
    CLASS_RODIAN,
    CLASS_SEEKER, // droid
    CLASS_SENTRY,
    CLASS_SHADOWTROOPER,
    CLASS_SABOTEUR,
    CLASS_STORMTROOPER,
    CLASS_SWAMP,
    CLASS_SWAMPTROOPER,
    CLASS_NOGHRI,
    CLASS_TAVION,
    CLASS_ALORA,
    CLASS_TRANDOSHAN,
    CLASS_UGNAUGHT,
    CLASS_JAWA,
    CLASS_WEEQUAY,
    CLASS_TUSKEN,
    CLASS_BOBAFETT,
    CLASS_ROCKETTROOPER,
    CLASS_SABER_DROID,
    CLASS_ASSASSIN_DROID,
    CLASS_HAZARD_TROOPER,
    CLASS_PLAYER,
    CLASS_VEHICLE,

    CLASS_NUM_CLASSES,
}
