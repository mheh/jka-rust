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
use mp_qshared::shared::{qboolean, qfalse, qtrue};

use crate::collision_world::CollisionWorld;
use crate::common::common::Common;
use crate::qcommon::net_chan_cpp_consts::{FRAGMENT_BIT, FRAGMENT_SIZE, MAX_LOOPBACK};
use crate::qcommon::net_limits::MAX_MSGLEN;
use crate::qcommon::netchan_t::netchan_t;
use crate::qcommon::protocol::PORT_SERVER;

use mp_host_interface::engine_host::EngineHost;
use mp_qshared::shared::cvar::cvar_t;

// PORT-NOTE(engine-host-state): `RenderModels`'s real definition
// (`mp_renderer::tr_model::render_models::RenderModels`) is unreachable —
// `mp_renderer` depends on `mp_engine_qcommon`, so importing it here would
// cycle. Local placeholder struct matching the cm_load.rs/vm_fns.rs/
// cmd_common.rs precedent throughout this crate.
#[allow(dead_code)]
use crate::cm_load::RenderModels;

// PORT-NOTE(cvar-flags-reach): `CVAR_INIT`/`CVAR_TEMP` (q_shared.h) live in
// `mp_game::q_shared_cvar_flags`, a tier above this crate (`mp_game` depends
// on `mp_engine_qcommon`, so depending back would cycle) — same reachability
// gap as common_fns.rs/vm_fns.rs's identical `mp_game::q_shared_cvar_flags::*`
// references. Escalated in missing_symbols; local consts transcribed here
// pending the canonical q_shared.h flags home landing somewhere both crates
// can reach (e.g. mp_qshared).
/// Raven `CVAR_INIT`.
/// Source: `oracle/codemp/game/q_shared.h:1788`
const CVAR_INIT: c_int = 0x0000_0010;
/// Raven `CVAR_TEMP`.
/// Source: `oracle/codemp/game/q_shared.h:1799`
const CVAR_TEMP: c_int = 0x0000_0100;

// PORT-NOTE(cvar-globals): `showpackets`/`showdrop`/`qport`/
// `net_killdroppedfragments` are file-scope `cvar_t*` globals
// (net_chan.cpp:40-43) with no `EngineCvars`/`Common` home yet (grepped:
// `Common` has no cvar sub-struct). Following the existing `cl_shownet`
// precedent in `common.rs` (collapsed `->integer` read, cvar-registry not
// landed), these are referenced as bare `common.showpackets` / `.showdrop` /
// `.net_qport` / `.net_killdroppedfragments` plain-`i32` fields — escalated
// in missing_symbols for the finisher to wire once `EngineCvars`/the cvar
// sub-struct lands.
//
// PORT-NOTE(loopback-state): `loopbacks[2]` (net_chan.cpp:486) is genuine
// cross-frame state (fork-3 case 3) → an `Engine`/`Common`-owned field,
// `common.loopbacks: [loopback_t; 2]`. `loopback_t` has no rosetta row —
// referenced from its natural home (`crate::qcommon::loopback_t::loopback_t`,
// mirroring `netchan_t`'s file placement) though it does not exist in the
// tree; escalated in missing_symbols.
//
// PORT-NOTE(net-adr-to-string-buf): `NET_AdrToString`'s `static char s[64]`
// is a rotating single-slot return buffer (fork-3 case 2) but the resolved
// signature returns a raw `*const c_char` into it, so it must outlive the
// call — modeled as a `common.net_adr_to_string_buf: [u8; 64]` field per
// ruling 2/3 exactly as `MSG_ReadString`'s scratch buffers already sit on
// `Common`. The field does not exist yet; escalated in missing_symbols.

/// Raven `netsrcString[2]` (net_chan.cpp:45-48) — file-scope const table
/// (fork-3 case 1: const table, no mutation).
///
/// Source: `oracle/codemp/qcommon/net_chan.cpp:45-48`
const NETSRC_STRING: [&str; 2] = ["client", "server"];

// Genuinely-unported callees referenced at their canonical future homes
// (sweep: extern forward-declares eliminated). `Com_sprintf` is a q_shared.c
// helper whose qshared home is not yet landed; `Cvar_Get` awaits cvar.cpp.
// `Sys_StringToAdr`/`Sys_SendPacket` are `PlatformHost` methods with no
// free-fn home threaded through `NET_*`'s pinned signatures — left bare and
// reported (cycle/shape seam).
use crate::cvar_fns::Cvar_Get;
use mp_qshared::shared::q_string::Com_sprintf;

/// Raven `NET_AdrToString`.
///
/// Source: `oracle/codemp/qcommon/net_chan.cpp:406-426`
pub fn NET_AdrToString(common: &mut Common, a: netadr_t) -> *const c_char {
    let s_ptr = common.net_adr_to_string_buf.as_mut_ptr() as *mut c_char;
    let size = common.net_adr_to_string_buf.len() as c_int;

    unsafe {
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
    unsafe {
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

        Sys_SendPacket(length, data, to);
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
        // PORT-NOTE(Q_strncpyz): Raven copies into a fixed `char
        // base[MAX_STRING_CHARS]` via `Q_strncpyz` before scanning for ':';
        // collapsed to an owned `String` with the same truncation length.
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

        // PORT-NOTE(host-seam-gap): the resolved signature carries no host
        // receiver to reach `Sys_StringToAdr` (a `PlatformHost` method) —
        // called bare by its Raven name exactly as the packet's LAW
        // signature prints it; escalated (shape_mismatches/missing_symbols).
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
            // PORT-NOTE(atoi): manual leading-integer scan matching C
            // `atoi` semantics (leading whitespace/sign, stop at first
            // non-digit, 0 on no digits) rather than `str::parse`'s
            // all-or-nothing behavior.
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
        // PORT-NOTE(cvar-globals): see the file-level note — `->integer`
        // reads collapse the not-yet-landed cvar registry to plain `i32`
        // fields, following `common.rs`'s existing `cl_shownet` precedent.
        let showpackets_cvar = Cvar_Get(common, cm, rm, host, "showpackets", "0", CVAR_TEMP);
        common.showpackets = (*showpackets_cvar).integer;

        let showdrop_cvar = Cvar_Get(common, cm, rm, host, "showdrop", "0", CVAR_TEMP);
        common.showdrop = (*showdrop_cvar).integer;

        let qport_cvar = Cvar_Get(
            common,
            cm,
            rm,
            host,
            "net_qport",
            &format!("{port}"),
            CVAR_INIT,
        );
        common.net_qport = (*qport_cvar).integer;

        let killdropped_cvar = Cvar_Get(
            common,
            cm,
            rm,
            host,
            "net_killdroppedfragments",
            "0",
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
