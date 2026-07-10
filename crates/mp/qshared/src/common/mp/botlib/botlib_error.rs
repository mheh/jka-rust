//! MP `botlib.h` botlib error codes.
//!
//! Plain `#define`s (not an enum), so §C8 makes them `const`s directly.
//!
//! Source: `oracle/codemp/game/botlib.h:50-63`

use core::ffi::c_int;

/// Raven `BLERR_NOERROR` — no error.
///
/// Source: `oracle/codemp/game/botlib.h:51`
pub const BLERR_NOERROR: c_int = 0;

/// Raven `BLERR_LIBRARYNOTSETUP` — library not setup.
///
/// Source: `oracle/codemp/game/botlib.h:52`
pub const BLERR_LIBRARYNOTSETUP: c_int = 1;

/// Raven `BLERR_INVALIDENTITYNUMBER` — invalid entity number.
///
/// Source: `oracle/codemp/game/botlib.h:53`
pub const BLERR_INVALIDENTITYNUMBER: c_int = 2;

/// Raven `BLERR_NOAASFILE` — no AAS file available.
///
/// Source: `oracle/codemp/game/botlib.h:54`
pub const BLERR_NOAASFILE: c_int = 3;

/// Raven `BLERR_CANNOTOPENAASFILE` — cannot open AAS file.
///
/// Source: `oracle/codemp/game/botlib.h:55`
pub const BLERR_CANNOTOPENAASFILE: c_int = 4;

/// Raven `BLERR_WRONGAASFILEID` — incorrect AAS file id.
///
/// Source: `oracle/codemp/game/botlib.h:56`
pub const BLERR_WRONGAASFILEID: c_int = 5;

/// Raven `BLERR_WRONGAASFILEVERSION` — incorrect AAS file version.
///
/// Source: `oracle/codemp/game/botlib.h:57`
pub const BLERR_WRONGAASFILEVERSION: c_int = 6;

/// Raven `BLERR_CANNOTREADAASLUMP` — cannot read AAS file lump.
///
/// Source: `oracle/codemp/game/botlib.h:58`
pub const BLERR_CANNOTREADAASLUMP: c_int = 7;

/// Raven `BLERR_CANNOTLOADICHAT` — cannot load initial chats.
///
/// Source: `oracle/codemp/game/botlib.h:59`
pub const BLERR_CANNOTLOADICHAT: c_int = 8;

/// Raven `BLERR_CANNOTLOADITEMWEIGHTS` — cannot load item weights.
///
/// Source: `oracle/codemp/game/botlib.h:60`
pub const BLERR_CANNOTLOADITEMWEIGHTS: c_int = 9;

/// Raven `BLERR_CANNOTLOADITEMCONFIG` — cannot load item config.
///
/// Source: `oracle/codemp/game/botlib.h:61`
pub const BLERR_CANNOTLOADITEMCONFIG: c_int = 10;

/// Raven `BLERR_CANNOTLOADWEAPONWEIGHTS` — cannot load weapon weights.
///
/// Source: `oracle/codemp/game/botlib.h:62`
pub const BLERR_CANNOTLOADWEAPONWEIGHTS: c_int = 11;

/// Raven `BLERR_CANNOTLOADWEAPONCONFIG` — cannot load weapon config.
///
/// Source: `oracle/codemp/game/botlib.h:63`
pub const BLERR_CANNOTLOADWEAPONCONFIG: c_int = 12;
