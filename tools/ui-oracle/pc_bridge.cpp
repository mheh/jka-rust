// pc_bridge.cpp — compiled as C++ (matches l_precomp.cpp/l_script.cpp's real
// dialect); see pc_bridge.h for why this file exists.
#include <cstdlib>

#include "q_shared.h"
#include "l_script.h"
#include "l_precomp.h"
#include "pc_bridge.h"

// l_precomp.cpp's handle table; declared there as a plain (non-static)
// global but not re-declared in l_precomp.h.
extern source_t *sourceFiles[];

// l_precomp.cpp's DEFINEHASHING-mode global-defines bucket table (also a
// plain global not re-declared in l_precomp.h). LoadSourceFile lazily
// allocates it before calling PC_AddGlobalDefinesToSource
// (l_precomp.cpp:3046-3051); LoadSourceMemory does NOT — a genuine Raven
// asymmetry bug (LoadSourceMemory used standalone, with no prior
// LoadSourceFile call, null-derefs inside PC_AddGlobalDefinesToSource's
// `globaldefines[i]` scan). Since this harness never calls LoadSourceFile
// and never registers any global define, replicating LoadSourceFile's exact
// lazy allocation here (matching its zeroed-bucket-table content byte for
// byte) sidesteps the crash without changing observable behavior — the
// oracle §19 UB-divergence pattern (diverge only where Raven is UB, pick
// the one defined behavior, note it here).
#define DEFINEHASHSIZE 1024 // l_precomp.cpp's own (unexported) constant
extern define_t **globaldefines;

void ui_oracle_install_source(int handle, char *data, int len, const char *name) {
	if (!globaldefines) {
		globaldefines = (define_t **)calloc(DEFINEHASHSIZE, sizeof(define_t *));
	}
	source_t *source = LoadSourceMemory(data, len, (char *)name);
	sourceFiles[handle] = source;
}

int ui_oracle_PC_ReadTokenHandle(int handle, void *pc_token) {
	return PC_ReadTokenHandle(handle, (pc_token_t *)pc_token);
}

int ui_oracle_PC_SourceFileAndLine(int handle, char *filename, int *line) {
	return PC_SourceFileAndLine(handle, filename, line);
}

// l_precomp.cpp's PC_PrintDefineHashTable calls this (dead: only reachable
// via a commented-out debug call, never invoked from any live PC_* path);
// a real (empty) definition is simpler than fighting stubs.c's C-linkage
// trick for one C++-mangled variadic symbol.
#include <cstdarg>
void Log_Write(char *fmt, ...) { (void)fmt; }
