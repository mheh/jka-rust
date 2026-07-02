#![allow(non_camel_case_types, non_snake_case)]

/// Raven `otherhudbits_t` — HUD display element type enumeration.
///
/// Type definition source: `oracle/oracle/code/cgame/cg_media.h:61-75`
#[repr(i32)]
pub enum otherhudbits_t {
    OHB_HEALTHAMOUNT = 0,
    OHB_ARMORAMOUNT,
    OHB_FORCEAMOUNT,
    OHB_AMMOAMOUNT,
    OHB_SABERSTYLE_STRONG,
    OHB_SABERSTYLE_MEDIUM,
    OHB_SABERSTYLE_FAST,
    OHB_SCANLINE_LEFT,
    OHB_SCANLINE_RIGHT,
    OHB_FRAME_LEFT,
    OHB_FRAME_RIGHT,
    OHB_MAX,
}
