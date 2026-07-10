use std::os::raw::c_int;

/// Raven alternate route goal flags (`ALTROUTEGOAL_*`) — `aas_altroutegoal_t` selectors.
///
/// Source: `oracle/codemp/game/be_aas.h:172-174`
pub const ALTROUTEGOAL_ALL: c_int = 1;
/// Cluster portals only.
pub const ALTROUTEGOAL_CLUSTERPORTALS: c_int = 2;
/// View portals only.
pub const ALTROUTEGOAL_VIEWPORTALS: c_int = 4;
