// Stub q_shared.h for compiling the UNMODIFIED codemp RMG/terrain TUs
// (cm_terrain.cpp, cm_randomterrain.cpp, RM_Manager.cpp) standalone. Provides
// only the shared types, math macros, and prototypes those TUs reference. The
// vec math macros and PLANE_/PlaneTypeForNormal are transcribed verbatim from
// oracle/codemp/game/q_shared.h so the compiled behavior is bit-faithful.
#pragma once
#include <stdlib.h>
#include <string.h>
#include <strings.h>
#include <math.h>
#include <stdio.h>
#include <assert.h>

// --- case-insensitive compares: Raven ASCII behavior == strcasecmp
//     (fixtures/config keys are ASCII). Source: q_shared.c string helpers.
#define Q_stricmp   strcasecmp
#define Q_stricmpn  strncasecmp
#ifndef stricmp
#define stricmp     strcasecmp
#endif
#ifndef strnicmp
#define strnicmp    strncasecmp
#endif

typedef enum { qfalse = 0, qtrue } qboolean;
typedef unsigned char byte;

typedef float  vec_t;
typedef vec_t  vec2_t[2];
typedef vec_t  vec3_t[3];
typedef vec_t  vec4_t[4];
typedef vec_t  vec5_t[5];
typedef vec3_t vec3pair_t[2];
typedef int    ivec3_t[3];

// Raven `typedef int thandle_t`. Source: q_shared.h / native rosetta.
typedef int thandle_t;

#define MAX_QPATH        64
#define MAX_INFO_STRING  1024
#define BIG_INFO_STRING  8192
#define BIG_INFO_KEY     8192
#define BIG_INFO_VALUE   8192

// World bounds + plane constants. Source: oracle/codemp/game/q_shared.h:18-19,1844-1856
#define MAX_WORLD_COORD  ( 64 * 1024 )
#define MIN_WORLD_COORD  ( -64 * 1024 )
#define PLANE_X          0
#define PLANE_Y          1
#define PLANE_Z          2
#define PLANE_NON_AXIAL  3
#define PlaneTypeForNormal(x) (x[0] == 1.0 ? PLANE_X : (x[1] == 1.0 ? PLANE_Y : (x[2] == 1.0 ? PLANE_Z : PLANE_NON_AXIAL) ) )

// Vector math. Source: oracle/codemp/game/q_shared.h:1358-1399
#define DotProduct(x,y)               ((x)[0]*(y)[0]+(x)[1]*(y)[1]+(x)[2]*(y)[2])
#define VectorSubtract(a,b,c)         ((c)[0]=(a)[0]-(b)[0],(c)[1]=(a)[1]-(b)[1],(c)[2]=(a)[2]-(b)[2])
#define VectorAdd(a,b,c)              ((c)[0]=(a)[0]+(b)[0],(c)[1]=(a)[1]+(b)[1],(c)[2]=(a)[2]+(b)[2])
#define VectorCopy(a,b)               ((b)[0]=(a)[0],(b)[1]=(a)[1],(b)[2]=(a)[2])
#define VectorScale(v, s, o)          ((o)[0]=(v)[0]*(s),(o)[1]=(v)[1]*(s),(o)[2]=(v)[2]*(s))
#define VectorMA(v, s, b, o)          ((o)[0]=(v)[0]+(b)[0]*(s),(o)[1]=(v)[1]+(b)[1]*(s),(o)[2]=(v)[2]+(b)[2]*(s))
#define VectorSet(v, x, y, z)         ((v)[0]=(x), (v)[1]=(y), (v)[2]=(z))
#define VectorInc(v)                  ((v)[0] += 1.0f,(v)[1] += 1.0f,(v)[2] +=1.0f)
#define VectorDec(v)                  ((v)[0] -= 1.0f,(v)[1] -= 1.0f,(v)[2] -=1.0f)
#define VectorScaleVectorAdd(c,a,b,o) ((o)[0]=(c)[0]+((a)[0]*(b)[0]),(o)[1]=(c)[1]+((a)[1]*(b)[1]),(o)[2]=(c)[2]+((a)[2]*(b)[2]))
#define VectorInverseScaleVector(a,b,c) ((c)[0]=(a)[0]/(b)[0],(c)[1]=(a)[1]/(b)[1],(c)[2]=(a)[2]/(b)[2])
#define CrossProduct(a,b,c)           ((c)[0]=(a)[1]*(b)[2]-(a)[2]*(b)[1],(c)[1]=(a)[2]*(b)[0]-(a)[0]*(b)[2],(c)[2]=(a)[0]*(b)[1]-(a)[1]*(b)[0])
#define minimum(x,y) ((x)<(y)?(x):(y))
#define maximum(x,y) ((x)>(y)?(x):(y))
#define SURFACE_CLIP_EPSILON (0.125)

// Source: oracle/codemp/game/q_math.c:VectorLength / VectorNormalize (faithful).
static inline vec_t VectorLength( const vec3_t v ) {
	return (vec_t)sqrt( v[0]*v[0] + v[1]*v[1] + v[2]*v[2] );
}
static inline vec_t VectorNormalize( vec3_t v ) {
	float length = (float)sqrt( v[0]*v[0] + v[1]*v[1] + v[2]*v[2] );
	if ( length ) {
		float ilength = 1.0f / length;
		v[0] *= ilength; v[1] *= ilength; v[2] *= ilength;
	}
	return length;
}
static inline vec_t Distance( const vec3_t p1, const vec3_t p2 ) {
	vec3_t v; VectorSubtract( p2, p1, v ); return VectorLength( v );
}

// Raven `cplane_t`. Source: oracle/codemp/game/q_shared.h:1860-1866
typedef struct cplane_s {
	vec3_t normal;
	float  dist;
	byte   type;
	byte   signbits;
	byte   pad[2];
} cplane_t;

// Raven `trace_t`. Source: oracle/codemp/game/q_shared.h:1893-1912 (reduced to
// the fields the terrain TUs name; layout is not ABI-crossing here — §F).
typedef struct {
	byte     allsolid;
	byte     startsolid;
	short    entityNum;
	float    fraction;
	vec3_t   endpos;
	cplane_t plane;
	int      surfaceFlags;
	int      contents;
} trace_t;

// Raven `sphere_t` (named only inside traceWork_s; collision paths are
// dead-stripped, so only the shape needs to compile).
typedef struct {
	qboolean use;
	float    radius;
	float    halfheight;
	vec3_t   offset;
} sphere_t;

// Raven `cvar_t` (the terrain TUs only take the address of com_terrainPhysics /
// com_newtrace; the real fields are unused here).
typedef struct cvar_s {
	char *name;
	char *string;
	int   integer;
	float value;
} cvar_t;

void  SetPlaneSignbits( cplane_t *out );
char *Info_ValueForKey( const char *s, const char *key );
void          Rand_Init( int seed );
float         flrand( float min, float max );
int           irand( int min, int max );
unsigned long rng_state( void ); // harness accessor: raw holdrand (LCG anchor)
