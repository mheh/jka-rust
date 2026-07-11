#![allow(
    non_snake_case,
    non_camel_case_types,
    unused_variables,
    unused_mut,
    unused_unsafe,
    unused_assignments,
    unused_parens,
    clippy::too_many_arguments
)]

//! MP botlib `l_crc.cpp` — the CRC-16 (CCITT) checksum routines used by the
//! precompiler/AAS file loaders.
//!
//! Source: `oracle/codemp/botlib/l_crc.cpp`

use core::ffi::{c_char, c_int, c_uchar, c_ushort};

use crate::l_crc::crc_consts::{CRC_INIT_VALUE, CRC_XOR_VALUE};
use crate::BotLib;

//TODO: Port byte
// Source: oracle/codemp/game/q_shared.h:349 (mp_game prelude — mp_engine_botlib
// has no dependency on mp_game yet; see missing_symbols).
use mp_game::prelude::byte;

/// Raven `CRC_Init` — sets the CRC accumulator to its initial value.
///
/// Source: `oracle/codemp/botlib/l_crc.cpp:75-78`
pub fn CRC_Init(crcvalue: *mut c_ushort) {
    unsafe {
        *crcvalue = CRC_INIT_VALUE;
    }
}

/// Raven `CRC_ProcessByte` — folds one byte into the running CRC.
///
/// Source: `oracle/codemp/botlib/l_crc.cpp:85-88`
pub fn CRC_ProcessByte(bot: &mut BotLib, crcvalue: *mut c_ushort, data: byte) {
    unsafe {
        *crcvalue = (*crcvalue << 8) ^ bot.crctable[((*crcvalue >> 8) ^ data as c_ushort) as usize];
    }
}

/// Raven `CRC_Value` — finalizes the CRC accumulator with the XOR mask.
///
/// Source: `oracle/codemp/botlib/l_crc.cpp:95-98`
pub fn CRC_Value(crcvalue: c_ushort) -> c_ushort {
    crcvalue ^ CRC_XOR_VALUE
}

/// Raven `CRC_ContinueProcessString` — folds `length` bytes of `data` into an
/// already-initialized CRC accumulator.
///
/// Source: `oracle/codemp/botlib/l_crc.cpp:126-134`
pub fn CRC_ContinueProcessString(
    bot: &mut BotLib,
    crc: *mut c_ushort,
    data: *mut c_char,
    length: c_int,
) {
    unsafe {
        for i in 0..length {
            let byte_val = *data.offset(i as isize) as c_uchar;
            *crc = (*crc << 8) ^ bot.crctable[(((*crc >> 8) as c_uchar) ^ byte_val) as usize];
        }
    }
}

/// Raven `CRC_ProcessString` — initializes, processes `length` bytes of
/// `data`, and finalizes the CRC in one call.
///
/// Source: `oracle/codemp/botlib/l_crc.cpp:105-119`
pub fn CRC_ProcessString(bot: &mut BotLib, data: *mut c_uchar, length: c_int) -> c_ushort {
    let mut crcvalue: c_ushort = 0;
    CRC_Init(&mut crcvalue);

    unsafe {
        for i in 0..length {
            let mut ind: c_int = (crcvalue >> 8) as c_int ^ (*data.offset(i as isize) as c_int);
            if ind < 0 || ind > 256 {
                ind = 0;
            }
            crcvalue = (crcvalue << 8) ^ bot.crctable[ind as usize];
        }
    }

    CRC_Value(crcvalue)
}
