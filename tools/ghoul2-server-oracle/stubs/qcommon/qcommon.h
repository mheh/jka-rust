// ghoul2-server-oracle stub (seeded from tools/trmodel-oracle) for codemp/qcommon/qcommon.h
//
// Declares the qcommon engine seam the unmodified ghoul2 server TUs reference.
// Only the handful actually reached on the arena/bolt/surface code paths get a
// body in host.cpp (Com_Error, Com_Printf); every other declaration here is
// compile-only — the code that would call it (FS, zone, cvar registry) is
// dead-stripped at link (-Wl,-dead_strip), so no host implementation is needed.
// oracle/ is never edited.
#ifndef GHOUL2_ORACLE_QCOMMON_H
#define GHOUL2_ORACLE_QCOMMON_H

#include "../game/q_shared.h"

#ifdef __cplusplus
extern "C" {
#endif

// --- console ---
void Com_Printf(const char *fmt, ...);
void Com_DPrintf(const char *fmt, ...);
void Com_Error(int level, const char *fmt, ...);

// --- cvar system ---
cvar_t *Cvar_Get(const char *name, const char *value, int flags);

// Engine globals G2_API.cpp reads to decide server-vs-client registration
// (G2_ShouldRegisterServer, G2_API.cpp:570-584). Owned by the harness host;
// the arena dumper leaves them at their headless-server defaults.
typedef struct vm_s vm_t;
extern vm_t   *gvm;             // G2_API.cpp:23
extern vm_t   *currentVM;       // G2_API.cpp:24
extern cvar_t *com_cl_running;  // G2_API.cpp:574
extern cvar_t *com_dedicated;   // G2_API.cpp:577

// --- zone allocator (per-tag byte sums; Z_MemSize == sum of live iAllocSize) ---
void *Z_Malloc(int iSize, memtag_t eTag, qboolean bZeroit);
void  Z_Free(void *ptr);
void  Z_MorphMallocTag(void *pvBuffer, memtag_t eDesiredTag);
int   Z_MemSize(memtag_t eTag);

// --- hunk (harness maps to malloc; the dumper is short-lived) ---
#define h_low 0
void *Hunk_Alloc(int size, int preference);

// --- filesystem (fixture-backed) ---
int    FS_ReadFile(const char *qpath, void **buffer);
void   FS_FreeFile(void *buffer);
int    FS_FileIsInPAK(const char *filename, int *pChecksum);   // 1 in-pure-pak, else -1

#ifdef __cplusplus
}
#endif

#endif // GHOUL2_ORACLE_QCOMMON_H
