// pc_bridge.h — plain-C-linkage seam onto botlib's real (C++-compiled)
// preprocessor (l_precomp.cpp/l_script.cpp). ui_shared.c, q_shared.c,
// main.cpp and stubs.c all compile as plain C (mirroring
// jampgame-oracle/run_gcombat.sh's `cc`, so stubs.c's K&R argless-stub trick
// — the C linker binds by name alone, no C++ mangling — keeps working for
// whatever ui_shared.c references that this harness never exercises);
// l_precomp.cpp/l_script.cpp/l_memory.cpp keep their real .cpp dialect
// (compiled with `c++`). pc_bridge.cpp is the one TU compiled as C++ that
// calls the unmangled botlib entry points directly and re-exports them with
// C linkage so the C side can call them by plain name.
#ifndef UI_ORACLE_PC_BRIDGE_H
#define UI_ORACLE_PC_BRIDGE_H

#ifdef __cplusplus
extern "C" {
#endif

// Loads `data` (length `len`, source name `name`) via the UNMODIFIED
// LoadSourceMemory and installs it at l_precomp.cpp's own handle-table slot
// `handle` (its `sourceFiles[handle]`) — bypassing trap_PC_LoadSource's
// filesystem path entirely, matching the Rust test's LoadSourceMemory +
// direct `bot.sourceFiles[handle]` install.
void ui_oracle_install_source(int handle, char *data, int len, const char *name);

// Direct forwards onto PC_ReadTokenHandle / PC_SourceFileAndLine.
int ui_oracle_PC_ReadTokenHandle(int handle, void *pc_token);
int ui_oracle_PC_SourceFileAndLine(int handle, char *filename, int *line);

#ifdef __cplusplus
}
#endif

#endif
