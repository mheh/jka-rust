//! MP `class_t` — NPC class enumeration (`teams.h`).
//!
//! `teams.h` is pulled in by `q_shared.h` (`oracle/codemp/game/q_shared.h:16`),
//! so `class_t` is part of the shared server<->game surface: the engine server
//! (`mp_engine_server::sv_world`, which cannot reach `mp_game`) compares
//! `entity->s.NPC_class == CLASS_VEHICLE`. Following the `Q3_INFINITE`
//! migration precedent (NAV-D3 / RULING 39d, sibling `q3_infinite.rs`) it lives
//! here at the shared tier; `mp_game::teams::class` re-exports it so its own
//! call sites keep resolving.
//!
//! Type definition source: `oracle/codemp/game/teams.h:17-77`

#![allow(non_camel_case_types)]

/// Raven `class_t`.
///
/// Raven: made up from the model directories; MUST be in the same order as the
/// `ClassNames` array in `NPC_stats.cpp`.
/// Type definition source: `oracle/codemp/game/teams.h:17-77`
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
    CLASS_STORMTROOPER,
    CLASS_SWAMP,
    CLASS_SWAMPTROOPER,
    CLASS_TAVION,
    CLASS_TRANDOSHAN,
    CLASS_UGNAUGHT,
    CLASS_JAWA,
    CLASS_WEEQUAY,
    CLASS_BOBAFETT,
    CLASS_VEHICLE,
    CLASS_RANCOR,
    CLASS_WAMPA,

    CLASS_NUM_CLASSES,
}
