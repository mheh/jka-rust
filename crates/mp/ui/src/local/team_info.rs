//! `TeamInfo` — Raven `teamInfo`.

use core::ffi::c_int;

use mp_qshared::shared::qhandle_t;

/// Raven `#define TEAM_MEMBERS 8//5`.
///
/// Source: `oracle/codemp/ui/ui_local.h:581`
pub const TEAM_MEMBERS: usize = 8;

/// Raven `teamInfo` — per-team UI data (name, image, member list, icons).
///
/// Type definition source: `oracle/codemp/ui/ui_local.h:614-622`
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[doc(alias = "teamInfo")]
#[allow(non_snake_case)]
pub struct TeamInfo {
    pub teamName: String,
    pub imageName: String,
    pub teamMembers: [String; TEAM_MEMBERS],
    pub teamIcon: qhandle_t,
    pub teamIcon_Metal: qhandle_t,
    pub teamIcon_Name: qhandle_t,
    pub cinematic: c_int,
}
