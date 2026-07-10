// roff-oracle stub — minimal q_shared.h subset the UNMODIFIED
// codemp/qcommon/RoffSystem.cpp (+ RoffSystem.h) references. Only the types,
// constants, vector macros, and free-function decls ROFF touches are provided;
// everything else in the real header is irrelevant to this TU. oracle/ untouched.
//
// Real definitions live in oracle/codemp/game/q_shared.h — cites below.
#ifndef __Q_SHARED_H
#define __Q_SHARED_H

#include <string.h>
#include <math.h>

// q_shared.h:1 typedef enum {qfalse, qtrue} qboolean;
typedef enum { qfalse, qtrue } qboolean;

typedef unsigned char byte;             // q_shared.h basic type

// q_shared.h:530-532
typedef float vec_t;
typedef vec_t vec3_t[3];

#ifndef NULL
#define NULL ((void *)0)
#endif

// q_shared.h angle indexes
#define PITCH 0
#define YAW   1
#define ROLL  2

// q_shared.h:548
#define M_PI 3.14159265358979323846f

#define MAX_QPATH 64                    // q_shared.h

// q_shared.h:1161-1163
#define S_COLOR_RED    "^1"
#define S_COLOR_GREEN  "^2"
#define S_COLOR_YELLOW "^3"

// q_shared.h:2644-2652 — trType_t (full faithful ordering; ROFF uses
// TR_STATIONARY / TR_LINEAR).
typedef enum {
	TR_STATIONARY,
	TR_INTERPOLATE,
	TR_LINEAR,
	TR_LINEAR_STOP,
	TR_NONLINEAR_STOP,
	TR_SINE,
	TR_GRAVITY
} trType_t;

// q_shared.h:2653-2660
typedef struct {
	trType_t trType;
	int      trTime;
	int      trDuration;
	vec3_t   trBase;
	vec3_t   trDelta;
} trajectory_t;

// q_shared.h:1363,1381,1380,1397 — the vector macros ROFF uses (macro form is
// bit-identical to the _Vector* function form under -ffp-contract=off).
#define VectorCopy(a,b)      ((b)[0]=(a)[0],(b)[1]=(a)[1],(b)[2]=(a)[2])
#define VectorScale(v,s,o)   ((o)[0]=(v)[0]*(s),(o)[1]=(v)[1]*(s),(o)[2]=(v)[2]*(s))
#define VectorMA(v,s,b,o)    ((o)[0]=(v)[0]+(b)[0]*(s),(o)[1]=(v)[1]+(b)[1]*(s),(o)[2]=(v)[2]+(b)[2]*(s))
#define VectorClear(a)       ((a)[0]=(a)[1]=(a)[2]=0)

#ifdef __cplusplus
extern "C" {
#endif

// q_shared.h:1623 / q_shared.c AngleVectors — reproduced faithfully in host.cpp.
void AngleVectors( const vec3_t angles, vec3_t forward, vec3_t right, vec3_t up );
// q_shared.h:1633 / q_shared.c
void COM_StripExtension( const char *in, char *out );
// qcommon Com_Printf / va — supplied by host.cpp.
void Com_Printf( const char *fmt, ... );
char *va( const char *fmt, ... );

#ifdef __cplusplus
}
#endif

#endif // __Q_SHARED_H
