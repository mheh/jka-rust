//! Shared mock-engine + lifecycle-drive machinery for the ABI smoke tests.
//!
//! Both `abi_smoke.rs` (our Rust `jampgame` cdylib) and `oracle_smoke.rs`
//! (Raven's UNMODIFIED jampgame built by `tools/referee-oracle/build.sh`) drive a
//! loadable module through the exact engine/module contract — `dllEntry(syscall)`
//! then `vmMain(command, arg0..arg11)` — against ONE mock engine, asserting the
//! module survives `GAME_INIT` -> warm-up `GAME_RUN_FRAME`s -> `CLIENT_CONNECT` ->
//! `CLIENT_BEGIN` -> connected frames -> `CLIENT_COMMAND` -> `CLIENT_DISCONNECT`
//! -> `GAME_SHUTDOWN` and produces the structural side effects the engine relies
//! on. Raven's DLL calling our mock through the SEAM-D11 trampoline is the
//! referee acceptance proof.
//!
//! Transport wiring (all real, no shims of our own):
//!   * the module is loaded through `native_platform::module_loader::sys_load_dll`
//!     — the ported libloading loader; it resolves and invokes `dllEntry`, handing
//!     the module the engine syscall trampoline;
//!   * the syscall trampoline handed over is `mp_engine_qcommon`'s real C-variadic
//!     `game_syscall_trampoline` (SEAM-D11); the module's `trap_*` wrappers call
//!     it variadically and it forwards a flat 16-word frame to
//!     `game_syscall_trampoline_words`, which reads the armed `EngineSlot` and
//!     dispatches into our fixed-arity `mock_syscall`;
//!   * `arm_game_slot` injects `(ctx, mock_syscall)` into that slot before the
//!     first `vmMain`.
//!
//! The oracle DLL's `vmMain`/syscall use Raven's original 32-bit `int` widths
//! while the loader's `AbiWord` is `isize`; on the arm64 host this is benign for
//! the small non-negative lifecycle values (writes to a 32-bit result register
//! zero-extend), and the variadic syscall path is width-agnostic (pointers pass
//! at their natural width). So the identical trampoline drives both modules.

#![allow(non_snake_case)]
#![allow(dead_code)]

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::ffi::{c_char, c_int, c_void, CStr, CString};
use std::path::PathBuf;

use mp_abi::game::exports::MpGameExport;
use mp_abi::game::imports::MpGameImport;
use mp_engine_qcommon::vm::{arm_game_slot, game_syscall_trampoline};
use mp_qshared::shared::cvar::vmCvar_t;
use native_platform::entrypoints::{AbiWord, RawSyscall, RawVmMain};
use native_platform::module_loader::{
    sys_load_dll, ModuleNaming, ModuleSearchPolicy, SearchStep,
};

// ---------------------------------------------------------------------------
// Import-number constants (wire values). Used as `match` patterns in the mock.
// ---------------------------------------------------------------------------

const G_PRINT: isize = MpGameImport::G_PRINT as isize;
const G_ERROR: isize = MpGameImport::G_ERROR as isize;
const G_MILLISECONDS: isize = MpGameImport::G_MILLISECONDS as isize;
const G_CVAR_REGISTER: isize = MpGameImport::G_CVAR_REGISTER as isize;
const G_CVAR_UPDATE: isize = MpGameImport::G_CVAR_UPDATE as isize;
const G_CVAR_SET: isize = MpGameImport::G_CVAR_SET as isize;
const G_CVAR_VARIABLE_INTEGER_VALUE: isize = MpGameImport::G_CVAR_VARIABLE_INTEGER_VALUE as isize;
const G_CVAR_VARIABLE_STRING_BUFFER: isize = MpGameImport::G_CVAR_VARIABLE_STRING_BUFFER as isize;
const G_ARGC: isize = MpGameImport::G_ARGC as isize;
const G_ARGV: isize = MpGameImport::G_ARGV as isize;
const G_FS_FOPEN_FILE: isize = MpGameImport::G_FS_FOPEN_FILE as isize;
const G_FS_READ: isize = MpGameImport::G_FS_READ as isize;
const G_FS_WRITE: isize = MpGameImport::G_FS_WRITE as isize;
const G_FS_FCLOSE_FILE: isize = MpGameImport::G_FS_FCLOSE_FILE as isize;
const G_FS_GETFILELIST: isize = MpGameImport::G_FS_GETFILELIST as isize;
const G_GET_SERVERINFO: isize = MpGameImport::G_GET_SERVERINFO as isize;
const G_GET_USERINFO: isize = MpGameImport::G_GET_USERINFO as isize;
const G_SET_CONFIGSTRING: isize = MpGameImport::G_SET_CONFIGSTRING as isize;
const G_GET_CONFIGSTRING: isize = MpGameImport::G_GET_CONFIGSTRING as isize;
const G_GET_ENTITY_TOKEN: isize = MpGameImport::G_GET_ENTITY_TOKEN as isize;
const G_LOCATE_GAME_DATA: isize = MpGameImport::G_LOCATE_GAME_DATA as isize;
const G_SET_USERINFO: isize = MpGameImport::G_SET_USERINFO as isize;

/// Realistic userinfo string for client 0, exercising the keys
/// `ClientUserinfoChanged` reads (g_client.c:1888). A drop-in of what a real
/// client submits: name, rate/snaps, model + team_model, colors, handicap, sex,
/// item prediction, teamtask, an empty password, and a full `forcepowers` config.
///
/// The `forcepowers` value is Raven's own canonical `<rank>-<side>-<18 power
/// digits>` string (its `BG_LegalizedForcePowers` fallback, bg_misc.c:457).
/// NUM_FORCE_POWERS is 18, so the 18 digits fully populate the force-power array.
/// This is REQUIRED, not decorative: a real client always sends `forcepowers`,
/// and Raven's `WP_InitForcePowers`/`BG_LegalizedForcePowers` (w_force.c:277,
/// bg_misc.c:439) leaves its `int final_Powers[NUM_FORCE_POWERS]` stack array
/// UNINITIALIZED when the string is empty — then reads garbage as a force-power
/// level and indexes `bgForcePowerCost[i][countDown]` out of bounds, segfaulting
/// (ClientBegin -> WP_InitForcePowers). The Rust port survives an empty string
/// only because Rust zero-inits the array; the unmodified oracle does not.
const CLIENT0_USERINFO: &str = "\\name\\Padawan\\rate\\25000\\snaps\\20\\model\\kyle/default\
\\team_model\\kyle/default\\color1\\4\\color2\\4\\handicap\\100\\sex\\male\
\\cg_predictItems\\1\\teamtask\\0\\forcepowers\\7-1-032330000000001333\\password\\";

// ---------------------------------------------------------------------------
// Mock engine state
// ---------------------------------------------------------------------------

/// What `G_LOCATE_GAME_DATA` handed the engine — the core structural side effect
/// of `GAME_INIT` (`g_public.h:145`: the game tells the server where and how big
/// the entity/client arrays are).
#[derive(Clone, Copy, Debug)]
struct LocateData {
    g_ents: *mut c_void,
    num_g_entities: c_int,
    sizeof_g_entity_t: c_int,
    clients: *mut c_void,
    sizeof_g_client: c_int,
}

struct MockEngine {
    /// Monotonic `trap_Milliseconds` clock.
    millis: c_int,
    /// In-memory cvar table keyed by name (the engine's persisted values).
    cvars: BTreeMap<String, String>,
    /// `vmCvar_t.handle` → cvar name, so `G_CVAR_UPDATE` (which passes only the
    /// vmCvar pointer) can re-serve the right value.
    handle_names: BTreeMap<c_int, String>,
    next_handle: c_int,
    /// Configstrings the game set (`G_SET_CONFIGSTRING`), recorded by index.
    configstrings: BTreeMap<c_int, String>,
    /// Per-client userinfo strings served by `G_GET_USERINFO` and mutated by
    /// `G_SET_USERINFO` (the game rewrites userinfo on illegal name changes).
    userinfos: BTreeMap<c_int, String>,
    /// Minimal BSP entity token stream served by `G_GET_ENTITY_TOKEN`.
    tokens: Vec<CString>,
    token_idx: usize,
    /// Current command tokens served by `G_ARGC`/`G_ARGV` (the engine's parsed
    /// `Cmd_Argv` view) while a `GAME_CLIENT_COMMAND`/`GAME_CONSOLE_COMMAND` runs.
    cmd_args: Vec<CString>,
    /// Captured `G_LOCATE_GAME_DATA` payload (None until INIT calls it).
    locate: Option<LocateData>,
    /// `G_PRINT` capture.
    prints: Vec<String>,
    /// Set iff `G_ERROR` fired (a real `Com_Error` — a test failure).
    g_error: Option<String>,
    /// Per-import call counts for the coverage report.
    counts: BTreeMap<isize, u32>,
    /// Imports hit that the mock does not model (served the permissive `0`).
    logged_only: BTreeMap<isize, u32>,
}

impl MockEngine {
    fn new() -> Self {
        // Serve a plain FFA, single-server-slice environment. g_gametype=0
        // (FFA), bot_enable/dedicated=0, sv_maxclients modest. The module's own
        // cvar table defaults are echoed back by `G_CVAR_REGISTER` when a name
        // is not pre-seeded here, so only overrides need listing.
        let mut cvars = BTreeMap::new();
        for (k, v) in [
            ("g_gametype", "0"),
            ("bot_enable", "0"),
            ("dedicated", "0"),
            ("sv_maxclients", "16"),
            ("com_buildScript", "0"),
        ] {
            cvars.insert(k.to_string(), v.to_string());
        }

        // A minimal playable map: a `worldspawn` followed by one FFA spawn point.
        // G_ParseSpawnVars expects `{`, then key/value pairs, then `}`; the first
        // entity must be `worldspawn` (SP_worldspawn errors otherwise). The
        // `info_player_deathmatch` is required so ClientSpawn's SelectSpawnPoint
        // finds a spot — with none, Raven's `G_Error("Couldn't find a spawn
        // point")` drops the game (g_client.c SelectSpawnPoint). After the stream
        // `G_GET_ENTITY_TOKEN` returns qfalse (end of entity string).
        // Contract: `qboolean trap_GetEntityToken( char *buffer, int bufferSize )`
        // — Source: oracle/oracle/codemp/game/g_public.h:221.
        let tokens = [
            "{", "classname", "worldspawn", "}", //
            "{", "classname", "info_player_deathmatch", "origin", "0 0 100", "angle", "0", "}",
        ]
        .into_iter()
        .map(|s| CString::new(s).unwrap())
        .collect();

        // Client 0 arrives with a realistic userinfo; other slots are empty
        // until a (hypothetical) connect populates them.
        let mut userinfos = BTreeMap::new();
        userinfos.insert(0, CLIENT0_USERINFO.to_string());

        MockEngine {
            millis: 0,
            cvars,
            handle_names: BTreeMap::new(),
            next_handle: 1,
            configstrings: BTreeMap::new(),
            userinfos,
            tokens,
            token_idx: 0,
            cmd_args: Vec::new(),
            locate: None,
            prints: Vec::new(),
            g_error: None,
            counts: BTreeMap::new(),
            logged_only: BTreeMap::new(),
        }
    }

    /// Install the tokenized command the engine's `Cmd_Argc`/`Cmd_Argv` view
    /// serves for the duration of one `ClientCommand`/`ConsoleCommand` dispatch.
    fn set_cmd(&mut self, tokens: &[&str]) {
        self.cmd_args = tokens.iter().map(|t| CString::new(*t).unwrap()).collect();
    }

    /// Clear the command view once the dispatch returns.
    fn clear_cmd(&mut self) {
        self.cmd_args.clear();
    }
}

// The mock is reached only through the single-threaded engine slot; a
// thread-local `RefCell` gives the fixed-arity syscall fn access to it without a
// `static mut`.
thread_local! {
    static MOCK: RefCell<MockEngine> = RefCell::new(MockEngine::new());
}

// ---------------------------------------------------------------------------
// Raw-frame helpers
// ---------------------------------------------------------------------------

/// Read argument word `i` from the trampoline's flat frame. `args[0]` is the
/// import number; `args[1..]` are the syscall's own words in encode order.
///
/// # Safety
/// `args` is the shim's 16-word frame; `i` must be < 16.
unsafe fn word(args: *const isize, i: usize) -> isize {
    *args.add(i)
}

/// Borrow a NUL-terminated C string argument as an owned `String` (lossy).
unsafe fn c_str(ptr: isize) -> String {
    if ptr == 0 {
        return String::new();
    }
    CStr::from_ptr(ptr as *const c_char)
        .to_string_lossy()
        .into_owned()
}

/// Write `value` (NUL-terminated, truncated to `size`) into a C char buffer.
unsafe fn write_c_buffer(buf: isize, size: isize, value: &str) {
    if buf == 0 || size <= 0 {
        return;
    }
    let cap = size as usize;
    let dst = buf as *mut c_char;
    let bytes = value.as_bytes();
    let n = bytes.len().min(cap.saturating_sub(1));
    for (k, &b) in bytes.iter().take(n).enumerate() {
        *dst.add(k) = b as c_char;
    }
    *dst.add(n) = 0;
}

// ---------------------------------------------------------------------------
// The mock engine syscall entrypoint (matches `SlotSyscall`)
// ---------------------------------------------------------------------------

/// `extern "C-unwind" fn(ctx, args) -> isize` — the injected engine dispatch
/// target the inbound trampoline forwards to (`vm.cpp:471-472,506`). `args[0]` is
/// the `MpGameImport` number; the rest are the syscall's words.
///
/// Strategy: PERMISSIVE DEFAULT — an unmodeled syscall is logged and returns 0,
/// so gaps are visible (in the coverage report), never fatal. Only the syscalls
/// `GAME_INIT`/frame/shutdown actually need substance are implemented.
extern "C-unwind" fn mock_syscall(_ctx: *mut c_void, args: *const isize) -> isize {
    let n = unsafe { word(args, 0) };

    MOCK.with(|m| {
        let mut m = m.borrow_mut();
        *m.counts.entry(n).or_insert(0) += 1;

        match n {
            G_PRINT => {
                let s = unsafe { c_str(word(args, 1)) };
                eprint!("[G_PRINT] {s}");
                m.prints.push(s);
                0
            }
            G_ERROR => {
                // A real Com_Error must fail the test loudly (SEAM-D12: this
                // panic unwinds back through the C-unwind trampoline frame).
                let s = unsafe { c_str(word(args, 1)) };
                m.g_error = Some(s.clone());
                panic!("module raised G_ERROR: {s}");
            }
            G_MILLISECONDS => {
                let t = m.millis;
                m.millis += 1;
                t as isize
            }
            // ---- cvar family ------------------------------------------------
            G_CVAR_REGISTER => {
                // ( vmCvar_t *vmCvar, const char *varName, const char *default, int flags )
                let cvar_ptr = unsafe { word(args, 1) } as *mut vmCvar_t;
                let name = unsafe { c_str(word(args, 2)) };
                let default = unsafe { c_str(word(args, 3)) };
                let value = m.cvars.entry(name.clone()).or_insert(default).clone();
                let handle = m.next_handle;
                m.next_handle += 1;
                m.handle_names.insert(handle, name);
                unsafe { write_vmcvar(cvar_ptr, handle, &value) };
                0
            }
            G_CVAR_UPDATE => {
                // ( vmCvar_t *vmCvar ) — recover the name via the stored handle.
                let cvar_ptr = unsafe { word(args, 1) } as *mut vmCvar_t;
                if !cvar_ptr.is_null() {
                    let handle = unsafe { (*cvar_ptr).handle };
                    if let Some(name) = m.handle_names.get(&handle).cloned() {
                        if let Some(value) = m.cvars.get(&name).cloned() {
                            unsafe { write_vmcvar(cvar_ptr, handle, &value) };
                        }
                    }
                }
                0
            }
            G_CVAR_SET => {
                // ( const char *var_name, const char *value )
                let name = unsafe { c_str(word(args, 1)) };
                let value = unsafe { c_str(word(args, 2)) };
                m.cvars.insert(name, value);
                0
            }
            G_CVAR_VARIABLE_INTEGER_VALUE => {
                let name = unsafe { c_str(word(args, 1)) };
                m.cvars
                    .get(&name)
                    .and_then(|v| v.trim().parse::<i32>().ok())
                    .unwrap_or(0) as isize
            }
            G_CVAR_VARIABLE_STRING_BUFFER => {
                let name = unsafe { c_str(word(args, 1)) };
                let value = m.cvars.get(&name).cloned().unwrap_or_default();
                unsafe { write_c_buffer(word(args, 2), word(args, 3), &value) };
                0
            }
            // ---- command args ----------------------------------------------
            // `int trap_Argc( void )` / `void trap_Argv( int n, char *buffer,
            // int bufferSize )` — the module's tokenized view of the current
            // command. Source: oracle/oracle/codemp/game/g_public.h (trap_Argc/Argv).
            G_ARGC => m.cmd_args.len() as isize,
            G_ARGV => {
                let idx = unsafe { word(args, 1) } as usize;
                let s = m
                    .cmd_args
                    .get(idx)
                    .map(|c| c.to_str().unwrap())
                    .unwrap_or("");
                unsafe { write_c_buffer(word(args, 2), word(args, 3), s) };
                0
            }
            // ---- filesystem: everything missing ----------------------------
            G_FS_FOPEN_FILE => {
                // ( const char *qpath, fileHandle_t *f, fsMode_t mode ) -> int len
                // Missing file: handle 0, length -1. All optional BG data loads
                // (sabers, vehicles, anim cfg, logfile) take their skip paths.
                let handle_out = unsafe { word(args, 2) } as *mut c_int;
                if !handle_out.is_null() {
                    unsafe { *handle_out = 0 };
                }
                -1
            }
            G_FS_READ | G_FS_WRITE | G_FS_FCLOSE_FILE => 0,
            G_FS_GETFILELIST => 0,
            // ---- server info / config strings ------------------------------
            G_GET_SERVERINFO => {
                unsafe {
                    write_c_buffer(
                        word(args, 1),
                        word(args, 2),
                        "\\g_gametype\\0\\sv_maxclients\\16\\mapname\\smoke",
                    )
                };
                0
            }
            G_GET_USERINFO => {
                // `void trap_GetUserinfo( int num, char *buffer, int bufferSize )`.
                // A realistic client userinfo drives ClientUserinfoChanged's key
                // reads (name/model/color/sex/handicap/cg_predictItems/team_model
                // /snaps/rate) faithfully; unknown clients get an empty string.
                // Source: oracle/oracle/codemp/game/g_client.c:1912,2269 (trap_GetUserinfo).
                let num = unsafe { word(args, 1) } as c_int;
                let s = m.userinfos.get(&num).cloned().unwrap_or_default();
                unsafe { write_c_buffer(word(args, 2), word(args, 3), &s) };
                0
            }
            G_SET_USERINFO => {
                // `void trap_SetUserinfo( int num, const char *buffer )`.
                // Source: oracle/oracle/codemp/game/g_public.h (trap_SetUserinfo).
                let num = unsafe { word(args, 1) } as c_int;
                let s = unsafe { c_str(word(args, 2)) };
                m.userinfos.insert(num, s);
                0
            }
            G_SET_CONFIGSTRING => {
                let num = unsafe { word(args, 1) } as c_int;
                let s = unsafe { c_str(word(args, 2)) };
                m.configstrings.insert(num, s);
                0
            }
            G_GET_CONFIGSTRING => {
                let num = unsafe { word(args, 1) } as c_int;
                let s = m.configstrings.get(&num).cloned().unwrap_or_default();
                unsafe { write_c_buffer(word(args, 2), word(args, 3), &s) };
                0
            }
            G_GET_ENTITY_TOKEN => {
                // qboolean ( char *buffer, int bufferSize )
                if m.token_idx < m.tokens.len() {
                    let tok = m.tokens[m.token_idx].clone();
                    m.token_idx += 1;
                    unsafe { write_c_buffer(word(args, 1), word(args, 2), tok.to_str().unwrap()) };
                    1 // qtrue
                } else {
                    0 // qfalse — end of entity string
                }
            }
            G_LOCATE_GAME_DATA => {
                m.locate = Some(LocateData {
                    g_ents: unsafe { word(args, 1) } as *mut c_void,
                    num_g_entities: unsafe { word(args, 2) } as c_int,
                    sizeof_g_entity_t: unsafe { word(args, 3) } as c_int,
                    clients: unsafe { word(args, 4) } as *mut c_void,
                    sizeof_g_client: unsafe { word(args, 5) } as c_int,
                });
                0
            }
            // ---- permissive default: log + return 0 ------------------------
            other => {
                *m.logged_only.entry(other).or_insert(0) += 1;
                0
            }
        }
    })
}

/// Write a `vmCvar_t` back into the module's memory (`q_shared.h:1818-1830`):
/// `handle`, `modificationCount`, `value = atof(str)`, `integer = atoi(str)`,
/// and the NUL-terminated `string`.
unsafe fn write_vmcvar(ptr: *mut vmCvar_t, handle: c_int, value: &str) {
    if ptr.is_null() {
        return;
    }
    let integer = value.trim().parse::<i32>().unwrap_or(0);
    let float = value.trim().parse::<f32>().unwrap_or(0.0);
    (*ptr).handle = handle;
    (*ptr).modificationCount = 1;
    (*ptr).value = float;
    (*ptr).integer = integer;
    let bytes = value.as_bytes();
    let n = bytes.len().min((*ptr).string.len() - 1);
    for (k, &b) in bytes.iter().take(n).enumerate() {
        (*ptr).string[k] = b as c_char;
    }
    (*ptr).string[n] = 0;
}

// ---------------------------------------------------------------------------
// Artifact location helpers
// ---------------------------------------------------------------------------

/// Platform cdylib filename for a `[lib] name = "<base>"` crate:
/// `"jampgame"` → `libjampgame.dylib` / `libjampgame.so` / `jampgame.dll`.
pub fn dylib_filename(base: &str) -> String {
    #[cfg(target_os = "macos")]
    {
        format!("lib{base}.dylib")
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        format!("lib{base}.so")
    }
    #[cfg(windows)]
    {
        format!("{base}.dll")
    }
}

/// Locate a cargo-built cdylib next to the test binary. Integration tests run
/// from `target/<profile>/deps/`; cargo places the cdylib in both `deps/` and its
/// parent `target/<profile>/`. `cargo build --workspace` (CI) or `cargo test -p
/// jampgame` (which builds the package's cdylib target) produces it beforehand.
pub fn locate_cargo_cdylib(filename: &str) -> PathBuf {
    let exe = std::env::current_exe().expect("current_exe");
    let deps = exe.parent().expect("deps dir");
    for dir in [deps, deps.parent().expect("target/<profile> dir")] {
        let candidate = dir.join(filename);
        if candidate.exists() {
            return candidate;
        }
    }
    panic!(
        "built cdylib `{filename}` not found next to test binary ({}). \
         Run `cargo build --workspace` (or `cargo test -p jampgame`, which builds \
         the cdylib target) first — the test never spawns cargo.",
        deps.display()
    );
}

// ---------------------------------------------------------------------------
// The single lifecycle drive (with all structural assertions)
// ---------------------------------------------------------------------------

/// `GameWorld` / Raven's `level` island (MAX_GENTITIES entities + clients +
/// shared buffers) is many megabytes and is built by value inside `vmMain`'s
/// GAME_INIT bootstrap. The default 2 MiB test-thread stack overflows on it; the
/// real engine drives `vmMain` on its large main-thread stack. Run the whole
/// single-shot lifecycle on a thread with a generous stack to match. (The oracle
/// DLL's GAME_INIT allocates just as big.)
pub fn run_on_engine_thread(dylib: PathBuf) {
    let handle = std::thread::Builder::new()
        .name("abi-smoke-engine".into())
        .stack_size(64 * 1024 * 1024)
        .spawn(move || run_lifecycle(dylib))
        .expect("spawn engine thread");
    handle.join().expect("engine lifecycle thread panicked");
}

fn run_lifecycle(dylib: PathBuf) {
    let dir = dylib.parent().unwrap().to_path_buf();
    let file = dylib
        .file_name()
        .expect("dylib filename")
        .to_str()
        .expect("utf8 dylib filename")
        .to_string();
    eprintln!("[smoke] loading {}", dylib.display());

    // Load through the real ported loader. A single FsPath step whose
    // base/gamedir join to the artifact directory, plus explicit naming, drives
    // the loader straight at our file — exercising its FS probe + the
    // `dllEntry(syscall)` handshake (win_main.cpp:858-887). The syscall handed
    // over is the real C-variadic inbound trampoline.
    // `name` borrows `file` (any lifetime); `suffix` must be `&'static str`
    // (`ModuleNaming.suffix: Option<&'static str>`) so it comes from the
    // platform-fixed literal, not a slice of the local filename.
    let name: &str = split_dylib_stem(&file);
    let policy = ModuleSearchPolicy {
        naming: ModuleNaming {
            suffix: Some(platform_dylib_suffix()),
        },
        direct_first: false,
        steps: vec![SearchStep::FsPath {
            base: dir,
            gamedir: String::new(),
        }],
    };

    // Arm the engine slot BEFORE any module syscall can fire (the first one is
    // inside GAME_INIT). dllEntry (called by the loader) only stores the pointer.
    arm_game_slot(std::ptr::null_mut(), mock_syscall);

    let syscall: RawSyscall = game_syscall_trampoline as *const c_void;
    let module = sys_load_dll(&policy, name, syscall)
        .expect("sys_load_dll resolved dllEntry+vmMain and completed the handshake");
    let vm_main: RawVmMain = module.entry();

    // ---- GAME_INIT -------------------------------------------------------
    // ( int levelTime, int randomSeed, int restart ). qfalse restart.
    let init_ret = call_vm(vm_main, MpGameExport::GAME_INIT, &[600, 42, 0]);
    eprintln!("[smoke] GAME_INIT returned {init_ret}");

    // ---- structural assertions on INIT side effects ----------------------
    MOCK.with(|m| {
        let m = m.borrow();

        assert!(
            m.g_error.is_none(),
            "GAME_INIT raised G_ERROR: {:?}",
            m.g_error
        );

        let locate = m
            .locate
            .expect("GAME_INIT must call G_LOCATE_GAME_DATA (g_public.h:145)");
        assert!(
            !locate.g_ents.is_null(),
            "LOCATE_GAME_DATA gentities base is null"
        );
        assert!(
            !locate.clients.is_null(),
            "LOCATE_GAME_DATA clients base is null"
        );
        assert!(
            locate.num_g_entities >= 1,
            "LOCATE_GAME_DATA numGEntities = {} (< 1: no worldspawn slot)",
            locate.num_g_entities
        );
        assert!(
            locate.sizeof_g_entity_t > 0,
            "LOCATE_GAME_DATA sizeofGEntity_t = {} (must be > 0)",
            locate.sizeof_g_entity_t
        );
        assert!(
            locate.sizeof_g_client > 0,
            "LOCATE_GAME_DATA sizeofGameClient = {} (must be > 0)",
            locate.sizeof_g_client
        );

        assert!(
            !m.configstrings.is_empty(),
            "GAME_INIT set no configstrings"
        );

        // The worldspawn token stream should have been fully consumed.
        assert_eq!(
            m.token_idx,
            m.tokens.len(),
            "GAME_INIT did not consume the entity token stream"
        );
    });

    // ---- warm-up frames before the client connects -----------------------
    // The engine runs a few game frames before wiring up a client (g_main.c:2949
    // "we run 3 game frames before calling Connect"). +50ms steps.
    let mut t = 600;
    let run_frame = |t: AbiWord| {
        let ret = call_vm(vm_main, MpGameExport::GAME_RUN_FRAME, &[t]);
        assert_eq!(ret, 0, "GAME_RUN_FRAME(t={t}) returned {ret}, expected 0");
        MOCK.with(|m| {
            assert!(
                m.borrow().g_error.is_none(),
                "GAME_RUN_FRAME raised G_ERROR"
            );
        });
    };
    for _ in 0..3 {
        t += 50;
        run_frame(t);
    }
    eprintln!("[smoke] 3 warm-up frames survived (level time now {t})");

    // ---- GAME_CLIENT_CONNECT(0, firstTime=qtrue, isBot=qfalse) ------------
    // Contract (g_client.c ClientConnect): returns NULL (0) to admit the client,
    // or a pointer to a rejection string on failure. Source: g_client.c:2258.
    let connect_ret = call_vm(
        vm_main,
        MpGameExport::GAME_CLIENT_CONNECT,
        &[0, 1 /*qtrue*/, 0 /*qfalse*/],
    );
    if connect_ret != 0 {
        let reason = unsafe { c_str(connect_ret) };
        panic!("GAME_CLIENT_CONNECT(0) rejected the client: {reason:?}");
    }
    MOCK.with(|m| {
        let m = m.borrow();
        assert!(m.g_error.is_none(), "GAME_CLIENT_CONNECT raised G_ERROR");
        // ClientConnect → ClientUserinfoChanged reads the client's userinfo
        // (g_client.c:2269,2347). Prove the substantive G_GET_USERINFO path ran.
        assert!(
            *m.counts.get(&G_GET_USERINFO).unwrap_or(&0) > 0,
            "GAME_CLIENT_CONNECT never read client userinfo (G_GET_USERINFO)"
        );
    });
    eprintln!("[smoke] GAME_CLIENT_CONNECT(0) admitted (returned NULL)");

    // GAME_CLIENT_USERINFO_CHANGED is NOT driven here: the engine only issues it
    // on an actual userinfo change; the connect/begin flow calls it internally
    // (ClientConnect g_client.c:2347, ClientBegin g_client.c:2437). We drive only
    // what the engine drives.

    // ---- GAME_CLIENT_BEGIN(0) --------------------------------------------
    // vmMain calls ClientBegin(arg0, qtrue) — spawns the client into the level
    // (ClientSpawn → WP_InitForcePowers etc.). Source: g_main.c:534, g_client.c:2393.
    let begin_ret = call_vm(vm_main, MpGameExport::GAME_CLIENT_BEGIN, &[0]);
    assert_eq!(begin_ret, 0, "GAME_CLIENT_BEGIN returned {begin_ret}");
    MOCK.with(|m| {
        assert!(
            m.borrow().g_error.is_none(),
            "GAME_CLIENT_BEGIN raised G_ERROR"
        );
    });
    eprintln!("[smoke] GAME_CLIENT_BEGIN(0) spawned the client");

    // ---- GAME_RUN_FRAME x10 with the client connected --------------------
    for _ in 0..10 {
        t += 50;
        run_frame(t);
    }
    eprintln!("[smoke] 10 connected frames survived (level time now {t})");

    // ---- GAME_CLIENT_COMMAND(0, "say hello") -----------------------------
    // vmMain calls ClientCommand(arg0), which reads the command via trap_Argc/
    // trap_Argv (g_main.c:537, g_cmds.c ClientCommand). The mock serves the
    // command tokens through G_ARGC/G_ARGV below.
    MOCK.with(|m| m.borrow_mut().set_cmd(&["say", "hello"]));
    let cmd_ret = call_vm(vm_main, MpGameExport::GAME_CLIENT_COMMAND, &[0]);
    assert_eq!(cmd_ret, 0, "GAME_CLIENT_COMMAND returned {cmd_ret}");
    MOCK.with(|m| {
        let mut m = m.borrow_mut();
        assert!(m.g_error.is_none(), "GAME_CLIENT_COMMAND raised G_ERROR");
        m.clear_cmd();
    });
    eprintln!("[smoke] GAME_CLIENT_COMMAND(0, \"say hello\") survived");

    // A couple more frames with the client still in.
    for _ in 0..2 {
        t += 50;
        run_frame(t);
    }

    // ---- GAME_CLIENT_DISCONNECT(0) ---------------------------------------
    // Source: g_main.c:531, g_client.c:3816.
    let disc_ret = call_vm(vm_main, MpGameExport::GAME_CLIENT_DISCONNECT, &[0]);
    assert_eq!(disc_ret, 0, "GAME_CLIENT_DISCONNECT returned {disc_ret}");
    MOCK.with(|m| {
        assert!(
            m.borrow().g_error.is_none(),
            "GAME_CLIENT_DISCONNECT raised G_ERROR"
        );
    });
    eprintln!("[smoke] GAME_CLIENT_DISCONNECT(0) clean");

    // A frame or two after the disconnect.
    for _ in 0..2 {
        t += 50;
        run_frame(t);
    }
    eprintln!("[smoke] post-disconnect frames survived (level time now {t})");

    // ---- GAME_SHUTDOWN (restart qfalse) ----------------------------------
    let sd_ret = call_vm(vm_main, MpGameExport::GAME_SHUTDOWN, &[0]);
    assert_eq!(sd_ret, 0, "GAME_SHUTDOWN returned {sd_ret}, expected 0");
    MOCK.with(|m| {
        assert!(
            m.borrow().g_error.is_none(),
            "GAME_SHUTDOWN raised G_ERROR"
        );
    });
    eprintln!("[smoke] GAME_SHUTDOWN clean");

    // ---- coverage report -------------------------------------------------
    MOCK.with(|m| {
        let m = m.borrow();
        eprintln!("\n===== mock syscall coverage =====");
        eprintln!("implemented syscalls (name: count):");
        for (&num, &count) in &m.counts {
            if !m.logged_only.contains_key(&num) {
                eprintln!("  {:<32} {count}", import_name(num));
            }
        }
        eprintln!("logged-only (permissive default 0) syscalls (name: count):");
        for (&num, &count) in &m.logged_only {
            eprintln!("  {:<32} {count}", import_name(num));
        }
        eprintln!(
            "totals: {} distinct imports, {} prints, {} configstrings set",
            m.counts.len(),
            m.prints.len(),
            m.configstrings.len()
        );
        eprintln!("=================================\n");
    });
}

/// The stem of a platform cdylib filename — the part the loader recombines with
/// `suffix` via `format!("{name}{suffix}")`: `libjampgame.dylib` → `libjampgame`,
/// `jampgame.dll` → `jampgame`.
fn split_dylib_stem(file: &str) -> &str {
    let dot = file.find('.').expect("dylib name has an extension");
    &file[..dot]
}

/// Platform dylib suffix as a `&'static str` (required by `ModuleNaming.suffix`).
fn platform_dylib_suffix() -> &'static str {
    #[cfg(target_os = "macos")]
    {
        ".dylib"
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        ".so"
    }
    #[cfg(windows)]
    {
        ".dll"
    }
}

/// Invoke `vmMain(command, arg0..arg11)`, zero-filling the unused words.
fn call_vm(vm_main: RawVmMain, command: MpGameExport, args: &[AbiWord]) -> AbiWord {
    let mut a = [0 as AbiWord; 12];
    a[..args.len()].copy_from_slice(args);
    vm_main(
        command as c_int,
        a[0], a[1], a[2], a[3], a[4], a[5], a[6], a[7], a[8], a[9], a[10], a[11],
    )
}

/// Best-effort import name for the coverage report (the handled set plus the
/// syscalls GAME_INIT/frame/shutdown are known to raise); anything else prints
/// as its raw wire number.
fn import_name(n: isize) -> String {
    macro_rules! named {
        ($($v:ident),* $(,)?) => {
            $(if n == MpGameImport::$v as isize { return stringify!($v).to_string(); })*
        };
    }
    named!(
        G_PRINT, G_ERROR, G_MILLISECONDS,
        G_CVAR_REGISTER, G_CVAR_UPDATE, G_CVAR_SET,
        G_CVAR_VARIABLE_INTEGER_VALUE, G_CVAR_VARIABLE_STRING_BUFFER,
        G_ARGC, G_ARGV,
        G_FS_FOPEN_FILE, G_FS_READ, G_FS_WRITE, G_FS_FCLOSE_FILE, G_FS_GETFILELIST,
        G_GET_SERVERINFO, G_GET_USERINFO,
        G_SET_CONFIGSTRING, G_GET_CONFIGSTRING, G_GET_ENTITY_TOKEN, G_LOCATE_GAME_DATA,
        G_SET_SHARED_BUFFER, G_G2_CLEANENTATTACHMENTS, G_ICARUS_INIT, G_ICARUS_SHUTDOWN,
        G_NAV_LOAD, G_NAV_SETPATHSCALCULATED, G_SET_SERVER_CULL,
        G_G2_HAVEWEGHOULMODELS, G_ROFF_CLEAN, G_SEND_SERVER_COMMAND, G_SEND_CONSOLE_COMMAND,
        G_LINKENTITY, G_UNLINKENTITY, G_GET_USERCMD,
        G_ROFF_UPDATE_ENTITIES, G_ICARUS_FREEENT, G_ICARUS_INITENT,
        G_NAV_SAVE, G_NAV_CALCULATEPATHS, G_NAV_CLEARALLFAILEDEDGES,
        G_NAV_CHECKBLOCKEDEDGES, G_NAV_CLEARCHECKEDNODES, G_G2_INITGHOUL2MODEL,
    );
    format!("syscall#{n}")
}
