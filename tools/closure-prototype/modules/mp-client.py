# client.h pulls tr_public/ui_public/keys/snd_public/cg_public/bg_public.
# keys.h -> ../ui/keycodes.h (MP keycodes are ui-owned). snd_local.h pulls
# vendored-but-parseable OpenAL headers + mp3struct.h (channel_t embeds
# MP3STREAM by value); its eax includes are patched out at parse time
# (windows COM; nothing swept embeds EAX types). Skipped: BinkVideo.h
# (vendored Bink SDK, Xbox), snd_local_console.h (Xbox),
# client/keycodes.h (Xbox orphan; PC uses ui/keycodes.h).

SPEC = dict(
    name="mp-client",
    lang="c++", entry=["codemp/game/q_shared.h", "codemp/qcommon/qcommon.h",
                       "codemp/client/client.h", "codemp/client/snd_local.h",
                       "codemp/client/snd_music.h", "codemp/client/snd_ambient.h",
                       "codemp/client/fffx.h", "codemp/client/FxScheduler.h",
                       "codemp/client/FXExport.h"],
    includes=["codemp/client", "codemp/game", "codemp/qcommon", "codemp/renderer",
              "codemp/ui", "codemp/cgame", "codemp",
              "../tools/closure-prototype/glshim"],
    defines=["NDEBUG", "MISSIONPACK", "_JK2",
             # The same win32 spellings mp-engine-ded supplies: client.h
             # reaches renderer and platform headers that assume an active
             # platform section. Pointer-size handles keep layout correct.
             "LPCTSTR=const char *", "COLORREF=unsigned int",
             "DWORD=unsigned int", "WORD=unsigned short",
             "BYTE=unsigned char", "HANDLE=void *", "LPVOID=void *",
             "stricmp=strcasecmp", "strnicmp=strncasecmp",
             "USHORT=unsigned short", "BOOL=int", "UINT=unsigned int",
             "FLOAT=float", "HDC=void *", "HGLRC=void *",
             "DECLARE_HANDLE(name)=typedef void *name"],
    flags=["-fdeclspec", "-fno-operator-names"],
    # The jamp-client transcription set (DEC-55, DEC-57; survey link set).
    # FX C++ TUs go the section-F design route, never blind packets.
    # Excluded as dead or replaced: 0_SH_Leak, the console/Xbox twins,
    # OpenAL/eax arms, mp3code/ (minimp3 replaces it), win32/ (winit/wgpu/
    # cpal supersede it, DEC-56).
    srcglob=["codemp/client/cl_main.cpp", "codemp/client/cl_parse.cpp",
             "codemp/client/cl_net_chan.cpp", "codemp/client/cl_input.cpp",
             "codemp/client/cl_keys.cpp", "codemp/client/cl_cgame.cpp",
             "codemp/client/cl_ui.cpp", "codemp/client/cl_console.cpp",
             "codemp/client/cl_scrn.cpp", "codemp/client/cl_cin.cpp",
             "codemp/client/snd_dma.cpp", "codemp/client/snd_mem.cpp",
             "codemp/client/snd_mix.cpp", "codemp/client/snd_mp3.cpp",
             "codemp/client/snd_music.cpp", "codemp/client/snd_ambient.cpp",
             "codemp/qcommon/cm_terrainmap.cpp",
             "codemp/qcommon/cm_draw.cpp"],

    # ---- chain fields (enginesweep / engineorder / enginepackets) ----
    label="client",
    sweep_title="mp-client (jamp client island)",
    sweep_desc=(
        "the `mp-client` profile (jamp client island compile set: 16 "
        "`codemp/client/*.cpp` TUs + `cm_terrainmap.cpp` + `cm_draw.cpp`; "
        "FX C++ TUs are design-track and stay out of the srcglob)"),
    subsystems=["qcommon", "client"],
    order=dict(
        # The client island TU list is the profile srcglob (DEC-55 scope law).
        extra_subsystems=[],
        md_title="mp-client (jamp client island)",
    ),
    packets=dict(
        # The engine rosetta carries the ported qshared/engine and wire types.
        # Client-only types have no rows yet and surface as report items.
        rosetta="out/engine/type-rosetta.tsv",
        digest="modules/mp-client-rulings-digest.md",
        digest_heading="## CLIENT ISLAND RULINGS DIGEST (verbatim)",
        parse_desc="the pinned per-file libclang parse of the `mp-client` "
                   "profile",
        # No frozen §F doc routes inside the island: the FX subsystem stays
        # out of the srcglob, and CDraw32/CTerrainMap methods surface as
        # undocumented-cpp referee items (their design home is DEC-55.4).
        doc={},
        doc_kind={},
        s20_files=set(),
        stringed_files=set(),
        trmodel_files=set(),
        rmg_folded=set(),
        ghoul2_classes=set(),
        gp2_dir="crates/mp/engine/qcommon/src/gp2/",
        gp2_classes=set(),
        crate_src={
            "qcommon": "crates/mp/engine/qcommon/src",
            "client": "crates/mp/engine/client/src",
        },
        # DEC-57 rule-20 drops: the OpenAL/EAX-only functions in snd_dma.cpp.
        drop_list="modules/mp-client-drops.json",
        # The island's callee surface is heavily already ported. Out-of-set
        # callees resolve to their real worktree signatures as LAW.
        law_from_tree=True,
        # Only a crate the client island links may supply a LAW signature.
        # The list order is the collision rank (DEC-32 canonical home first).
        law_crates=["crates/mp/engine/", "crates/mp/renderer",
                    "crates/native/", "crates/mp/qshared/",
                    "crates/mp/host-interface/", "crates/mp/abi/"],
    ),
)
