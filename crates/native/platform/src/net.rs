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

use core::ffi::{c_char, c_int, c_void};
use std::ffi::{CStr, CString};
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

use native_string::latin1_to_string;

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

/// Raven `ip_socket = NET_IPSocket(...)` — publish the bound fd (or 0 on
/// failure) so `Sys_SendPacket`/`Sys_GetPacket`/`NET_Sleep` see it. Set by
/// `NET_OpenIP` one tier up as it walks the port range.
///
/// Source: `oracle/codemp/unix/unix_net.c:468`
pub fn set_ip_socket(fd: i32) {
    IP_SOCKET.store(fd, Ordering::Relaxed);
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
            Some(e) if e == libc::EWOULDBLOCK || e == libc::ECONNREFUSED => {
                NetRecvResult::WouldBlock
            }
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
    Some(latin1_to_string(line))
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

/// Raven `PORT_ANY` — the "bind to any free port" sentinel `NET_IPSocket`
/// checks; named locally because the qcommon constant is above this tier.
///
/// Source: `oracle/codemp/qcommon/qcommon.h:104`
const PORT_ANY: i32 = -1;

/// Raven `NET_ErrorString` (unix) — `strerror(errno)`. Named here so
/// `net_ip_socket`'s error prints reproduce the oracle's `Com_Printf` text
/// byte-for-byte (the qcommon-tier prints route the returned string through
/// `com_printf`).
///
/// Source: `oracle/codemp/unix/unix_net.c:573-579`
fn net_error_string() -> String {
    let code = std::io::Error::last_os_error().raw_os_error().unwrap_or(0);
    // SAFETY: strerror returns a pointer to a static/thread-local NUL-terminated
    // C string; copied out immediately.
    let p = unsafe { libc::strerror(code) };
    unsafe { CStr::from_ptr(p) }.to_string_lossy().into_owned()
}

/// Raven `NET_IPSocket` (unix) — open a non-blocking, broadcast-capable UDP
/// socket bound to `interface`:`port` (INADDR_ANY unless a real interface is
/// named; `port == PORT_ANY` binds a free port). Returns the fd, or 0 on any
/// failure exactly as the oracle does. `print` carries every `Com_Printf` line
/// (the "Opening IP socket" banner and the `NET_ErrorString` errors) up to the
/// qcommon tier, which cannot be named from here.
///
/// Source: `oracle/codemp/unix/unix_net.c:499-552`
pub fn net_ip_socket(interface: Option<&str>, port: i32, print: &mut dyn FnMut(&str)) -> i32 {
    match interface {
        Some(iface) => print(&format!("Opening IP socket: {iface}:{port}\n")),
        None => print(&format!("Opening IP socket: localhost:{port}\n")),
    }

    // SAFETY: socket() with constant AF/type args; the returned fd is checked.
    let newsocket = unsafe { libc::socket(libc::PF_INET, libc::SOCK_DGRAM, libc::IPPROTO_UDP) };
    if newsocket == -1 {
        print(&format!(
            "ERROR: UDP_OpenSocket: socket: {}",
            net_error_string()
        ));
        return 0;
    }

    // make it non-blocking: ioctl(newsocket, FIONBIO, &qtrue).
    let nb: c_int = 1;
    // SAFETY: FIONBIO reads one int through the pointer; `nb` outlives the call.
    if unsafe { libc::ioctl(newsocket, libc::FIONBIO, &nb as *const c_int) } == -1 {
        print(&format!(
            "ERROR: UDP_OpenSocket: ioctl FIONBIO:{}\n",
            net_error_string()
        ));
        return 0;
    }

    // make it broadcast capable: setsockopt(SOL_SOCKET, SO_BROADCAST, &i=1).
    let i: c_int = 1;
    // SAFETY: SO_BROADCAST reads a sizeof(int) option value through the pointer.
    if unsafe {
        libc::setsockopt(
            newsocket,
            libc::SOL_SOCKET,
            libc::SO_BROADCAST,
            &i as *const c_int as *const c_void,
            core::mem::size_of::<c_int>() as libc::socklen_t,
        )
    } == -1
    {
        print(&format!(
            "ERROR: UDP_OpenSocket: setsockopt SO_BROADCAST:{}\n",
            net_error_string()
        ));
        return 0;
    }

    let mut address: libc::sockaddr_in = unsafe { core::mem::zeroed() };
    // if (!net_interface || !net_interface[0] || !Q_stricmp(..,"localhost"))
    //     INADDR_ANY; else Sys_StringToSockaddr(net_interface).
    let use_any = match interface {
        None => true,
        Some(s) => s.is_empty() || s.eq_ignore_ascii_case("localhost"),
    };
    if use_any {
        address.sin_addr.s_addr = libc::INADDR_ANY;
    } else if let Some(s) = interface {
        // Raven ignores Sys_StringToSockaddr's return: a resolve failure leaves
        // the zeroed sockaddr (INADDR_ANY), so `None` here is that same path.
        if let Some(ip) = net_string_to_ip(s) {
            address.sin_addr.s_addr = u32::from_ne_bytes(ip);
        }
    }

    if port == PORT_ANY {
        address.sin_port = 0;
    } else {
        // htons((short)port).
        address.sin_port = (port as u16).to_be();
    }
    address.sin_family = libc::AF_INET as libc::sa_family_t;

    // SAFETY: `address` is a fully initialized sockaddr_in; the len matches.
    if unsafe {
        libc::bind(
            newsocket,
            &address as *const libc::sockaddr_in as *const libc::sockaddr,
            core::mem::size_of::<libc::sockaddr_in>() as libc::socklen_t,
        )
    } == -1
    {
        print(&format!(
            "ERROR: UDP_OpenSocket: bind: {}\n",
            net_error_string()
        ));
        // SAFETY: closing the fd we just opened.
        unsafe { libc::close(newsocket) };
        return 0;
    }

    newsocket
}

extern "C" {
    /// `gethostbyname(3)`. The `libc` crate does not bind it on every target,
    /// so the native layer declares it; the symbol links from the platform's
    /// libc (libSystem on macOS). `hostent` is `libc`-provided.
    fn gethostbyname(name: *const c_char) -> *mut libc::hostent;
}

/// Raven `NET_GetLocalAddress` (unix, the DEFAULT `gethostbyname` variant) —
/// resolve this host's name to its interface addresses, filling the `localIP[]`
/// table via [`net_add_local_address`] and echoing Raven's `Com_Printf` lines
/// (`Hostname:`/`Alias:`/`IP:`) through `print`. The `#ifdef MACOS_X` twin
/// (`unix_net.c:315-405`, a `getifaddrs`/`SIOCGIFCONF` interface walk) is a
/// deliberately-skipped per-OS variant, not a port gap.
///
/// Source: `oracle/codemp/unix/unix_net.c:408-447`
pub fn net_collect_local_addresses(print: &mut dyn FnMut(&str)) {
    // numIP = 0
    LOCAL_IP.lock().unwrap().clear();

    let mut hostname = [0 as c_char; 256];
    // SAFETY: gethostname writes at most 256 bytes into `hostname`.
    if unsafe { libc::gethostname(hostname.as_mut_ptr(), 256) } == -1 {
        return;
    }

    // SAFETY: `hostname` is a NUL-terminated C string after a successful
    // gethostname; gethostbyname reads it and returns a static hostent or NULL.
    let host_info = unsafe { gethostbyname(hostname.as_ptr()) };
    if host_info.is_null() {
        return;
    }
    // SAFETY: gethostbyname returned a live (static) hostent.
    let hi = unsafe { &*host_info };

    // SAFETY: h_name is a NUL-terminated C string in the hostent.
    print(&format!(
        "Hostname: {}\n",
        unsafe { CStr::from_ptr(hi.h_name) }.to_string_lossy()
    ));

    // while ( (p = hostInfo->h_aliases[n++]) != NULL ) Com_Printf("Alias: ...")
    let mut n: isize = 0;
    loop {
        // SAFETY: h_aliases is a NULL-terminated array of C strings.
        let p = unsafe { *hi.h_aliases.offset(n) };
        if p.is_null() {
            break;
        }
        print(&format!(
            "Alias: {}\n",
            unsafe { CStr::from_ptr(p) }.to_string_lossy()
        ));
        n += 1;
    }

    if hi.h_addrtype != libc::AF_INET {
        return;
    }

    // while ( (p = h_addr_list[numIP]) != NULL && numIP < MAX_IPS )
    let mut idx: isize = 0;
    while (idx as usize) < MAX_IPS {
        // SAFETY: h_addr_list is a NULL-terminated array of h_length-byte
        // (4 for AF_INET) address buffers.
        let p = unsafe { *hi.h_addr_list.offset(idx) };
        if p.is_null() {
            break;
        }
        // localIP[numIP][k] = p[k]; the printed ntohl form equals p[0..4] in
        // order, so store and print the raw network-order bytes directly.
        let bytes = unsafe { core::slice::from_raw_parts(p as *const u8, 4) };
        let ip = [bytes[0], bytes[1], bytes[2], bytes[3]];
        print(&format!("IP: {}.{}.{}.{}\n", ip[0], ip[1], ip[2], ip[3]));
        net_add_local_address(ip);
        idx += 1;
    }
}

/// Raven `NET_Sleep`'s `select` (unix) — block up to `msec` ms until `sock` (or,
/// when stdin is still live, fd 0) is readable. The oracle's `!ip_socket ||
/// !com_dedicated->integer` early-out is a cvar gate owned one tier up; this is
/// only the raw `select` core.
///
/// Source: `oracle/codemp/unix/unix_net.c:591-597`
pub fn net_select_sleep(sock: i32, msec: i32) {
    let mut fdset: libc::fd_set = unsafe { core::mem::zeroed() };
    // SAFETY: FD_ZERO/FD_SET operate on the local, initialized fdset; the fds
    // are valid (sock is the bound socket, 0 is stdin).
    unsafe {
        libc::FD_ZERO(&mut fdset);
        if STDIN_ACTIVE.load(Ordering::Relaxed) {
            libc::FD_SET(0, &mut fdset); // stdin is processed too
        }
        libc::FD_SET(sock, &mut fdset); // network socket
    }
    let mut timeout = libc::timeval {
        tv_sec: (msec / 1000) as libc::time_t,
        tv_usec: ((msec % 1000) * 1000) as libc::suseconds_t,
    };
    // SAFETY: select reads/writes the local fdset and timeout; nfds = sock+1.
    unsafe {
        libc::select(
            sock + 1,
            &mut fdset,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &mut timeout,
        );
    }
}
