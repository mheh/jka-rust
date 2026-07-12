// ghoul2-server-oracle stub for codemp/game/q_shared.h
//
// Minimal shared types the unmodified ghoul2 server TUs (G2_bolts.cpp,
// G2_surfaces.cpp, the G2_API.cpp arena) + mdx_format.h / ghoul2_shared.h need.
// The full renderer/qcommon closure is deliberately NOT pulled; the engine seam
// is declared in ../qcommon/qcommon.h and implemented by the deterministic
// harness host (host.cpp). Seeded from tools/trmodel-oracle's stub (same shared
// surface). oracle/ is never edited.
#ifndef GHOUL2_ORACLE_Q_SHARED_H
#define GHOUL2_ORACLE_Q_SHARED_H

#include <string.h>
#include <stdio.h>
#include <stdlib.h>
#include <ctype.h>
#include <stdarg.h>
#include <math.h>
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
	TAG_MODEL_GLA,
	TAG_GHOUL2      // ghoul2 bone/surface scratch (G2_surfaces.cpp:371 etc.)
};

// ForceReload_e — q_shared.h:3167-3173 (verbatim ordinals).
typedef enum {
	eForceReload_NOTHING,
	eForceReload_MODELS,
	eForceReload_ALL
} ForceReload_e;

#define S_COLOR_RED    "^1" // q_shared.h:1161
#define S_COLOR_YELLOW "^3" // q_shared.h (§20 client-path Com_Printf strings)

// errorParm_t — verbatim ordinals (q_shared.h:451-457); ghoul2's arena New()
// calls Com_Error(ERR_FATAL, ...) on slot exhaustion (G2_API.cpp:388).
#ifdef ERR_FATAL
#undef ERR_FATAL
#endif
typedef enum {
	ERR_FATAL,
	ERR_DROP,
	ERR_SERVERDISCONNECT,
	ERR_DISCONNECT,
	ERR_NEED_CD
} errorParm_t;

// Eorientations — verbatim ordinals (q_shared.h:3086-3094). Named in the
// G2_local.h bone-angle prototypes (must be declared even though the bolt/
// surface/arena dumpers never call those bone entry points).
typedef enum Eorientations {
	ORIGIN = 0,
	POSITIVE_X,
	POSITIVE_Z,
	POSITIVE_Y,
	NEGATIVE_X,
	NEGATIVE_Z,
	NEGATIVE_Y
};

// orientation_t + vector helpers — only reached by the §20 client tag/bounds
// code (R_LerpTag/R_ModelBounds), which must compile but is never exercised.
typedef struct { vec3_t origin; vec3_t axis[3]; } orientation_t;
#define VectorCopy(a,b)   ((b)[0]=(a)[0],(b)[1]=(a)[1],(b)[2]=(a)[2])
#define VectorClear(a)    ((a)[0]=(a)[1]=(a)[2]=0)
#define VectorAdd(a,b,c)  ((c)[0]=(a)[0]+(b)[0],(c)[1]=(a)[1]+(b)[1],(c)[2]=(a)[2]+(b)[2]) // q_shared.h:1360
#define AxisClear(ax)     (VectorClear((ax)[0]),VectorClear((ax)[1]),VectorClear((ax)[2]))
vec_t VectorNormalize(vec3_t v);
static inline vec_t VectorLength(const vec3_t v) {          // q_shared.h:1460
	return (vec_t)sqrt(v[0]*v[0]+v[1]*v[1]+v[2]*v[2]);
}
float Com_Clamp(float min, float max, float value);        // q_shared.h:1630
void  Com_sprintf(char *dest, int size, const char *fmt, ...);

// QDECL — MSVC __cdecl calling-convention marker (q_shared.h:63); empty on g++.
// G2_API.cpp's static QsortDistance is declared `static int QDECL QsortDistance`.
#ifndef QDECL
#define QDECL
#endif

// cplane_t + trace_t — verbatim member sets (q_shared.h:1860-1878). G2_gore.h's
// SRagDollEffectorCollision holds a `const trace_t &`; only declared for compile.
typedef struct cplane_s {
	vec3_t	normal;
	float	dist;
	byte	type;
	byte	signbits;
	byte	pad[2];
} cplane_t;
typedef struct {
	byte		allsolid;
	byte		startsolid;
	short		entityNum;
	float		fraction;
	vec3_t		endpos;
	cplane_t	plane;
	int			surfaceFlags;
	int			contents;
} trace_t;

// CollisionRecord_t — verbatim (q_shared.h:1870-1884). Named in the G2_local.h
// collision prototypes (G2_TraceModels/G2API_CollisionDetect); the bolt/surface/
// arena dumpers never populate one, but the type must be declared.
typedef struct {
	float		mDistance;
	int			mEntityNum;
	int			mModelIndex;
	int			mPolyIndex;
	int			mSurfaceIndex;
	vec3_t		mCollisionPosition;
	vec3_t		mCollisionNormal;
	int			mFlags;
	int			mMaterial;
	int			mLocation;
	float		mBarycentricI;
	float		mBarycentricJ;
} CollisionRecord_t;
#define MAX_G2_COLLISIONS 16                              // q_shared.h:1886
typedef CollisionRecord_t G2Trace_t[MAX_G2_COLLISIONS];  // q_shared.h:1888

// SSkinGoreData (SSkinGoreData_s) — verbatim member set (q_shared.h:3112-...);
// the _G2_GORE arm names it in G2_TraceModels' signature. Only declared, never
// populated by the bolt/surface/arena slice.
typedef struct SSkinGoreData_s {
	vec3_t	angles, position;
	int		currentTime, entNum;
	vec3_t	rayDirection, hitLocation, scale;
	float	SSize, TSize, theta;
	int		growDuration;
	float	goreScaleStartFraction;
	qboolean frontFaces, backFaces, baseModelOnly;
	int		lifeTime, fadeOutTime, shrinkOutTime;
	float	alphaModulate;
	vec3_t	tint;
	float	impactStrength;
	int		shader, myIndex;
	qboolean fadeRGB;
} SSkinGoreData;

// IK-solver param blocks (q_shared.h sharedSetBoneIKStateParams_t /
// sharedIKMoveParams_t) — named only in the G2API_SetBoneIKState/IKMove
// prototypes the bolt/surface/arena TUs never call. Opaque forward types.
typedef struct sharedSetBoneIKStateParams_s sharedSetBoneIKStateParams_t;
typedef struct sharedIKMoveParams_s          sharedIKMoveParams_t;

// CMiniHeap — the transform-vert scratch heap (qcommon/MiniHeap.h); named as a
// pointer in G2_local.h transform prototypes. Forward-declare suffices for the
// bolt/surface/arena TUs (which never allocate from it).
class CMiniHeap;

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

#endif // GHOUL2_ORACLE_Q_SHARED_H
