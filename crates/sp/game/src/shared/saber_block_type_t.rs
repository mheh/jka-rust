#![allow(non_camel_case_types, non_snake_case)]

/// Raven `saberBlockType_t` — saber block type.
///
/// Raven: Block only attacks and shots around the saber itself, a bbox of around 12x12x12 / Block all attacks in an area around the player in a rough arc of 180 degrees.
/// Type definition source: `oracle/code/game/g_shared.h:352-356`
#[repr(i32)]
pub enum saberBlockType_t {
    BLK_NO = 0,
    BLK_TIGHT = 1,  // Block only attacks and shots around the saber itself, a bbox of around 12x12x12
    BLK_WIDE = 2,   // Block all attacks in an area around the player in a rough arc of 180 degrees
}
