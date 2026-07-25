//! `ColorRangeDef` — Raven `colorRangeDef_t`.

use mp_qshared::shared::vec4_t;

/// Raven `colorRangeDef_t` — a color range definition.
///
/// Type definition source: `oracle/codemp/ui/ui_shared.h:148-152`
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[doc(alias = "colorRangeDef_t")]
pub struct ColorRangeDef {
    pub color: vec4_t,
    pub low: f32,
    pub high: f32,
}
