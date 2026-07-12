//! Engine-tier `Sys_*` net glue whose oracle (unix) bodies shape `netadr_t`/
//! `msg_t` around `native_platform`'s raw socket syscalls.
//!
//! Raven's `Sys_SendPacket`/`Sys_StringToAdr`/`Sys_GetPacket` live in
//! `oracle/codemp/unix/unix_net.c`, but they name `netadr_t`/`msg_t` and print
//! through `Com_Printf`/`NET_AdrToString` — all ABOVE `native_platform` (which
//! cannot take an uphill edge to `mp_qshared` without a dependency cycle). So
//! the address/port shaping (`NetadrToSockadr`/`SockadrToNetadr`) is hosted
//! here and the bare OS calls are delegated downward (per §B state is threaded,
//! not reached). Behavior source: `oracle/codemp/unix/unix_net.c`.
//!
//! The UDP socket itself is never opened yet: nothing in qcommon calls
//! `NET_Init`/`NET_OpenIP`, so `native_platform::net::ip_socket()` stays 0 and
//! every send/recv here short-circuits exactly as the oracle does on an
//! unopened socket. Wiring `NET_OpenIP` is deferred to the server/client boot.

#![allow(non_snake_case)]

use core::ffi::c_int;

use native_platform::net::{ip_socket, ipx_socket, net_is_lan_ip, net_recvfrom, net_sendto, NetRecvResult};

use mp_qshared::common::mp::qcommon::msg_t::msg_t;
use mp_qshared::common::mp::qcommon::netadr_t::netadr_t;
use mp_qshared::common::mp::qcommon::netadrtype_t::netadrtype_t;
use mp_qshared::shared::error_parm::errorParm_t;

use crate::common::{com_error, com_printf, Common};
use crate::net_chan::NET_AdrToString;

/// `NET_AdrToString` renders into a `Common`-owned buffer and returns a pointer
/// into it; copy it out as an owned `String` so the follow-on `com_printf`
/// (which reborrows `Common`) does not alias the buffer.
fn adr_to_string(common: &mut Common, a: netadr_t) -> String {
    let p = NET_AdrToString(common, a);
    unsafe { core::ffi::CStr::from_ptr(p).to_string_lossy().into_owned() }
}

/// Raven `Sys_StringToAdr` (unix) — `Sys_StringToSockaddr` (dotted-quad or DNS
/// resolve) followed by `SockadrToNetadr` (address bytes in, `sin_port` = 0,
/// `NA_IP`). The caller (`NET_StringToAdr`) overwrites the port afterward.
///
/// Source: `oracle/codemp/unix/unix_net.c:122-131` (`SockadrToNetadr`
/// `unix_net.c:63-68`)
pub fn Sys_StringToAdr(s: &str, a: &mut netadr_t) -> bool {
    match native_platform::net::net_string_to_ip(s) {
        None => false,
        Some(ip) => {
            a.ip = ip;
            a.port = 0;
            a.r#type = netadrtype_t::NA_IP;
            true
        }
    }
}

/// Raven `Sys_SendPacket` (unix) — pick the socket for `to`'s address type,
/// build the `sockaddr_in` (`NetadrToSockadr`), and `sendto`. An unopened
/// socket is a no-op; a `sendto` failure prints Raven's `Com_Printf` error.
///
/// Source: `oracle/codemp/unix/unix_net.c:190-228` (`NetadrToSockadr`
/// `unix_net.c:43-59`)
///
/// # Safety
/// `data` must point to `length` readable bytes.
pub unsafe fn Sys_SendPacket(common: &mut Common, length: c_int, data: *const (), to: netadr_t) {
    // NetadrToSockadr: only NA_BROADCAST/NA_IP set sin_addr (-1 / a->ip); the
    // IPX types leave it 0 but their socket is 0, so nothing is sent.
    let (net_socket, ip) = match to.r#type {
        netadrtype_t::NA_BROADCAST => (ip_socket(), [255u8, 255, 255, 255]),
        netadrtype_t::NA_IP => (ip_socket(), to.ip),
        netadrtype_t::NA_IPX => (ipx_socket(), to.ip),
        netadrtype_t::NA_BROADCAST_IPX => (ipx_socket(), [255u8, 255, 255, 255]),
        _ => com_error(
            errorParm_t::ERR_FATAL,
            "NET_SendPacket: bad address type".to_string(),
        ),
    };

    if net_socket == 0 {
        return;
    }

    // sin_port = a->port (already network order).
    match net_sendto(net_socket, ip, to.port, data as *const core::ffi::c_void, length) {
        Ok(()) => {}
        Err(err) => {
            let adr = adr_to_string(common, to);
            com_printf(common, &format!("NET_SendPacket ERROR: {err} to {adr}\n"));
        }
    }
}

/// Raven `Sys_GetPacket` (unix) — poll the IP then IPX socket for one datagram,
/// filling `net_from` (`SockadrToNetadr`) and `net_message`. EWOULDBLOCK/
/// ECONNREFUSED and oversize packets are skipped; other errors print.
///
/// Source: `oracle/codemp/unix/unix_net.c:137-185`
pub fn Sys_GetPacket(common: &mut Common, net_from: &mut netadr_t, net_message: &mut msg_t) -> bool {
    for protocol in 0..2 {
        let net_socket = if protocol == 0 { ip_socket() } else { ipx_socket() };
        if net_socket == 0 {
            continue;
        }

        let buf = unsafe {
            core::slice::from_raw_parts_mut(net_message.data, net_message.maxsize as usize)
        };
        // bk000305: was missing.
        net_message.readcount = 0;

        match net_recvfrom(net_socket, buf) {
            NetRecvResult::WouldBlock => continue,
            NetRecvResult::Error(err) => {
                let adr = adr_to_string(common, *net_from);
                com_printf(common, &format!("NET_GetPacket: {err} from {adr}\n"));
                continue;
            }
            NetRecvResult::Received {
                from_ip,
                from_port,
                len,
            } => {
                // SockadrToNetadr.
                net_from.ip = from_ip;
                net_from.port = from_port;
                net_from.r#type = netadrtype_t::NA_IP;

                if len == net_message.maxsize as usize {
                    let adr = adr_to_string(common, *net_from);
                    com_printf(common, &format!("Oversize packet from {adr}\n"));
                    continue;
                }

                net_message.cursize = len as c_int;
                return true;
            }
        }
    }

    false
}

/// Raven unix `Sys_IsLANAddress` — loopback and IPX are always LAN; non-IP is
/// never LAN; an IP address takes the class-C `localIP[]` comparison (the
/// class-A/B blocks are commented out in the oracle), delegated to
/// `native_platform` where the interface table lives.
///
/// Source: `oracle/codemp/unix/unix_net.c:240-293`
pub fn Sys_IsLANAddress(adr: &netadr_t) -> bool {
    if adr.r#type == netadrtype_t::NA_LOOPBACK {
        return true;
    }
    if adr.r#type == netadrtype_t::NA_IPX {
        return true;
    }
    if adr.r#type != netadrtype_t::NA_IP {
        return false;
    }
    net_is_lan_ip(adr.ip)
}
