// roff-oracle stub — the server-spine seam ROFF reads/writes. Real defs live in
// oracle/codemp/server/server.h (+ game/g_public.h, game/q_shared.h); only the
// fields ApplyROFF/Play/ClearLerp/ProcessNote touch are modelled. Layout is the
// oracle's own internal representation (no ABI crossing here), so a minimal
// struct suffices. oracle/ untouched.
#ifndef SERVER_H_INC
#define SERVER_H_INC

// entityState_t — g_public.h references q_shared.h:2676-2677. ROFF touches only
// s.pos / s.apos.
typedef struct {
	trajectory_t pos;   // q_shared.h:2676
	trajectory_t apos;  // q_shared.h:2677
} entityState_t;

// entityShared_t — g_public.h:60-95. ROFF touches currentOrigin / currentAngles /
// mIsRoffing.
typedef struct {
	vec3_t   currentOrigin;   // g_public.h:79
	vec3_t   currentAngles;   // g_public.h:80
	qboolean mIsRoffing;      // g_public.h:81 — qtrue when the entity is being roffed
} entityShared_t;

// sharedEntity_t — g_public.h:685-715. ROFF touches s, r, next_roff_time.
typedef struct {
	entityState_t  s;               // g_public.h:685
	entityShared_t r;               // g_public.h:691
	int            next_roff_time;  // g_public.h:714 — npc's need to know when they're getting roff'd
} sharedEntity_t;

// GAME_ROFF_NOTETRACK_CALLBACK — g_public.h:766 (int entnum, char *notetrack).
// Only the numeric value matters to the dumper (host logs the callnum).
#define GAME_ROFF_NOTETRACK_CALLBACK 12

// serverStatic_t — server.h:211,232; ROFF reads svs.time.
typedef struct {
	int time;
} serverStatic_t;

// vm_t — opaque VM handle; ROFF passes gvm to VM_Call (server note-track arm).
typedef struct vm_s vm_t;

#ifdef __cplusplus
extern "C" {
#endif

extern serverStatic_t svs;                 // server.h:232
extern vm_t          *gvm;                  // the game VM handle

sharedEntity_t *SV_GentityNum( int num );   // server.h:349
int             VM_Call( vm_t *vm, int callNum, ... );

#ifdef __cplusplus
}
#endif

#endif // SERVER_H_INC
