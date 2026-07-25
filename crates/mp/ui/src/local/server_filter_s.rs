//! `ServerFilter` — Raven `serverFilter_t`.

/// Raven `serverFilter_s` (typedef `serverFilter_t`) — one row of the
/// server-browser mod filter table.
///
/// The only instance is the compiled-in `static const serverFilter_t
/// serverFilters[]`, so both strings stay `&'static str`.
///
/// Type definition source: `oracle/codemp/ui/ui_local.h:649-652`
/// Source: `oracle/codemp/ui/ui_main.c:896-900` (`serverFilters`)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[doc(alias = "serverFilter_s")]
#[doc(alias = "serverFilter_t")]
pub struct ServerFilter {
    pub description: &'static str,
    pub basedir: &'static str,
}
