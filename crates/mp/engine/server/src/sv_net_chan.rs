//! `sv_net_chan.cpp` — server-side netchan encode/decode/transmit/process.
//!
//! Source: `oracle/codemp/server/sv_net_chan.cpp`

use core::ffi::c_int;

use mp_engine_qcommon::net_chan::{Netchan_Transmit, Netchan_TransmitNextFragment};
use mp_engine_qcommon::collision_world::CollisionWorld;
use mp_engine_qcommon::cm_load::RenderModels;
use mp_engine_qcommon::common::Common;
use mp_engine_qcommon::msg::{MSG_ReadLong, MSG_WriteByte};
use mp_engine_qcommon::net_chan::Netchan_Process;
use mp_engine_qcommon::qcommon::huffman_consts::{SV_DECODE_START, SV_ENCODE_START};
use mp_engine_qcommon::qcommon::netchan_t::netchan_t;
use mp_engine_qcommon::qcommon::net_limits::MAX_RELIABLE_COMMANDS;
use mp_engine_qcommon::qcommon::svc_ops_e::svc_ops_e;
use mp_host_interface::engine_host::EngineHost;
use mp_qshared::common::mp::qcommon::msg_t::msg_t;
use mp_qshared::shared::{qboolean, qfalse, qtrue};

use crate::server::client_s::client_t;

/// Raven `SV_Netchan_Encode` — first four bytes of `data` are always the
/// `long reliableAcknowledge`. XOR byte-scramble; wire-format-critical, so the
/// key-derivation loop is transcribed exactly (`'.' << (i&1)` on `%`, else
/// `string[index] << (i&1)`, `index` cycling on the NUL terminator).
///
/// Source: `oracle/codemp/server/sv_net_chan.cpp:17-63`
fn SV_Netchan_Encode(common: &mut Common, client: *mut client_t, msg: *mut msg_t) {
    unsafe {
        if (*msg).cursize < SV_ENCODE_START {
            return;
        }

        let srdc = (*msg).readcount;
        let sbit = (*msg).bit;
        let soob = (*msg).oob;

        (*msg).bit = 0;
        (*msg).readcount = 0;
        (*msg).oob = 0;

        // Raven reads `reliableAcknowledge` here but never uses it (readcount is
        // restored below, so the read has no effect); kept for fidelity.
        let _reliable_acknowledge: c_int = MSG_ReadLong(common, msg);

        (*msg).oob = soob;
        (*msg).bit = sbit;
        (*msg).readcount = srdc;

        let string: *mut u8 = (*client).lastClientCommandString.as_mut_ptr() as *mut u8;
        let mut index: i32 = 0;

        // xor the client challenge with the netchan sequence number
        let mut key: u8 = ((*client).challenge ^ (*client).netchan.outgoingSequence) as u8;

        let mut i = SV_ENCODE_START;
        while i < (*msg).cursize {
            // modify the key with the last received and with this message
            // acknowledged client command
            if *string.add(index as usize) == 0 {
                index = 0;
            }
            if *string.add(index as usize) == b'%' {
                key ^= (b'.' as i32).wrapping_shl((i & 1) as u32) as u8;
            } else {
                key ^= (*string.add(index as usize) as i32).wrapping_shl((i & 1) as u32) as u8;
            }
            index += 1;
            // encode the data with this key
            *(*msg).data.offset(i as isize) = *(*msg).data.offset(i as isize) ^ key;
            i += 1;
        }
    }
}

/// Raven `SV_Netchan_Decode` — first 12 bytes of `data` are always
/// `serverId`/`messageAcknowledge`/`reliableAcknowledge`. XOR byte-scramble;
/// wire-format-critical, transcribed exactly.
///
/// Source: `oracle/codemp/server/sv_net_chan.cpp:76-118`
fn SV_Netchan_Decode(common: &mut Common, client: *mut client_t, msg: *mut msg_t) {
    unsafe {
        let srdc = (*msg).readcount;
        let sbit = (*msg).bit;
        let soob = (*msg).oob;

        (*msg).oob = 0;

        let server_id: c_int = MSG_ReadLong(common, msg);
        let message_acknowledge: c_int = MSG_ReadLong(common, msg);
        let reliable_acknowledge: c_int = MSG_ReadLong(common, msg);

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

/// Raven `SV_Netchan_TransmitNextFragment`.
///
/// Source: `oracle/codemp/server/sv_net_chan.cpp:126-128`
pub fn SV_Netchan_TransmitNextFragment(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    chan: *mut netchan_t,
) {
    // Callee's canonical home is qcommon `net_chan`; not yet ported there
    // (honest missing symbol).
    Netchan_TransmitNextFragment(common, cm, rm, host, chan);
}

/// Raven `SV_Netchan_Transmit`. The trailing `svc_EOF`, `SV_Netchan_Encode`
/// scramble, and `Netchan_Transmit` order is preserved; the oracle's
/// commented-out `Huff_Compress`/checksum scaffolding is dead code, not ported.
///
/// Source: `oracle/codemp/server/sv_net_chan.cpp:138-147`
pub fn SV_Netchan_Transmit(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    client: *mut client_t,
    msg: *mut msg_t,
) {
    unsafe {
        MSG_WriteByte(common, msg, svc_ops_e::svc_EOF as c_int);
        SV_Netchan_Encode(common, client, msg);
        // Callee's canonical home is qcommon `net_chan`; not yet ported there
        // (honest missing symbol).
        Netchan_Transmit(
            common,
            cm,
            rm,
            host,
            &mut (*client).netchan,
            (*msg).cursize,
            (*msg).data,
        );
    }
}

/// Raven `SV_Netchan_Process`.
///
/// Source: `oracle/codemp/server/sv_net_chan.cpp:154-168`
pub fn SV_Netchan_Process(common: &mut Common, client: *mut client_t, msg: *mut msg_t) -> qboolean {
    unsafe {
        let ret = Netchan_Process(common, &mut (*client).netchan, msg);
        if ret == qfalse {
            return qfalse;
        }
        SV_Netchan_Decode(common, client, msg);
        // Huff_Decompress / checksum verification is commented out in the oracle
        // (dead code, never executed) — not transcribed.
        qtrue
    }
}
