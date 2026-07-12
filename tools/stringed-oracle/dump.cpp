// stringed-oracle — golden dumper for the StringEd localization port
// (docs/subsystems/stringed.md § Verification strategy).
//
// The two UNMODIFIED oracle TUs are #included into this dumper TU (copied into
// build/ by build.sh; oracle/ is never edited). Unlike gp2-/icarus-oracle,
// which link the oracle .cpp separately and drive it through a public HEADER
// class, StringEd's `CStringEdPackage` and its `TheStringPackage` global live
// INSIDE stringed_ingame.cpp with no header — so enumerating `m_StringEntries`
// in std::map (BTreeMap) sorted order requires TU-level visibility. Including
// the byte-for-byte source is still "compile the unmodified TU standalone"
// (porting-rules §18); the engine services resolve to host.cpp at link.
//
//   dump <mode> [fixtureRoot]
//     mode = parse_lookup | reference_stability | filelist_scan
//
// stdout is the golden; Com_Printf/Com_DPrintf are routed to stderr by the host.

#include "stubs/game/q_shared.h"
#include "stubs/qcommon/qcommon.h"

#include <cstdio>
#include <cstring>
#include <string>

// the unmodified oracle TUs (one aggregated dumper TU)
#include "build/codemp/qcommon/stringed_ingame.cpp"
#include "build/codemp/qcommon/stringed_interface.cpp"

// harness host hooks (host.cpp)
extern void Host_Init();
extern void Host_SetFixtureRoot(const char *root);
extern void Host_CvarSet(const char *name, const char *value);

// ---- escaped string printer (keeps control bytes on one golden line) --------
static void putEsc(const char *s) {
    for (const unsigned char *p = (const unsigned char *)s; *p; p++) {
        unsigned char c = *p;
        if (c == '\n')      fputs("\\n", stdout);
        else if (c == '\t') fputs("\\t", stdout);
        else if (c == '\r') fputs("\\r", stdout);
        else if (c == '\\') fputs("\\\\", stdout);
        else if (c < 0x20 || c >= 0x7f) printf("\\x%02x", c);
        else putchar(c);
    }
}

static void dumpEntries(const char *header) {
    printf("== %s ==\n", header);
    for (mapStringEntries_t::iterator it = TheStringPackage.m_StringEntries.begin();
         it != TheStringPackage.m_StringEntries.end(); ++it) {
        printf("E |%s| str=|", it->first.c_str());
        putEsc(it->second.m_strString.c_str());
        printf("| dbg=|");
        putEsc(it->second.m_strDebug.c_str());
        printf("| flags=%d\n", it->second.m_iFlags);
    }
}

static void dumpFlags() {
    int n = SE_GetNumFlags();
    printf("numFlags=%d\n", n);
    for (int i = 0; i < n; i++) {
        LPCSTR name = SE_GetFlagName(i);
        printf("FLAG %d |%s| mask=%d\n", i, name, SE_GetFlagMask(name));
    }
    // SE-V4: signed index compared against size_t -> negative wraps out of range.
    printf("FLAG oor |%s|\n", SE_GetFlagName(99));
    printf("FLAG neg |%s|\n", SE_GetFlagName(-1));
}

static const char *okOrMsg(LPCSTR m) { return m ? m : "(null=ok)"; }

// ===========================================================================
// Golden A — parse / lookup (doc § Verification strategy, "Golden A")
// ===========================================================================
static void mode_parse_lookup() {
    SE_Init(); // registers se_language/se_debug/sp_leet, loads english (debug on)

    dumpEntries("ENTRIES (map-sorted)"); // pins BTreeMap sorted order, SE-D1(4)

    printf("== FLAGS ==\n"); // encounter-order bit assignment, AddFlagReference
    dumpFlags();

    printf("== LOOKUP se_debug=0 ==\n"); // se_debug default "0"
    printf("hit           |%s|\n", SE_GetString("OBJECTIVES_MISSION01"));
    printf("uppercasefold |%s|\n", SE_GetString("objectives_mission01"));
    printf("2arg          |%s|\n", SE_GetString("OBJECTIVES", "MISSION01"));
    printf("2arg menus    |%s|\n", SE_GetString("MENUS", "START_GAME"));
    printf("greeting(crlf)|"); putEsc(SE_GetString("OBJECTIVES_GREETING")); printf("|\n");
    printf("hichar        |"); putEsc(SE_GetString("OBJECTIVES_HICHAR")); printf("|\n");
    printf("miss          |%s|\n", SE_GetString("NOPE_NOPE"));            // -> ""
    printf("flags m01     %d\n", SE_GetFlags("OBJECTIVES_MISSION01"));    // CAPTION|TYPEMATIC
    printf("flags m02(2a) %d\n", SE_GetFlags("OBJECTIVES", "MISSION02")); // VOICED
    printf("flags miss    %d\n", SE_GetFlags("NOPE_NOPE"));               // SE-V3 -> 0

    printf("== LOOKUP se_debug=1 ==\n"); // debug branch: m_strDebug (SE_GetString)
    Host_CvarSet("se_debug", "1");
    printf("dbg m01       |%s|\n", SE_GetString("OBJECTIVES_MISSION01"));
    printf("dbg menus     |%s|\n", SE_GetString("MENUS_QUIT"));
    Host_CvarSet("se_debug", "0");

    printf("== LEET sp_leet=42 ==\n"); // Leetify char-substitution on reload
    Host_CvarSet("sp_leet", "42");
    SE_LoadLanguage("english");
    dumpEntries("ENTRIES-LEET");
    Host_CvarSet("sp_leet", "0");

    printf("== ERRORS (SE_Load_Actual direct, non-critical) ==\n"); // ParseLine msgs
    // Reset parse state so the truncated probe fires: the oracle clears the
    // end-marker flag only in Clear(), not SetupNewFileParse — a prior loaded
    // file's ENDMARKER would otherwise mask a later truncation (faithful quirk;
    // in normal flow SE_LoadLanguage's SE_NewLanguage resets it first).
    SE_NewLanguage();
    printf("truncated  |%s|\n", okOrMsg(SE_Load_Actual("misc/truncated.str",  SE_FALSE, SE_FALSE)));
    printf("badversion |%s|\n", okOrMsg(SE_Load_Actual("misc/badversion.str", SE_FALSE, SE_FALSE)));
    printf("unknownkw  |%s|\n", okOrMsg(SE_Load_Actual("misc/unknownkw.str",  SE_FALSE, SE_FALSE)));
    printf("missing    |%s|\n", okOrMsg(SE_Load_Actual("misc/nope.str",       SE_FALSE, SE_FALSE)));
}

// ===========================================================================
// Golden B — reference stability + language reload (doc § "Golden B")
// ===========================================================================
static void mode_reference_stability() {
    SE_Init(); // loads english (debug on)

    printf("== BEFORE (english) ==\n");
    printf("m01 str  |%s|\n", SE_GetString("OBJECTIVES_MISSION01"));
    printf("m01 flags %d\n", SE_GetFlags("OBJECTIVES_MISSION01"));
    dumpFlags();

    printf("== SE_NewLanguage : Clear(SE_TRUE) ==\n"); // flag tables survive
    SE_NewLanguage();
    printf("numFlags=%d (name table survives)\n", SE_GetNumFlags());
    printf("m01 after clear |%s| (entries cleared)\n", SE_GetString("OBJECTIVES_MISSION01"));

    printf("== reload german (NewLanguage keeps flag masks) ==\n");
    Host_CvarSet("se_language", "german");
    SE_LoadLanguage("german"); // loads .str then .ste override
    printf("m01 german |%s|\n", SE_GetString("OBJECTIVES_MISSION01")); // .ste override
    printf("m02 #same  |%s|\n", SE_GetString("OBJECTIVES_MISSION02")); // -> cached english
    printf("numFlags=%d (masks persist; german entries carry 0)\n", SE_GetNumFlags());
    printf("m01 flags  %d (rebuilt entry, no FLAGS lines)\n", SE_GetFlags("OBJECTIVES_MISSION01"));

    printf("== SE_CheckForLanguageUpdates (cvar_take_modified flow) ==\n");
    Host_CvarSet("se_language", "english"); // sets modified=qtrue
    SE_CheckForLanguageUpdates();           // reload english, clear modified
    printf("m01 after update |%s|\n", SE_GetString("OBJECTIVES_MISSION01"));
    SE_CheckForLanguageUpdates();           // modified now false -> no-op
    printf("m01 second call  |%s| (no-op reload)\n", SE_GetString("OBJECTIVES_MISSION01"));

    printf("== SE_ShutDown : Clear(SE_FALSE) ==\n"); // flag tables cleared
    SE_ShutDown();
    printf("numFlags=%d (flags cleared)\n", SE_GetNumFlags());
}

// ===========================================================================
// Golden C — file-list scan + language enumeration (doc § "Golden C")
// ===========================================================================
static void mode_filelist_scan() {
    string results;
    int count = SE_BuildFileList("strings", results); // ext "/" subdirs vs ".str"
    printf("== SE_BuildFileList(\"strings\") ==\n");
    printf("count=%d\n", count);
    printf("results=|%s|\n", results.c_str()); // ';'-delimited, deterministic sort

    printf("== SE_GetNumLanguages (dedup, english-first) ==\n");
    int nl = SE_GetNumLanguages();
    printf("numLanguages=%d\n", nl);
    for (int i = 0; i < nl; i++) {
        LPCSTR name = SE_GetLanguageName(i);
        printf("LANG %d name=|%s| dir=|%s|\n", i, name, SE_GetLanguageDir(i));
    }
    // SE-V3/V4: out-of-range / negative index -> "" (release assert is a no-op)
    printf("name oor=|%s| neg=|%s|\n", SE_GetLanguageName(99), SE_GetLanguageName(-1));
    printf("dir  oor=|%s| neg=|%s|\n", SE_GetLanguageDir(99),  SE_GetLanguageDir(-1));
}

int main(int argc, char **argv) {
    if (argc < 2) {
        fprintf(stderr, "usage: %s <parse_lookup|reference_stability|filelist_scan> [fixtureRoot]\n", argv[0]);
        return 2;
    }
    Host_Init();
    Host_SetFixtureRoot(argc >= 3 ? argv[2] : "fixtures");

    std::string mode = argv[1];
    if (mode == "parse_lookup")            mode_parse_lookup();
    else if (mode == "reference_stability") mode_reference_stability();
    else if (mode == "filelist_scan")       mode_filelist_scan();
    else { fprintf(stderr, "unknown mode %s\n", argv[1]); return 2; }
    return 0;
}
