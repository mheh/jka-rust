#![allow(non_camel_case_types, non_snake_case)]

/// Raven `CM_ANGLE1` — `usercmd_t` delta bit: angles[0] changed.
/// Source: `oracle/codemp/qcommon/msg.cpp:668`
pub const CM_ANGLE1: i32 = 1 << 0;

/// Raven `CM_ANGLE2` — `usercmd_t` delta bit: angles[1] changed.
/// Source: `oracle/codemp/qcommon/msg.cpp:669`
pub const CM_ANGLE2: i32 = 1 << 1;

/// Raven `CM_ANGLE3` — `usercmd_t` delta bit: angles[2] changed.
/// Source: `oracle/codemp/qcommon/msg.cpp:670`
pub const CM_ANGLE3: i32 = 1 << 2;

/// Raven `CM_FORWARD` — `usercmd_t` delta bit: forwardmove changed.
/// Source: `oracle/codemp/qcommon/msg.cpp:671`
pub const CM_FORWARD: i32 = 1 << 3;

/// Raven `CM_SIDE` — `usercmd_t` delta bit: rightmove changed.
/// Source: `oracle/codemp/qcommon/msg.cpp:672`
pub const CM_SIDE: i32 = 1 << 4;

/// Raven `CM_UP` — `usercmd_t` delta bit: upmove changed.
/// Source: `oracle/codemp/qcommon/msg.cpp:673`
pub const CM_UP: i32 = 1 << 5;

/// Raven `CM_BUTTONS` — `usercmd_t` delta bit: buttons changed.
/// Source: `oracle/codemp/qcommon/msg.cpp:674`
pub const CM_BUTTONS: i32 = 1 << 6;

/// Raven `CM_WEAPON` — `usercmd_t` delta bit: weapon changed.
/// Source: `oracle/codemp/qcommon/msg.cpp:675`
pub const CM_WEAPON: i32 = 1 << 7;

/// Raven `CM_FORCE` — `usercmd_t` delta bit: force-power selection changed.
/// Source: `oracle/codemp/qcommon/msg.cpp:677`
pub const CM_FORCE: i32 = 1 << 8;

/// Raven `CM_INVEN` — `usercmd_t` delta bit: inventory selection changed.
/// Source: `oracle/codemp/qcommon/msg.cpp:678`
pub const CM_INVEN: i32 = 1 << 9;

/// Raven `FLOAT_INT_BITS` — bit width used to pack a small float as an int in
/// a delta message.
/// Source: `oracle/codemp/qcommon/msg.cpp:1055`
pub const FLOAT_INT_BITS: i32 = 13;

/// Raven `FLOAT_INT_BIAS` — bias added when packing/unpacking `FLOAT_INT_BITS`
/// values.
/// Source: `oracle/codemp/qcommon/msg.cpp:1056`
pub const FLOAT_INT_BIAS: i32 = 1 << (FLOAT_INT_BITS - 1);
