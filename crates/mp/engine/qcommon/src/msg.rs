#![allow(non_snake_case, non_camel_case_types)]
//! `msg.cpp` — the bit-stream read/write layer (`msg_t`) used for net-channel
//! messages, demo/save serialization, and the entity/playerstate delta coder.
//!
//! Source: `oracle/codemp/qcommon/msg.cpp`

use core::ffi::{c_char, c_int};

use mp_qshared::common::mp::qcommon::entity_state::entityState_t;
use mp_qshared::common::mp::qcommon::msg_t::msg_t;
use mp_qshared::common::mp::qcommon::player_state::playerState_t;
use mp_qshared::common::mp::qcommon::usercmd::usercmd_t;
use mp_qshared::shared::{errorParm_t, qboolean, qfalse, qtrue};

use crate::common::Common;

// PORT-NOTE(q_math-reach): `Q_strncpyz` (q_shared primitive) is ported in
// `mp_game`, a tier above this crate's dependency graph (cm_load.rs/
// files_common.rs precedent) — not reachable here. Referenced by its exact
// Raven name.
extern "Rust" {
    fn Q_strncpyz(dest: *mut c_char, src: *const c_char, destsize: c_int);
}

// PORT-NOTE(cross-crate): `Server`/`SV_GentityNum` (oracle/codemp/qcommon/../
// server/server.h:233; server/sv_game.cpp:58) live in `mp_engine_server`,
// which itself depends on THIS crate (`mp_engine_qcommon`) — importing them
// here would be a dependency cycle. Referenced by bare (unqualified) name
// per the resolved signature exactly as the packet prints it; reported as a
// shape_mismatch for the integration pass to resolve (likely: move these two
// fns' destination, or split a shared sub-crate).
//
// PORT-NOTE(same-file callees): `MSG_WriteBits`/`MSG_ReadBits`/`MSG_WriteByte`/
// `MSG_WriteShort`/`MSG_WriteData`/`MSG_ReadLong` land in this SAME
// destination module (`msg.rs`) via sibling packets not in this shard;
// called unqualified, no import needed once they land.
//
// PORT-NOTE(cross-crate): `Com_Error` (ruling 1, receiverless panic) is
// referenced unqualified; its landing module is common's error path, not yet
// present at transcription time.

/// Raven `MSG_Clear`.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:94-98`
pub fn MSG_Clear(buf: *mut msg_t) {
    unsafe {
        (*buf).cursize = 0;
        (*buf).overflowed = qfalse;
        (*buf).bit = 0; //<- in bits
    }
}

/// Raven `MSG_Bitstream`.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:101-103`
pub fn MSG_Bitstream(buf: *mut msg_t) {
    unsafe {
        (*buf).oob = qfalse;
    }
}

/// Raven `MSG_BeginReading`.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:105-109`
pub fn MSG_BeginReading(msg: *mut msg_t) {
    unsafe {
        (*msg).readcount = 0;
        (*msg).bit = 0;
        (*msg).oob = qfalse;
    }
}

/// Raven `MSG_BeginReadingOOB`.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:111-115`
pub fn MSG_BeginReadingOOB(msg: *mut msg_t) {
    unsafe {
        (*msg).readcount = 0;
        (*msg).bit = 0;
        (*msg).oob = qtrue;
    }
}

/// Raven `MSG_shutdownHuffman`. `_NEWHUFFTABLE_`'s `fp`-close body is
/// debug-only tooling with no reachable `fp` global in this port; the
/// `#ifdef` gate is faithfully dead under our build (no `_NEWHUFFTABLE_`).
///
/// Source: `oracle/codemp/qcommon/msg.cpp:3274-3282`
pub fn MSG_shutdownHuffman() {
    // PORT-NOTE(_NEWHUFFTABLE_): the `fp`/`fclose` debug body is compiled out
    // (no `_NEWHUFFTABLE_` define); nothing to port.
}

/// Raven `msg_hData` — static Huffman frequency table for `MSG_initHuffman`.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:2958-3215`
pub const MSG_H_DATA: [i32; 256] = [
    250315, 41193, 6292, 7106, 3730, 3750, 6110, 23283, 33317, 6950, 7838, 9714, 9257, 17259, 3949,
    1778, 8288, 1604, 1590, 1663, 1100, 1213, 1238, 1134, 1749, 1059, 1246, 1149, 1273, 4486, 2805,
    3472, 21819, 1159, 1670, 1066, 1043, 1012, 1053, 1070, 1726, 888, 1180, 850, 960, 780, 1752,
    3296, 10630, 4514, 5881, 2685, 4650, 3837, 2093, 1867, 2584, 1949, 1972, 940, 1134, 1788, 1670,
    1206, 5719, 6128, 7222, 6654, 3710, 3795, 1492, 1524, 2215, 1140, 1355, 971, 2180, 1248, 1328,
    1195, 1770, 1078, 1264, 1266, 1168, 965, 1155, 1186, 1347, 1228, 1529, 1600, 2617, 2048, 2546,
    3275, 2410, 3585, 2504, 2800, 2675, 6146, 3663, 2840, 14253, 3164, 2221, 1687, 3208, 2739,
    3512, 4796, 4091, 3515, 5288, 4016, 7937, 6031, 5360, 3924, 4892, 3743, 4566, 4807, 5852, 6400,
    6225, 8291, 23243, 7838, 7073, 8935, 5437, 4483, 3641, 5256, 5312, 5328, 5370, 3492, 2458,
    1694, 1821, 2121, 1916, 1149, 1516, 1367, 1236, 1029, 1258, 1104, 1245, 1006, 1149, 1025, 1241,
    952, 1287, 997, 1713, 1009, 1187, 879, 1099, 929, 1078, 951, 1656, 930, 1153, 1030, 1262, 1062,
    1214, 1060, 1621, 930, 1106, 912, 1034, 892, 1158, 990, 1175, 850, 1121, 903, 1087, 920, 1144,
    1056, 3462, 2240, 4397, 12136, 7758, 1345, 1307, 3278, 1950, 886, 1023, 1112, 1077, 1042, 1061,
    1071, 1484, 1001, 1096, 915, 1052, 995, 1070, 876, 1111, 851, 1059, 805, 1112, 923, 1103, 817,
    1899, 1872, 976, 841, 1127, 956, 1159, 950, 7791, 954, 1289, 933, 1127, 3207, 1020, 927, 1355,
    768, 1040, 745, 952, 805, 1073, 740, 1013, 805, 1008, 796, 996, 1057, 11457, 13504,
];

/// Raven `MSG_initHuffman`.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:3219-3234`
pub fn MSG_initHuffman(common: &mut Common) {
    // PORT-NOTE(_NEWHUFFTABLE_): `fp=fopen(...)` debug-log open is compiled
    // out (no `_NEWHUFFTABLE_` define).
    common.msg_init = qtrue;
    //TODO: Port Huff_Init
    // Source: oracle/codemp/qcommon/msg.cpp:3227
    crate::qcommon::huff::Huff_Init(&mut common.msg_huff);
    for i in 0..256usize {
        for _j in 0..MSG_H_DATA[i] {
            //TODO: Port Huff_addRef
            // Source: oracle/codemp/qcommon/msg.cpp:3230-3231
            crate::qcommon::huff::Huff_addRef(&mut common.msg_huff.compressor, i as u8); // Do update
            crate::qcommon::huff::Huff_addRef(&mut common.msg_huff.decompressor, i as u8);
            // Do update
        }
    }
}

/// Raven `MSG_WriteChar`. `PARANOID` range-check guard compiles out (no
/// `PARANOID` define in this build).
///
/// Source: `oracle/codemp/qcommon/msg.cpp:280-287`
pub fn MSG_WriteChar(common: &mut Common, sb: *mut msg_t, c: c_int) {
    let _ = errorParm_t::ERR_FATAL; // PARANOID range-check compiles out; cited for parity reference
    MSG_WriteBits(common, sb, c, 8);
}

/// Raven `MSG_WriteFloat`.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:318-326`
pub fn MSG_WriteFloat(common: &mut Common, sb: *mut msg_t, f: f32) {
    let l: i32 = f.to_bits() as i32;
    MSG_WriteBits(common, sb, l, 32);
}

/// Raven `MSG_WriteBigString`.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:356-383`
pub fn MSG_WriteBigString(common: &mut Common, sb: *mut msg_t, s: *const c_char) {
    unsafe {
        if s.is_null() {
            MSG_WriteData(common, sb, b"\0".as_ptr() as *const (), 1);
        } else {
            let l = crate::common_fns::strlen(s);
            if l >= mp_qshared::shared::limits::BIG_INFO_STRING {
                crate::common::com_printf(common, "MSG_WriteString: BIG_INFO_STRING");
                MSG_WriteData(common, sb, b"\0".as_ptr() as *const (), 1);
                return;
            }
            let mut string = [0u8; mp_qshared::shared::limits::BIG_INFO_STRING];
            Q_strncpyz(string.as_mut_ptr() as *mut c_char, s, string.len() as c_int);

            // eurofix: remove this so we can chat in european languages...	-ste
            // (0xff-strip loop left commented in the oracle; not ported)

            MSG_WriteData(common, sb, string.as_ptr() as *const (), l as c_int + 1);
        }
    }
}

/// Raven `MSG_WriteAngle`.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:385-387`
pub fn MSG_WriteAngle(common: &mut Common, sb: *mut msg_t, f: f32) {
    MSG_WriteByte(common, sb, (f * 256.0 / 360.0) as c_int & 255);
}

/// Raven `MSG_WriteAngle16`. `ANGLE2SHORT(x)` = `(int)((x)*65536/360) & 65535`.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:389-391`
pub fn MSG_WriteAngle16(common: &mut Common, sb: *mut msg_t, f: f32) {
    MSG_WriteShort(common, sb, ((f * 65536.0 / 360.0) as c_int) & 65535);
}

/// Raven `MSG_ReadChar`.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:401-410`
pub fn MSG_ReadChar(common: &mut Common, msg: *mut msg_t) -> c_int {
    let mut c = MSG_ReadBits(common, msg, 8) as i8 as c_int;
    unsafe {
        if (*msg).readcount > (*msg).cursize {
            c = -1;
        }
    }
    c
}

/// Raven `MSG_ReadByte`.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:412-420`
pub fn MSG_ReadByte(common: &mut Common, msg: *mut msg_t) -> c_int {
    let mut c = MSG_ReadBits(common, msg, 8) as u8 as c_int;
    unsafe {
        if (*msg).readcount > (*msg).cursize {
            c = -1;
        }
    }
    c
}

/// Raven `MSG_ReadShort`.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:422-431`
pub fn MSG_ReadShort(common: &mut Common, msg: *mut msg_t) -> c_int {
    let mut c = MSG_ReadBits(common, msg, 16) as i16 as c_int;
    unsafe {
        if (*msg).readcount > (*msg).cursize {
            c = -1;
        }
    }
    c
}

/// Raven `MSG_ReadFloat`.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:444-457`
pub fn MSG_ReadFloat(common: &mut Common, msg: *mut msg_t) -> f32 {
    let l = MSG_ReadBits(common, msg, 32);
    let mut f = f32::from_bits(l as u32);
    unsafe {
        if (*msg).readcount > (*msg).cursize {
            f = -1.0;
        }
    }
    f
}

/// Raven `MSG_WriteDelta`.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:567-574`
pub fn MSG_WriteDelta(common: &mut Common, msg: *mut msg_t, oldV: c_int, newV: c_int, bits: c_int) {
    if oldV == newV {
        MSG_WriteBits(common, msg, 0, 1);
        return;
    }
    MSG_WriteBits(common, msg, 1, 1);
    MSG_WriteBits(common, msg, newV, bits);
}

/// Raven `MSG_ReadDelta`.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:576-581`
pub fn MSG_ReadDelta(common: &mut Common, msg: *mut msg_t, oldV: c_int, bits: c_int) -> c_int {
    if MSG_ReadBits(common, msg, 1) != 0 {
        return MSG_ReadBits(common, msg, bits);
    }
    oldV
}

/// Raven `MSG_WriteDeltaFloat`.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:583-590`
pub fn MSG_WriteDeltaFloat(common: &mut Common, msg: *mut msg_t, oldV: f32, newV: f32) {
    if oldV == newV {
        MSG_WriteBits(common, msg, 0, 1);
        return;
    }
    MSG_WriteBits(common, msg, 1, 1);
    MSG_WriteBits(common, msg, newV.to_bits() as i32, 32);
}

/// Raven `MSG_ReadDeltaFloat`.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:592-600`
pub fn MSG_ReadDeltaFloat(common: &mut Common, msg: *mut msg_t, oldV: f32) -> f32 {
    if MSG_ReadBits(common, msg, 1) != 0 {
        let newV = f32::from_bits(MSG_ReadBits(common, msg, 32) as u32);
        return newV;
    }
    oldV
}

/// Raven `kbitmask` — precomputed `(1 << bits) - 1` masks for
/// `MSG_ReadDeltaKey`.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:610-619`
pub const KBITMASK: [i32; 32] = [
    0x00000001, 0x00000003, 0x00000007, 0x0000000F, 0x0000001F, 0x0000003F, 0x0000007F, 0x000000FF,
    0x000001FF, 0x000003FF, 0x000007FF, 0x00000FFF, 0x00001FFF, 0x00003FFF, 0x00007FFF, 0x0000FFFF,
    0x0001FFFF, 0x0003FFFF, 0x0007FFFF, 0x000FFFFF, 0x001FFFFF, 0x003FFFFF, 0x007FFFFF, 0x00FFFFFF,
    0x01FFFFFF, 0x03FFFFFF, 0x07FFFFFF, 0x0FFFFFFF, 0x1FFFFFFF, 0x3FFFFFFF, 0x7FFFFFFF, -1i32,
];

/// Raven `MSG_WriteDeltaKey`.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:621-630`
pub fn MSG_WriteDeltaKey(
    common: &mut Common,
    msg: *mut msg_t,
    key: c_int,
    oldV: c_int,
    newV: c_int,
    bits: c_int,
) {
    if oldV == newV {
        MSG_WriteBits(common, msg, 0, 1);
        return;
    }
    MSG_WriteBits(common, msg, 1, 1);
    MSG_WriteBits(common, msg, (newV ^ key) & ((1 << bits) - 1), bits);
}

/// Raven `MSG_ReadDeltaKey`.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:632-637`
pub fn MSG_ReadDeltaKey(
    common: &mut Common,
    msg: *mut msg_t,
    key: c_int,
    oldV: c_int,
    bits: c_int,
) -> c_int {
    if MSG_ReadBits(common, msg, 1) != 0 {
        return MSG_ReadBits(common, msg, bits) ^ (key & KBITMASK[bits as usize]);
    }
    oldV
}

/// Raven `MSG_WriteDeltaKeyFloat`.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:639-646`
pub fn MSG_WriteDeltaKeyFloat(
    common: &mut Common,
    msg: *mut msg_t,
    key: c_int,
    oldV: f32,
    newV: f32,
) {
    if oldV == newV {
        MSG_WriteBits(common, msg, 0, 1);
        return;
    }
    MSG_WriteBits(common, msg, 1, 1);
    MSG_WriteBits(common, msg, (newV.to_bits() as i32) ^ key, 32);
}

/// Raven `MSG_ReadDeltaKeyFloat`.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:648-656`
pub fn MSG_ReadDeltaKeyFloat(common: &mut Common, msg: *mut msg_t, key: c_int, oldV: f32) -> f32 {
    if MSG_ReadBits(common, msg, 1) != 0 {
        let newV = f32::from_bits((MSG_ReadBits(common, msg, 32) ^ key) as u32);
        return newV;
    }
    oldV
}

/// Raven `MSG_ReportChangeVectors_f`. Debug tooling gated by
/// `!_XBOX && !FINAL_BUILD`; `FINAL_BUILD` is undefined for this build
/// (porting-rules ledger), so the body is live.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:3291-3316`
pub fn MSG_ReportChangeVectors_f(common: &mut Common) {
    //TODO: Port entityStateFields
    // Source: oracle/codemp/qcommon/msg.cpp:859-1051
    let num_entity_fields = common.entity_state_fields.len();
    crate::common::com_printf(common, "Entity State Fields:\n");
    for field in common.entity_state_fields.iter_mut() {
        crate::common::com_printf(common, &format!("{}\t\t{}\n", field.name, field.mCount));
        field.mCount = 0;
    }
    let _ = num_entity_fields;

    //TODO: Port playerStateFields
    // Source: oracle/codemp/qcommon/msg.cpp:1410-1568
    crate::common::com_printf(common, "\nPlayer State Fields:\n");
    for field in common.player_state_fields.iter_mut() {
        crate::common::com_printf(common, &format!("{}\t\t{}\n", field.name, field.mCount));
        field.mCount = 0;
    }
}

/// Raven `MSG_ReadString`.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:459-495`
pub fn MSG_ReadString(common: &mut Common, msg: *mut msg_t) -> *mut c_char {
    // §19: Raven's `static char string[MAX_STRING_CHARS]` scratch buffer —
    // ruling-3 rotating-scratch case, owned as a field on `Common` in place
    // of a hidden `static`.
    let cap = mp_qshared::shared::limits::MAX_STRING_CHARS;
    let mut l: usize = 0;
    loop {
        let c = MSG_ReadByte(common, msg);
        if c == -1 || c == 0 {
            break;
        }
        let mut c = c;
        // translate all fmt spec to avoid crash bugs
        if c == '%' as c_int {
            c = '.' as c_int;
        }
        common.msg_read_string_buf[l] = c as u8;
        l += 1;
        if l > cap - 1 {
            break;
        }
    }
    // some bonus protection, shouldn't occur cause server doesn't write such things
    if l <= cap - 1 {
        common.msg_read_string_buf[l] = 0;
    } else {
        common.msg_read_string_buf[cap - 1] = 0;
    }
    common.msg_read_string_buf.as_mut_ptr() as *mut c_char
}

/// Raven `MSG_ReadBigString`.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:497-519`
pub fn MSG_ReadBigString(common: &mut Common, msg: *mut msg_t) -> *mut c_char {
    let cap = mp_qshared::shared::limits::BIG_INFO_STRING;
    let mut l: usize = 0;
    loop {
        let c = MSG_ReadByte(common, msg);
        if c == -1 || c == 0 {
            break;
        }
        let mut c = c;
        // translate all fmt spec to avoid crash bugs
        if c == '%' as c_int {
            c = '.' as c_int;
        }
        common.msg_read_big_string_buf[l] = c as u8;
        l += 1;
        if l >= cap - 1 {
            break;
        }
    }
    common.msg_read_big_string_buf[l] = 0;
    common.msg_read_big_string_buf.as_mut_ptr() as *mut c_char
}

/// Raven `MSG_ReadStringLine`.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:521-542`
pub fn MSG_ReadStringLine(common: &mut Common, msg: *mut msg_t) -> *mut c_char {
    let cap = mp_qshared::shared::limits::MAX_STRING_CHARS;
    let mut l: usize = 0;
    loop {
        let c = MSG_ReadByte(common, msg);
        if c == -1 || c == 0 || c == '\n' as c_int {
            break;
        }
        let mut c = c;
        // translate all fmt spec to avoid crash bugs
        if c == '%' as c_int {
            c = '.' as c_int;
        }
        common.msg_read_string_line_buf[l] = c as u8;
        l += 1;
        if l >= cap - 1 {
            break;
        }
    }
    common.msg_read_string_line_buf[l] = 0;
    common.msg_read_string_line_buf.as_mut_ptr() as *mut c_char
}

/// Raven `MSG_ReadAngle16`. `SHORT2ANGLE(x)` = `(x)*(360.0/65536)`.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:544-546`
pub fn MSG_ReadAngle16(common: &mut Common, msg: *mut msg_t) -> f32 {
    (MSG_ReadShort(common, msg) as f32) * (360.0 / 65536.0)
}

/// Raven `MSG_ReadData`.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:548-554`
pub fn MSG_ReadData(common: &mut Common, msg: *mut msg_t, data: *mut (), len: c_int) {
    unsafe {
        let out = data as *mut u8;
        for i in 0..len {
            *out.offset(i as isize) = MSG_ReadByte(common, msg) as u8;
        }
    }
}

/// Raven `MSG_WriteDeltaUsercmd`.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:685-706`
pub fn MSG_WriteDeltaUsercmd(
    common: &mut Common,
    msg: *mut msg_t,
    from: *mut usercmd_t,
    to: *mut usercmd_t,
) {
    unsafe {
        if (*to).serverTime - (*from).serverTime < 256 {
            MSG_WriteBits(common, msg, 1, 1);
            MSG_WriteBits(common, msg, (*to).serverTime - (*from).serverTime, 8);
        } else {
            MSG_WriteBits(common, msg, 0, 1);
            MSG_WriteBits(common, msg, (*to).serverTime, 32);
        }
        MSG_WriteDelta(common, msg, (*from).angles[0], (*to).angles[0], 16);
        MSG_WriteDelta(common, msg, (*from).angles[1], (*to).angles[1], 16);
        MSG_WriteDelta(common, msg, (*from).angles[2], (*to).angles[2], 16);
        MSG_WriteDelta(
            common,
            msg,
            (*from).forwardmove as c_int,
            (*to).forwardmove as c_int,
            8,
        );
        MSG_WriteDelta(
            common,
            msg,
            (*from).rightmove as c_int,
            (*to).rightmove as c_int,
            8,
        );
        MSG_WriteDelta(
            common,
            msg,
            (*from).upmove as c_int,
            (*to).upmove as c_int,
            8,
        );
        MSG_WriteDelta(common, msg, (*from).buttons, (*to).buttons, 16);
        MSG_WriteDelta(
            common,
            msg,
            (*from).weapon as c_int,
            (*to).weapon as c_int,
            8,
        );

        MSG_WriteDelta(
            common,
            msg,
            (*from).forcesel as c_int,
            (*to).forcesel as c_int,
            8,
        );
        MSG_WriteDelta(
            common,
            msg,
            (*from).invensel as c_int,
            (*to).invensel as c_int,
            8,
        );

        MSG_WriteDelta(
            common,
            msg,
            (*from).generic_cmd as c_int,
            (*to).generic_cmd as c_int,
            8,
        );
    }
}

/// Raven `MSG_ReadDeltaUsercmd`.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:714-733`
pub fn MSG_ReadDeltaUsercmd(
    common: &mut Common,
    msg: *mut msg_t,
    from: *mut usercmd_t,
    to: *mut usercmd_t,
) {
    unsafe {
        if MSG_ReadBits(common, msg, 1) != 0 {
            (*to).serverTime = (*from).serverTime + MSG_ReadBits(common, msg, 8);
        } else {
            (*to).serverTime = MSG_ReadBits(common, msg, 32);
        }
        (*to).angles[0] = MSG_ReadDelta(common, msg, (*from).angles[0], 16);
        (*to).angles[1] = MSG_ReadDelta(common, msg, (*from).angles[1], 16);
        (*to).angles[2] = MSG_ReadDelta(common, msg, (*from).angles[2], 16);
        (*to).forwardmove = MSG_ReadDelta(common, msg, (*from).forwardmove as c_int, 8) as _;
        (*to).rightmove = MSG_ReadDelta(common, msg, (*from).rightmove as c_int, 8) as _;
        (*to).upmove = MSG_ReadDelta(common, msg, (*from).upmove as c_int, 8) as _;
        (*to).buttons = MSG_ReadDelta(common, msg, (*from).buttons, 16);
        (*to).weapon = MSG_ReadDelta(common, msg, (*from).weapon as c_int, 8) as _;

        (*to).forcesel = MSG_ReadDelta(common, msg, (*from).forcesel as c_int, 8) as _;
        (*to).invensel = MSG_ReadDelta(common, msg, (*from).invensel as c_int, 8) as _;

        (*to).generic_cmd = MSG_ReadDelta(common, msg, (*from).generic_cmd as c_int, 8) as _;
    }
}

/// Raven `MSG_WriteDeltaUsercmdKey`.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:740-778`
pub fn MSG_WriteDeltaUsercmdKey(
    common: &mut Common,
    msg: *mut msg_t,
    key: c_int,
    from: *mut usercmd_t,
    to: *mut usercmd_t,
) {
    unsafe {
        if (*to).serverTime - (*from).serverTime < 256 {
            MSG_WriteBits(common, msg, 1, 1);
            MSG_WriteBits(common, msg, (*to).serverTime - (*from).serverTime, 8);
        } else {
            MSG_WriteBits(common, msg, 0, 1);
            MSG_WriteBits(common, msg, (*to).serverTime, 32);
        }
        if (*from).angles[0] == (*to).angles[0]
            && (*from).angles[1] == (*to).angles[1]
            && (*from).angles[2] == (*to).angles[2]
            && (*from).forwardmove == (*to).forwardmove
            && (*from).rightmove == (*to).rightmove
            && (*from).upmove == (*to).upmove
            && (*from).buttons == (*to).buttons
            && (*from).weapon == (*to).weapon
            && (*from).forcesel == (*to).forcesel
            && (*from).invensel == (*to).invensel
            && (*from).generic_cmd == (*to).generic_cmd
        {
            MSG_WriteBits(common, msg, 0, 1); // no change
            common.oldsize += 7;
            return;
        }
        let key = key ^ (*to).serverTime;
        MSG_WriteBits(common, msg, 1, 1);
        MSG_WriteDeltaKey(common, msg, key, (*from).angles[0], (*to).angles[0], 16);
        MSG_WriteDeltaKey(common, msg, key, (*from).angles[1], (*to).angles[1], 16);
        MSG_WriteDeltaKey(common, msg, key, (*from).angles[2], (*to).angles[2], 16);
        MSG_WriteDeltaKey(
            common,
            msg,
            key,
            (*from).forwardmove as c_int,
            (*to).forwardmove as c_int,
            8,
        );
        MSG_WriteDeltaKey(
            common,
            msg,
            key,
            (*from).rightmove as c_int,
            (*to).rightmove as c_int,
            8,
        );
        MSG_WriteDeltaKey(
            common,
            msg,
            key,
            (*from).upmove as c_int,
            (*to).upmove as c_int,
            8,
        );
        MSG_WriteDeltaKey(common, msg, key, (*from).buttons, (*to).buttons, 16);
        MSG_WriteDeltaKey(
            common,
            msg,
            key,
            (*from).weapon as c_int,
            (*to).weapon as c_int,
            8,
        );

        MSG_WriteDeltaKey(
            common,
            msg,
            key,
            (*from).forcesel as c_int,
            (*to).forcesel as c_int,
            8,
        );
        MSG_WriteDeltaKey(
            common,
            msg,
            key,
            (*from).invensel as c_int,
            (*to).invensel as c_int,
            8,
        );

        MSG_WriteDeltaKey(
            common,
            msg,
            key,
            (*from).generic_cmd as c_int,
            (*to).generic_cmd as c_int,
            8,
        );
    }
}

/// Raven `MSG_ReadDeltaUsercmdKey`.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:786-822`
pub fn MSG_ReadDeltaUsercmdKey(
    common: &mut Common,
    msg: *mut msg_t,
    key: c_int,
    from: *mut usercmd_t,
    to: *mut usercmd_t,
) {
    unsafe {
        if MSG_ReadBits(common, msg, 1) != 0 {
            (*to).serverTime = (*from).serverTime + MSG_ReadBits(common, msg, 8);
        } else {
            (*to).serverTime = MSG_ReadBits(common, msg, 32);
        }
        if MSG_ReadBits(common, msg, 1) != 0 {
            let key = key ^ (*to).serverTime;
            (*to).angles[0] = MSG_ReadDeltaKey(common, msg, key, (*from).angles[0], 16);
            (*to).angles[1] = MSG_ReadDeltaKey(common, msg, key, (*from).angles[1], 16);
            (*to).angles[2] = MSG_ReadDeltaKey(common, msg, key, (*from).angles[2], 16);
            (*to).forwardmove =
                MSG_ReadDeltaKey(common, msg, key, (*from).forwardmove as c_int, 8) as _;
            (*to).rightmove =
                MSG_ReadDeltaKey(common, msg, key, (*from).rightmove as c_int, 8) as _;
            (*to).upmove = MSG_ReadDeltaKey(common, msg, key, (*from).upmove as c_int, 8) as _;
            (*to).buttons = MSG_ReadDeltaKey(common, msg, key, (*from).buttons, 16);
            (*to).weapon = MSG_ReadDeltaKey(common, msg, key, (*from).weapon as c_int, 8) as _;

            (*to).forcesel = MSG_ReadDeltaKey(common, msg, key, (*from).forcesel as c_int, 8) as _;
            (*to).invensel = MSG_ReadDeltaKey(common, msg, key, (*from).invensel as c_int, 8) as _;

            (*to).generic_cmd =
                MSG_ReadDeltaKey(common, msg, key, (*from).generic_cmd as c_int, 8) as _;
        } else {
            (*to).angles[0] = (*from).angles[0];
            (*to).angles[1] = (*from).angles[1];
            (*to).angles[2] = (*from).angles[2];
            (*to).forwardmove = (*from).forwardmove;
            (*to).rightmove = (*from).rightmove;
            (*to).upmove = (*from).upmove;
            (*to).buttons = (*from).buttons;
            (*to).weapon = (*from).weapon;

            (*to).forcesel = (*from).forcesel;
            (*to).invensel = (*from).invensel;

            (*to).generic_cmd = (*from).generic_cmd;
        }
    }
}

/// Raven `MSG_ReadDeltaEntity`.
///
/// STATE: `cl_shownet` (cvar read), `entityStateFields`, `sv` — see
/// missing_symbols/shape_mismatches for the unresolved cvar/table/cross-crate
/// items (netField_t has no rosetta row; `Server` lives in a crate this one
/// does not depend on).
///
/// Source: `oracle/codemp/qcommon/msg.cpp:1228-1383`
pub fn MSG_ReadDeltaEntity(
    common: &mut Common,
    sv: &mut Server,
    msg: *mut msg_t,
    from: *mut entityState_t,
    to: *mut entityState_t,
    number: c_int,
) {
    unsafe {
        if !(0..mp_qshared::shared::limits::MAX_GENTITIES as c_int).contains(&number) {
            //TODO: Port Com_Error
            // Source: oracle/codemp/qcommon/msg.cpp:1239 (ruling 1: receiverless panic)
            Com_Error(
                errorParm_t::ERR_DROP,
                &format!("Bad delta entity number: {number}"),
            );
        }

        let startBit = (*msg).bit;

        // check for a remove
        if MSG_ReadBits(common, msg, 1) == 1 {
            crate::common_fns::Com_Memset(to as *mut (), 0, core::mem::size_of::<entityState_t>());
            (*to).number = mp_qshared::shared::limits::MAX_GENTITIES as c_int - 1;
            //TODO: Port cl_shownet
            // Source: oracle/codemp/qcommon/msg.cpp:12
            if common.cl_shownet >= 2 || common.cl_shownet == -1 {
                crate::common::com_printf(
                    common,
                    &format!("{:3}: #{:<3} remove\n", (*msg).readcount, number),
                );
            }
            return;
        }

        // check for no delta
        if MSG_ReadBits(common, msg, 1) == 0 {
            *to = *from;
            (*to).number = number;
            return;
        }

        //TODO: Port entityStateFields
        // Source: oracle/codemp/qcommon/msg.cpp:859-1051
        let num_fields = common.entity_state_fields.len();
        let lc = MSG_ReadByte(common, msg);

        // shownet 2/3 will interleave with other printed info, -1 will
        // just print the delta records`
        let print = if common.cl_shownet >= 2 || common.cl_shownet == -1 {
            if sv.sv.state != 0 {
                //TODO: Port SV_GentityNum
                // Source: oracle/codemp/server/sv_game.cpp:58
                let classname_ptr = (*SV_GentityNum(sv, number)).classname;
                let classname = std::ffi::CStr::from_ptr(classname_ptr).to_string_lossy();
                crate::common::com_printf(
                    common,
                    &format!("{:3}: #{:<3} ({}) ", (*msg).readcount, number, classname),
                );
            } else {
                crate::common::com_printf(
                    common,
                    &format!("{:3}: #{:<3} ", (*msg).readcount, number),
                );
            }
            true
        } else {
            false
        };

        (*to).number = number;

        for i in 0..lc {
            let field = &mut common.entity_state_fields[i as usize];
            let fromF = (from as *const u8).add(field.offset as usize) as *const c_int;
            let toF = (to as *mut u8).add(field.offset as usize) as *mut c_int;

            if MSG_ReadBits(common, msg, 1) == 0 {
                // no change
                *toF = *fromF;
            } else if field.bits == 0 {
                // float
                if MSG_ReadBits(common, msg, 1) == 0 {
                    *(toF as *mut f32) = 0.0f32;
                } else if MSG_ReadBits(common, msg, 1) == 0 {
                    // integral float
                    let mut trunc =
                        MSG_ReadBits(common, msg, crate::qcommon::msg_consts::FLOAT_INT_BITS);
                    // bias to allow equal parts positive and negative
                    trunc -= crate::qcommon::msg_consts::FLOAT_INT_BIAS;
                    *(toF as *mut f32) = trunc as f32;
                    if print {
                        crate::common::com_printf(common, &format!("{}:{} ", field.name, trunc));
                    }
                } else {
                    // full floating point value
                    *toF = MSG_ReadBits(common, msg, 32);
                    if print {
                        crate::common::com_printf(
                            common,
                            &format!("{}:{} ", field.name, *(toF as *mut f32)),
                        );
                    }
                }
            } else if MSG_ReadBits(common, msg, 1) == 0 {
                *toF = 0;
            } else {
                // integer
                *toF = MSG_ReadBits(common, msg, field.bits);
                if print {
                    crate::common::com_printf(common, &format!("{}:{} ", field.name, *toF));
                }
            }
        }
        for i in lc..(num_fields as c_int) {
            let field = &common.entity_state_fields[i as usize];
            let fromF = (from as *const u8).add(field.offset as usize) as *const c_int;
            let toF = (to as *mut u8).add(field.offset as usize) as *mut c_int;
            // no change
            *toF = *fromF;
        }

        if print {
            let endBit = (*msg).bit;
            crate::common::com_printf(common, &format!(" ({} bits)\n", endBit - startBit));
        }
    }
}

/// Raven `MSG_ReadDeltaPlayerstate`.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:2455-2692`
pub fn MSG_ReadDeltaPlayerstate(
    common: &mut Common,
    msg: *mut msg_t,
    from: *mut playerState_t,
    to: *mut playerState_t,
    isVehiclePS: qboolean,
) {
    let _ = isVehiclePS; // _OPTIMIZED_VEHICLE_NETWORKING gate below is not defined for this build
    unsafe {
        let mut dummy: playerState_t;
        let from = if from.is_null() {
            dummy = core::mem::zeroed();
            crate::common_fns::Com_Memset(
                &mut dummy as *mut playerState_t as *mut (),
                0,
                core::mem::size_of::<playerState_t>(),
            );
            &mut dummy as *mut playerState_t
        } else {
            from
        };
        *to = *from;

        let startBit = if (*msg).bit == 0 {
            (*msg).readcount * 8 - crate::qcommon::msg_consts::GENTITYNUM_BITS
        } else {
            ((*msg).readcount - 1) * 8 + (*msg).bit - crate::qcommon::msg_consts::GENTITYNUM_BITS
        };

        // shownet 2/3 will interleave with other printed info, -2 will
        // just print the delta records
        let print = if common.cl_shownet >= 2 || common.cl_shownet == -2 {
            crate::common::com_printf(common, &format!("{:3}: playerstate ", (*msg).readcount));
            true
        } else {
            false
        };

        //=====_OPTIMIZED_VEHICLE_NETWORKING not defined for this build=====
        //TODO: Port playerStateFields
        // Source: oracle/codemp/qcommon/msg.cpp:1410-1568
        let num_fields = common.player_state_fields.len();

        let lc = MSG_ReadByte(common, msg);

        for i in 0..lc {
            let field = &mut common.player_state_fields[i as usize];
            let fromF = (from as *const u8).add(field.offset as usize) as *const c_int;
            let toF = (to as *mut u8).add(field.offset as usize) as *mut c_int;

            if MSG_ReadBits(common, msg, 1) == 0 {
                // no change
                *toF = *fromF;
            } else if field.bits == 0 {
                // float
                if MSG_ReadBits(common, msg, 1) == 0 {
                    // integral float
                    let mut trunc =
                        MSG_ReadBits(common, msg, crate::qcommon::msg_consts::FLOAT_INT_BITS);
                    // bias to allow equal parts positive and negative
                    trunc -= crate::qcommon::msg_consts::FLOAT_INT_BIAS;
                    *(toF as *mut f32) = trunc as f32;
                    if print {
                        crate::common::com_printf(common, &format!("{}:{} ", field.name, trunc));
                    }
                } else {
                    // full floating point value
                    *toF = MSG_ReadBits(common, msg, 32);
                    if print {
                        crate::common::com_printf(
                            common,
                            &format!("{}:{} ", field.name, *(toF as *mut f32)),
                        );
                    }
                }
            } else {
                // integer
                *toF = MSG_ReadBits(common, msg, field.bits);
                if print {
                    crate::common::com_printf(common, &format!("{}:{} ", field.name, *toF));
                }
            }
        }
        for i in lc..(num_fields as c_int) {
            let field = &common.player_state_fields[i as usize];
            let fromF = (from as *const u8).add(field.offset as usize) as *const c_int;
            let toF = (to as *mut u8).add(field.offset as usize) as *mut c_int;
            // no change
            *toF = *fromF;
        }

        // read the arrays
        if MSG_ReadBits(common, msg, 1) != 0 {
            // parse stats
            if MSG_ReadBits(common, msg, 1) != 0 {
                //TODO: Port LOG
                // Source: oracle/codemp/qcommon/msg.cpp:2588 ("PS_STATS")
                let bits = MSG_ReadShort(common, msg);
                for i in 0..16 {
                    if bits & (1 << i) != 0 {
                        if i == crate::qcommon::msg_consts::STAT_WEAPONS {
                            (*to).stats[i as usize] = MSG_ReadBits(
                                common,
                                msg,
                                mp_qshared::common::mp::qcommon::player_state::MAX_WEAPONS as c_int,
                            ) as i16;
                        } else {
                            (*to).stats[i as usize] = MSG_ReadShort(common, msg) as i16;
                        }
                    }
                }
            }

            // parse persistant stats
            if MSG_ReadBits(common, msg, 1) != 0 {
                //TODO: Port LOG
                // Source: oracle/codemp/qcommon/msg.cpp:2614 ("PS_PERSISTANT")
                let bits = MSG_ReadShort(common, msg);
                for i in 0..16 {
                    if bits & (1 << i) != 0 {
                        (*to).persistant[i as usize] = MSG_ReadShort(common, msg) as i16;
                    }
                }
            }

            // parse ammo
            if MSG_ReadBits(common, msg, 1) != 0 {
                //TODO: Port LOG
                // Source: oracle/codemp/qcommon/msg.cpp:2632 ("PS_AMMO")
                let bits = MSG_ReadShort(common, msg);
                for i in 0..16 {
                    if bits & (1 << i) != 0 {
                        (*to).ammo[i as usize] = MSG_ReadShort(common, msg) as i16;
                    }
                }
            }

            // parse powerups
            if MSG_ReadBits(common, msg, 1) != 0 {
                //TODO: Port LOG
                // Source: oracle/codemp/qcommon/msg.cpp:2650 ("PS_POWERUPS")
                let bits = MSG_ReadShort(common, msg);
                for i in 0..16 {
                    if bits & (1 << i) != 0 {
                        (*to).powerups[i as usize] = MSG_ReadLong(common, msg);
                    }
                }
            }
        }

        if print {
            let endBit = if (*msg).bit == 0 {
                (*msg).readcount * 8 - crate::qcommon::msg_consts::GENTITYNUM_BITS
            } else {
                ((*msg).readcount - 1) * 8 + (*msg).bit
                    - crate::qcommon::msg_consts::GENTITYNUM_BITS
            };
            crate::common::com_printf(common, &format!(" ({} bits)\n", endBit - startBit));
        }

        // _ONEBIT_COMBO not defined for this build; the mask-replay tail is dead.
    }
}
