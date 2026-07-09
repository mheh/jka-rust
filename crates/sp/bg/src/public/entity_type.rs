//! SP `bg_public.h` entity type enumeration.
//!
//! Type definition source: `oracle/code/game/bg_public.h:713-732`

#![allow(non_camel_case_types)]

/// Raven `entityType_t`.
///
/// Type definition source: `oracle/code/game/bg_public.h:713-732`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum entityType_t {
    ET_GENERAL = 0,
    ET_PLAYER = 1,
    ET_ITEM = 2,
    ET_MISSILE = 3,
    ET_MOVER = 4,
    ET_BEAM = 5,
    ET_PORTAL = 6,
    ET_SPEAKER = 7,
    ET_PUSH_TRIGGER = 8,
    ET_TELEPORT_TRIGGER = 9,
    ET_INVISIBLE = 10,
    ET_THINKER = 11,
    ET_CLOUD = 12,  // dumb
    ET_TERRAIN = 13,
    ET_EVENTS = 14,         // any of the EV_* events can be added freestanding
                            // by setting eType to ET_EVENTS + eventNum
                            // this avoids having to set eFlags and eventNum
}
