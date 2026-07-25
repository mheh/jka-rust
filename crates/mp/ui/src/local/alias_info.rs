//! `AliasInfo` — Raven `aliasInfo`.

/// Raven `aliasInfo` — one bot-alias row parsed out of the team arena file.
///
/// Type definition source: `oracle/codemp/ui/ui_local.h:608-612`
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[doc(alias = "aliasInfo")]
pub struct AliasInfo {
    pub name: String,
    pub ai: String,
    pub action: String,
}
