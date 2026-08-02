SPEC = dict(
    name="mp-icarus",
    lang="c++", entry=["codemp/game/q_shared.h", "codemp/qcommon/qcommon.h",
                       # interface.h fn tables reference sharedEntity_t
                       "codemp/game/g_public.h",
                       "codemp/icarus/tokenizer.h", "codemp/icarus/blockstream.h",
                       "codemp/icarus/interpreter.h", "codemp/icarus/interface.h",
                       "codemp/icarus/sequence.h", "codemp/icarus/taskmanager.h",
                       "codemp/icarus/sequencer.h", "codemp/icarus/module.h",
                       "codemp/icarus/instance.h",
                       "codemp/icarus/icarus.h", "codemp/icarus/Q3_Interface.h",
                       "codemp/icarus/Q3_Registers.h", "codemp/icarus/GameInterface.h"],
    includes=["codemp/icarus", "codemp/game", "codemp/qcommon", "codemp"],
    # tokenizer.h uses the win32 LPCTSTR spelling; platform.h only defines
    # it under _WIN32, so supply it directly for layout purposes.
    defines=["NDEBUG", "MISSIONPACK", "_JK2", "LPCTSTR=const char *"],
)
