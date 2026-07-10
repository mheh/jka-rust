use std::os::raw::c_int;

/// Raven client movement prediction stop events (`SE_*`) — `aas_clientmove_t::stopevent` flags.
///
/// Source: `oracle/codemp/game/be_aas.h:144-157`
pub const SE_NONE: c_int = 0;
/// The ground is hit.
pub const SE_HITGROUND: c_int = 1;
/// There's no ground.
pub const SE_LEAVEGROUND: c_int = 2;
/// Water is entered.
pub const SE_ENTERWATER: c_int = 4;
/// Slime is entered.
pub const SE_ENTERSLIME: c_int = 8;
/// Lava is entered.
pub const SE_ENTERLAVA: c_int = 16;
/// The ground is hit with damage.
pub const SE_HITGROUNDDAMAGE: c_int = 32;
/// There's a gap.
pub const SE_GAP: c_int = 64;
/// Touching a jump pad area.
pub const SE_TOUCHJUMPPAD: c_int = 128;
/// Touching teleporter.
pub const SE_TOUCHTELEPORTER: c_int = 256;
/// The given stoparea is entered.
pub const SE_ENTERAREA: c_int = 512;
/// A ground face in the area is hit.
pub const SE_HITGROUNDAREA: c_int = 1024;
/// Hit the specified bounding box.
pub const SE_HITBOUNDINGBOX: c_int = 2048;
/// Touching a cluster portal.
pub const SE_TOUCHCLUSTERPORTAL: c_int = 4096;
