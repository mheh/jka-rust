// Stub cm_local.h — the real clipmap-local header pulls the whole collision
// world; the terrain TUs only reference the brush/shader collision structs and
// a few clipmap-owned collision entry points (which live on the dead-stripped
// PatchCollide/WaterCollide paths under DEDICATED). Types transcribed from
// oracle/codemp/qcommon/cm_local.h (§18, oracle never edited).
#pragma once
#include "q_shared.h"

// Raven cbrushside_s / cbrush_s (non-_XBOX). Source: cm_local.h:60-75
typedef struct cbrushside_s {
	cplane_t *plane;
	int       shaderNum;
} cbrushside_t;

typedef struct cbrush_s {
	int            shaderNum;
	int            contents;
	vec3_t         bounds[2];
	cbrushside_t  *sides;
	unsigned short numsides;
	unsigned short checkcount;
} cbrush_t;

// Raven CCMShader — the wider-clipmap shader record CM_GetShaderInfo returns.
// LoadTerrainDef reads contentFlags/surfaceFlags. Source: cm_local.h:77-89
class CCMShader {
public:
	char             shader[MAX_QPATH];
	class CCMShader *mNext;
	int              surfaceFlags;
	int              contentFlags;

	const char      *GetName( void ) const { return shader; }
	class CCMShader *GetNext( void ) const { return mNext; }
	void             SetNext( class CCMShader *next ) { mNext = next; }
	void             Destroy( void ) {}
};

// The SETTLED extern binding LoadTerrainDef reaches (RMG-D5 / ruling 41). Owned
// by the cm C-track packet, NOT ported by the RMG doc. Here it is the harness
// stub (see src/rmg_host_stubs.cpp for the contract). Source: cm_local.h:303
CCMShader *CM_GetShaderInfo( const char *name );

// Collision entry points named by the dead-stripped PatchCollide (In-scope item
// 4 lives on CollisionWorld in the Rust port; under DEDICATED with no trace
// caller these are dead-stripped in the harness). Source: cm_public.h:56-57
struct traceWork_s;
class CCMPatch;
void CM_HandlePatchCollision( struct traceWork_s *tw, trace_t &trace, const vec3_t tStart, const vec3_t tEnd, class CCMPatch *patch, int checkcount );
void CM_CalcExtents( const vec3_t start, const vec3_t end, const struct traceWork_s *tw, vec3pair_t bounds );

// traceWork_s — named by PatchCollide/WaterCollide (dead-stripped). Source:
// cm_local.h:238-264 (reduced to the fields the terrain TU names).
typedef struct traceWork_s {
	vec3_t       start;
	vec3_t       end;
	vec3_t       size[2];
	vec3_t       offsets[8];
	float        maxOffset;
	vec3_t       extents;
	vec3_t       modelOrigin;
	int          contents;
	qboolean     isPoint;
	sphere_t     sphere;
	vec3pair_t   bounds;
	vec3pair_t   localBounds;
	float        baseEnterFrac;
	float        baseLeaveFrac;
	float        enterFrac;
	float        leaveFrac;
	cbrushside_t *leadside;
	cplane_t     *clipplane;
	bool         startout;
	bool         getout;
} traceWork_t;
