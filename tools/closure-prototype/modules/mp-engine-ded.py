# DEDICATED-SERVER function sweep (WinDed.vcproj Release). Unlike mp-engine
# (header-only type sweep), this carries a srcglob so fnsweep.py can unity-
# parse whole-subsystem .cpp bodies. Defines mirror WinDed Release exactly:
#   WIN32,NDEBUG,_CONSOLE,DEDICATED,BOTLIB,_WINDOWS
# WIN32/_CONSOLE/_WINDOWS dropped per NOTES decision #1 (gate macros/asm,
# not layouts/bodies); DEDICATED+BOTLIB kept — DEDICATED is the whole point
# (it #ifndef's out the client/GL/sound halves of qcommon/server/renderer,
# leaving exactly the headless host + server-side model/G2 loading). NB the
# WinDed *Release* config oddly omits _JK2/MISSIONPACK (its Debug config has
# _JK2); kept faithful. The whole engine is C++ (.cpp), so lang=c++.
# srcglob is the WinDed compile set minus win32/null-device/vendored:
#   qcommon+server+ghoul2+botlib+icarus+RMG in full, plus the 9 renderer
#   sources WinDed links for server-side G2/model/shader loading
#   (tr_model/mesh/ghoul2/image/shader/init/main/backend + matcomp), which
#   compile down to their non-DEDICATED remainder. null_renderer/null_*
#   are the stub device layer (our Rust host supplies its own) — excluded.
# Renderer files pull tr_local.h->qgl.h->GL, so the glshim include dir +
# GL/win32 scalar-typedef defines from mp-renderer are merged in; -fdeclspec
# for tr_local's __declspec(align), -fno-operator-names for its `or` fields.

# The five FROZEN §F design docs (ruling 7 + the doc-session rulings). A function
# routed to a doc is NOT given a mechanical signature — the doc's Method-
# transcription table IS its work order (porting-rules §F). Classification is by
# the evidence in engine-port-order (subsystem dir / owning file / owner class),
# confirmed against each doc's own class coverage.
_DOC = {
    "icarus": "docs/subsystems/icarus.md",
    "rmg": "docs/subsystems/rmg-terrain.md",
    "ghoul2": "docs/subsystems/ghoul2-server.md",
    "npcnav": "docs/subsystems/npcnav.md",
    "roff": "docs/subsystems/roff.md",
    # Rulings 50/51 (2026-07-09): the sixth and seventh §F docs.
    "stringed": "docs/subsystems/stringed.md",
    "trmodel": "docs/subsystems/tr-model.md",
}

SPEC = dict(
    name="mp-engine-ded",
    lang="c++", entry="codemp/qcommon/qcommon.h",
    includes=["codemp/qcommon", "codemp/server", "codemp/botlib",
              "codemp/ghoul2", "codemp/icarus", "codemp/RMG",
              "codemp/renderer", "codemp/cgame", "codemp/game", "codemp",
              "../tools/closure-prototype/glshim"],
    defines=["NDEBUG", "DEDICATED", "BOTLIB",
             # win32 spellings the headers assume from an active platform
             # section (icarus tokenizer.h, RMG). Pointer-size handles keep
             # layout correct; only used where the sweep reads bodies.
             "LPCTSTR=const char *", "COLORREF=unsigned int",
             "DWORD=unsigned int", "WORD=unsigned short",
             "BYTE=unsigned char", "HANDLE=void *", "LPVOID=void *",
             # Raven leans on the MSVC case-insensitive str* spellings;
             # POSIX names them strcasecmp/strncasecmp (rescues RMG/icarus).
             "stricmp=strcasecmp", "strnicmp=strncasecmp",
             "USHORT=unsigned short", "BOOL=int", "UINT=unsigned int",
             "FLOAT=float", "HDC=void *", "HGLRC=void *",
             "DECLARE_HANDLE(name)=typedef void *name"],
    # -fdeclspec for __declspec(align); -fno-operator-names for `or` fields.
    # (q_shared SnapVector's MSVC __asm{} can't parse on an arm64 host —
    # -fasm-blocks needs an x86 target which would break 64-bit layout
    # parity — so clang drops that one header-inline and recovers; benign.)
    flags=["-fdeclspec", "-fno-operator-names"],
    srcglob=["codemp/qcommon/*.cpp", "codemp/server/*.cpp",
             "codemp/ghoul2/*.cpp", "codemp/botlib/*.cpp",
             "codemp/icarus/*.cpp", "codemp/RMG/*.cpp",
             "codemp/renderer/tr_model.cpp", "codemp/renderer/tr_mesh.cpp",
             "codemp/renderer/tr_ghoul2.cpp", "codemp/renderer/tr_image.cpp",
             "codemp/renderer/tr_shader.cpp", "codemp/renderer/tr_init.cpp",
             "codemp/renderer/tr_main.cpp", "codemp/renderer/tr_backend.cpp",
             "codemp/renderer/matcomp.c"],

    # ---- chain fields (enginesweep / engineorder / enginepackets) ----
    label="engine",
    sweep_title="mp-engine (DEDICATED server)",
    sweep_desc=(
        "the `mp-engine-ded` profile (WinDed.vcproj Release compile set: "
        "qcommon+server+ghoul2+botlib+icarus+RMG + the 9 model-loading "
        "renderer sources, `-DDEDICATED -DBOTLIB`)"),
    subsystems=["qcommon", "server", "ghoul2", "botlib", "icarus", "RMG",
                "renderer"],
    order=dict(
        # engineorder derives the per-file TU list from the vcproj link set,
        # not the srcglob (see the engineorder docstring, defects 3).
        vcproj="codemp/WinDed.vcproj",
        # The OS seam the Rust host implements natively (std::net, std::time,
        # module loader) — not ported 1:1, mirrors the existing Sys_*
        # externals stance.
        exclude_platform={"null/win_main.cpp", "win32/win_net.cpp",
                          "win32/win_shared.cpp"},
        # Vendored third-party code — supplied by Rust crates (flate2/png),
        # parity gated by pk3/png golden fixtures, per the established
        # vendored-code policy.
        exclude_vendored={"png/png.cpp", "zlib32/deflate.cpp",
                          "zlib32/inflate.cpp", "zlib32/zipcommon.cpp"},
        extra_subsystems=["null"],
        md_title="mp-engine (DEDICATED server)",
    ),
    packets=dict(
        rosetta="out/engine/type-rosetta.tsv",
        digest="docs/handoffs/engine-fork-discovery.md",
        digest_heading="## ENGINE FORK RULINGS (verbatim — all 48 settled)",
        parse_desc="the pinned per-file WinDed-Release libclang parse",
        doc=_DOC,
        # A §F doc pointer → the receiver kind of the state it owns. A C-track
        # fn that CALLS a doc-routed fn gains that kind PLUS `host` (rulings
        # 11/24: §F seam fns take `(&mut <Subsystem>, &mut dyn EngineHost, …)`).
        # GP2-routed callees (cpp-done) need no receiver (the GP2 reimpl
        # threads none). stringed folds into `common` (ruling 50), trmodel
        # into `rm` (ruling 51).
        doc_kind={
            _DOC["icarus"]: "icarus",
            _DOC["rmg"]: "rmg",
            _DOC["ghoul2"]: "g2",
            _DOC["npcnav"]: "nav",
            _DOC["roff"]: "roff",
            _DOC["stringed"]: "common",
            _DOC["trmodel"]: "rm",
        },
        # Ruling 49: CDraw32 (all of cm_draw.cpp) is §20-dropped — sole caller
        # CTerrainMap is header-only in the link set; addendum in rmg-terrain.md.
        s20_files={"cm_draw.cpp"},
        # Ruling 50/51 file scopes: whole-TU doc routing (free fns included —
        # the docs' Method-transcription tables are their work orders).
        stringed_files={"stringed_ingame.cpp", "stringed_interface.cpp"},
        # tr-model.md owns tr_model.cpp + matcomp.c live surface AND the
        # §20/§C10 classification of the DEDICATED-dead renderer TUs (ruling 54).
        trmodel_files={"tr_model.cpp", "matcomp.c", "tr_shader.cpp",
                       "tr_image.cpp", "tr_init.cpp", "tr_main.cpp",
                       "tr_mesh.cpp", "null_renderer.cpp"},
        # RMG qcommon terrain twins folded into rmg-terrain.md (ruling 16/28);
        # class set confirmed present in rmg-terrain.md.
        rmg_folded={"CCMLandScape", "CRandomTerrain", "CTerrainMap",
                    "CPathInfo", "CArea", "CCMPatch", "CCMHeightDetails",
                    "CCMShaderText"},
        # ghoul2 render internals (G2SV) confirmed in ghoul2-server.md.
        ghoul2_classes={"CBoneCache", "CTransformBone"},
        # GP2 is the DONE C++ pilot (porting-rules §F exemplar) — not a
        # docs/subsystems doc; its work order is the landed reimplementation.
        gp2_dir="crates/mp/engine/qcommon/src/gp2/",
        gp2_classes={"CGPGroup", "CGPValue", "CGPObject", "CGenericParser2",
                     "CTextPool"},
        # DESTINATION PATHS — one Rust module per oracle source file, at the
        # owning crate's src root, named by the oracle stem (`cm_load.cpp` →
        # `<root>/cm_load.rs`).
        crate_src={
            "qcommon": "crates/mp/engine/qcommon/src",
            "botlib": "crates/mp/engine/botlib/src",
            "server": "crates/mp/engine/server/src",
            "renderer": "crates/mp/renderer/src",
            "null": "crates/mp/engine/client/src/null",
        },
        # No rule-20 drop-list file: the engine's only §20 drop is the
        # cm_draw.cpp file route above (ruling 49).
        drop_list=None,
        # The engine is unported, so every signature derives from the clang
        # cursor. No worktree LAW scan.
        law_from_tree=False,
    ),
)
