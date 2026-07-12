#![allow(non_camel_case_types, non_snake_case)]

/// Raven `characters_t` — player character selector enum, Voyager crew manifest.
///
/// Raven: player character selector enum from Voyager's crew; organized by HazTeam Alpha,
/// HazTeam Beta, Senior Crew, Other Crew, and Generic Crew variants.
/// Type definition source: `oracle/code/game/characters.h:1-45`
#[repr(i32)]
pub enum characters_t {
    // HazTeam Alpha
    CHARACTER_FOSTER = 0,
    CHARACTER_TELSIA,
    CHARACTER_BIESSMAN,
    CHARACTER_CHANG,
    CHARACTER_CHELL,
    CHARACTER_JUROT,
    // HazTeam Beta
    CHARACTER_LAIRD,
    CHARACTER_KENN,
    CHARACTER_OVIEDO,
    CHARACTER_ODELL,
    CHARACTER_NELSON,
    CHARACTER_JAWORSKI,
    CHARACTER_CSATLOS,
    // Senior Crew
    CHARACTER_JANEWAY,
    CHARACTER_CHAKOTAY,
    CHARACTER_TUVOK,
    CHARACTER_TUVOKHAZ,
    CHARACTER_TORRES,
    CHARACTER_PARIS,
    CHARACTER_KIM,
    CHARACTER_DOCTOR,
    CHARACTER_SEVEN,
    CHARACTER_SEVENHAZ,
    CHARACTER_NEELIX,
    // Other Crew
    CHARACTER_PELLETIER,
    // Generic Crew
    CHARACTER_CREWMAN,
    CHARACTER_LT,
    CHARACTER_COMM,
    CHARACTER_CAPT,
    CHARACTER_GENERIC1,
    CHARACTER_GENERIC2,
    CHARACTER_GENERIC3,
    CHARACTER_GENERIC4,
    CHARACTER_NUM_CHARS,
}
