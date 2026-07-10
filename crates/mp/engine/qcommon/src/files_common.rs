/Users/milohehmsoth/Developer/Milo/jka-rust/.claude/worktrees/engine-cpass/crates/mp/engine/qcommon/src/files_common.rs:

#![allow(
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals,
    unused_variables,
    unused_mut,
    unused_unsafe,
    clippy::too_many_arguments
)]

//! `files_common.cpp` — platform-independent filesystem core: init/shutdown
//! state, search-path build helpers, filename normalization/comparison, and
//! the write-file/printf-to-file surface.
//!
//! Source: `oracle/codemp/qcommon/files_common.cpp`
//!
//! PORT-NOTE(state): `Common` does not yet carry the `fs_*` filesystem
//! globals (`fs_searchpaths`, `fs_loadStack`, `fs_basepath`, `fs_gamedir`,
//! `fs_gamedirvar`, `lastValidBase`, `lastValidGame`, `initialized`) nor the
//! two `FS_BuildOSPath` overloads' fn-scope statics (`ospath`/`toggle`,
//! fork-3 category 3 — the returned pointer must outlive the call, so they
//! thread as host fields, not owned return values). Referenced verbatim by
//! their exact Raven names per the files_pc.rs precedent; every one is
//! reported in missing_symbols for the finisher to add once the struct
//! lands.
//!
//! PORT-NOTE(rm-types): `RenderModels` (state-receiver type pinned by the
//! engine-fork-discovery preamble's receiver order; rmg-terrain.md/
//! tr-model.md own its real shape) has not landed in this crate yet —
//! imported by its resolved-signature name per the no-stub rule
//! (cm_load.rs precedent); reported as a missing symbol.

use core::ffi::{c_char, c_int};

use mp_host_interface::engine_host::EngineHost;
use mp_qshared::shared::error_parm::errorParm_t;
use mp_qshared::shared::{qboolean, qfalse, qtrue};
use native_types::fileHandle_t;

use crate::collision_world::CollisionWorld;
use crate::common::Common;
use crate::files::files_consts::BASEGAME;
// PORT-NOTE(rm-types): see module doc above.
use crate::render_models::RenderModels;
// PORT-NOTE(MAX_OSPATH): the packet's rosetta row points at
// `mp_engine_client::client::client_connection_t::MAX_OSPATH`, but that
// const is a private, file-local `const MAX_OSPATH: usize = 1024;` inside a
// crate `mp_engine_qcommon` has no Cargo dependency on — not importable as
// printed. Referenced by name anyway per the no-invent rule; reported in
// both missing_symbols and shape_mismatches for the finisher (likely fix:
// promote it to `pub` in a shared qcommon/qshared home and add the edge, or
// land a qcommon-local `MAX_OSPATH` at the same value).
use mp_engine_client::client::client_connection_t::MAX_OSPATH;

// Callees not yet landed in this crate (stub-free order violated by
// concurrent parallel porting — referenced by exact Raven name per the
// cm_load.rs `extern "Rust"` precedent); every one is reported in
// missing_symbols.
extern "Rust" {
    /// `Com_Error` — ruling 1's `longjmp`-as-panic landing pad. The packet
    /// at `qcommon__1592_CM_DeleteCachedMap.md` resolves its full signature
    /// as `(common, cm, sv, rm, rmg, host, code, fmt, ...)`; none of this
    /// file's callers carry `sv`/`rmg`/`cm`/`rm`/`host`, so this narrower
    /// `(common, code, msg)` shape is called instead (shape_mismatches).
    fn Com_Error(common: &mut Common, code: c_int, msg: &str);
    /// `Com_Printf` — narrowed to `(common, msg)` for the same reason as
    /// `Com_Error` above (its full resolved shape also carries `cm`/`rm`/
    /// `host`).
    fn Com_Printf(common: &mut Common, msg: &str);
    /// `Com_StartupVariable` — resolved signature per
    /// `qcommon__1763_Com_StartupVariable.md`.
    fn Com_StartupVariable(
        common: &mut Common,
        cm: &mut CollisionWorld,
        rm: &mut RenderModels,
        host: &mut dyn EngineHost,
        r#match: *const c_char,
    );
    /// `FS_Write` — resolved signature per `qcommon__1592_CM_DeleteCachedMap.md`.
    fn FS_Write(common: &mut Common, buffer: *const (), len: c_int, h: fileHandle_t) -> c_int;
    /// `FS_FOpenFileWrite` — resolved signature per
    /// `qcommon__1592_CM_DeleteCachedMap.md`.
    fn FS_FOpenFileWrite(common: &mut Common, filename: *const c_char) -> fileHandle_t;
    /// `FS_FCloseFile` — resolved signature per
    /// `qcommon__1592_CM_DeleteCachedMap.md`.
    fn FS_FCloseFile(common: &mut Common, f: fileHandle_t);
    /// `FS_ReadFile` — resolved signature per
    /// `qcommon__1592_CM_DeleteCachedMap.md`.
    fn FS_ReadFile(
        common: &mut Common,
        cm: &mut CollisionWorld,
        rm: &mut RenderModels,
        host: &mut dyn EngineHost,
        qpath: *const c_char,
        buffer: *mut *mut (),
    ) -> c_int;
    /// `FS_SetRestrictions` — resolved signature per
    /// `qcommon__1592_CM_DeleteCachedMap.md`.
    fn FS_SetRestrictions(
        common: &mut Common,
        cm: &mut CollisionWorld,
        rm: &mut RenderModels,
        host: &mut dyn EngineHost,
    );
    /// `FS_Startup` — resolved signature per
    /// `qcommon__1592_CM_DeleteCachedMap.md`.
    fn FS_Startup(
        common: &mut Common,
        cm: &mut CollisionWorld,
        rm: &mut RenderModels,
        host: &mut dyn EngineHost,
        gameName: *const c_char,
    );
    // PORT-NOTE(q_math-reach): `Com_sprintf`/`Q_strncpyz` (q_shared/q_math
    // primitives) are ported in `mp_game`, a tier above this crate's
    // dependency graph (cm_load.rs precedent) — not reachable here.
    // Referenced by their exact Raven names; reported as missing symbols.
    // Narrowed to a pre-formatted `&str` (Rust has no safe C-variadic fn
    // definitions) matching the net_chan.rs `Com_sprintf` call precedent.
    fn Com_sprintf(dest: *mut c_char, size: c_int, fmt: &str);
    fn Q_strncpyz(dest: *mut c_char, src: *const c_char, destsize: c_int);
}

/// Raven `FS_Initialized`.
///
/// Source: `oracle/codemp/qcommon/files_common.cpp:243-245`
pub fn FS_Initialized(common: &mut Common) -> qboolean {
    // PORT-NOTE(state): see module doc — `fs_searchpaths` referenced
    // verbatim.
    if !common.fs_searchpaths.is_null() {
        qtrue
    } else {
        qfalse
    }
}

/// Raven `FS_LoadStack`.
///
/// Source: `oracle/codemp/qcommon/files_common.cpp:253-256`
pub fn FS_LoadStack(common: &mut Common) -> c_int {
    // PORT-NOTE(state): see module doc — `fs_loadStack` referenced verbatim.
    common.fs_loadStack
}

/// Raven `FS_ReplaceSeparators`.
///
/// Source: `oracle/codemp/qcommon/files_common.cpp:277-285`
pub fn FS_ReplaceSeparators(path: *mut c_char) {
    // PORT-NOTE(PATH_SEP): `PATH_SEP` is a platform macro with no rosetta
    // row (seam-supplied, files_pc.rs precedent) — normalized here as `/`
    // per ruling 8's unix-semantics stance.
    unsafe {
        let mut s = path;
        while *s != 0 {
            if *s == b'/' as c_char || *s == b'\\' as c_char {
                *s = b'/' as c_char;
            }
            s = s.add(1);
        }
    }
}

/// Raven `FS_FilenameCompare`.
///
/// Source: `oracle/codemp/qcommon/files_common.cpp:345-372`
pub fn FS_FilenameCompare(s1: *const c_char, s2: *const c_char) -> qboolean {
    unsafe {
        let mut p1 = s1;
        let mut p2 = s2;
        loop {
            let mut c1 = *p1 as c_int;
            let mut c2 = *p2 as c_int;
            p1 = p1.add(1);
            p2 = p2.add(1);

            if c1 >= b'a' as c_int && c1 <= b'z' as c_int {
                c1 -= b'a' as c_int - b'A' as c_int;
            }
            if c2 >= b'a' as c_int && c2 <= b'z' as c_int {
                c2 -= b'a' as c_int - b'A' as c_int;
            }

            if c1 == b'\\' as c_int || c1 == b':' as c_int {
                c1 = b'/' as c_int;
            }
            if c2 == b'\\' as c_int || c2 == b':' as c_int {
                c2 = b'/' as c_int;
            }

            if c1 != c2 {
                return -1; // strings not equal
            }
            if c1 == 0 {
                break;
            }
        }
        0 // strings are equal
    }
}

/// Raven `FS_BuildOSPath` (single-`qpath` overload).
///
/// Source: `oracle/codemp/qcommon/files_common.cpp:294-315`
pub fn FS_BuildOSPath(common: &mut Common, mut qpath: *const c_char) -> *mut c_char {
    let mut temp: [c_char; 1024] = [0; 1024];
    // PORT-NOTE(state): see module doc — `ospath`/`toggle` (fork-3 category
    // 3: the returned pointer must outlive the call) threaded as
    // `common.fs_build_os_path_buf`/`common.fs_build_os_path_toggle`.
    common.fs_build_os_path_toggle ^= 1; // flip-flop to allow two returns without clash

    unsafe {
        // Fix for filenames that are given to FS with a leading "/" (/botfiles/Foo)
        if *qpath == b'\\' as c_char || *qpath == b'/' as c_char {
            qpath = qpath.add(1);
        }

        // FIXME VVFIXME Holy crap this is wrong.
        //	Com_sprintf( temp, sizeof(temp), "/%s/%s", fs_gamedirvar->string, qpath );
        let qpath_str = core::ffi::CStr::from_ptr(qpath).to_string_lossy();
        Com_sprintf(
            temp.as_mut_ptr(),
            temp.len() as c_int,
            &format!("/{}/{}", "base", qpath_str),
        );

        FS_ReplaceSeparators(temp.as_mut_ptr());

        let toggle = common.fs_build_os_path_toggle as usize;
        let base_str = core::ffi::CStr::from_ptr((*common.fs_basepath).string).to_string_lossy();
        let temp_str = core::ffi::CStr::from_ptr(temp.as_ptr()).to_string_lossy();
        let ospath_ptr = common.fs_build_os_path_buf[toggle].as_mut_ptr();
        Com_sprintf(
            ospath_ptr,
            common.fs_build_os_path_buf[toggle].len() as c_int,
            &format!("{}{}", base_str, temp_str),
        );

        ospath_ptr
    }
}

/// Raven `FS_BuildOSPath` (`base`/`game`/`qpath` overload).
///
/// PORT-NOTE(overload): Raven overloads `FS_BuildOSPath` by arity (C++
/// name mangling distinguishes them); Rust has no fn overloading, so this
/// 4-arg overload is named `FS_BuildOSPath4` (shape_mismatches).
///
/// Source: `oracle/codemp/qcommon/files_common.cpp:317-336`
pub fn FS_BuildOSPath4(
    common: &mut Common,
    base: *const c_char,
    mut game: *const c_char,
    qpath: *const c_char,
) -> *mut c_char {
    let mut temp: [c_char; 1024] = [0; 1024];
    // PORT-NOTE(state): see module doc — this overload's own `ospath[4]`/
    // `toggle` are a SEPARATE fn-scope-static pair from the single-`qpath`
    // overload above (Raven gives each overload its own statics); threaded
    // as `common.fs_build_os_path4_buf`/`common.fs_build_os_path4_toggle`.
    common.fs_build_os_path4_toggle = (common.fs_build_os_path4_toggle + 1) & 3; // allows four returns without clash (increased from 2 during fs_copyfiles 2 enhancement)

    unsafe {
        if game.is_null() || *game == 0 {
            // PORT-NOTE(state): `fs_gamedir` referenced verbatim (see module doc).
            game = common.fs_gamedir.as_ptr();
        }

        let game_str = core::ffi::CStr::from_ptr(game).to_string_lossy();
        let qpath_str = core::ffi::CStr::from_ptr(qpath).to_string_lossy();
        Com_sprintf(
            temp.as_mut_ptr(),
            temp.len() as c_int,
            &format!("/{}/{}", game_str, qpath_str),
        );
        FS_ReplaceSeparators(temp.as_mut_ptr());

        let toggle = common.fs_build_os_path4_toggle as usize;
        let base_str = core::ffi::CStr::from_ptr(base).to_string_lossy();
        let temp_str = core::ffi::CStr::from_ptr(temp.as_ptr()).to_string_lossy();
        let ospath_ptr = common.fs_build_os_path4_buf[toggle].as_mut_ptr();
        Com_sprintf(
            ospath_ptr,
            common.fs_build_os_path4_buf[toggle].len() as c_int,
            &format!("{}{}", base_str, temp_str),
        );

        ospath_ptr
    }
}

/// Raven `FS_CheckInit`.
///
/// Source: `oracle/codemp/qcommon/files_common.cpp:229-235`
pub fn FS_CheckInit(common: &mut Common) {
    // PORT-NOTE(state): `initialized` referenced verbatim (see module doc).
    if common.initialized == qfalse {
        unsafe {
            Com_Error(
                common,
                errorParm_t::ERR_FATAL as c_int,
                "Filesystem call made without initialization\n",
            );
        }
    }
}

/// Raven `FS_Printf`.
///
/// Source: `oracle/codemp/qcommon/files_common.cpp:375-384`
// PORT-NOTE(variadic): Raven's C `... ` variadic collapses to a
// pre-formatted `msg: &str` — Rust has no safe C-variadic fn definitions.
// `vsprintf`/`strlen` (the externals the packet lists) are subsumed by the
// caller having already formatted `msg`; `FS_Write` receives its bytes
// exactly as Raven's vsprintf-then-FS_Write body does.
pub fn FS_Printf(common: &mut Common, h: fileHandle_t, msg: &str) {
    let bytes = msg.as_bytes();
    unsafe {
        FS_Write(common, bytes.as_ptr() as *const (), bytes.len() as c_int, h);
    }
}

/// Raven `FS_WriteFile`.
///
/// Source: `oracle/codemp/qcommon/files_common.cpp:401-421`
pub fn FS_WriteFile(common: &mut Common, qpath: *const c_char, buffer: *const (), size: c_int) {
    unsafe {
        // PORT-NOTE(state): `fs_searchpaths` referenced verbatim (see module doc).
        if common.fs_searchpaths.is_null() {
            Com_Error(
                common,
                errorParm_t::ERR_FATAL as c_int,
                "Filesystem call made without initialization\n",
            );
        }

        if qpath.is_null() || buffer.is_null() {
            Com_Error(
                common,
                errorParm_t::ERR_FATAL as c_int,
                "FS_WriteFile: NULL parameter",
            );
        }

        let f = FS_FOpenFileWrite(common, qpath);
        if f == 0 {
            let qpath_str = core::ffi::CStr::from_ptr(qpath).to_string_lossy();
            Com_Printf(common, &format!("Failed to open {}\n", qpath_str));
            return;
        }

        FS_Write(common, buffer, size, f);

        FS_FCloseFile(common, f);
    }
}

/// Raven `FS_InitFilesystem`.
///
/// Source: `oracle/codemp/qcommon/files_common.cpp:480-511`
pub fn FS_InitFilesystem(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
) {
    unsafe {
        // allow command line parms to override our defaults
        // we have to specially handle this, because normal command
        // line variable sets don't happen until after the filesystem
        // has already been initialized
        Com_StartupVariable(common, cm, rm, host, c"fs_cdpath".as_ptr());
        Com_StartupVariable(common, cm, rm, host, c"fs_basepath".as_ptr());
        Com_StartupVariable(common, cm, rm, host, c"fs_homepath".as_ptr());
        Com_StartupVariable(common, cm, rm, host, c"fs_game".as_ptr());
        Com_StartupVariable(common, cm, rm, host, c"fs_copyfiles".as_ptr());
        Com_StartupVariable(common, cm, rm, host, c"fs_restrict".as_ptr());

        // try to start up normally
        let basegame = std::ffi::CString::new(BASEGAME).unwrap();
        FS_Startup(common, cm, rm, host, basegame.as_ptr());
        // PORT-NOTE(state): `initialized` referenced verbatim (see module doc).
        common.initialized = qtrue;

        // see if we are going to allow add-ons
        FS_SetRestrictions(common, cm, rm, host);

        // if we can't find default.cfg, assume that the paths are
        // busted and error out now, rather than getting an unreadable
        // graphics screen when the font fails to load
        let mut buffer: *mut () = core::ptr::null_mut();
        if FS_ReadFile(
            common,
            cm,
            rm,
            host,
            c"mpdefault.cfg".as_ptr(),
            &mut buffer as *mut *mut (),
        ) <= 0
        {
            // bk001208 - SafeMode see below, FIXME?
            // PORT-NOTE(shape): Com_Error's full resolved signature
            // (qcommon__1592_CM_DeleteCachedMap.md) also carries `sv`/`rmg`;
            // not available here (shape_mismatches).
            Com_Error(
                common,
                errorParm_t::ERR_FATAL as c_int,
                "Couldn't load mpdefault.cfg",
            );
        }

        // PORT-NOTE(state): `lastValidBase`/`lastValidGame`/`fs_basepath`/
        // `fs_gamedirvar` referenced verbatim (see module doc).
        Q_strncpyz(
            common.lastValidBase.as_mut_ptr(),
            (*common.fs_basepath).string,
            common.lastValidBase.len() as c_int,
        );
        Q_strncpyz(
            common.lastValidGame.as_mut_ptr(),
            (*common.fs_gamedirvar).string,
            common.lastValidGame.len() as c_int,
        );

        // bk001208 - SafeMode see below, FIXME?
    }
}
