use core::ffi::c_int;

/// Raven `#define TINYCHAR_WIDTH (SMALLCHAR_WIDTH)`.
/// Source: `oracle/codemp/game/q_shared.h:1032`
pub const TINYCHAR_WIDTH: c_int = SMALLCHAR_WIDTH;
/// Raven `#define TINYCHAR_HEIGHT (SMALLCHAR_HEIGHT/2)`.
/// Source: `oracle/codemp/game/q_shared.h:1033`
pub const TINYCHAR_HEIGHT: c_int = SMALLCHAR_HEIGHT / 2;

/// Raven `#define SMALLCHAR_WIDTH 8`.
/// Source: `oracle/codemp/game/q_shared.h:1035`
pub const SMALLCHAR_WIDTH: c_int = 8;
/// Raven `#define SMALLCHAR_HEIGHT 16`.
/// Source: `oracle/codemp/game/q_shared.h:1036`
pub const SMALLCHAR_HEIGHT: c_int = 16;

/// Raven `#define BIGCHAR_WIDTH 16`.
/// Source: `oracle/codemp/game/q_shared.h:1038`
pub const BIGCHAR_WIDTH: c_int = 16;
/// Raven `#define BIGCHAR_HEIGHT 16`.
/// Source: `oracle/codemp/game/q_shared.h:1039`
pub const BIGCHAR_HEIGHT: c_int = 16;

/// Raven `#define GIANTCHAR_WIDTH 32`.
/// Source: `oracle/codemp/game/q_shared.h:1041`
pub const GIANTCHAR_WIDTH: c_int = 32;
/// Raven `#define GIANTCHAR_HEIGHT 48`.
/// Source: `oracle/codemp/game/q_shared.h:1042`
pub const GIANTCHAR_HEIGHT: c_int = 48;
