# Same shape as mp-client; SP additionally has client_ui.h, vmachine.h
# (SP's vm_t), cl_mp3.h, cl_input_hotswap.h, and its keycodes.h lives in
# client/ (not ui/). Same eax patch + skips as MP.

SPEC = dict(
    name="sp-client",
    lang="c++", entry=["code/game/q_shared.h", "code/qcommon/qcommon.h",
                       "code/client/client.h", "code/client/client_ui.h",
                       "code/client/vmachine.h", "code/client/snd_local.h",
                       "code/client/cl_mp3.h", "code/client/snd_music.h",
                       "code/client/snd_ambient.h", "code/client/fffx.h",
                       "code/client/cl_input_hotswap.h"],
    includes=["code/client", "code/game", "code/qcommon", "code/renderer",
              "code/ui", "code/cgame", "code"],
    defines=["NDEBUG", "_IMMERSION"],
)
