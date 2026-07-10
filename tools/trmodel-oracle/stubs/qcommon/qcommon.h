// trmodel-oracle stub for codemp/qcommon/qcommon.h
//
// Declares the engine services the unmodified tr_model.cpp calls. All are
// implemented by the deterministic harness host (host.cpp): a fixture-backed
// filesystem (FS_ReadFile from fixtures/, FS_FileIsInPAK from a pak-checksum
// fixture map honouring the 1/-1 convention, ruling 59), a zone allocator that
// tracks per-tag byte sums (Z_MemSize parity), a cvar registry, and captured
// console output. oracle/ is never edited.
#ifndef TRMODEL_ORACLE_QCOMMON_H
#define TRMODEL_ORACLE_QCOMMON_H

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

#endif // TRMODEL_ORACLE_QCOMMON_H
