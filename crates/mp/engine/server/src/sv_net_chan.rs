//! `sv_net_chan.cpp` — server-side netchan decode/process.
//!
//! Source: `oracle/codemp/server/sv_net_chan.cpp`

use core::ffi::c_int;

use mp_engine_qcommon::common::Common;
use mp_engine_qcommon::qcommon::huffman_consts::SV_DECODE_START;
use mp_engine_qcommon::qcommon::net_limits::MAX_RELIABLE_COMMANDS;
use mp_qshared::shared::{qboolean, qfalse, qtrue};

use crate::server::client_s::client_t;
use mp_qshared::common::mp::qcommon::msg_t::msg_t;

/// Raven `SV_Netchan_Decode`.
///
/// Source: `oracle/codemp/server/sv_net_chan.cpp:76-118`
pub fn SV_Netchan_Decode(common: &mut Common, client: *mut client_t, msg: *mut msg_t) {
    unsafe {
        let srdc = (*msg).readcount;
        let sbit = (*msg).bit;
        let soob = (*msg).oob;

        (*msg).oob = 0;

        let server_id: c_int = mp_engine_qcommon::msg::MSG_ReadLong(common, msg);
        let message_acknowledge: c_int = mp_engine_qcommon::msg::MSG_ReadLong(common, msg);
        let reliable_acknowledge: c_int = mp_engine_qcommon::msg::MSG_ReadLong(common, msg);

        (*msg).oob = soob;
        (*msg).bit = sbit;
        (*msg).readcount = srdc;

        let string: *mut u8 = (*client).reliableCommands
            [(reliable_acknowledge as usize) & (MAX_RELIABLE_COMMANDS - 1)]
            .as_mut_ptr() as *mut u8;
        let mut index: i32 = 0;

        let mut key: u8 = ((*client).challenge ^ server_id ^ message_acknowledge) as u8;

        let mut i = (*msg).readcount + SV_DECODE_START;
        while i < (*msg).cursize {
            // modify the key with the last sent and acknowledged server command
            if *string.add(index as usize) == 0 {
                index = 0;
            }
            if *string.add(index as usize) == b'%' {
                key ^= (b'.' as i32).wrapping_shl((i & 1) as u32) as u8;
            } else {
                key ^= (*string.add(index as usize) as i32).wrapping_shl((i & 1) as u32) as u8;
            }
            index += 1;
            // decode the data with this key
            *(*msg).data.offset(i as isize) = *(*msg).data.offset(i as isize) ^ key;
            i += 1;
        }
    }
}

/// Raven `SV_Netchan_Process`.
///
/// Source: `oracle/codemp/server/sv_net_chan.cpp:154-168`
pub fn SV_Netchan_Process(common: &mut Common, client: *mut client_t, msg: *mut msg_t) -> qboolean {
    unsafe {
        let ret = mp_engine_qcommon::net_chan::Netchan_Process(common, &mut (*client).netchan, msg);
        if ret == qfalse {
            return qfalse;
        }
        SV_Netchan_Decode(common, client, msg);
        // Huff_Decompress / checksum verification is commented out in the oracle
        // (dead code, never executed) — not transcribed.
        qtrue
    }
}
