//! Client netchan encode/decode/transmit wrappers.
//!
//! Source: `oracle/codemp/client/cl_net_chan.cpp`.

use core::ffi::c_int;

use mp_engine_qcommon::common::common::Common;
use mp_engine_qcommon::common::engine_host_view::EngineHostView;
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
use native_types::qfalse;
use native_types::qtrue;

use crate::client_host::Client;

/// Raven `CL_Netchan_Encode`.
///
/// This xors the outgoing message body with a key derived from the last
/// acknowledged server command, restoring the read cursor afterward.
///
/// Source: `oracle/codemp/client/cl_net_chan.cpp:19-67`
pub fn CL_Netchan_Encode(common: &mut Common, cl: &mut Client, msg: *mut msg_t) {
    unsafe {
        if (*msg).cursize <= CL_ENCODE_START {
            return;
        }

        let srdc = (*msg).readcount;
        let sbit = (*msg).bit;
        let soob = (*msg).oob;

        (*msg).bit = 0;
        (*msg).readcount = 0;
        (*msg).oob = qfalse;

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
pub fn CL_Netchan_Decode(common: &mut Common, cl: &mut Client, msg: *mut msg_t) {
    unsafe {
        let srdc = (*msg).readcount;
        let sbit = (*msg).bit;
        let soob = (*msg).oob;

        (*msg).oob = qfalse;

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
pub fn CL_Netchan_TransmitNextFragment(view: &mut EngineHostView, chan: *mut netchan_t) {
    Netchan_TransmitNextFragment(view, chan);
}

/// Raven `CL_Netchan_Transmit`.
///
/// This appends the end-of-frame marker, encodes the buffer, and hands it to
/// the netchan for send.
///
/// Source: `oracle/codemp/client/cl_net_chan.cpp:137-147`
pub fn CL_Netchan_Transmit(
    view: &mut EngineHostView,
    cl: &mut Client,
    chan: *mut netchan_t,
    msg: *mut msg_t,
) {
    unsafe {
        MSG_WriteByte(view.common, msg, clc_EOF as c_int);
        CL_Netchan_Encode(view.common, cl, msg);
        Netchan_Transmit(view, chan, (*msg).cursize, (*msg).data as *const byte);
    }
}

/// Raven `CL_Netchan_Process`.
///
/// This returns false when the netchan drops the fragment, otherwise decodes
/// the payload and accumulates the running saved-byte counter.
///
/// Source: `oracle/codemp/client/cl_net_chan.cpp:157-175`
pub fn CL_Netchan_Process(
    common: &mut Common,
    cl: &mut Client,
    chan: *mut netchan_t,
    msg: *mut msg_t,
) -> qboolean {
    // PORT-NOTE(newsize): Raven's commented-out `static int newsize` (three-kind
    // rule kind 3, genuine cross-frame state) has no field on `Client` in this
    // packet's state table beyond the write access. Threaded as a Client field
    // pending integration wiring.
    let ret = Netchan_Process(common, chan, msg);
    if ret == qfalse {
        return qfalse;
    }
    CL_Netchan_Decode(common, cl, msg);
    cl.newsize += unsafe { (*msg).cursize };
    qtrue
}
