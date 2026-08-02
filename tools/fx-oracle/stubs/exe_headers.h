// fx-oracle stub for `oracle/codemp/qcommon/exe_headers.h`.
//
// The real header pulls in `qcommon.h`, which drags the whole engine (netchan,
// the VM loader, the file system, the zone allocator) into every FX TU. The FX
// code needs six names out of that header, so the harness declares those and
// keeps the rest out.
// Source: `oracle/codemp/qcommon/exe_headers.h:1-5`
#ifndef FX_ORACLE_EXE_HEADERS_H
#define FX_ORACLE_EXE_HEADERS_H

#include "../game/q_shared.h"

#include <math.h>

// `Round` is an inline in the real qcommon.h. CFxScheduler::PlayEffect rounds
// the spawn count with it, so the body must stay identical.
// Source: `oracle/codemp/qcommon/qcommon.h:1094-1097`
inline int Round(float value)
{
	return ((int)floorf(value + 0.5f));
}

// The VM handle is opaque to the FX code: it only ever names `cgvm`.
typedef struct vm_s vm_t;
int VM_Call(vm_t *vm, int callNum, ...);

// The cvar and console surface the FX code reaches.
// Source: `oracle/codemp/qcommon/qcommon.h:697-712`
cvar_t *Cvar_Get(const char *var_name, const char *value, int flags);
void Com_Printf(const char *fmt, ...);
void Com_DPrintf(const char *fmt, ...);
void Com_Error(int level, const char *fmt, ...);

// CParticle::UpdateOrigin skips the point-contents probe on RMG maps. The
// harness leaves the cvar null, which takes the same branch a normal map does.
// Source: `oracle/codemp/client/FxPrimitives.cpp:232`
extern cvar_t *com_RMG;

// GenericParser2's text pool allocates through the zone.
// Source: `oracle/codemp/qcommon/qcommon.h:787,791`
void *Z_Malloc(int iSize, memtag_t eTag, qboolean bZeroit = qfalse, int iAlign = 4);
void Z_Free(void *ptr);

// The file surface CFxScheduler::RegisterEffect reads effect files through.
// Source: `oracle/codemp/qcommon/qcommon.h:591,596,600`
int FS_FOpenFileByMode(const char *qpath, fileHandle_t *f, fsMode_t mode);
int FS_Read2(void *buffer, int len, fileHandle_t f);
void FS_FCloseFile(fileHandle_t f);

#endif // FX_ORACLE_EXE_HEADERS_H
