#![allow(non_camel_case_types)]

/// Raven `ha_pref` Hunk_Alloc allocation preference.
///
/// Type definition source: `oracle/codemp/game/q_shared.h:504-508`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ha_pref {
    h_high,
    h_low,
    h_dontcare,
}
