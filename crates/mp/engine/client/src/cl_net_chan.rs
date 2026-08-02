//! Client netchan encode/decode/transmit wrappers.
//!
//! Source: `oracle/codemp/client/cl_net_chan.cpp`.

use core::ffi::c_int;

use mp_engine_qcommon::msg::MSG_ReadLong;
use mp_engine_qcommon::msg::MSG_WriteByte;
use mp_engine_qcommon::net_chan::Netchan_Process;
use mp_engine_qcommon::net_chan::Netchan_Transmit;
use mp_engine_qcommon::net_chan::Netchan_TransmitNextFragment;
use mp_engine_qcommon::qcommon::huffman_consts::CL_DECODE_START;
use mp_engine_qcommon::qcommon::huffman_consts::CL_ENCODE_START;
use mp_engine_qcommon::qcommon::net_limits::MAX_RELIABLE_COMMANDS;
use mp_engine_qcommon::qcommon::clc_ops_e::clc_ops_e::clc_EOF;
use mp_engine_qcommon::qcommon::netchan_t::netchan_t;
use mp_game::prelude::byte;
use mp_qshared::common::mp::qcommon::msg_t::msg_t;
use mp_qshared::shared::swap::LittleLong;
use native_types::qboolean;

use crate::client_host::Client;

/// Msg id for the client end-of-frame marker.
///
/// PORT-NOTE(clc_EOF): the `clc_EOF` client-to-server message id is not in
/// this packet's rosetta. Referenced exactly as the oracle names it below.
/// Source: `oracle/codemp/client/cl_net_chan.cpp:139`
// missing_symbols: clc_EOF

/// Raven `CL_Netchan_Encode`.
///
/// This xors the outgoing message body with a key derived from the last
/// acknowledged server command, restoring the read cursor afterward.
///
/// Source: `oracle/codemp/client/cl_net_chan.cpp:19-67`
pub fn CL_Netchan_Encode(cl: &mut Client, msg: *mut msg_t) {
    unsafe {
        if (*msg).cursize <= CL_ENCODE_START {
            return;
        }

        let srdc = (*msg).readcount;
        let sbit = (*msg).bit;
        let soob = (*msg).oob;

        (*msg).bit = 0;
        (*msg).readcount = 0;
        (*msg).oob = qboolean::qfalse;

        // PORT-NOTE(msg-receiver): `MSG_ReadLong` takes `common: &mut Common`,
        // a receiver this fn's LAW signature does not carry.
        let serverId: c_int = MSG_ReadLong(common, msg);
        let messageAcknowledge: c_int = MSG_ReadLong(common, msg);
        let reliableAcknowledge: c_int = MSG_ReadLong(common, msg);

        (*msg).oob = soob;
        (*msg).bit = sbit;
        (*msg).readcount = srdc;

        let cmd_index = (reliableAcknowledge as usize) & (MAX_RELIABLE_COMMANDS - 1);
        let string: *const byte = cl.clc.serverCommands[cmd_index].as_ptr() as *const byte;
        let mut index: usize = 0;

        let mut key: byte = (cl.clc.challenge as byte) ^ (serverId as byte) ^ (messageAcknowledge as byte);
        let mut i = CL_ENCODE_START;
        while i < (*msg).cursize {
            // Modify the key with the last received now acknowledged server command.
            if *string.add(index) == 0 {
                index = 0;
            }
            if *string.add(index) == b'%' {
                key ^= (b'.' as byte) << (i & 1);
            } else {
                key ^= (*string.add(index)) << (i & 1);
            }
            index += 1;
            // Encode the data with this key.
            let p = (*msg).data.add(i as usize);
            *p = (*p) ^ key;
            i += 1;
        }
    }
}

/// Raven `CL_Netchan_Decode`.
///
/// This xors the incoming message body back using the client challenge and
/// the netchan sequence number, restoring the read cursor afterward.
///
/// Source: `oracle/codemp/client/cl_net_chan.cpp:78-118`
pub fn CL_Netchan_Decode(cl: &mut Client, msg: *mut msg_t) {
    unsafe {
        let srdc = (*msg).readcount;
        let sbit = (*msg).bit;
        let soob = (*msg).oob;

        (*msg).oob = qboolean::qfalse;

        // PORT-NOTE(msg-receiver): `MSG_ReadLong` takes `common: &mut Common`,
        // a receiver this fn's LAW signature does not carry.
        let reliableAcknowledge: c_int = MSG_ReadLong(common, msg);

        (*msg).oob = soob;
        (*msg).bit = sbit;
        (*msg).readcount = srdc;

        let cmd_index = (reliableAcknowledge as usize) & (MAX_RELIABLE_COMMANDS - 1);
        let string: *const byte = cl.clc.reliableCommands[cmd_index].as_ptr() as *const byte;
        let mut index: usize = 0;
        // Xor the client challenge with the netchan sequence number (need
        // something that changes every message).
        let seq = LittleLong(*((*msg).data as *const c_int));
        let mut key: byte = (cl.clc.challenge as byte) ^ (seq as byte);

        let mut i = (*msg).readcount + CL_DECODE_START;
        while i < (*msg).cursize {
            // Modify the key with the last sent and with this message
            // acknowledged client command.
            if *string.add(index) == 0 {
                index = 0;
            }
            if *string.add(index) == b'%' {
                key ^= (b'.' as byte) << (i & 1);
            } else {
                key ^= (*string.add(index)) << (i & 1);
            }
            index += 1;
            // Decode the data with this key.
            let p = (*msg).data.add(i as usize);
            *p = (*p) ^ key;
            i += 1;
        }
    }
}

/// Raven `CL_Netchan_TransmitNextFragment`.
///
/// Source: `oracle/codemp/client/cl_net_chan.cpp:126-128`
pub fn CL_Netchan_TransmitNextFragment(chan: *mut netchan_t) {
    // PORT-NOTE(view-receiver): `Netchan_TransmitNextFragment` takes
    // `view: &mut EngineHostView`, a receiver this fn's LAW signature does
    // not carry.
    Netchan_TransmitNextFragment(view, chan);
}

/// Raven `CL_Netchan_Transmit`.
///
/// This appends the end-of-frame marker, encodes the buffer, and hands it to
/// the netchan for send.
///
/// Source: `oracle/codemp/client/cl_net_chan.cpp:137-147`
pub fn CL_Netchan_Transmit(cl: &mut Client, chan: *mut netchan_t, msg: *mut msg_t) {
    unsafe {
        // PORT-NOTE(msg-receiver): `MSG_WriteByte` takes `common: &mut Common`,
        // a receiver this fn's LAW signature does not carry.
        MSG_WriteByte(common, msg, clc_EOF as c_int);
        CL_Netchan_Encode(cl, msg);
        // PORT-NOTE(view-receiver): `Netchan_Transmit` takes
        // `view: &mut EngineHostView`, a receiver this fn's LAW signature
        // does not carry.
        Netchan_Transmit(view, chan, (*msg).cursize, (*msg).data as *const byte);
    }
}

/// Raven `CL_Netchan_Process`.
///
/// This returns false when the netchan drops the fragment, otherwise decodes
/// the payload and accumulates the running saved-byte counter.
///
/// Source: `oracle/codemp/client/cl_net_chan.cpp:157-175`
pub fn CL_Netchan_Process(cl: &mut Client, chan: *mut netchan_t, msg: *mut msg_t) -> qboolean {
    // PORT-NOTE(newsize): Raven's commented-out `static int newsize` (three-kind
    // rule kind 3, genuine cross-frame state) has no field on `Client` in this
    // packet's state table beyond the write access. Threaded as a Client field
    // pending integration wiring.
    // PORT-NOTE(common-receiver): `Netchan_Process` takes `common: &mut Common`,
    // a receiver this fn's LAW signature does not carry.
    let ret = Netchan_Process(common, chan, msg);
    if ret == qboolean::qfalse {
        return qboolean::qfalse;
    }
    CL_Netchan_Decode(cl, msg);
    cl.newsize += unsafe { (*msg).cursize };
    qboolean::qtrue
}
