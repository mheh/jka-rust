//! Differential parity: the Rust `mp_engine_qcommon::stringed` §F port must
//! reproduce, byte for byte, the dumps produced by the UNMODIFIED Raven C++
//! StringEd TUs compiled by `tools/stringed-oracle/build.sh` (goldens under
//! `tools/stringed-oracle/goldens/`).
//!
//! The three units mirror `docs/subsystems/stringed.md` § Verification strategy
//! and the three `tools/stringed-oracle/dump.cpp` modes exactly:
//! * [`parse_lookup_matches_oracle_golden`] mirrors `mode_parse_lookup`
//!   (Golden A — parse/lookup).
//! * [`reference_stability_matches_oracle_golden`] mirrors
//!   `mode_reference_stability` (Golden B — reference stability + reload).
//! * [`filelist_scan_matches_oracle_golden`] mirrors `mode_filelist_scan`
//!   (Golden C — file-list scan + language enumeration).
//!
//! Fixtures/goldens are read from `tools/stringed-oracle/` and are never edited.
//! The port is driven through [`mp_host_interface::mock::MockHost`] (RULING
//! 32/55): the oracle `host.cpp`'s in-memory cvar registry + fixture-backed VFS
//! are that mock's cvar registry (`string` authoritative, `integer` via `atoi`)
//! and its FS-fixture map (`files`, keyed by qpath; `fs_list_files` sorts). The
//! Rust dump format matches each `printf`/`putEsc` character for character.

use std::fmt::Write as _;

use mp_engine_qcommon::stringed::api::{
    se_check_for_language_updates, se_get_language_dir, se_get_language_name, se_get_num_languages,
    se_init, se_load_actual, se_load_language, se_new_language, se_shut_down,
};
use mp_engine_qcommon::stringed::interface::se_build_file_list;
use mp_engine_qcommon::stringed::StringEdPackage;
use mp_host_interface::mock::MockHost;
use testkit::{oracle_root, walk_fixtures};

/// A `MockHost` whose FS-fixture map (`files`) is seeded from every file under
/// `tools/stringed-oracle/fixtures/`, keyed by its path relative to that root
/// (eg `misc/truncated.str`, `strings/english/menus.str`). This is the Rust
/// twin of the oracle `host.cpp` VFS with `Host_SetFixtureRoot("fixtures")`
/// (files are raw bytes — CP1252 hi-char fixtures load unmodified).
fn fixture_host() -> MockHost {
    let mut host = MockHost::new();
    let fixtures = oracle_root("stringed-oracle").join("fixtures");
    walk_fixtures(&fixtures, &mut |key, bytes| {
        host.files.insert(key, bytes);
    });
    host
}

/// Escaped string printer — mirrors `dump.cpp`'s `putEsc` byte for byte: keeps
/// control bytes on one golden line (`\n`/`\t`/`\r`/`\\`, else `\xHH` for
/// `< 0x20` or `>= 0x7f`, else the raw char). Operates on the stored `String`'s
/// bytes so a stored newline (the `\n` CR-literal) escapes to `\n`.
fn put_esc(out: &mut String, s: &[u8]) {
    for &c in s {
        match c {
            b'\n' => out.push_str("\\n"),
            b'\t' => out.push_str("\\t"),
            b'\r' => out.push_str("\\r"),
            b'\\' => out.push_str("\\\\"),
            c if c < 0x20 || c >= 0x7f => write!(out, "\\x{c:02x}").unwrap(),
            c => out.push(c as char),
        }
    }
}

/// Mirror `dump.cpp`'s `dumpEntries`: the header then every `m_StringEntries`
/// key → (`m_strString`, `m_strDebug`, `m_iFlags`) in `BTreeMap`-sorted order
/// (SE-D1(4)). `E |KEY| str=|<esc>| dbg=|<esc>| flags=N`.
fn dump_entries(out: &mut String, pkg: &StringEdPackage, header: &str) {
    writeln!(out, "== {header} ==").unwrap();
    for (key, entry) in &pkg.string_entries {
        write!(out, "E |{key}| str=|").unwrap();
        put_esc(out, entry.m_str_string.as_bytes());
        out.push_str("| dbg=|");
        put_esc(out, entry.m_str_debug.as_bytes());
        writeln!(out, "| flags={}", entry.m_i_flags).unwrap();
    }
}

/// Mirror `dump.cpp`'s `dumpFlags`: `SE_GetNumFlags`, then each flag's
/// name/mask in encounter order, then the SE-V4 out-of-range / negative probes
/// (both `""`). The free-function `SE_Get*Flag*` are `StringEdPackage` seam
/// methods here (SE-D7/RULING 57).
fn dump_flags(out: &mut String, pkg: &StringEdPackage) {
    let n = pkg.get_num_flags();
    writeln!(out, "numFlags={n}").unwrap();
    for i in 0..n {
        let name = pkg.get_flag_name(i);
        writeln!(out, "FLAG {i} |{name}| mask={}", pkg.get_flag_mask(name)).unwrap();
    }
    // SE-V4: signed index compared against size_t -> negative wraps out of range.
    writeln!(out, "FLAG oor |{}|", pkg.get_flag_name(99)).unwrap();
    writeln!(out, "FLAG neg |{}|", pkg.get_flag_name(-1)).unwrap();
}

/// Mirror `dump.cpp`'s `okOrMsg`: a `None` (ok) load prints `(null=ok)`, else
/// the error message verbatim.
fn ok_or_msg(m: &Option<String>) -> &str {
    match m {
        Some(msg) => msg,
        None => "(null=ok)",
    }
}

// ===========================================================================
// Golden A — parse / lookup (mode_parse_lookup)
// ===========================================================================

/// Reproduce `dump.cpp`'s `mode_parse_lookup` through the ported API.
fn dump_parse_lookup() -> String {
    let mut host = fixture_host();
    let mut pkg = StringEdPackage::default();
    let mut out = String::new();

    // SE_Init: registers se_language/se_debug/sp_leet, loads english (debug on).
    se_init(&mut pkg, &mut host);

    dump_entries(&mut out, &pkg, "ENTRIES (map-sorted)"); // BTreeMap order, SE-D1(4)

    out.push_str("== FLAGS ==\n"); // encounter-order bit assignment, AddFlagReference
    dump_flags(&mut out, &pkg);

    out.push_str("== LOOKUP se_debug=0 ==\n"); // se_debug default "0"
    writeln!(
        out,
        "hit           |{}|",
        pkg.get_string("OBJECTIVES_MISSION01", &mut host)
    )
    .unwrap();
    writeln!(
        out,
        "uppercasefold |{}|",
        pkg.get_string("objectives_mission01", &mut host)
    )
    .unwrap();
    writeln!(
        out,
        "2arg          |{}|",
        pkg.get_string2("OBJECTIVES", "MISSION01", &mut host)
    )
    .unwrap();
    writeln!(
        out,
        "2arg menus    |{}|",
        pkg.get_string2("MENUS", "START_GAME", &mut host)
    )
    .unwrap();
    out.push_str("greeting(crlf)|");
    put_esc(
        &mut out,
        pkg.get_string("OBJECTIVES_GREETING", &mut host).as_bytes(),
    );
    out.push_str("|\n");
    out.push_str("hichar        |");
    put_esc(
        &mut out,
        pkg.get_string("OBJECTIVES_HICHAR", &mut host).as_bytes(),
    );
    out.push_str("|\n");
    writeln!(
        out,
        "miss          |{}|",
        pkg.get_string("NOPE_NOPE", &mut host)
    )
    .unwrap(); // -> ""
    writeln!(
        out,
        "flags m01     {}",
        pkg.get_flags("OBJECTIVES_MISSION01")
    )
    .unwrap();
    writeln!(
        out,
        "flags m02(2a) {}",
        pkg.get_flags2("OBJECTIVES", "MISSION02")
    )
    .unwrap();
    writeln!(out, "flags miss    {}", pkg.get_flags("NOPE_NOPE")).unwrap(); // SE-V3 -> 0

    out.push_str("== LOOKUP se_debug=1 ==\n"); // debug branch: m_strDebug
    host.set_cvar("se_debug", "1");
    writeln!(
        out,
        "dbg m01       |{}|",
        pkg.get_string("OBJECTIVES_MISSION01", &mut host)
    )
    .unwrap();
    writeln!(
        out,
        "dbg menus     |{}|",
        pkg.get_string("MENUS_QUIT", &mut host)
    )
    .unwrap();
    host.set_cvar("se_debug", "0");

    out.push_str("== LEET sp_leet=42 ==\n"); // Leetify char-substitution on reload
    host.set_cvar("sp_leet", "42");
    se_load_language(&mut pkg, "english", true, &mut host);
    dump_entries(&mut out, &pkg, "ENTRIES-LEET");
    host.set_cvar("sp_leet", "0");

    out.push_str("== ERRORS (SE_Load_Actual direct, non-critical) ==\n"); // ParseLine msgs
                                                                          // Reset parse state so the truncated probe fires (faithful quirk: the oracle
                                                                          // clears the end-marker flag only in Clear(), not SetupNewFileParse — a
                                                                          // prior file's ENDMARKER would otherwise mask a later truncation).
    se_new_language(&mut pkg);
    writeln!(
        out,
        "truncated  |{}|",
        ok_or_msg(&se_load_actual(
            &mut pkg,
            "misc/truncated.str",
            false,
            false,
            &mut host
        ))
    )
    .unwrap();
    writeln!(
        out,
        "badversion |{}|",
        ok_or_msg(&se_load_actual(
            &mut pkg,
            "misc/badversion.str",
            false,
            false,
            &mut host
        ))
    )
    .unwrap();
    writeln!(
        out,
        "unknownkw  |{}|",
        ok_or_msg(&se_load_actual(
            &mut pkg,
            "misc/unknownkw.str",
            false,
            false,
            &mut host
        ))
    )
    .unwrap();
    writeln!(
        out,
        "missing    |{}|",
        ok_or_msg(&se_load_actual(
            &mut pkg,
            "misc/nope.str",
            false,
            false,
            &mut host
        ))
    )
    .unwrap();

    out
}

#[test]
fn parse_lookup_matches_oracle_golden() {
    let golden_path = oracle_root("stringed-oracle").join("goldens/parse_lookup.txt");
    let golden = std::fs::read_to_string(&golden_path).unwrap_or_else(|_| {
        panic!("missing golden {golden_path:?} — run tools/stringed-oracle/build.sh --regen")
    });
    assert_eq!(
        dump_parse_lookup(),
        golden,
        "parse_lookup diverges from the StringEd C++ oracle"
    );
}

// ===========================================================================
// Golden B — reference stability + language reload (mode_reference_stability)
// ===========================================================================

/// Reproduce `dump.cpp`'s `mode_reference_stability` through the ported API.
fn dump_reference_stability() -> String {
    let mut host = fixture_host();
    let mut pkg = StringEdPackage::default();
    let mut out = String::new();

    se_init(&mut pkg, &mut host); // loads english (debug on)

    out.push_str("== BEFORE (english) ==\n");
    writeln!(
        out,
        "m01 str  |{}|",
        pkg.get_string("OBJECTIVES_MISSION01", &mut host)
    )
    .unwrap();
    writeln!(out, "m01 flags {}", pkg.get_flags("OBJECTIVES_MISSION01")).unwrap();
    dump_flags(&mut out, &pkg);

    out.push_str("== SE_NewLanguage : Clear(SE_TRUE) ==\n"); // flag tables survive
    se_new_language(&mut pkg);
    writeln!(
        out,
        "numFlags={} (name table survives)",
        pkg.get_num_flags()
    )
    .unwrap();
    writeln!(
        out,
        "m01 after clear |{}| (entries cleared)",
        pkg.get_string("OBJECTIVES_MISSION01", &mut host)
    )
    .unwrap();

    out.push_str("== reload german (NewLanguage keeps flag masks) ==\n");
    host.set_cvar("se_language", "german");
    se_load_language(&mut pkg, "german", true, &mut host); // loads .str then .ste override
    writeln!(
        out,
        "m01 german |{}|",
        pkg.get_string("OBJECTIVES_MISSION01", &mut host)
    )
    .unwrap(); // .ste override
    writeln!(
        out,
        "m02 #same  |{}|",
        pkg.get_string("OBJECTIVES_MISSION02", &mut host)
    )
    .unwrap(); // -> cached english
    writeln!(
        out,
        "numFlags={} (masks persist; german entries carry 0)",
        pkg.get_num_flags()
    )
    .unwrap();
    writeln!(
        out,
        "m01 flags  {} (rebuilt entry, no FLAGS lines)",
        pkg.get_flags("OBJECTIVES_MISSION01")
    )
    .unwrap();

    out.push_str("== SE_CheckForLanguageUpdates (cvar_take_modified flow) ==\n");
    host.set_cvar("se_language", "english"); // sets modified=qtrue
    se_check_for_language_updates(&mut pkg, &mut host); // reload english, clear modified
    writeln!(
        out,
        "m01 after update |{}|",
        pkg.get_string("OBJECTIVES_MISSION01", &mut host)
    )
    .unwrap();
    se_check_for_language_updates(&mut pkg, &mut host); // modified now false -> no-op
    writeln!(
        out,
        "m01 second call  |{}| (no-op reload)",
        pkg.get_string("OBJECTIVES_MISSION01", &mut host)
    )
    .unwrap();

    out.push_str("== SE_ShutDown : Clear(SE_FALSE) ==\n"); // flag tables cleared
    se_shut_down(&mut pkg);
    writeln!(out, "numFlags={} (flags cleared)", pkg.get_num_flags()).unwrap();

    out
}

#[test]
fn reference_stability_matches_oracle_golden() {
    let golden_path = oracle_root("stringed-oracle").join("goldens/reference_stability.txt");
    let golden = std::fs::read_to_string(&golden_path).unwrap_or_else(|_| {
        panic!("missing golden {golden_path:?} — run tools/stringed-oracle/build.sh --regen")
    });
    assert_eq!(
        dump_reference_stability(),
        golden,
        "reference_stability diverges from the StringEd C++ oracle"
    );
}

// ===========================================================================
// Golden C — file-list scan + language enumeration (mode_filelist_scan)
// ===========================================================================

/// Reproduce `dump.cpp`'s `mode_filelist_scan` through the ported API.
fn dump_filelist_scan() -> String {
    let mut host = fixture_host();
    let mut pkg = StringEdPackage::default();
    let mut out = String::new();

    // SE_BuildFileList("strings"): ext "/" subdirs vs ".str" files; Raven's
    // mutated `string &` + `giFilesFound` return as (results, count) (SE-D1(3)).
    let (results, count) = se_build_file_list("strings", &mut host);
    out.push_str("== SE_BuildFileList(\"strings\") ==\n");
    writeln!(out, "count={count}").unwrap();
    writeln!(out, "results=|{results}|").unwrap(); // ';'-delimited, deterministic sort

    out.push_str("== SE_GetNumLanguages (dedup, english-first) ==\n");
    let nl = se_get_num_languages(&mut pkg, &mut host);
    writeln!(out, "numLanguages={nl}").unwrap();
    for i in 0..nl {
        writeln!(
            out,
            "LANG {i} name=|{}| dir=|{}|",
            se_get_language_name(&pkg, i),
            se_get_language_dir(&pkg, i)
        )
        .unwrap();
    }
    // SE-V3/V4: out-of-range / negative index -> "" (release assert is a no-op).
    writeln!(
        out,
        "name oor=|{}| neg=|{}|",
        se_get_language_name(&pkg, 99),
        se_get_language_name(&pkg, -1)
    )
    .unwrap();
    writeln!(
        out,
        "dir  oor=|{}| neg=|{}|",
        se_get_language_dir(&pkg, 99),
        se_get_language_dir(&pkg, -1)
    )
    .unwrap();

    out
}

#[test]
fn filelist_scan_matches_oracle_golden() {
    let golden_path = oracle_root("stringed-oracle").join("goldens/filelist_scan.txt");
    let golden = std::fs::read_to_string(&golden_path).unwrap_or_else(|_| {
        panic!("missing golden {golden_path:?} — run tools/stringed-oracle/build.sh --regen")
    });
    assert_eq!(
        dump_filelist_scan(),
        golden,
        "filelist_scan diverges from the StringEd C++ oracle"
    );
}
