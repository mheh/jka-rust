//! `ModInfo` — Raven `modInfo_t`.

/// Raven `modInfo_t` — a single loadable-mod list entry.
///
/// PORT-NOTE: the frozen `#[repr(C)]` twin stays in `native_types` for SP's
/// still-faithful `mp_ui`-sibling tree; MP's list is module-private (Class C),
/// so it lands owned here.
///
/// Type definition source: `oracle/codemp/ui/ui_local.h:711-714`
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[doc(alias = "modInfo_t")]
#[allow(non_snake_case)]
pub struct ModInfo {
    pub modName: String,
    pub modDescr: String,
}
