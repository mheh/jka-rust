// pmworld.h -- the axial-brush world model + trace/pointcontents/snap stubs
// shared by BOTH the raw-trace dumper (main_trace.c) and the pmove dumper
// (main_pmove.c). This is the ONLY collision the pmove differential slice
// exercises; the Rust parity test (TestTraps) transcribes the identical
// algorithm.
//
// The trace is Q3's CM_ClipBoxToBrush restricted to axis-aligned box brushes
// (the general nonaxial-plane machinery is unnecessary because every brush is
// an AABB). This reproduces exactly the semantics pmove depends on:
// allsolid/startsolid, fraction, endpos, an axial plane.normal, per-brush
// surfaceFlags, contents=CONTENTS_SOLID, and entityNum = fraction<1 ?
// ENTITYNUM_WORLD : ENTITYNUM_NONE.
//
// *** BIT-IDENTITY RULES (must match the Rust port exactly) ***
//   (a) Every float literal carries the `f` suffix. A bare `0.125` promotes
//       the whole subexpression to double; the Rust f32 side would diverge.
//   (b) Only f32 + - * / and compares appear -- NO libm, fabs, sqrt, macros.
//       Axial normals are exact (0,+/-1) so no VectorNormalize is ever needed.
//   Combined with the harness's -ffp-contract=off, the result is
//   IEEE-deterministic on both the C and Rust sides.
//
// snap_vector is pinned to rintf() <-> f32::round_ties_even (may differ from
// the real jamp-engine SnapVector -- see README, engine-seam revisit).
#ifndef JAMPGAME_ORACLE_PMWORLD_H
#define JAMPGAME_ORACLE_PMWORLD_H

#include "q_shared.h"
#include "bg_public.h"
#include <math.h>   // rintf only (snap_vector); NEVER used inside the trace

#define PMW_SURFACE_CLIP_EPSILON 0.125f
#define PMW_MAX_BRUSHES 64

typedef struct {
	float mins[3];
	float maxs[3];
	int   surfaceFlags;
} pmw_brush_t;

static pmw_brush_t g_pmw_brushes[PMW_MAX_BRUSHES];
static int         g_pmw_numBrushes = 0;

// Trace-call tripwire: bumped on every pm_trace() call (dumped as `ntr`).
static long        g_pmw_traceCount = 0;

static void pmw_reset_world(void) {
	g_pmw_numBrushes = 0;
	g_pmw_traceCount = 0;
}

static void pmw_add_brush(float x0, float y0, float z0,
                          float x1, float y1, float z1, int surf) {
	pmw_brush_t *b;
	if (g_pmw_numBrushes >= PMW_MAX_BRUSHES) {
		fprintf(stderr, "pmw: too many brushes\n");
		exit(2);
	}
	b = &g_pmw_brushes[g_pmw_numBrushes++];
	b->mins[0] = x0; b->mins[1] = y0; b->mins[2] = z0;
	b->maxs[0] = x1; b->maxs[1] = y1; b->maxs[2] = z1;
	b->surfaceFlags = surf;
}

// The six outward axial face planes of one AABB brush, filled on demand.
// normal is exactly one of (+/-1,0,0),(0,+/-1,0),(0,0,+/-1); dist is the world
// plane distance so that normal . p == dist on the face.
static void pmw_brush_plane(const pmw_brush_t *b, int face,
                            float normal[3], float *dist, int *axis) {
	int a = face >> 1;          // 0,1,2
	int positive = !(face & 1); // even face = +axis, odd = -axis
	normal[0] = 0.0f; normal[1] = 0.0f; normal[2] = 0.0f;
	*axis = a;
	if (positive) {
		normal[a] = 1.0f;
		*dist = b->maxs[a];
	} else {
		normal[a] = -1.0f;
		*dist = -b->mins[a];
	}
}

// Q3 CM_ClipBoxToBrush for a single AABB brush; updates *tw in place. A `return`
// means "this brush does not clip the sweep" (advance to the next brush).
static void pmw_clip_box_to_brush(trace_t *tw,
                                  const float start[3], const float end[3],
                                  const float tw_mins[3], const float tw_maxs[3],
                                  const pmw_brush_t *brush) {
	int   face;
	float enterFrac = -1.0f;
	float leaveFrac = 1.0f;
	float clipNormal[3] = { 0.0f, 0.0f, 0.0f };
	float clipDist = 0.0f;
	int   clipAxis = 0;
	int   clipPositive = 0;
	int   getout = 0;
	int   startout = 0;

	for (face = 0; face < 6; face++) {
		float normal[3];
		float planeDist;
		int   axis;
		float ofs[3];
		float dist, d1, d2, f;

		pmw_brush_plane(brush, face, normal, &planeDist, &axis);

		ofs[0] = normal[0] < 0.0f ? tw_maxs[0] : tw_mins[0];
		ofs[1] = normal[1] < 0.0f ? tw_maxs[1] : tw_mins[1];
		ofs[2] = normal[2] < 0.0f ? tw_maxs[2] : tw_mins[2];

		// dist = planeDist - dot(ofs, normal); normal is axial so only ofs[axis]
		// contributes.
		dist = planeDist - (ofs[0] * normal[0] + ofs[1] * normal[1] + ofs[2] * normal[2]);

		d1 = (start[0] * normal[0] + start[1] * normal[1] + start[2] * normal[2]) - dist;
		d2 = (end[0]   * normal[0] + end[1]   * normal[1] + end[2]   * normal[2]) - dist;

		if (d2 > 0.0f) getout = 1;   // endpoint is not in solid
		if (d1 > 0.0f) startout = 1;

		// completely in front of this face -> no intersection with the brush
		if (d1 > 0.0f && (d2 >= PMW_SURFACE_CLIP_EPSILON || d2 >= d1)) {
			return;
		}
		// doesn't cross this plane -> plane is irrelevant
		if (d1 <= 0.0f && d2 <= 0.0f) {
			continue;
		}

		if (d1 > d2) {   // entering
			f = (d1 - PMW_SURFACE_CLIP_EPSILON) / (d1 - d2);
			if (f < 0.0f) f = 0.0f;
			if (f > enterFrac) {
				enterFrac = f;
				clipNormal[0] = normal[0];
				clipNormal[1] = normal[1];
				clipNormal[2] = normal[2];
				clipDist = planeDist;
				clipAxis = axis;
				clipPositive = (normal[axis] > 0.0f);
			}
		} else {         // leaving
			f = (d1 + PMW_SURFACE_CLIP_EPSILON) / (d1 - d2);
			if (f > 1.0f) f = 1.0f;
			if (f < leaveFrac) {
				leaveFrac = f;
			}
		}
	}

	if (!startout) {
		// original point was inside this brush
		tw->startsolid = 1;
		if (!getout) {
			tw->allsolid = 1;
			tw->fraction = 0.0f;
			tw->contents = CONTENTS_SOLID;
		}
		return;
	}

	if (enterFrac < leaveFrac) {
		if (enterFrac > -1.0f && enterFrac < tw->fraction) {
			if (enterFrac < 0.0f) enterFrac = 0.0f;
			tw->fraction = enterFrac;
			tw->plane.normal[0] = clipNormal[0];
			tw->plane.normal[1] = clipNormal[1];
			tw->plane.normal[2] = clipNormal[2];
			tw->plane.dist = clipDist;
			tw->plane.type = (byte)clipAxis;             // 0,1,2 = axial
			// signbits: bit set per negative normal component
			tw->plane.signbits = (byte)((clipNormal[0] < 0.0f ? 1 : 0)
			                          | (clipNormal[1] < 0.0f ? 2 : 0)
			                          | (clipNormal[2] < 0.0f ? 4 : 0));
			(void)clipPositive;
			tw->surfaceFlags = brush->surfaceFlags;
			tw->contents = CONTENTS_SOLID;
		}
	}
}

// Sweep an AABB [tw_mins,tw_maxs] from start to end through all brushes.
static void pm_trace_impl(trace_t *results,
                          const vec3_t start, const vec3_t mins,
                          const vec3_t maxs, const vec3_t end) {
	trace_t tr;
	int i;

	g_pmw_traceCount++;

	memset(&tr, 0, sizeof(tr));
	tr.fraction = 1.0f;

	for (i = 0; i < g_pmw_numBrushes; i++) {
		pmw_clip_box_to_brush(&tr, start, end, mins, maxs, &g_pmw_brushes[i]);
		if (tr.allsolid) {
			break;   // can't get any more solid
		}
	}

	if (tr.allsolid) {
		tr.startsolid = 1;
	}

	tr.endpos[0] = start[0] + tr.fraction * (end[0] - start[0]);
	tr.endpos[1] = start[1] + tr.fraction * (end[1] - start[1]);
	tr.endpos[2] = start[2] + tr.fraction * (end[2] - start[2]);

	if (tr.fraction < 1.0f) {
		tr.entityNum = ENTITYNUM_WORLD;
		tr.contents = CONTENTS_SOLID;
	} else {
		tr.entityNum = ENTITYNUM_NONE;
	}

	*results = tr;
}

// pmove_t.trace / .pointcontents function-pointer signatures.
static void pm_trace(trace_t *results, const vec3_t start, const vec3_t mins,
                     const vec3_t maxs, const vec3_t end,
                     int passEntityNum, int contentMask) {
	(void)passEntityNum; (void)contentMask;
	pm_trace_impl(results, start, mins, maxs, end);
}

static int pm_pointcontents(const vec3_t point, int passEntityNum) {
	int i;
	(void)passEntityNum;
	for (i = 0; i < g_pmw_numBrushes; i++) {
		const pmw_brush_t *b = &g_pmw_brushes[i];
		if (point[0] >= b->mins[0] && point[0] <= b->maxs[0] &&
		    point[1] >= b->mins[1] && point[1] <= b->maxs[1] &&
		    point[2] >= b->mins[2] && point[2] <= b->maxs[2]) {
			return CONTENTS_SOLID;
		}
	}
	return 0;
}

// trap_SnapVector: pinned rintf (round-to-nearest, ties-to-even) <->
// f32::round_ties_even on the Rust side.
static void pmw_snapvector(float *v) {
	v[0] = rintf(v[0]);
	v[1] = rintf(v[1]);
	v[2] = rintf(v[2]);
}

#endif // JAMPGAME_ORACLE_PMWORLD_H
