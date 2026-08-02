// terrainmap-oracle stub: the precompiled-header umbrella cm_draw.cpp and
// cm_terrainmap.cpp include first. Declares only what those two TUs name.
// The oracle sources are never edited (porting-rules §F18).
#pragma once

#include <cassert>
#include <cmath>
#include <cstdio>
#include <cstdlib>
#include <cstring>

typedef unsigned char byte;

typedef enum { qfalse, qtrue } qboolean;

typedef float vec_t;
typedef vec_t vec3_t[3];

#define MAX_QPATH 64

// windows.h POINT; cm_draw.h keeps the windows.h include commented out, so the
// real MP build gets this from the precompiled header.
typedef struct tagPOINT {
	long x;
	long y;
} POINT;

// Memory: the harness backs Raven's zone with the C allocator.
void *Z_Malloc(int size, int tag, qboolean zeroit);
void Z_Free(void *ptr);

char *va(const char *format, ...);

void RotatePointAroundVector(vec3_t dst, const vec3_t dir, const vec3_t point, float degrees);
