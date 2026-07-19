//! Engine-tier `Sys_*` wrappers whose oracle (unix) bodies reach FS/cvar state.
//!
//! `Sys_LoadDll`, `Sys_StreamedRead`, and `Sys_StreamSeek` live in
//! `oracle/codemp/unix/unix_main.c`, but their bodies call engine-tier state
//! (`Cvar_VariableString`/`FS_BuildOSPath`/`FS_Read`/`FS_Seek`) that sits ABOVE
//! `native_platform`, so they are hosted here in `qcommon` (per §B state is
//! threaded, not reached). Behavior source: `oracle/codemp/unix/unix_main.c`.
//!
//! The `Sys_QueEvent`/`Sys_GetEvent` event pump also lives here: its ring is
//! `Common.sys_events` and it allocates event payloads through `Z_Malloc`, both
//! engine-tier, while the console/packet polling is delegated to
//! `native_platform`/`sys_net`.

#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_long, c_void};

use mp_qshared::common::mp::qcommon::msg_t::msg_t;
use mp_qshared::common::mp::qcommon::netadr_t::netadr_t;
use mp_qshared::shared::error_parm::errorParm_t;
use mp_qshared::shared::qfalse;
use native_types::fileHandle_t;

use crate::common::engine_host_view::EngineHostView;
use crate::common::{com_error, com_printf, Common, MASK_QUED_EVENTS, MAX_QUED_EVENTS};
use crate::cvar_fns::{Cvar_Set, Cvar_VariableString};
use crate::files_common::{FS_BuildOSPath4, FS_Read};
use crate::files_pc::FS_Seek;
use crate::msg::MSG_Init;
use crate::qcommon::net_limits::MAX_MSGLEN;
use crate::qcommon::sys_event_t::sysEvent_t;
use crate::qcommon::sys_event_type_t::sysEventType_t;
use crate::sys_net::Sys_GetPacket;
use crate::timing::sys_milliseconds;
use crate::z_memman_pc::{Z_Free, Z_Malloc};
use mp_qshared::common::mp::qcommon::tags::memtag_t;

/// The module `dllEntry` C entry: `void (*)(int (*syscallptr)(int, ...))`.
/// The syscall arg is our SEAM-D11 `isize`-variadic trampoline; a fn pointer is
/// one register regardless of its declared prototype, so passing the trampoline
/// through the module's `int(*)(int,...)` slot is ABI-identical.
/// Source: `oracle/codemp/unix/unix_main.c:328`
type DllEntryFn = unsafe extern "C" fn(Option<unsafe extern "C-unwind" fn(isize, ...) -> isize>);

/// Fetch `dlerror()` as an owned string for the verbose Com_Printf messages
/// (empty when `dlerror()` returns NULL).
fn dlerror_string() -> String {
    let err = unsafe { libc::dlerror() };
    if err.is_null() {
        String::new()
    } else {
        unsafe {
            core::ffi::CStr::from_ptr(err)
                .to_string_lossy()
                .into_owned()
        }
    }
}

/// Raven `Sys_LoadDll` (unix) — load a native game dll instead of a QVM: resolve
/// the module OS path (`fs_basepath`/`fs_game`, then `fs_cdpath`), `dlopen` it,
/// `dlsym` `dllEntry`/`vmMain`, hand `dllEntry` the syscall trampoline.
///
/// Source: `oracle/codemp/unix/unix_main.c:323-447`
///
/// # Safety
/// `name` must be a valid NUL-terminated C string; `entryPoint`/`systemcalls`
/// cross the module ABI seam (porting-rules §D11 exemption).
pub unsafe fn Sys_LoadDll(
    common: &mut Common,
    name: *const c_char,
    entryPoint: &mut Option<native_platform::entrypoints::RawVmMain>,
    systemcalls: Option<unsafe extern "C-unwind" fn(isize, ...) -> isize>,
) -> *mut c_void {
    // Raven fills `curpath` via `getcwd` only for the dead `#if 0` install-dir
    // path; dropped here as it has no live effect.

    // Raven's arch suffix: this unix target resolves the `__i386__` release
    // branch (`%si386.so`) — the only unix arch the oracle defines and the name
    // CI ships (`jampgamei386.so`).
    // Source: `oracle/codemp/unix/unix_main.c:342-356`
    let name_str = core::ffi::CStr::from_ptr(name)
        .to_string_lossy()
        .into_owned();
    let fname = std::ffi::CString::new(format!("{name_str}i386.so")).unwrap_or_default();

    // bk001129 - was RTLD_LAZY: `#define Q_RTLD RTLD_NOW`.
    let q_rtld = libc::RTLD_NOW;

    let basepath = std::ffi::CString::new(Cvar_VariableString(common, "fs_basepath")).unwrap();
    let cdpath = std::ffi::CString::new(Cvar_VariableString(common, "fs_cdpath")).unwrap();
    let gamedir = std::ffi::CString::new(Cvar_VariableString(common, "fs_game")).unwrap();

    let mut path = FS_BuildOSPath4(common, basepath.as_ptr(), gamedir.as_ptr(), fname.as_ptr());
    // bk001206 - verbose
    let path_str = core::ffi::CStr::from_ptr(path)
        .to_string_lossy()
        .into_owned();
    com_printf(common, &format!("Sys_LoadDll({path_str})... \n"));

    // bk001129 - from cvs1.17 (mkv), was fname not fn
    let mut lib_handle = libc::dlopen(path, q_rtld);

    if lib_handle.is_null() {
        if !cdpath.to_bytes().is_empty() {
            // bk001206 - report any problem
            com_printf(
                common,
                &format!("Sys_LoadDll({path_str}) failed: \"{}\"\n", dlerror_string()),
            );

            path = FS_BuildOSPath4(common, cdpath.as_ptr(), gamedir.as_ptr(), fname.as_ptr());
            lib_handle = libc::dlopen(path, q_rtld);
            let path2 = core::ffi::CStr::from_ptr(path)
                .to_string_lossy()
                .into_owned();
            if lib_handle.is_null() {
                // bk001206 - report any problem
                com_printf(
                    common,
                    &format!("Sys_LoadDll({path2}) failed: \"{}\"\n", dlerror_string()),
                );
            } else {
                com_printf(common, &format!("Sys_LoadDll({path2}): succeeded ...\n"));
            }
        } else {
            com_printf(common, &format!("Sys_LoadDll({path_str}): succeeded ...\n"));
        }

        if lib_handle.is_null() {
            // NDEBUG (retail release) branch: abort on failure.
            com_error(
                errorParm_t::ERR_FATAL,
                format!("Sys_LoadDll({name_str}) failed dlopen() completely!\n"),
            );
        }
    }

    let dll_entry = libc::dlsym(lib_handle, c"dllEntry".as_ptr());
    if dll_entry.is_null() {
        let err = dlerror_string();
        com_printf(
            common,
            &format!("Sys_LoadDLL({name_str}) failed dlsym(dllEntry): \"{err}\" ! \n"),
        );
    }

    let vm_main = libc::dlsym(lib_handle, c"vmMain".as_ptr());
    *entryPoint = if vm_main.is_null() {
        None
    } else {
        Some(core::mem::transmute::<
            *mut c_void,
            native_platform::entrypoints::RawVmMain,
        >(vm_main))
    };
    if entryPoint.is_none() || dll_entry.is_null() {
        // NDEBUG (retail release) branch: abort on failure.
        com_error(
            errorParm_t::ERR_FATAL,
            format!(
                "Sys_LoadDll({name_str}) failed dlsym(vmMain): \"{}\" !\n",
                dlerror_string()
            ),
        );
    }

    // bk001212
    com_printf(
        common,
        &format!("Sys_LoadDll({name_str}) found **vmMain** at  {vm_main:p}  \n"),
    );
    let dll_entry: DllEntryFn = core::mem::transmute::<*mut c_void, DllEntryFn>(dll_entry);
    dll_entry(systemcalls);
    com_printf(common, &format!("Sys_LoadDll({name_str}) succeeded!\n"));
    lib_handle
}

/// Raven `Sys_StreamedRead` (unix) — the `#if 1` non-async build's thin
/// `FS_Read( buffer, size * count, f )` wrapper.
///
/// Source: `oracle/codemp/unix/unix_main.c:758-760`
pub fn Sys_StreamedRead(
    common: &mut Common,
    buffer: *mut (),
    size: c_int,
    count: c_int,
    f: fileHandle_t,
) -> c_int {
    FS_Read(common, buffer, size * count, f)
}

/// Raven `Sys_StreamSeek` (unix) — the `#if 1` non-async build's thin
/// `FS_Seek( f, offset, origin )` wrapper.
///
/// Source: `oracle/codemp/unix/unix_main.c:762-764`
///
/// Raven's `int offset` param is widened to `c_long` to match the `FS_Seek` seam
/// (the caller already holds a `c_long` offset).
pub fn Sys_StreamSeek(view: &mut EngineHostView, f: fileHandle_t, offset: c_long, origin: c_int) {
    FS_Seek(view, f, offset, origin);
}

/// Raven `Sys_QueEvent` (unix) — push one event onto the 256-entry
/// `eventQue`/`eventHead`/`eventTail` ring (`Common.sys_events`), warning and
/// discarding (freeing any payload) the oldest on overflow.
///
/// Source: `oracle/codemp/unix/unix_main.c:960-988`
///
/// # Safety
/// `ptr` (when non-null) must be a `Z_Malloc`'d block the event owner later
/// frees, per Raven's contract.
pub unsafe fn Sys_QueEvent(
    common: &mut Common,
    time: c_int,
    r#type: sysEventType_t,
    value: c_int,
    value2: c_int,
    ptrLength: c_int,
    ptr: *mut c_void,
) {
    let idx = (common.sys_events.head as usize) & MASK_QUED_EVENTS;

    // bk000305 - was missing
    if common.sys_events.head - common.sys_events.tail >= MAX_QUED_EVENTS as c_int {
        com_printf(common, "Sys_QueEvent: overflow\n");
        // we are discarding an event, but don't leak memory
        let evPtr = common.sys_events.que[idx].evPtr;
        if !evPtr.is_null() {
            Z_Free(common, evPtr as *mut ());
        }
        common.sys_events.tail += 1;
    }

    common.sys_events.head += 1;

    let time = if time == 0 {
        sys_milliseconds(common)
    } else {
        time
    };

    let ev = &mut common.sys_events.que[idx];
    ev.evTime = time;
    ev.evType = r#type;
    ev.evValue = value;
    ev.evValue2 = value2;
    ev.evPtrLength = ptrLength;
    ev.evPtr = ptr;
}

/// Raven `Sys_GetEvent` (unix) — drain the `eventQue`, else pump the system:
/// `Sys_ConsoleInput` (queued as `SE_CONSOLE`) and `Sys_GetPacket` (queued as
/// `SE_PACKET`), returning the next queued event or an empty timestamped one.
///
/// `Sys_SendKeyEvents`/`IN_Frame` are X11 window/input pumps with no window in
/// the headless module host, so they queue nothing and are elided.
///
/// Source: `oracle/codemp/unix/unix_main.c:995-1051`
pub fn Sys_GetEvent(view: &mut EngineHostView) -> sysEvent_t {
    // return if we have data
    if view.common.sys_events.head > view.common.sys_events.tail {
        view.common.sys_events.tail += 1;
        return view.common.sys_events.que
            [((view.common.sys_events.tail - 1) as usize) & MASK_QUED_EVENTS];
    }

    // check for console commands
    if let Some(s) = native_platform::net::sys_console_input() {
        let bytes = s.as_bytes();
        let len = bytes.len() as c_int + 1;
        let b = Z_Malloc(view, len, memtag_t::TAG_EVENT, qfalse, 4) as *mut u8;
        // strcpy( b, s ): copy the line and NUL-terminate.
        unsafe {
            core::ptr::copy_nonoverlapping(bytes.as_ptr(), b, bytes.len());
            *b.add(bytes.len()) = 0;
            Sys_QueEvent(
                view.common,
                0,
                sysEventType_t::SE_CONSOLE,
                0,
                0,
                len,
                b as *mut c_void,
            );
        }
    }

    // check for network packets
    let mut netmsg: msg_t = unsafe { core::mem::zeroed() };
    let pkt = view.common.sys_packetReceived.as_mut_ptr();
    MSG_Init(view, &mut netmsg, pkt, MAX_MSGLEN as c_int);
    let mut adr: netadr_t = unsafe { core::mem::zeroed() };
    if Sys_GetPacket(view.common, &mut adr, &mut netmsg) {
        // copy out to a seperate buffer for qeueing
        let len = core::mem::size_of::<netadr_t>() as c_int + netmsg.cursize;
        let buf = Z_Malloc(view, len, memtag_t::TAG_EVENT, qfalse, 4) as *mut netadr_t;
        unsafe {
            *buf = adr;
            core::ptr::copy_nonoverlapping(
                netmsg.data,
                buf.add(1) as *mut u8,
                netmsg.cursize as usize,
            );
            Sys_QueEvent(
                view.common,
                0,
                sysEventType_t::SE_PACKET,
                0,
                0,
                len,
                buf as *mut c_void,
            );
        }
    }

    // return if we have data
    if view.common.sys_events.head > view.common.sys_events.tail {
        view.common.sys_events.tail += 1;
        return view.common.sys_events.que
            [((view.common.sys_events.tail - 1) as usize) & MASK_QUED_EVENTS];
    }

    // create an empty event to return
    let mut ev: sysEvent_t = unsafe { core::mem::zeroed() };
    ev.evTime = sys_milliseconds(view.common);
    ev
}

/// Raven's unix `arch` cvar value (`Sys_Init`'s `#if` chain): only the linux
/// i386 build has a specific string; every other target this port builds for
/// falls to Raven's own `#else` arms (`"linux unknown"` / `"unknown"`).
///
/// Source: `oracle/codemp/unix/unix_main.c:164-200`
#[cfg(all(target_os = "linux", target_arch = "x86"))]
const SYS_ARCH: &str = "linux i386";
#[cfg(all(target_os = "linux", not(target_arch = "x86")))]
const SYS_ARCH: &str = "linux unknown";
#[cfg(not(target_os = "linux"))]
const SYS_ARCH: &str = "unknown";

/// Raven unix `Sys_Init` — the `arch`/`username` cvar writes. The input-layer
/// tail is client-shell slice work:
//TODO: Port Sys_In_Restart_f + IN_Init (client-shell slice)
// Source: oracle/codemp/unix/unix_main.c:162,204
///
/// Source: `oracle/codemp/unix/unix_main.c:160-206`
pub fn Sys_Init(view: &mut EngineHostView) {
    Cvar_Set(view, "arch", SYS_ARCH);
    let username = native_platform::sys_main::Sys_GetCurrentUser();
    Cvar_Set(view, "username", &username);
}
