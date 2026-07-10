use std::os::raw::c_int;

/// Raven route prediction stop events (`RSE_*`) — `aas_predictroute_t::stopevent` flags.
///
/// Source: `oracle/codemp/game/be_aas.h:180-184`
pub const RSE_NONE: c_int = 0;
/// No route to goal.
pub const RSE_NOROUTE: c_int = 1;
/// Stop as soon as one of the given travel types is used.
pub const RSE_USETRAVELTYPE: c_int = 2;
/// Stop when entering the given contents.
pub const RSE_ENTERCONTENTS: c_int = 4;
/// Stop when entering the given area.
pub const RSE_ENTERAREA: c_int = 8;
