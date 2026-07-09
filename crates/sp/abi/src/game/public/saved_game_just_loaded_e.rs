#![allow(non_camel_case_types, non_snake_case)]

/// Raven `SavedGameJustLoaded_e` — saved game state indicator.
///
/// Type definition source: `oracle/code/game/g_public.h:54-59`
#[repr(i32)]
pub enum SavedGameJustLoaded_e {
    eNO = 0,
    eFULL = 1,
    eAUTO = 2,
}
