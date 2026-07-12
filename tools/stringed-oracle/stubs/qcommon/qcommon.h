// stringed-oracle stub for codemp/qcommon/qcommon.h
//
// Declares the engine services the two unmodified StringEd TUs call. All are
// implemented by the deterministic harness host (host.cpp): a fixture-backed
// virtual filesystem and an in-memory cvar registry. oracle/ is never edited.
#ifndef STRINGED_ORACLE_QCOMMON_H
#define STRINGED_ORACLE_QCOMMON_H

#include "../game/q_shared.h"

#ifdef __cplusplus
extern "C" {
#endif

// --- cvar system ---
cvar_t *Cvar_Get(const char *name, const char *value, int flags);

// --- console / fatal ---
void Com_Printf(const char *fmt, ...);
void Com_DPrintf(const char *fmt, ...);
void Com_Error(int level, const char *fmt, ...);

// --- misc ---
void COM_DefaultExtension(char *path, int maxSize, const char *extension);

// --- zone allocator (harness maps to malloc/free) ---
void *Z_Malloc(int size, int tag, qboolean zeroit);
void  Z_Free(void *ptr);

// --- filesystem (fixture-backed) ---
int    FS_ReadFile(const char *qpath, void **buffer);
void   FS_FreeFile(void *buffer);
char **FS_ListFiles(const char *directory, const char *extension, int *numfiles);
void   FS_FreeFileList(char **list);

#ifdef __cplusplus
}
#endif

#endif // STRINGED_ORACLE_QCOMMON_H
