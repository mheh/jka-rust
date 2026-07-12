#![allow(non_snake_case, non_camel_case_types, unused_variables)]
//! `net_chan.cpp` — network channel/loopback plumbing: address formatting,
//! the loopback (localhost) transport, netchan sequencing/fragmentation, and
//! out-of-band datagram send.
//!
//! Source: `oracle/codemp/qcommon/net_chan.cpp`

use core::ffi::{c_char, c_int};

use mp_qshared::common::mp::qcommon::msg_t::msg_t;
use mp_qshared::common::mp::qcommon::netadr_t::netadr_t;
use mp_qshared::common::mp::qcommon::netadrtype_t::netadrtype_t;
use mp_qshared::common::mp::qcommon::netsrc_t::netsrc_t;
use mp_qshared::shared::limits::MAX_STRING_CHARS;
use mp_qshared::shared::{errorParm_t, qboolean, qfalse, qtrue};

use crate::collision_world::CollisionWorld;
use crate::common::com_error;
use crate::common::common::Common;
use crate::msg::{MSG_InitOOB, MSG_WriteData, MSG_WriteLong, MSG_WriteShort};
use crate::qcommon::net_chan_cpp_consts::{
    FRAGMENT_BIT, FRAGMENT_SIZE, MAX_LOOPBACK, MAX_PACKETLEN,
};
use crate::qcommon::net_limits::MAX_MSGLEN;
use crate::qcommon::netchan_t::netchan_t;
use crate::qcommon::protocol::PORT_SERVER;

use mp_host_interface::engine_host::EngineHost;

// `RenderModels` here is a local placeholder: `mp_renderer` depends on this crate, so importing the real type would cycle (matches cm_load.rs's precedent).
#[allow(dead_code)]
use crate::cm_load::RenderModels;

/// Raven `CVAR_INIT`.
/// Source: `oracle/codemp/game/q_shared.h:1788`
const CVAR_INIT: c_int = 0x0000_0010;
/// Raven `CVAR_TEMP`.
/// Source: `oracle/codemp/game/q_shared.h:1799`
const CVAR_TEMP: c_int = 0x0000_0100;

/// Raven `netsrcString[2]` (net_chan.cpp:45-48) — file-scope const table
/// (fork-3 case 1: const table, no mutation).
///
/// Source: `oracle/codemp/qcommon/net_chan.cpp:45-48`
const NETSRC_STRING: [&str; 2] = ["client", "server"];

// `Com_sprintf` is a q_shared.c helper whose qshared home is not yet landed;
// `Cvar_Get` awaits cvar.cpp.
use crate::cvar_fns::Cvar_Get;
use crate::sys_net::{Sys_SendPacket, Sys_StringToAdr};
use mp_qshared::shared::q_string::Com_sprintf;

/// Raven `NET_AdrToString`.
///
/// Source: `oracle/codemp/qcommon/net_chan.cpp:406-426`
pub fn NET_AdrToString(common: &mut Common, a: netadr_t) -> *const c_char {
    let s_ptr = common.net_adr_to_string_buf.as_mut_ptr() as *mut c_char;
    let size = common.net_adr_to_string_buf.len() as c_int;

    match a.r#type {
        netadrtype_t::NA_LOOPBACK => {
            Com_sprintf(s_ptr, size, "loopback");
        }
        netadrtype_t::NA_BOT => {
            Com_sprintf(s_ptr, size, "bot");
        }
        netadrtype_t::NA_IP => {
            // BigShort(a.port): host is little-endian (matches the referee
            // platform), so BigShort is a byte-swap; %i vararg promotion
            // sign-extends the `short` return, reproduced via the i16 hop.
            let port = (a.port.swap_bytes() as i16) as i32;
            Com_sprintf(
                s_ptr,
                size,
                &format!("{}.{}.{}.{}:{}", a.ip[0], a.ip[1], a.ip[2], a.ip[3], port),
            );
        }
        netadrtype_t::NA_BAD => {
            Com_sprintf(s_ptr, size, "BAD");
        }
        _ => {
            let port = (a.port.swap_bytes() as i16) as i32;
            Com_sprintf(
                s_ptr,
                size,
                &format!(
                    "{:02x}{:02x}{:02x}{:02x}.{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}:{}",
                    a.ipx[0],
                    a.ipx[1],
                    a.ipx[2],
                    a.ipx[3],
                    a.ipx[4],
                    a.ipx[5],
                    a.ipx[6],
                    a.ipx[7],
                    a.ipx[8],
                    a.ipx[9],
                    port
                ),
            );
        }
    }

    s_ptr as *const c_char
}

/// Raven `NET_IsLocalAddress`.
///
/// Source: `oracle/codemp/qcommon/net_chan.cpp:458-460`
pub fn NET_IsLocalAddress(adr: netadr_t) -> qboolean {
    if matches!(adr.r#type, netadrtype_t::NA_LOOPBACK) {
        qtrue
    } else {
        qfalse
    }
}

/// Raven `Netchan_Setup`.
///
/// Source: `oracle/codemp/qcommon/net_chan.cpp:71-79`
pub fn Netchan_Setup(sock: netsrc_t, chan: *mut netchan_t, adr: netadr_t, qport: c_int) {
    unsafe {
        crate::common_fns::Com_Memset(chan as *mut (), 0, core::mem::size_of::<netchan_t>());

        (*chan).sock = sock;
        (*chan).remoteAddress = adr;
        (*chan).qport = qport;
        (*chan).incomingSequence = 0;
        (*chan).outgoingSequence = 1;
    }
}

/// Raven `NET_GetLoopPacket`.
///
/// Source: `oracle/codemp/qcommon/net_chan.cpp:489-511`
pub fn NET_GetLoopPacket(
    common: &mut Common,
    sock: netsrc_t,
    net_from: *mut netadr_t,
    net_message: *mut msg_t,
) -> qboolean {
    unsafe {
        let loop_ = &mut common.loopbacks[sock as usize];

        if loop_.send - loop_.get > MAX_LOOPBACK {
            loop_.get = loop_.send - MAX_LOOPBACK;
        }

        if loop_.get >= loop_.send {
            return qfalse;
        }

        let i = (loop_.get & (MAX_LOOPBACK - 1)) as usize;
        loop_.get += 1;

        crate::common_fns::Com_Memcpy(
            (*net_message).data as *mut (),
            loop_.msgs[i].data.as_ptr() as *const (),
            loop_.msgs[i].datalen as usize,
        );
        (*net_message).cursize = loop_.msgs[i].datalen;
        crate::common_fns::Com_Memset(net_from as *mut (), 0, core::mem::size_of::<netadr_t>());
        (*net_from).r#type = netadrtype_t::NA_LOOPBACK;
        qtrue
    }
}

/// Raven `NET_SendLoopPacket`.
///
/// Source: `oracle/codemp/qcommon/net_chan.cpp:514-526`
pub fn NET_SendLoopPacket(
    common: &mut Common,
    sock: netsrc_t,
    length: c_int,
    data: *const (),
    to: netadr_t,
) {
    let _ = to;
    let loop_ = &mut common.loopbacks[(sock as usize) ^ 1];

    let i = (loop_.send & (MAX_LOOPBACK - 1)) as usize;
    loop_.send += 1;

    crate::common_fns::Com_Memcpy(
        loop_.msgs[i].data.as_mut_ptr() as *mut (),
        data,
        length as usize,
    );
    loop_.msgs[i].datalen = length;
}

/// Raven `NET_SendPacket`.
///
/// Source: `oracle/codemp/qcommon/net_chan.cpp:531-549`
pub fn NET_SendPacket(
    common: &mut Common,
    sock: netsrc_t,
    length: c_int,
    data: *const (),
    to: netadr_t,
) {
    unsafe {
        // sequenced packets are shown in netchan, so just show oob
        if common.showpackets != 0 && *(data as *const c_int) == -1 {
            crate::common::common::com_printf(common, &format!("send packet {length:4}\n"));
        }

        if to.r#type == netadrtype_t::NA_LOOPBACK {
            NET_SendLoopPacket(common, sock, length, data, to);
            return;
        }
        if to.r#type == netadrtype_t::NA_BOT {
            return;
        }
        if to.r#type == netadrtype_t::NA_BAD {
            return;
        }

        Sys_SendPacket(common, length, data, to);
    }
}

/// Raven `NET_StringToAdr`.
///
/// Source: `oracle/codemp/qcommon/net_chan.cpp:617-656`
pub fn NET_StringToAdr(s: *const c_char, a: *mut netadr_t) -> qboolean {
    unsafe {
        let s_str = core::ffi::CStr::from_ptr(s).to_string_lossy().into_owned();

        if s_str == "localhost" {
            crate::common_fns::Com_Memset(a as *mut (), 0, core::mem::size_of::<netadr_t>());
            (*a).r#type = netadrtype_t::NA_LOOPBACK;
            return qtrue;
        }

        // look for a port number
        // Raven copies into a fixed `char base[MAX_STRING_CHARS]` via `Q_strncpyz` before scanning for ':'; collapsed here to an owned `String` truncated to the same length.
        let mut base = s_str;
        if base.len() > (MAX_STRING_CHARS - 1) {
            base.truncate(MAX_STRING_CHARS - 1);
        }
        let port_str: Option<String> = if let Some(idx) = base.find(':') {
            let p = base[idx + 1..].to_string();
            base.truncate(idx);
            Some(p)
        } else {
            None
        };

        let r: bool = Sys_StringToAdr(&base, &mut *a);

        if !r {
            (*a).r#type = netadrtype_t::NA_BAD;
            return qfalse;
        }

        // inet_addr returns this if out of range
        if (*a).ip == [255u8, 255, 255, 255] {
            (*a).r#type = netadrtype_t::NA_BAD;
            return qfalse;
        }

        if let Some(p) = port_str {
            // Manual leading-integer scan matches C `atoi` semantics (leading whitespace/sign, stop at first non-digit, 0 on no digits) rather than `str::parse`'s all-or-nothing behavior.
            let trimmed = p.trim_start();
            let neg = trimmed.starts_with('-');
            let digits: String = trimmed
                .trim_start_matches(['+', '-'])
                .chars()
                .take_while(|c| c.is_ascii_digit())
                .collect();
            let mut v: i64 = digits.parse().unwrap_or(0);
            if neg {
                v = -v;
            }
            let port_num = v as i32;
            (*a).port = (port_num as i16 as u16).swap_bytes();
        } else {
            (*a).port = (PORT_SERVER as u16).swap_bytes();
        }

        qtrue
    }
}

/// Raven `Netchan_Init`.
///
/// Source: `oracle/codemp/qcommon/net_chan.cpp:56-62`
pub fn Netchan_Init(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    port: c_int,
) {
    let port = port & 0xffff;

    unsafe {
        // `->integer` is cached into a plain `i32` field once here, not re-read live like Raven's `cvar_t*` — a runtime `/set` won't be reflected after init.
        let showpackets_cvar = Cvar_Get(common, cm, rm, host, c"showpackets".as_ptr(), c"0".as_ptr(), CVAR_TEMP);
        common.showpackets = (*showpackets_cvar).integer;

        let showdrop_cvar = Cvar_Get(common, cm, rm, host, c"showdrop".as_ptr(), c"0".as_ptr(), CVAR_TEMP);
        common.showdrop = (*showdrop_cvar).integer;

        let qport_val = std::ffi::CString::new(format!("{port}")).unwrap();
        let qport_cvar = Cvar_Get(
            common,
            cm,
            rm,
            host,
            c"net_qport".as_ptr(),
            qport_val.as_ptr(),
            CVAR_INIT,
        );
        common.net_qport = (*qport_cvar).integer;

        let killdropped_cvar = Cvar_Get(
            common,
            cm,
            rm,
            host,
            c"net_killdroppedfragments".as_ptr(),
            c"0".as_ptr(),
            CVAR_TEMP,
        );
        common.net_killdroppedfragments = (*killdropped_cvar).integer;
    }
}

/// Raven `NET_CompareBaseAdr`.
///
/// Source: `oracle/codemp/qcommon/net_chan.cpp:378-404`
pub fn NET_CompareBaseAdr(common: &mut Common, a: netadr_t, b: netadr_t) -> qboolean {
    if a.r#type as i32 != b.r#type as i32 {
        return qfalse;
    }

    if matches!(a.r#type, netadrtype_t::NA_LOOPBACK) {
        return qtrue;
    }

    if matches!(a.r#type, netadrtype_t::NA_IP) {
        if a.ip == b.ip {
            return qtrue;
        }
        return qfalse;
    }

    // #ifndef _XBOX // No IPX
    if matches!(a.r#type, netadrtype_t::NA_IPX) {
        let eq = unsafe {
            libc::memcmp(
                a.ipx.as_ptr() as *const libc::c_void,
                b.ipx.as_ptr() as *const libc::c_void,
                10,
            ) == 0
        };
        if eq {
            return qtrue;
        }
        return qfalse;
    }

    crate::common::common::com_printf(common, "NET_CompareBaseAdr: bad address type\n");
    qfalse
}

/// Raven `NET_CompareAdr` — full-address (base + port) equality.
///
/// Source: `oracle/codemp/qcommon/net_chan.cpp:429-455`
pub fn NET_CompareAdr(common: &mut Common, a: netadr_t, b: netadr_t) -> qboolean {
    if a.r#type as i32 != b.r#type as i32 {
        return qfalse;
    }

    if matches!(a.r#type, netadrtype_t::NA_LOOPBACK) {
        return qtrue;
    }

    if matches!(a.r#type, netadrtype_t::NA_IP) {
        if a.ip[0] == b.ip[0]
            && a.ip[1] == b.ip[1]
            && a.ip[2] == b.ip[2]
            && a.ip[3] == b.ip[3]
            && a.port == b.port
        {
            return qtrue;
        }
        return qfalse;
    }

    // #ifndef _XBOX // No IPX
    if matches!(a.r#type, netadrtype_t::NA_IPX) {
        let eq = unsafe {
            libc::memcmp(
                a.ipx.as_ptr() as *const libc::c_void,
                b.ipx.as_ptr() as *const libc::c_void,
                10,
            ) == 0
        };
        if eq && a.port == b.port {
            return qtrue;
        }
        return qfalse;
    }

    crate::common::common::com_printf(common, "NET_CompareAdr: bad address type\n");
    qfalse
}

/// Raven `NET_OutOfBandPrint` — send a text message in an out-of-band datagram.
///
/// Raven's variadic `format, ...` is pre-rendered by callers into `s` (the
/// established `&str`/`String` reshape); Raven's `QDECL` (`__cdecl`) macro has
/// no Rust equivalent and is dropped.
///
/// Source: `oracle/codemp/qcommon/net_chan.cpp:559-576`
pub fn NET_OutOfBandPrint(common: &mut Common, sock: netsrc_t, adr: netadr_t, s: String) {
    // §19: Raven's `char string[MAX_MSGLEN]` local is written header-first then
    // filled by `vsprintf(string+4,...)` before the `strlen` read; zero-init
    // here satisfies definite-init for the same reachable-before-read shape.
    let mut string = [0u8; MAX_MSGLEN as usize];

    // set the header
    string[0] = 0xff;
    string[1] = 0xff;
    string[2] = 0xff;
    string[3] = 0xff;

    // vsprintf(string+4, format, argptr): the rendered text lands after the
    // 4-byte header. Faithful to the fixed scratch, an over-long render runs
    // past MAX_MSGLEN and panics on the Rust bounds check where C reads/writes
    // adjacent stack (UB); OOB prints are short ("challengeResponse ...").
    let bytes = s.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        string[4 + i] = b;
    }

    // strlen(string): the 0xff header is non-NUL, so this is 4 + the rendered
    // length (vsprintf output carries no interior NUL).
    let len = 4 + bytes.len();

    // send the datagram
    NET_SendPacket(common, sock, len as c_int, string.as_ptr() as *const (), adr);
}

/// Raven `NET_OutOfBandData`.
///
/// Note: Raven's `QDECL` is a calling-convention macro (`__cdecl`); it has no
/// Rust equivalent and is dropped from the signature (no behavior it gates).
///
/// Source: `oracle/codemp/qcommon/net_chan.cpp:585-605`
pub fn NET_OutOfBandData(
    common: &mut Common,
    sock: netsrc_t,
    adr: netadr_t,
    format: *mut native_types::byte,
    len: c_int,
) {
    // §19: Raven's `byte string[MAX_MSGLEN*2]` local is written header-first
    // then filled by the loop below before any read; zero-init here satisfies
    // Rust's definite-init rule for the same reachable-before-read shape.
    let mut string = [0u8; (MAX_MSGLEN * 2) as usize];
    string[0] = 0xff;
    string[1] = 0xff;
    string[2] = 0xff;
    string[3] = 0xff;

    unsafe {
        for i in 0..len as isize {
            string[(i + 4) as usize] = *format.offset(i);
        }

        let mut mbuf = msg_t {
            allowoverflow: qfalse,
            overflowed: qfalse,
            oob: qfalse,
            data: string.as_mut_ptr(),
            maxsize: 0,
            cursize: len + 4,
            readcount: 0,
            bit: 0,
        };

        // set the header
        crate::qcommon::huff::Huff_Compress(&mut mbuf, 12);
        // send the datagram
        NET_SendPacket(common, sock, mbuf.cursize, mbuf.data as *const (), adr);
    }
}

/// Raven `Netchan_TransmitNextFragment` — send one fragment of the current
/// message. Only builds the OOB header here (`MSG_InitOOB`). `qport->integer`
/// is cached on `Common` (`net_qport`); `showpackets->integer` likewise.
///
/// Source: `oracle/codemp/qcommon/net_chan.cpp:80-134`
pub fn Netchan_TransmitNextFragment(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    chan: *mut netchan_t,
) {
    unsafe {
        let mut send_buf = [0u8; MAX_PACKETLEN as usize];
        let mut send: msg_t = core::mem::zeroed();

        // write the packet header
        MSG_InitOOB(common, cm, rm, host, &mut send, send_buf.as_mut_ptr(), send_buf.len() as c_int); // <-- only do the oob here

        MSG_WriteLong(common, &mut send, (*chan).outgoingSequence | FRAGMENT_BIT);

        // send the qport if we are a client
        if matches!((*chan).sock, netsrc_t::NS_CLIENT) {
            let qport = common.net_qport;
            MSG_WriteShort(common, &mut send, qport);
        }

        // copy the reliable message to the packet first
        let mut fragmentLength = FRAGMENT_SIZE;
        if (*chan).unsentFragmentStart + fragmentLength > (*chan).unsentLength {
            fragmentLength = (*chan).unsentLength - (*chan).unsentFragmentStart;
        }

        MSG_WriteShort(common, &mut send, (*chan).unsentFragmentStart);
        MSG_WriteShort(common, &mut send, fragmentLength);
        MSG_WriteData(
            common,
            &mut send,
            (*chan).unsentBuffer.as_ptr().add((*chan).unsentFragmentStart as usize) as *const (),
            fragmentLength,
        );

        // send the datagram
        NET_SendPacket(
            common,
            (*chan).sock,
            send.cursize,
            send.data as *const (),
            (*chan).remoteAddress,
        );

        if common.showpackets != 0 {
            crate::common::common::com_printf(
                common,
                &format!(
                    "{} send {:4} : s={} fragment={},{}\n",
                    NETSRC_STRING[(*chan).sock as usize],
                    send.cursize,
                    (*chan).outgoingSequence - 1,
                    (*chan).unsentFragmentStart,
                    fragmentLength
                ),
            );
        }

        (*chan).unsentFragmentStart += fragmentLength;

        // this exit condition is a little tricky, because a packet
        // that is exactly the fragment length still needs to send
        // a second packet of zero length so that the other side
        // can tell there aren't more to follow
        if (*chan).unsentFragmentStart == (*chan).unsentLength && fragmentLength != FRAGMENT_SIZE {
            (*chan).outgoingSequence += 1;
            (*chan).unsentFragments = qfalse;
        }
    }
}

/// Raven `Netchan_Transmit` — send a message to a connection, fragmenting if
/// necessary. A 0 length still generates a packet. Raven's variadic-free
/// `(chan, length, data)` signature is preserved.
///
/// Source: `oracle/codemp/qcommon/net_chan.cpp:143-191`
pub fn Netchan_Transmit(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    chan: *mut netchan_t,
    length: c_int,
    data: *const native_types::byte,
) {
    unsafe {
        let mut send_buf = [0u8; MAX_PACKETLEN as usize];
        let mut send: msg_t = core::mem::zeroed();

        if length > MAX_MSGLEN as c_int {
            com_error(errorParm_t::ERR_DROP, format!("Netchan_Transmit: length = {length}"));
        }
        (*chan).unsentFragmentStart = 0;

        if (*chan).unsentFragments != qfalse {
            crate::common::common::com_printf(
                common,
                &format!("[ISM] Stomping Unsent Fragments {}\n", NETSRC_STRING[(*chan).sock as usize]),
            );
        }

        // fragment large reliable messages
        if length >= FRAGMENT_SIZE {
            (*chan).unsentFragments = qtrue;
            (*chan).unsentLength = length;
            crate::common_fns::Com_Memcpy(
                (*chan).unsentBuffer.as_mut_ptr() as *mut (),
                data as *const (),
                length as usize,
            );

            // only send the first fragment now
            Netchan_TransmitNextFragment(common, cm, rm, host, chan);

            return;
        }

        // write the packet header
        MSG_InitOOB(common, cm, rm, host, &mut send, send_buf.as_mut_ptr(), send_buf.len() as c_int);

        MSG_WriteLong(common, &mut send, (*chan).outgoingSequence);
        (*chan).outgoingSequence += 1;

        // send the qport if we are a client
        if matches!((*chan).sock, netsrc_t::NS_CLIENT) {
            let qport = common.net_qport;
            MSG_WriteShort(common, &mut send, qport);
        }

        MSG_WriteData(common, &mut send, data as *const (), length);

        // send the datagram
        NET_SendPacket(
            common,
            (*chan).sock,
            send.cursize,
            send.data as *const (),
            (*chan).remoteAddress,
        );

        if common.showpackets != 0 {
            crate::common::common::com_printf(
                common,
                &format!(
                    "{} send {:4} : s={} ack={}\n",
                    NETSRC_STRING[(*chan).sock as usize],
                    send.cursize,
                    (*chan).outgoingSequence - 1,
                    (*chan).incomingSequence
                ),
            );
        }
    }
}

/// Raven `Netchan_Process`.
///
/// Source: `oracle/codemp/qcommon/net_chan.cpp:208-366`
pub fn Netchan_Process(common: &mut Common, chan: *mut netchan_t, msg: *mut msg_t) -> qboolean {
    unsafe {
        // get sequence numbers
        crate::msg::MSG_BeginReadingOOB(msg);
        let mut sequence = crate::msg::MSG_ReadLong(common, msg);

        // check for fragment information
        let fragmented;
        if sequence & FRAGMENT_BIT != 0 {
            sequence &= !FRAGMENT_BIT;
            fragmented = qtrue;
        } else {
            fragmented = qfalse;
        }

        // read the qport if we are a server
        if matches!((*chan).sock, netsrc_t::NS_SERVER) {
            let _qport = crate::msg::MSG_ReadShort(common, msg);
        }

        // read the fragment information
        let (fragment_start, fragment_length) = if fragmented != qfalse {
            let start = (crate::msg::MSG_ReadShort(common, msg) as u16) as i32;
            let length = (crate::msg::MSG_ReadShort(common, msg) as u16) as i32;
            (start, length)
        } else {
            (0, 0) // stop warning message
        };

        if common.showpackets != 0 {
            if fragmented != qfalse {
                crate::common::common::com_printf(
                    common,
                    &format!(
                        "{} recv {:4} : s={} fragment={},{}\n",
                        NETSRC_STRING[(*chan).sock as usize],
                        (*msg).cursize,
                        sequence,
                        fragment_start,
                        fragment_length
                    ),
                );
            } else {
                crate::common::common::com_printf(
                    common,
                    &format!(
                        "{} recv {:4} : s={}\n",
                        NETSRC_STRING[(*chan).sock as usize],
                        (*msg).cursize,
                        sequence
                    ),
                );
            }
        }

        //
        // discard out of order or duplicated packets
        //
        if sequence <= (*chan).incomingSequence {
            if common.showdrop != 0 || common.showpackets != 0 {
                let adr = NET_AdrToString(common, (*chan).remoteAddress);
                let adr_str = core::ffi::CStr::from_ptr(adr)
                    .to_string_lossy()
                    .into_owned();
                crate::common::common::com_printf(
                    common,
                    &format!(
                        "{}:Out of order packet {} at {}\n",
                        adr_str,
                        sequence,
                        (*chan).incomingSequence
                    ),
                );
            }
            return qfalse;
        }

        //
        // dropped packets don't keep the message from being used
        //
        (*chan).dropped = sequence - ((*chan).incomingSequence + 1);
        if (*chan).dropped > 0 && (common.showdrop != 0 || common.showpackets != 0) {
            let adr = NET_AdrToString(common, (*chan).remoteAddress);
            let adr_str = core::ffi::CStr::from_ptr(adr)
                .to_string_lossy()
                .into_owned();
            crate::common::common::com_printf(
                common,
                &format!(
                    "{}:Dropped {} packets at {}\n",
                    adr_str,
                    (*chan).dropped,
                    sequence
                ),
            );
        }

        //
        // if this is the final fragment of a reliable message,
        // bump incoming_reliable_sequence
        //
        if fragmented != qfalse {
            // make sure we
            if sequence != (*chan).fragmentSequence {
                (*chan).fragmentSequence = sequence;
                (*chan).fragmentLength = 0;
            }

            // if we missed a fragment, dump the message
            if fragment_start != (*chan).fragmentLength {
                if common.showdrop != 0 || common.showpackets != 0 {
                    let adr = NET_AdrToString(common, (*chan).remoteAddress);
                    let adr_str = core::ffi::CStr::from_ptr(adr)
                        .to_string_lossy()
                        .into_owned();
                    crate::common::common::com_printf(
                        common,
                        &format!("{}:Dropped a message fragment\n", adr_str),
                    );
                }
                // we can still keep the part that we have so far, so we
                // don't need to clear chan->fragmentLength — hell yeah we
                // have to dump the whole thing -gil / but I am scared - mw
                return qfalse;
            }

            // copy the fragment to the fragment buffer
            if fragment_length < 0
                || (*msg).readcount + fragment_length > (*msg).cursize
                || (*chan).fragmentLength + fragment_length > MAX_MSGLEN as i32
            {
                if common.showdrop != 0 || common.showpackets != 0 {
                    let adr = NET_AdrToString(common, (*chan).remoteAddress);
                    let adr_str = core::ffi::CStr::from_ptr(adr)
                        .to_string_lossy()
                        .into_owned();
                    crate::common::common::com_printf(
                        common,
                        &format!("{}:illegal fragment length\n", adr_str),
                    );
                }
                return qfalse;
            }

            crate::common_fns::Com_Memcpy(
                (*chan)
                    .fragmentBuffer
                    .as_mut_ptr()
                    .add((*chan).fragmentLength as usize) as *mut (),
                (*msg).data.add((*msg).readcount as usize) as *const (),
                fragment_length as usize,
            );

            (*chan).fragmentLength += fragment_length;

            // if this wasn't the last fragment, don't process anything
            if fragment_length == FRAGMENT_SIZE {
                return qfalse;
            }

            if (*chan).fragmentLength + 4 > (*msg).maxsize {
                let adr = NET_AdrToString(common, (*chan).remoteAddress);
                let adr_str = core::ffi::CStr::from_ptr(adr)
                    .to_string_lossy()
                    .into_owned();
                crate::common::common::com_printf(
                    common,
                    &format!(
                        "{}:fragmentLength {} > msg->maxsize\n",
                        adr_str,
                        (*chan).fragmentLength + 4
                    ),
                );
                return qfalse;
            }

            // copy the full message over the partial fragment

            // make sure the sequence number is still there
            // LittleLong(sequence): host is little-endian (referee
            // platform), so this is the identity transform.
            *((*msg).data as *mut i32) = sequence;

            crate::common_fns::Com_Memcpy(
                (*msg).data.add(4) as *mut (),
                (*chan).fragmentBuffer.as_ptr() as *const (),
                (*chan).fragmentLength as usize,
            );
            (*msg).cursize = (*chan).fragmentLength + 4;
            (*chan).fragmentLength = 0;
            (*msg).readcount = 4; // past the sequence number
            (*msg).bit = 32; // past the sequence number

            return qtrue;
        }

        //
        // the message can now be read from the current message pointer
        //
        (*chan).incomingSequence = sequence;

        qtrue
    }
}
