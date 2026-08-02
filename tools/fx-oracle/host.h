// fx-oracle - the harness state the driver shares with the host seam.
#ifndef FX_ORACLE_HOST_H
#define FX_ORACLE_HOST_H

#include "../game/q_shared.h"

// The scripted FX clock, in milliseconds. Nothing in the harness reads a wall
// clock. `advance` moves it and hands the absolute value to FX_AdjustTime.
extern int fx_oracle_clock_ms;

// Prints a float as the 8-hex-digit IEEE-754 bit pattern. The return value
// points into a rotating buffer set, so one printf may hold many of these.
const char *fxf(float v);

// Seeds a cvar before `init`, the way a config file would.
cvar_t *fx_oracle_cvar_set(const char *name, const char *value);

// --- scripted reply queues --------------------------------------------------
//
// Each queue answers one engine question. A queue is FIFO, and its last entry
// repeats forever once the queue drains. An empty queue answers with the
// documented miss reply. README.md holds the miss values.

// One CG_TRACE / CG_G2TRACE reply.
void fx_oracle_push_trace(float fraction, const vec3_t endpos, const vec3_t normal,
	int startsolid, int allsolid, int surfaceFlags, int entityNum);

// One CG_POINT_CONTENTS reply.
void fx_oracle_push_pointcontents(int contents);

// One G2API_GetBoltMatrix reply. `exists` 0 means the bolt is not there.
void fx_oracle_push_bolt(int exists, const vec3_t origin, const vec3_t axis0,
	const vec3_t axis1, const vec3_t axis2);

// One CG_GET_LERP_ORIGIN reply.
void fx_oracle_push_lerporigin(const vec3_t origin);

#endif // FX_ORACLE_HOST_H
