// trmodel-oracle stub for codemp/game/q_shared.h
//
// Minimal shared types the unmodified tr_model.cpp / matcomp.c + mdx_format.h /
// sstring.h need. The full renderer/qcommon closure is deliberately NOT pulled;
// the engine seam is declared in ../qcommon/qcommon.h and implemented by the
// deterministic harness host (host.cpp). oracle/ is never edited.
#ifndef TRMODEL_ORACLE_Q_SHARED_H
#define TRMODEL_ORACLE_Q_SHARED_H

#include <string.h>
#include <stdio.h>
#include <stdlib.h>
#include <ctype.h>
#include <stdarg.h>
#include <assert.h>     // Raven's MSVC PCH makes assert ambient; NDEBUG -> no-op

typedef const char *LPCSTR;  // Win32 type used verbatim in tr_model.cpp:376,430

#ifdef __cplusplus
extern "C" {
#endif

typedef unsigned char byte;
typedef enum { qfalse, qtrue } qboolean;   // int-width, matches Raven
typedef int qhandle_t;

typedef float vec_t;
typedef vec_t vec2_t[2];
typedef vec_t vec3_t[3];
typedef vec_t vec4_t[4];

#define MAX_QPATH 64        // q_shared.h:393

// Endian helpers — identity on this little-endian host, matching the shipped
// x86 dedicated build the port models (LittleLong/Short/Float, TRM-D3).
static inline int    LittleLong(int x)    { return x; }
static inline short  LittleShort(short x) { return x; }
static inline float  LittleFloat(float x) { return x; }

// memtag_t is 1 byte (q_shared.h:3107). Only the model/filesys tags the live
// server pipeline touches are enumerated here (distinct keys for the zone map).
typedef char memtag_t;
enum {
	TAG_FREE = 0,
	TAG_FILESYS,
	TAG_MODEL_MD3,
	TAG_MODEL_GLM,
	TAG_MODEL_GLA
};

// ForceReload_e — q_shared.h:3167-3173 (verbatim ordinals).
typedef enum {
	eForceReload_NOTHING,
	eForceReload_MODELS,
	eForceReload_ALL
} ForceReload_e;

#define S_COLOR_RED    "^1" // q_shared.h:1161
#define S_COLOR_YELLOW "^3" // q_shared.h (§20 client-path Com_Printf strings)

#define ERR_DROP 1          // errorParm_t (q_shared.h:453); value unused by loader

// orientation_t + vector helpers — only reached by the §20 client tag/bounds
// code (R_LerpTag/R_ModelBounds), which must compile but is never exercised.
typedef struct { vec3_t origin; vec3_t axis[3]; } orientation_t;
#define VectorCopy(a,b)   ((b)[0]=(a)[0],(b)[1]=(a)[1],(b)[2]=(a)[2])
#define VectorClear(a)    ((a)[0]=(a)[1]=(a)[2]=0)
#define AxisClear(ax)     (VectorClear((ax)[0]),VectorClear((ax)[1]),VectorClear((ax)[2]))
vec_t VectorNormalize(vec3_t v);
void  Com_sprintf(char *dest, int size, const char *fmt, ...);

// cvar_t — only the fields the loader reads (->integer) plus registry bookkeeping.
typedef struct cvar_s {
	char	*name;
	char	*string;
	int		integer;
	float	value;
} cvar_t;

// string helpers implemented in host.cpp (q_shared.h:1706-1714).
int   Q_stricmp(const char *s1, const char *s2);
int   Q_stricmpn(const char *s1, const char *s2, int n);
char *Q_strlwr(char *s1);
void  Q_strncpyz(char *dest, const char *src, int destsize);
char *va(const char *format, ...);

#ifndef stricmp
#define stricmp strcasecmp
#endif

#ifdef __cplusplus
}
#endif

#endif // TRMODEL_ORACLE_Q_SHARED_H
