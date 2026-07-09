//! `PlatformHost` — the `Sys_*` platform seam (fork-8 ruling).
//!
//! The fork-8 ruling scopes this trait to clock, console I/O, UDP, and file
//! listing (dylib loading already exists in `native_platform`); ruling 33a
//! lands the UDP surface here with faithful Raven signatures over the
//! relocated `mp_qshared` wire types (`netadr_t`/`msg_t`). The method set is
//! exactly the `Sys_*` net/console/clock/listing externals the WinDed link
//! set's non-platform sources call — `Sys_ShowIP` has zero non-platform
//! callers and is not ported (porting-rules §20).
//!
//! Consumers may inject `&mut dyn PlatformHost`, so the trait is
//! dyn-compatible (no generic methods, no by-value `Self` returns).

use mp_qshared::common::mp::qcommon::msg_t::msg_t;
use mp_qshared::common::mp::qcommon::netadr_t::netadr_t;

/// Raven's platform host surface for the dedicated server main loop.
pub trait PlatformHost {
    /// Raven `Sys_Milliseconds` — profiling/timing clock. `base_time` is
    /// Raven's defaulted `baseTime` parameter (returns the epoch-relative time
    /// on the first `false` call, then deltas).
    /// Source: `oracle/codemp/qcommon/qcommon.h:978`
    fn milliseconds(&mut self, base_time: bool) -> i32;

    /// Raven `Sys_Print` — write text to the system console/log verbatim
    /// (`Com_Printf` routes here; ruling 10: byte-identical, no decoration).
    /// Source: `oracle/codemp/qcommon/qcommon.h:970`
    fn sys_print(&mut self, msg: &str);

    /// Raven `Sys_ConsoleInput` — poll the console for a completed command
    /// line; `None` is Raven's `NULL` (nothing typed this frame).
    /// Source: `oracle/codemp/null/win_main.cpp:200`
    fn console_input(&mut self) -> Option<String>;

    /// Raven `Sys_GetPacket` — poll the UDP socket; on a packet, fill
    /// `net_from` and `net_message` (writing into `net_message.data`, capped
    /// at `maxsize`, setting `cursize`) and return `true`; `false` = nothing
    /// pending. Out-params kept 1:1 with the call site (transcription-first,
    /// as for `EngineHost::trace`).
    /// Source: `oracle/codemp/win32/win_local.h:30` (dedicated no-op body:
    /// `oracle/codemp/null/null_net.c:41`)
    fn get_packet(&mut self, net_from: &mut netadr_t, net_message: &mut msg_t) -> bool;

    /// Raven `Sys_SendPacket( int length, const void *data, netadr_t to )` —
    /// send one UDP datagram; `length`+`data` collapse to `&[u8]`. `to` is
    /// borrowed (Raven copies the 20-byte struct by value; `netadr_t` carries
    /// no `Copy`, and the callee only reads it).
    /// Source: `oracle/codemp/qcommon/qcommon.h:1002`
    fn send_packet(&mut self, data: &[u8], to: &netadr_t);

    /// Raven `Sys_StringToAdr` — resolve a host string to an address, writing
    /// `a`; `false` = lookup failed (out-param kept 1:1 with the call site
    /// `net_chan.cpp:636`).
    /// Source: `oracle/codemp/qcommon/qcommon.h:1007`
    fn string_to_adr(&mut self, s: &str, a: &mut netadr_t) -> bool;

    /// Raven `Sys_IsLANAddress( netadr_t adr )` — whether `adr` is loopback or
    /// on a local LAN interface. Borrowed for the same reason as
    /// [`send_packet`]'s `to`.
    /// Source: `oracle/codemp/qcommon/qcommon.h:1010`
    ///
    /// [`send_packet`]: PlatformHost::send_packet
    fn is_lan_address(&mut self, adr: &netadr_t) -> bool;

    /// Raven `Sys_ListFiles` — enumerate `directory` for entries matching
    /// `extension` (or `filter`), returning their names. `Sys_FreeFileList`
    /// collapses into the returned `Vec`'s drop. Enumeration order is pinned
    /// sorted (ruling 9). `want_subs` is Raven's `qboolean wantsubs`.
    /// Source: `oracle/codemp/qcommon/qcommon.h:1025`
    fn list_files(
        &mut self,
        directory: &str,
        extension: &str,
        filter: Option<&str>,
        want_subs: bool,
    ) -> Vec<String>;
}
