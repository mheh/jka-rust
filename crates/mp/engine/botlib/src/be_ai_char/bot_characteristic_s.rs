#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

/// Raven `bot_characteristic_t` + its `union cvalue` — a single bot
/// characteristic, redesigned (porting-rules §F17; botlib is statically linked,
/// so layout is free) from a `type`-tagged struct-over-union into a Rust sum
/// type. Raven's `type` tag (`CT_INTEGER`/`CT_FLOAT`/`CT_STRING`, or `0` when
/// the slot is uninitialized) becomes the enum discriminant, and the
/// `integer`/`_float`/`string` union arms become the variants' payloads. The
/// owned `String` replaces Raven's malloc'd `char *` (freed on drop, retiring
/// `BotFreeCharacterStrings`).
///
/// Type definition source: `oracle/codemp/botlib/be_ai_char.cpp:39-50`
#[derive(Clone, Default)]
pub enum Characteristic {
    /// Raven `type == 0` — an uninitialized slot (zero-init default), the state
    /// the parser and the `Check`/`Default`/`Interpolate` paths test for.
    #[default]
    None,
    /// Raven `CT_INTEGER` (`cvalue::integer`).
    Integer(c_int),
    /// Raven `CT_FLOAT` (`cvalue::_float`).
    Float(f32),
    /// Raven `CT_STRING` (`cvalue::string`).
    Str(String),
}
