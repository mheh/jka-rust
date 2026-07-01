#![allow(non_camel_case_types)]

/// Raven `waterHeightLevel_t` — how deep an entity is submerged. SP-only.
///
/// Type definition source: `oracle/oracle/code/game/q_shared.h:1603-1613`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum waterHeightLevel_t {
    WHL_NONE,
    WHL_ANKLES,
    WHL_KNEES,
    WHL_WAIST,
    WHL_TORSO,
    WHL_SHOULDERS,
    WHL_HEAD,
    WHL_UNDER,
}
