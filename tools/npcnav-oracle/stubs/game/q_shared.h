// npcnav-oracle stub for oracle/codemp/game/q_shared.h.
//
// §18 discipline: the oracle navigator.cpp/.h TU is compiled UNMODIFIED; this
// stub header supplies only the types/constants/macros/externs that TU touches,
// so we never edit oracle/. Mirrors the gp2-oracle stub arrangement.
//
// NAV-D1 / RULING 44 — the 4-byte-`long` shim.
// -------------------------------------------------------------------------
// Raven's `.nav` format was authored on Win32 where `long`/`unsigned long`
// are 4 bytes. This host is LP64 (`long` == 8 bytes), which would double the
// width of every nav/node header id in the emitted file. We must NOT edit the
// oracle source, so the shim is a compile-time arrangement (the same class of
// mechanism as the pinned-parse `LittleShort=` flags — flags/stub-headers are
// not source edits):
//
//   * All standard-library and system headers this TU needs are pulled in
//     HERE, at the top of the first include, BEFORE the macro is armed. Every
//     later `#include <...>` in navigator.cpp/.h is then an include-guard
//     no-op, so no libstdc++/libc header is ever parsed with the macro active
//     (all their `long` typedefs — ptrdiff_t, size_t, time_t — are frozen).
//   * `#define long int` at the very END of this header rewrites the bare
//     `long` KEYWORD tokens to `int` for exactly the code parsed afterwards:
//     navigator.h + navigator.cpp. The only bare-`long` tokens in that code
//     are the six `.nav`-format sites (navigator.cpp:388,428,557,559,614,676
//     — the NAV/NODE header ids and GetLong) plus the GetLong decl
//     (navigator.h:233). Every one is a field that is FS_Read/FS_Write'd, so
//     making them 4 bytes is exactly the retail on-disk width. (The lone other
//     `long` at navigator.cpp:1992 is inside a comment — preprocessed away.)
//
// Result: the emitted NAV id occupies 4 bytes, not 8; goldens, retail pk3
// `.nav` files, and the OpenJK referee all agree; the Rust port pins these
// fields to i32/u32. main.cpp asserts the width from the emitted bytes.
#ifndef NPCNAV_ORACLE_Q_SHARED_STUB
#define NPCNAV_ORACLE_Q_SHARED_STUB

// --- system/STL headers FIRST, before the long-shim is armed ---------------
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>
#include <time.h>
#include <assert.h>
#include <stdint.h>
#include <algorithm>
#include <map>
#include <vector>
#include <list>

// --- primitive types -------------------------------------------------------
typedef unsigned char byte;
typedef unsigned char BYTE;

typedef enum { qfalse, qtrue } qboolean;

typedef float  vec_t;
typedef vec_t  vec3_t[3];
typedef vec_t  vec4_t[4];
typedef vec_t  vec5_t[5];

typedef int fileHandle_t;

// Raven fsMode_t (files.h) — only READ/WRITE are exercised.
typedef enum { FS_READ, FS_WRITE, FS_APPEND, FS_APPEND_SYNC } fsMode_t;

typedef enum { ERR_FATAL, ERR_DROP, ERR_SERVERDISCONNECT, ERR_DISCONNECT, ERR_NEED_CD } errorParm_t;

// --- constants the TU references -------------------------------------------
#define MAX_WORLD_COORD   ( 64 * 1024 )
#define MIN_WORLD_COORD   ( -64 * 1024 )
#define WORLD_SIZE        ( MAX_WORLD_COORD - MIN_WORLD_COORD )
#define STEPSIZE          18
#define Q3_INFINITE       16777216

// gentity indexing (used only in trace/PVS paths, all stubbed) --------------
#define MAX_GENTITIES        1024
#define ENTITYNUM_NONE       ( MAX_GENTITIES - 1 )
#define ENTITYNUM_WORLD      ( MAX_GENTITIES - 2 )
#define MAX_FAILED_NODES     8

// content/trace masks (values only need to exist; SV_Trace is a clear stub) --
#define CONTENTS_SOLID        0x00000001
#define CONTENTS_BODY         0x02000000
#define CONTENTS_BOTCLIP      0x00040000
#define CONTENTS_MONSTERCLIP  0x00100000
#define CONTENTS_TERRAIN      0x40000000
#define MASK_SOLID            ( CONTENTS_SOLID | CONTENTS_TERRAIN )
#define MASK_NPCSOLID         ( CONTENTS_SOLID | CONTENTS_MONSTERCLIP | CONTENTS_BODY | CONTENTS_TERRAIN )

#define CVAR_CHEAT            512

// Constants referenced only by the trace/PVS/failed-node/edge paths, which are
// compiled but never executed by the fixture generator. Values match the
// oracle headers (harmless, keeps the TU honest).
#define MAX_STORED_WAYPOINTS  512
#define DEFAULT_MINS_2        -24
#define DEFAULT_MAXS_2        40
#define S_COLOR_RED           "^1"
#define ET_PLAYER             1     // bg_public.h eType enum
#define ET_NPC                13
#define EF_DEAD               (1<<1)

// serverStatic_t frame clock (server.h:232). Only the recheck-timer arms read
// svs.time; stubbed to a fixed value in main.cpp.
extern struct serverStatic_stub { int time; } svs;

// STL iteration macros. Raven defines these in icarus/icarus.h; navigator.cpp
// uses them but the real TU picks them up through its (much larger) include
// graph. Reproduced verbatim from oracle/codemp/icarus/icarus.h:16-17.
#define STL_ITERATE( a, b )   for ( a = b.begin(); a != b.end(); a++ )
#define STL_INSERT( a, b )    a.insert( a.end(), b );

// --- vector helpers (pure; exercised by HardConnect/GetProjectedNode) -------
#define DotProduct(a,b)       ((a)[0]*(b)[0]+(a)[1]*(b)[1]+(a)[2]*(b)[2])
#define VectorSubtract(a,b,c) ((c)[0]=(a)[0]-(b)[0],(c)[1]=(a)[1]-(b)[1],(c)[2]=(a)[2]-(b)[2])
#define VectorCopy(a,b)       ((b)[0]=(a)[0],(b)[1]=(a)[1],(b)[2]=(a)[2])
#define VectorSet(v,x,y,z)    ((v)[0]=(x),(v)[1]=(y),(v)[2]=(z))

static inline float VectorNormalize( vec3_t v ) {
    float length = (float)sqrt( DotProduct( v, v ) );
    if ( length ) {
        float ilength = 1.0f / length;
        v[0] *= ilength; v[1] *= ilength; v[2] *= ilength;
    }
    return length;
}
static inline float Distance( const vec3_t p1, const vec3_t p2 ) {
    vec3_t v; VectorSubtract( p2, p1, v );
    return (float)sqrt( DotProduct( v, v ) );
}
static inline float DistanceSquared( const vec3_t p1, const vec3_t p2 ) {
    vec3_t v; VectorSubtract( p2, p1, v );
    return DotProduct( v, v );
}

// --- ABI structs the TU dereferences ---------------------------------------
// Minimal shapes: the ent-taking nav methods COMPILE against these but are
// never executed by the fixture generator (trace/PVS/gentity paths are 3c).
typedef struct { int integer; float value; char *string; } cvar_t;

typedef struct {
    qboolean allsolid;
    qboolean startsolid;
    float    fraction;
    vec3_t   endpos;
    int      entityNum;
} trace_t;

typedef struct { int number; int eType; int eFlags; } entityState_t;
typedef struct { vec3_t currentOrigin; vec3_t mins; vec3_t maxs; } entityShared_t;

typedef struct sharedEntity_s {
    entityState_t  s;
    entityShared_t r;
    void          *client;
    int            clipmask;
    int            health;
    qboolean       inuse;
    int            waypoint;
    int            failedWaypoints[MAX_FAILED_NODES];
    int            failedWaypointCheckTime;
} sharedEntity_t;

// failedEdge_t — shared game/engine struct; its exact 16-byte layout is part
// of the `.nav` failed-edge block (Save writes the 32-entry array raw).
typedef struct failedEdge_e { int startID; int endID; int checkTime; int entID; } failedEdge_t;

// --- engine services (defined in main.cpp) ---------------------------------
#ifdef __cplusplus
extern "C" {
#endif
int   FS_FOpenFileByMode( const char *qpath, fileHandle_t *f, fsMode_t mode );
int   FS_Read( void *buffer, int len, fileHandle_t f );
int   FS_Write( const void *buffer, int len, fileHandle_t f );
void  FS_FCloseFile( fileHandle_t f );

cvar_t *Cvar_Get( const char *name, const char *value, int flags );

void  Com_Error( int level, const char *fmt, ... );
void  Com_Printf( const char *fmt, ... );
char *va( const char *format, ... );

int   Q_irand( int minVal, int maxVal );
#ifdef __cplusplus
}
#endif

// SV_* + VM_Call are declared where server.h is included; kept here too so the
// TU sees them regardless of include order. (All stubbed in main.cpp.)
sharedEntity_t *SV_GentityNum( int num );
qboolean        SV_inPVS( const vec3_t p1, const vec3_t p2 );
void            SV_Trace( trace_t *results, const vec3_t start, const vec3_t mins, const vec3_t maxs,
                          const vec3_t end, int passEntityNum, int contentmask, int capsule,
                          int traceFlags, int useLod );

// -------------------------------------------------------------------------
// NAV-D1 / RULING 44: arm the 4-byte-`long` shim. MUST be the last thing in
// this header — everything below (navigator.h + navigator.cpp) is parsed with
// bare `long` == `int`. See the file header for the full rationale.
#define long int

#endif // NPCNAV_ORACLE_Q_SHARED_STUB
