//! MP `bg_public.h` entity type definitions.
//!
//! Type definition source: `oracle/oracle/codemp/game/bg_public.h:1190-1213`

#![allow(non_camel_case_types)]

/// Raven `entityType_t`.
///
/// Type definition source: `oracle/oracle/codemp/game/bg_public.h:1190-1213`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum entityType_t {
    ET_GENERAL = 0,
    ET_PLAYER = 1,
    ET_ITEM = 2,
    ET_MISSILE = 3,
    ET_SPECIAL = 4,
    ET_HOLOCRON = 5,
    ET_MOVER = 6,
    ET_BEAM = 7,
    ET_PORTAL = 8,
    ET_SPEAKER = 9,
    ET_PUSH_TRIGGER = 10,
    ET_TELEPORT_TRIGGER = 11,
    ET_INVISIBLE = 12,
    ET_NPC = 13,
    ET_TEAM = 14,
    ET_BODY = 15,
    ET_TERRAIN = 16,
    ET_FX = 17,
    ET_EVENTS = 18,
}
