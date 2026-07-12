//! Raw UDP socket + console-input primitives, the native twin of Raven's unix
//! net/main platform layer.
//!
//! Raven keeps `ip_socket`/`ipx_socket` as file statics in
//! `oracle/codemp/unix/unix_net.c`; here the crate owns them behind
//! `AtomicI32` (fd, 0 = unopened, matching Raven's `int` sentinel — a live
//! socket never returns fd 0 while stdin is open). The `netadr_t`/`msg_t`
//! shaping lives one tier up in qcommon (those types cannot be named here
//! without an `mp_qshared -> native_platform` cycle); this module exposes only
//! the OS syscalls in terms of raw address bytes.
//!
//! Source: `oracle/codemp/unix/unix_net.c`, `oracle/codemp/unix/unix_main.c`

#![allow(non_snake_case)]

use core::ffi::c_void;
use std::ffi::CString;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

/// Raven `ip_socket` (unix) — the UDP socket fd; 0 until `NET_OpenIP` binds it.
///
/// Source: `oracle/codemp/unix/unix_net.c:31`
static IP_SOCKET: AtomicI32 = AtomicI32::new(0);

/// Raven `ipx_socket` (unix) — never opened on this platform (no IPX), so it
/// stays 0 and every IPX path short-circuits exactly as the oracle's does.
///
/// Source: `oracle/codemp/unix/unix_net.c:32`
static IPX_SOCKET: AtomicI32 = AtomicI32::new(0);

/// Current UDP socket fd (0 = unopened).
pub fn ip_socket() -> i32 {
    IP_SOCKET.load(Ordering::Relaxed)
}

/// Current IPX socket fd (always 0 on this platform).
pub fn ipx_socket() -> i32 {
    IPX_SOCKET.load(Ordering::Relaxed)
}

/// Outcome of a `net_recvfrom` poll, mirroring the branches of Raven's
/// `Sys_GetPacket` recvfrom handling (`unix_net.c:151-179`).
pub enum NetRecvResult {
    /// `recvfrom` == -1 with `errno` EWOULDBLOCK/ECONNREFUSED — no datagram.
    WouldBlock,
    /// `recvfrom` == -1 with any other `errno` — the `NET_ErrorString` text.
    Error(String),
    /// A datagram of `len` bytes arrived from `from_ip`:`from_port` (network
    /// order), copied into the caller's buffer.
    Received {
        from_ip: [u8; 4],
        from_port: u16,
        len: usize,
    },
}

/// Raven `Sys_StringToSockaddr`'s address resolve (unix) — a leading digit
/// takes `inet_addr` (INADDR_NONE = `255.255.255.255` passes straight through,
/// as the oracle relies on for its caller's out-of-range check), otherwise a
/// `getaddrinfo` (the thread-safe `gethostbyname` equivalent) first-A-record
/// lookup. Returns the 4 network-order address bytes, or `None` when the host
/// lookup fails.
///
/// Source: `oracle/codemp/unix/unix_net.c:87-110`
pub fn net_string_to_ip(s: &str) -> Option<[u8; 4]> {
    let cs = CString::new(s).ok()?;

    if s.as_bytes().first().is_some_and(u8::is_ascii_digit) {
        // *(int *)&sin_addr = inet_addr(s): dotted-quad octets are already the
        // network-order ip[] bytes; a malformed literal yields inet_addr's
        // INADDR_NONE (255.255.255.255), which the caller maps to NA_BAD.
        let octets = s
            .parse::<std::net::Ipv4Addr>()
            .map(|v| v.octets())
            .unwrap_or([255, 255, 255, 255]);
        return Some(octets);
    }

    let mut hints: libc::addrinfo = unsafe { core::mem::zeroed() };
    hints.ai_family = libc::AF_INET;
    let mut res: *mut libc::addrinfo = core::ptr::null_mut();
    // if (! (h = gethostbyname(s)) ) return qfalse;
    if unsafe { libc::getaddrinfo(cs.as_ptr(), core::ptr::null(), &hints, &mut res) } != 0 {
        return None;
    }
    if res.is_null() {
        return None;
    }
    // *(int *)&sin_addr = *(int *)h->h_addr_list[0]: first result's sin_addr.
    let sa = unsafe { (*res).ai_addr as *const libc::sockaddr_in };
    let ip = unsafe { (*sa).sin_addr.s_addr }.to_ne_bytes();
    unsafe { libc::freeaddrinfo(res) };
    Some(ip)
}

/// Build the `sockaddr_in` Raven's `NetadrToSockadr` produces from raw parts:
/// AF_INET, the caller-supplied network-order port, and the 4 network-order
/// address bytes.
///
/// Source: `oracle/codemp/unix/unix_net.c:43-59`
fn make_sockaddr_in(ip: [u8; 4], port_net: u16) -> libc::sockaddr_in {
    let mut addr: libc::sockaddr_in = unsafe { core::mem::zeroed() };
    addr.sin_family = libc::AF_INET as libc::sa_family_t;
    addr.sin_port = port_net;
    addr.sin_addr.s_addr = u32::from_ne_bytes(ip);
    addr
}

/// Raven `Sys_SendPacket`'s `sendto` (unix): fire one datagram at `ip`:`port_net`
/// through `sock`. A 0 fd (unopened socket) is a no-op, matching the oracle's
/// `if (!net_socket) return;`. Returns the `NET_ErrorString` text on `sendto`
/// failure so the caller can reproduce the oracle's `Com_Printf`.
///
/// Source: `oracle/codemp/unix/unix_net.c:214-227`
///
/// # Safety
/// `data` must point to `length` readable bytes.
pub unsafe fn net_sendto(
    sock: i32,
    ip: [u8; 4],
    port_net: u16,
    data: *const c_void,
    length: i32,
) -> Result<(), String> {
    if sock == 0 {
        return Ok(());
    }
    let addr = make_sockaddr_in(ip, port_net);
    let ret = libc::sendto(
        sock,
        data,
        length as usize,
        0,
        &addr as *const libc::sockaddr_in as *const libc::sockaddr,
        core::mem::size_of::<libc::sockaddr_in>() as libc::socklen_t,
    );
    if ret == -1 {
        Err(std::io::Error::last_os_error().to_string())
    } else {
        Ok(())
    }
}

/// Raven `Sys_GetPacket`'s `recvfrom` (unix): poll `sock` for one datagram into
/// `buf`, splitting `errno` into the WouldBlock (EWOULDBLOCK/ECONNREFUSED) and
/// real-error branches the oracle distinguishes.
///
/// Source: `oracle/codemp/unix/unix_net.c:151-179`
pub fn net_recvfrom(sock: i32, buf: &mut [u8]) -> NetRecvResult {
    let mut from: libc::sockaddr_in = unsafe { core::mem::zeroed() };
    let mut fromlen = core::mem::size_of::<libc::sockaddr_in>() as libc::socklen_t;
    let ret = unsafe {
        libc::recvfrom(
            sock,
            buf.as_mut_ptr() as *mut c_void,
            buf.len(),
            0,
            &mut from as *mut libc::sockaddr_in as *mut libc::sockaddr,
            &mut fromlen,
        )
    };
    if ret == -1 {
        let err = std::io::Error::last_os_error();
        return match err.raw_os_error() {
            Some(e) if e == libc::EWOULDBLOCK || e == libc::ECONNREFUSED => NetRecvResult::WouldBlock,
            _ => NetRecvResult::Error(err.to_string()),
        };
    }
    NetRecvResult::Received {
        from_ip: from.sin_addr.s_addr.to_ne_bytes(),
        from_port: from.sin_port,
        len: ret as usize,
    }
}

/// Raven `stdin_active` (unix) — cleared on stdin EOF so `Sys_ConsoleInput`
/// stops polling a closed pipe.
///
/// Source: `oracle/codemp/unix/unix_main.c:257`
static STDIN_ACTIVE: AtomicBool = AtomicBool::new(true);

/// Raven `Sys_ConsoleInput` (unix): a non-blocking `select`/`read` on stdin,
/// returning a completed line (newline stripped) or `None`. The oracle's
/// `com_dedicated`/`com_dedicated->value` gate lives in a cvar this base-tier
/// crate cannot reach; the poll is non-blocking so the gate is behaviorally
/// moot on a tty.
///
/// Source: `oracle/codemp/unix/unix_main.c:259-289`
pub fn sys_console_input() -> Option<String> {
    if !STDIN_ACTIVE.load(Ordering::Relaxed) {
        return None;
    }

    let mut fdset: libc::fd_set = unsafe { core::mem::zeroed() };
    unsafe {
        libc::FD_ZERO(&mut fdset);
        libc::FD_SET(0, &mut fdset); // stdin
    }
    let mut timeout = libc::timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    let sel = unsafe {
        libc::select(
            1,
            &mut fdset,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &mut timeout,
        )
    };
    if sel == -1 || !unsafe { libc::FD_ISSET(0, &fdset) } {
        return None;
    }

    let mut text = [0u8; 256];
    let len = unsafe { libc::read(0, text.as_mut_ptr() as *mut c_void, text.len()) };
    if len == 0 {
        // eof!
        STDIN_ACTIVE.store(false, Ordering::Relaxed);
        return None;
    }
    if len < 1 {
        return None;
    }

    // text[len-1] = 0: rip off the '\n' and terminate.
    let line = &text[..(len as usize - 1)];
    Some(String::from_utf8_lossy(line).into_owned())
}

/// Raven `MAX_IPS` — capacity of the local-interface address table.
///
/// Source: `oracle/codemp/unix/unix_net.c:34`
pub const MAX_IPS: usize = 16;

/// Raven `numIP`/`localIP[MAX_IPS][4]` (unix file statics) — the local
/// interface addresses `NET_GetLocalAddress` collects at `NET_Init`, read by
/// `Sys_IsLANAddress`'s class-C comparison. Owned here behind a `Mutex` (the
/// crate's file-static convention); empty until `NET_GetLocalAddress` fills it,
/// during which window `Sys_IsLANAddress`'s table loop matches nothing —
/// exactly the oracle's pre-`NET_Init` behavior.
///
/// Source: `oracle/codemp/unix/unix_net.c:35-36`
static LOCAL_IP: std::sync::Mutex<Vec<[u8; 4]>> = std::sync::Mutex::new(Vec::new());

/// `NET_GetLocalAddress`'s table write (the interface walk itself lives with
/// the boot-slice `NET_Init` port): record a local interface address, capped
/// at Raven's `MAX_IPS`.
///
/// Source: `oracle/codemp/unix/unix_net.c:305-341`
pub fn net_add_local_address(ip: [u8; 4]) {
    let mut t = LOCAL_IP.lock().unwrap();
    if t.len() < MAX_IPS {
        t.push(ip);
    }
}

/// The `NA_IP` arm of Raven `Sys_IsLANAddress` (unix): the class-C octet
/// comparison against `localIP[]` plus the RFC1918 192.168 block check. (The
/// class-A/B comparisons are commented out in the oracle; the loopback/IPX
/// type branches live with the `netadr_t`-shaped wrapper one tier up.)
///
/// Source: `oracle/codemp/unix/unix_net.c:255-289`
pub fn net_is_lan_ip(ip: [u8; 4]) -> bool {
    let t = LOCAL_IP.lock().unwrap();
    for local in t.iter() {
        // Class C
        if ip[0] == local[0] && ip[1] == local[1] && ip[2] == local[2] {
            return true;
        }
        // also check against the RFC1918 class c blocks
        if ip[0] == 192 && local[0] == 192 && ip[1] == 168 && local[1] == 168 {
            return true;
        }
    }
    false
}
