#![allow(non_camel_case_types)]

/// Raven `saberBlockType_t` saber-blocking coverage.
///
/// Type definition source: `oracle/oracle/codemp/game/q_shared.h:552-556`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum saberBlockType_t {
    BLK_NO,
    /// Raven: Block only attacks and shots around the saber itself, a bbox of around 12x12x12
    BLK_TIGHT,
    /// Raven: Block all attacks in an area around the player in a rough arc of 180 degrees
    BLK_WIDE,
}
