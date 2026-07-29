/// Which module this `MenuSystem` serves - Raven compiles `ui_shared.c`
/// twice (ui, and cgame with `CGAME` defined) and diverges a handful of
/// arms with `#ifndef CGAME`. One crate replaces the twin builds, so the
/// host is a runtime constant stamped at construction (DEC-36 D3, the
/// `BgHost` shape). Only the DEC-47.9 audit's genuinely-reachable
/// divergences branch on it: the `asset_model`/`asset_model_go` and
/// `isSaber`/`isSaber2` parse arms; the paint-family `#ifndef CGAME` sites
/// stay flagged as dead surface (cgame never enters the shared paint path).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiHost {
    Ui,
    Cgame,
}
