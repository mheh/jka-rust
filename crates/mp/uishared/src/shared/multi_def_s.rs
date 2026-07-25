//! `MultiDef` — Raven `multiDef_s`/`multiDef_t`.

/// Raven `#define MAX_MULTI_CVARS 32`.
///
/// Source: `oracle/codemp/ui/ui_shared.h:198`
pub const MAX_MULTI_CVARS: usize = 32;

/// Raven `multiDef_s` (typedef `multiDef_t`) — the multi-value cvar list a
/// combo-box item cycles through, one of the `itemDef_t::typeData` payloads.
///
/// PORT-NOTE: Raven's three parallel `[MAX_MULTI_CVARS]` arrays plus `count`
/// become parallel `Vec`s (`count` is `cvarList.len()`); `strDef` still selects
/// whether `cvarStr` or `cvarValue` carries the value.
///
/// Type definition source: `oracle/codemp/ui/ui_shared.h:200-206`
#[derive(Debug, Clone, PartialEq, Default)]
#[doc(alias = "multiDef_s")]
#[doc(alias = "multiDef_t")]
#[allow(non_snake_case)]
pub struct MultiDef {
    pub cvarList: Vec<String>,
    pub cvarStr: Vec<String>,
    pub cvarValue: Vec<f32>,
    pub strDef: bool,
}
