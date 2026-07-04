//! MP `g_public.h` server<->game-module interface consts: `entity->svFlags`
//! bits and Ghoul2 trace flags.
//!
//! Plain `#define` bit flags (not an enum), so §C8 makes them `const`s
//! directly.
//!
//! Source: `oracle/oracle/codemp/game/g_public.h:9-47`

use core::ffi::c_int;

/// Raven `Q3_INFINITE`.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:9`
pub const Q3_INFINITE: c_int = 16777216;

// entity->svFlags — the server does not know how to interpret most of the
// values in entityStates (level eType), so the game must explicitly flag
// special server behaviors.
pub const SVF_NOCLIENT: c_int = 0x0000_0001; // don't send entity to clients, even if it has effects
pub const SVF_BOT: c_int = 0x0000_0008; // set if the entity is a bot
pub const SVF_PLAYER_USABLE: c_int = 0x0000_0010; // player can use this with the use button
pub const SVF_BROADCAST: c_int = 0x0000_0020; // send to all connected clients
pub const SVF_PORTAL: c_int = 0x0000_0040; // merge a second pvs at origin2 into snapshots
pub const SVF_USE_CURRENT_ORIGIN: c_int = 0x0000_0080; // entity->r.currentOrigin instead of entity->s.origin for link position (missiles and movers)
pub const SVF_SINGLECLIENT: c_int = 0x0000_0100; // only send to a single client (entityShared_t->singleClient)
pub const SVF_NOSERVERINFO: c_int = 0x0000_0200; // don't send CS_SERVERINFO updates to this client
pub const SVF_CAPSULE: c_int = 0x0000_0400; // use capsule for collision detection instead of bbox
pub const SVF_NOTSINGLECLIENT: c_int = 0x0000_0800; // send entity to everyone but one client (entityShared_t->singleClient)
pub const SVF_OWNERNOTSHARED: c_int = 0x0000_1000; // if it's owned by something and another thing owned by that something hits it, it will still touch
pub const SVF_ICARUS_FREEZE: c_int = 0x0000_8000; // NPCs are frozen, ents don't execute ICARUS commands
pub const SVF_GLASS_BRUSH: c_int = 0x0800_0000; // Ent is a glass brush
pub const SVF_NO_BASIC_SOUNDS: c_int = 0x1000_0000; // No basic sounds
pub const SVF_NO_COMBAT_SOUNDS: c_int = 0x2000_0000; // No combat sounds
pub const SVF_NO_EXTRA_SOUNDS: c_int = 0x4000_0000; // No extra or jedi sounds

// rww - ghoul2 trace flags.
pub const G2TRFLAG_DOGHOULTRACE: c_int = 0x0000_0001; // do the ghoul2 trace
pub const G2TRFLAG_HITCORPSES: c_int = 0x0000_0002; // will try g2 collision on the ent even if it's EF_DEAD
pub const G2TRFLAG_GETSURFINDEX: c_int = 0x0000_0004; // will replace surfaceFlags with the ghoul2 surface index that was hit, if any
pub const G2TRFLAG_THICK: c_int = 0x0000_0008; // assures that the trace radius will be significantly large regardless of the trace box size
