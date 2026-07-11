//! Engine-tier `Sys_*` wrappers whose oracle (unix) bodies reach FS/cvar state.
//!
//! `Sys_LoadDll`, `Sys_StreamedRead`, and `Sys_StreamSeek` live in
//! `oracle/codemp/unix/unix_main.c`, but their bodies call engine-tier state
//! (`Cvar_VariableString`/`FS_BuildOSPath`/`FS_Read`/`FS_Seek`) that sits ABOVE
//! `native_platform`, so they are hosted here in `qcommon` (per §B state is
//! threaded, not reached). Behavior source: `oracle/codemp/unix/unix_main.c`.

#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_long, c_void};

use mp_host_interface::engine_host::EngineHost;
use mp_qshared::shared::error_parm::errorParm_t;
use native_types::fileHandle_t;

use crate::cm_load::RenderModels;
use crate::collision_world::CollisionWorld;
use crate::common::{com_error, com_printf, Common};
use crate::cvar_fns::Cvar_VariableString;
use crate::files_common::{FS_BuildOSPath4, FS_Read};
use crate::files_pc::FS_Seek;

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
        unsafe { core::ffi::CStr::from_ptr(err).to_string_lossy().into_owned() }
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
    entryPoint: &mut Option<unsafe extern "C" fn(i32, ...) -> i32>,
    systemcalls: Option<unsafe extern "C-unwind" fn(isize, ...) -> isize>,
) -> *mut c_void {
    // Raven fills `curpath` via `getcwd` only for the dead `#if 0` install-dir
    // path; dropped here as it has no live effect.

    // Raven's arch suffix: this unix target resolves the `__i386__` release
    // branch (`%si386.so`) — the only unix arch the oracle defines and the name
    // CI ships (`jampgamei386.so`).
    // Source: `oracle/codemp/unix/unix_main.c:342-356`
    let name_str = core::ffi::CStr::from_ptr(name).to_string_lossy().into_owned();
    let fname = std::ffi::CString::new(format!("{name_str}i386.so")).unwrap_or_default();

    // bk001129 - was RTLD_LAZY: `#define Q_RTLD RTLD_NOW`.
    let q_rtld = libc::RTLD_NOW;

    let basepath = Cvar_VariableString(common, c"fs_basepath".as_ptr());
    let cdpath = Cvar_VariableString(common, c"fs_cdpath".as_ptr());
    let gamedir = Cvar_VariableString(common, c"fs_game".as_ptr());

    let mut path = FS_BuildOSPath4(common, basepath, gamedir, fname.as_ptr());
    // bk001206 - verbose
    let path_str = core::ffi::CStr::from_ptr(path).to_string_lossy().into_owned();
    com_printf(common, &format!("Sys_LoadDll({path_str})... \n"));

    // bk001129 - from cvs1.17 (mkv), was fname not fn
    let mut lib_handle = libc::dlopen(path, q_rtld);

    if lib_handle.is_null() {
        if *cdpath != 0 {
            // bk001206 - report any problem
            com_printf(
                common,
                &format!("Sys_LoadDll({path_str}) failed: \"{}\"\n", dlerror_string()),
            );

            path = FS_BuildOSPath4(common, cdpath, gamedir, fname.as_ptr());
            lib_handle = libc::dlopen(path, q_rtld);
            let path2 = core::ffi::CStr::from_ptr(path).to_string_lossy().into_owned();
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
            unsafe extern "C" fn(i32, ...) -> i32,
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
pub fn Sys_StreamSeek(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    f: fileHandle_t,
    offset: c_long,
    origin: c_int,
) {
    FS_Seek(common, cm, rm, host, f, offset, origin);
}
