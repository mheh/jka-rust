// fx-oracle stub for `oracle/codemp/client/client.h`.
//
// The real header declares the whole client: the snapshot ring, the key state,
// the console, the UI and cgame VMs, the sound cache and the renderer import.
// The six FX translation units reach eight names out of it, so the harness
// declares those and keeps the rest out.
//
// Every declaration below keeps the oracle signature. The bodies live in
// host.cpp.
#ifndef FX_ORACLE_CLIENT_H
#define FX_ORACLE_CLIENT_H

#include "../game/q_shared.h"
#include "../cgame/tr_types.h"
#include "../cgame/cg_public.h"

// The two content masks the FX traces use. `bg_public.h` owns them, and that
// header drags in the weapon, animation and vehicle tables.
// Source: `oracle/codemp/game/bg_public.h:1171-1172`
#define MASK_SOLID (CONTENTS_SOLID | CONTENTS_TERRAIN)
#define MASK_PLAYERSOLID (CONTENTS_SOLID | CONTENTS_PLAYERCLIP | CONTENTS_BODY | CONTENTS_TERRAIN)

// The one field of `clientActive_t` the FX code touches. Every cgame trap the
// FX system makes marshals its arguments through this block.
// Source: `oracle/codemp/client/client.h:136`
typedef struct {
	char *mSharedMemory;
} fxOracleClientActive_t;

extern fxOracleClientActive_t cl;

// The cgame VM handle. `VM_Call` in the harness dispatches on the trap number
// and never looks at the handle.
// Source: `oracle/codemp/client/client.h:392`
extern vm_t *cgvm;

// The renderer import, cut down to the eight entry points the FX code calls.
// Source: `oracle/codemp/renderer/tr_public.h:30-65`
typedef struct {
	qhandle_t (*RegisterModel)(const char *name);
	qhandle_t (*RegisterShader)(const char *name);
	void (*AddRefEntityToScene)(const refEntity_t *re);
	void (*AddMiniRefEntityToScene)(const miniRefEntity_t *re);
	void (*AddPolyToScene)(qhandle_t hShader, int numVerts, const polyVert_t *verts, int num);
	void (*AddDecalToScene)(qhandle_t shader, const vec3_t origin, const vec3_t dir, float orientation,
		float r, float g, float b, float a, qboolean alphaFade, float radius, qboolean temporary);
	void (*AddLightToScene)(const vec3_t org, float intensity, float r, float g, float b);
	void (*DrawStretchPic)(float x, float y, float w, float h,
		float s1, float t1, float s2, float t2, qhandle_t hShader);
} fxOracleRefExport_t;

extern fxOracleRefExport_t re;

// The sound surface `SFxHelper` wraps.
// Source: `oracle/codemp/client/snd_public.h:11-12,57`
//
// Raven drops the volume and radius arguments inside `SFxHelper::PlaySound`
// (`oracle/codemp/client/FxSystem.h:91-95`), so nothing reaches the sound seam
// with them. The two default arguments make that loss visible in the golden:
// a `SOUND` record always carries `volume -1 radius -1`.
void S_StartSound(const vec3_t origin, int entnum, int entchannel, sfxHandle_t sfx,
	int volume = -1, int radius = -1);
void S_StartLocalSound(sfxHandle_t sfx, int channelNum);
sfxHandle_t S_RegisterSound(const char *sample);

#endif // FX_ORACLE_CLIENT_H
