//! MP `g_public.h` server<->game-module interface consts: `entity->svFlags`
//! bits and Ghoul2 trace flags.
//!
//! Plain `#define` bit flags (not an enum), so §C8 makes them `const`s
//! directly.
//!
//! Source: `oracle/codemp/game/g_public.h:9-47`

use core::ffi::c_int;

/// Raven `Q3_INFINITE`.
///
/// Source: `oracle/codemp/game/g_public.h:9`
pub const Q3_INFINITE: c_int = 16777216;

// entity->svFlags and Ghoul2 trace flags live at the shared tier
// (`mp_qshared::common::mp::game::g_public`) so the engine server can see this
// `g_public.h` server<->game contract; re-exported here so `mp_game`'s own call
// sites (and the `prelude` glob) keep resolving.
pub use mp_qshared::common::mp::game::g_public::{
    G2TRFLAG_DOGHOULTRACE, G2TRFLAG_GETSURFINDEX, G2TRFLAG_HITCORPSES, G2TRFLAG_THICK, SVF_BOT,
    SVF_BROADCAST, SVF_CAPSULE, SVF_GLASS_BRUSH, SVF_ICARUS_FREEZE, SVF_NOCLIENT, SVF_NOSERVERINFO,
    SVF_NOTSINGLECLIENT, SVF_NO_BASIC_SOUNDS, SVF_NO_COMBAT_SOUNDS, SVF_NO_EXTRA_SOUNDS,
    SVF_OWNERNOTSHARED, SVF_PLAYER_USABLE, SVF_PORTAL, SVF_SINGLECLIENT, SVF_USE_CURRENT_ORIGIN,
};
