// fx-oracle - deterministic host and capture sinks for the Raven FX TUs.
//
// The unmodified oracle TUs (FxSystem.cpp, FxScheduler.cpp, FxPrimitives.cpp,
// FxTemplate.cpp, FxUtil.cpp, FXExport.cpp) reach the engine seam through
// `SFxHelper`. This file supplies that seam. Every outbound call prints one
// record to stdout at the moment the FX code makes it, so a golden is the
// emission stream in call order.
//
// Four rules keep every golden run-twice byte-identical:
//   1. No wall clock. `fx_oracle_clock_ms` only moves when the script says so.
//   2. No pointer, address or size value ever enters a record.
//   3. Every float prints as its raw IEEE-754 bit pattern, never as %f.
//   4. Every engine answer (trace, point contents, bolt, lerp origin) comes
//      from a scripted queue, never from a simulation.
//
// oracle/ is never edited.
#include "../qcommon/exe_headers.h"
#include "client.h"
#include "../ghoul2/G2.h"
#include "../ghoul2/G2_local.h"
#include "host.h"

#include <cstdio>
#include <cstdlib>
#include <cstdarg>
#include <cstring>

// --- the scripted clock -----------------------------------------------------

int fx_oracle_clock_ms = 0;

// --- float printing ---------------------------------------------------------

// A rotating buffer set. One record prints up to twenty floats, and 64 slots
// leave headroom for the widest one (REFENT).
#define FX_ORACLE_FLOAT_SLOTS 64

const char *fxf(float v)
{
	static char slots[FX_ORACLE_FLOAT_SLOTS][16];
	static int next = 0;

	unsigned int bits;
	memcpy(&bits, &v, sizeof(bits));

	char *out = slots[next];
	next = (next + 1) % FX_ORACLE_FLOAT_SLOTS;
	snprintf(out, sizeof(slots[0]), "%08x", bits);
	return out;
}

// --- the console ------------------------------------------------------------

// `SFxHelper::Print` formats into a buffer and hands the result to Com_DPrintf,
// so both console entry points land here. One call prints one PRINT record with
// the trailing newline stripped.
static void fx_oracle_print_record(const char *fmt, va_list ap)
{
	char text[2048];
	vsnprintf(text, sizeof(text), fmt, ap);

	size_t len = strlen(text);
	while (len > 0 && (text[len - 1] == '\n' || text[len - 1] == '\r')) {
		text[--len] = 0;
	}
	printf("PRINT %s\n", text);
}

void Com_Printf(const char *fmt, ...)
{
	va_list ap;
	va_start(ap, fmt);
	fx_oracle_print_record(fmt, ap);
	va_end(ap);
}

void Com_DPrintf(const char *fmt, ...)
{
	va_list ap;
	va_start(ap, fmt);
	fx_oracle_print_record(fmt, ap);
	va_end(ap);
}

void Com_Error(int level, const char *fmt, ...)
{
	va_list ap;
	va_start(ap, fmt);
	fprintf(stderr, "Com_Error(%d): ", level);
	vfprintf(stderr, fmt, ap);
	fprintf(stderr, "\n");
	va_end(ap);
	exit(1);
}

// --- cvars ------------------------------------------------------------------

// A flat table. The driver seeds values with `cvar` before `init`, and Cvar_Get
// then keeps the seeded value instead of the Raven default, the way a config
// file would.
#define FX_ORACLE_MAX_CVARS 32
static cvar_t s_cvars[FX_ORACLE_MAX_CVARS];
static char s_cvarNames[FX_ORACLE_MAX_CVARS][64];
static char s_cvarStrings[FX_ORACLE_MAX_CVARS][64];
static int s_numCvars = 0;

// CParticle::UpdateOrigin reads this before the point-contents probe. A null
// pointer takes the same branch a normal (non-RMG) map does.
// Source: `oracle/codemp/client/FxPrimitives.cpp:232`
cvar_t *com_RMG = NULL;

static cvar_t *fx_oracle_cvar_find(const char *name)
{
	for (int i = 0; i < s_numCvars; i++) {
		if (strcmp(s_cvarNames[i], name) == 0) {
			return &s_cvars[i];
		}
	}
	return NULL;
}

cvar_t *fx_oracle_cvar_set(const char *name, const char *value)
{
	cvar_t *cv = fx_oracle_cvar_find(name);
	if (!cv) {
		if (s_numCvars == FX_ORACLE_MAX_CVARS) {
			fprintf(stderr, "fx-oracle: the cvar table is full at %s\n", name);
			exit(1);
		}
		cv = &s_cvars[s_numCvars];
		Q_strncpyz(s_cvarNames[s_numCvars], name, sizeof(s_cvarNames[0]));
		cv->name = s_cvarNames[s_numCvars];
		s_numCvars++;
	}

	int slot = (int)(cv - s_cvars);
	Q_strncpyz(s_cvarStrings[slot], value, sizeof(s_cvarStrings[0]));
	cv->string = s_cvarStrings[slot];
	cv->value = (float)atof(value);
	cv->integer = atoi(value);
	return cv;
}

cvar_t *Cvar_Get(const char *var_name, const char *value, int flags)
{
	cvar_t *cv = fx_oracle_cvar_find(var_name);
	if (cv) {
		cv->flags |= flags;
		return cv;
	}
	cv = fx_oracle_cvar_set(var_name, value);
	cv->flags = flags;
	return cv;
}

// --- the zone ---------------------------------------------------------------

// GenericParser2's text pool is the only zone user in the link set. No golden
// carries a size or an address, so a plain malloc answers.
void *Z_Malloc(int iSize, memtag_t eTag, qboolean bZeroit, int iAlign)
{
	(void)eTag;
	(void)iAlign;
	void *p = malloc((size_t)iSize);
	if (!p) {
		Com_Error(ERR_FATAL, "Z_Malloc: out of memory (%d bytes)", iSize);
	}
	if (bZeroit) {
		memset(p, 0, (size_t)iSize);
	}
	return p;
}

void Z_Free(void *ptr)
{
	free(ptr);
}

// --- the file system --------------------------------------------------------

// Every read resolves under fixtures/. CFxScheduler::RegisterEffect prepends
// `effects/` to any name that does not already start with it
// (`oracle/codemp/client/FxScheduler.cpp:295-302`), so the resolver strips that
// prefix back off and the fixture tree stays flat.
#define FX_ORACLE_MAX_HANDLES 8
static FILE *s_files[FX_ORACLE_MAX_HANDLES];

static void fx_oracle_fixture_path(const char *qpath, char *out, size_t outSize)
{
	const char *rel = qpath;
	if (Q_stricmpn(rel, "effects/", 8) == 0) {
		rel += 8;
	}
	snprintf(out, outSize, "fixtures/%s", rel);

	// Raven builds some names with the Windows separator. The fixture tree uses
	// forward slashes only.
	for (char *p = out; *p; p++) {
		if (*p == '\\') {
			*p = '/';
		}
	}
}

int FS_FOpenFileByMode(const char *qpath, fileHandle_t *f, fsMode_t mode)
{
	if (mode != FS_READ) {
		fprintf(stderr, "fx-oracle: the harness refuses the write of %s\n", qpath);
		exit(1);
	}

	char path[1024];
	fx_oracle_fixture_path(qpath, path, sizeof(path));

	FILE *fp = fopen(path, "rb");
	if (!fp) {
		if (f) {
			*f = 0;
		}
		return -1;
	}
	fseek(fp, 0, SEEK_END);
	long size = ftell(fp);
	fseek(fp, 0, SEEK_SET);

	for (int i = 1; i < FX_ORACLE_MAX_HANDLES; i++) {
		if (!s_files[i]) {
			s_files[i] = fp;
			if (f) {
				*f = i;
			}
			return (int)size;
		}
	}
	fclose(fp);
	Com_Error(ERR_FATAL, "FS_FOpenFileByMode: out of handles for %s", path);
	return -1;
}

int FS_Read2(void *buffer, int len, fileHandle_t f)
{
	if (f <= 0 || f >= FX_ORACLE_MAX_HANDLES || !s_files[f]) {
		return 0;
	}
	return (int)fread(buffer, 1, (size_t)len, s_files[f]);
}

void FS_FCloseFile(fileHandle_t f)
{
	if (f <= 0 || f >= FX_ORACLE_MAX_HANDLES || !s_files[f]) {
		return;
	}
	fclose(s_files[f]);
	s_files[f] = NULL;
}

// --- the media registry -----------------------------------------------------

// Shaders, models and sounds share one policy: an ordered name table hands out
// `index + 1` for a new name and the stored handle for a repeat. Every call
// prints a record, repeats included, so a golden carries the registration order
// the effect files drive.
#define FX_ORACLE_MAX_MEDIA 128
#define FX_ORACLE_MEDIA_NAME 96

struct FxOracleMediaTable {
	char names[FX_ORACLE_MAX_MEDIA][FX_ORACLE_MEDIA_NAME];
	int count;
};

static FxOracleMediaTable s_shaders;
static FxOracleMediaTable s_models;
static FxOracleMediaTable s_sounds;

static int fx_oracle_media_handle(FxOracleMediaTable *table, const char *name)
{
	for (int i = 0; i < table->count; i++) {
		if (strcmp(table->names[i], name) == 0) {
			return i + 1;
		}
	}
	if (table->count == FX_ORACLE_MAX_MEDIA) {
		fprintf(stderr, "fx-oracle: the media table is full at %s\n", name);
		exit(1);
	}
	Q_strncpyz(table->names[table->count], name, FX_ORACLE_MEDIA_NAME);
	table->count++;
	return table->count;
}

static qhandle_t fx_oracle_register_shader(const char *name)
{
	int handle = fx_oracle_media_handle(&s_shaders, name);
	printf("REGSHADER %s -> %d\n", name, handle);
	return handle;
}

static qhandle_t fx_oracle_register_model(const char *name)
{
	int handle = fx_oracle_media_handle(&s_models, name);
	printf("REGMODEL %s -> %d\n", name, handle);
	return handle;
}

sfxHandle_t S_RegisterSound(const char *sample)
{
	int handle = fx_oracle_media_handle(&s_sounds, sample);
	printf("REGSOUND %s -> %d\n", sample, handle);
	return handle;
}

// --- the sound seam ---------------------------------------------------------

// Raven drops the volume and radius arguments inside `SFxHelper::PlaySound`, so
// the two default arguments carry the loss into the record as `-1 -1`.
// Source: `oracle/codemp/client/FxSystem.h:91-95`
void S_StartSound(const vec3_t origin, int entnum, int entchannel, sfxHandle_t sfx,
	int volume, int radius)
{
	vec3_t org = { 0.0f, 0.0f, 0.0f };
	if (origin) {
		VectorCopy(origin, org);
	}
	printf("SOUND origin %s %s %s entnum %d entchannel %d sfx %d volume %d radius %d\n",
		fxf(org[0]), fxf(org[1]), fxf(org[2]), entnum, entchannel, (int)sfx, volume, radius);
}

void S_StartLocalSound(sfxHandle_t sfx, int channelNum)
{
	printf("LOCALSOUND sfx %d entchannel %d\n", (int)sfx, channelNum);
}

// --- the renderer seam ------------------------------------------------------

// The eight capture sinks. `AddFxToScene((miniRefEntity_t*)0)` at
// FxPrimitives.cpp:1345 hands a null through, so both entity sinks answer a
// null with NULLREFENT.
static void fx_oracle_print_refent_body(const miniRefEntity_t *e)
{
	printf("reType %d renderfx %d hModel %d origin %s %s %s oldorigin %s %s %s"
		" axis %s %s %s %s %s %s %s %s %s nonNormalizedAxes %d radius %s rotation %s"
		" shaderTime %s customShader %d shaderRGBA %d %d %d %d shaderTexCoord %s %s frame %d\n",
		(int)e->reType, e->renderfx, (int)e->hModel,
		fxf(e->origin[0]), fxf(e->origin[1]), fxf(e->origin[2]),
		fxf(e->oldorigin[0]), fxf(e->oldorigin[1]), fxf(e->oldorigin[2]),
		fxf(e->axis[0][0]), fxf(e->axis[0][1]), fxf(e->axis[0][2]),
		fxf(e->axis[1][0]), fxf(e->axis[1][1]), fxf(e->axis[1][2]),
		fxf(e->axis[2][0]), fxf(e->axis[2][1]), fxf(e->axis[2][2]),
		(int)e->nonNormalizedAxes, fxf(e->radius), fxf(e->rotation),
		fxf(e->shaderTime), (int)e->customShader,
		(int)e->shaderRGBA[0], (int)e->shaderRGBA[1], (int)e->shaderRGBA[2], (int)e->shaderRGBA[3],
		fxf(e->shaderTexCoord[0]), fxf(e->shaderTexCoord[1]), e->frame);
}

static void fx_oracle_add_ref_entity(const refEntity_t *e)
{
	if (!e) {
		printf("NULLREFENT\n");
		return;
	}
	// `refEntity_t` opens with a byte-identical copy of `miniRefEntity_t`.
	// Source: `oracle/codemp/cgame/tr_types.h:131-163`
	printf("REFENT ");
	fx_oracle_print_refent_body((const miniRefEntity_t *)e);
}

static void fx_oracle_add_mini_ref_entity(const miniRefEntity_t *e)
{
	if (!e) {
		printf("NULLREFENT\n");
		return;
	}
	printf("MINIREFENT ");
	fx_oracle_print_refent_body(e);
}

static void fx_oracle_add_poly(qhandle_t hShader, int numVerts, const polyVert_t *verts, int num)
{
	printf("POLY shader %d count %d\n", (int)hShader, numVerts);
	for (int i = 0; i < numVerts; i++) {
		printf("POLYV %d xyz %s %s %s st %s %s modulate %d %d %d %d\n", i,
			fxf(verts[i].xyz[0]), fxf(verts[i].xyz[1]), fxf(verts[i].xyz[2]),
			fxf(verts[i].st[0]), fxf(verts[i].st[1]),
			(int)verts[i].modulate[0], (int)verts[i].modulate[1],
			(int)verts[i].modulate[2], (int)verts[i].modulate[3]);
	}
	(void)num;
}

static void fx_oracle_add_decal(qhandle_t shader, const vec3_t origin, const vec3_t dir,
	float orientation, float r, float g, float b, float a, qboolean alphaFade,
	float radius, qboolean temporary)
{
	printf("DECAL shader %d origin %s %s %s dir %s %s %s orientation %s"
		" rgba %s %s %s %s alphaFade %d radius %s temporary %d\n",
		(int)shader, fxf(origin[0]), fxf(origin[1]), fxf(origin[2]),
		fxf(dir[0]), fxf(dir[1]), fxf(dir[2]), fxf(orientation),
		fxf(r), fxf(g), fxf(b), fxf(a), (int)alphaFade, fxf(radius), (int)temporary);
}

static void fx_oracle_add_light(const vec3_t org, float intensity, float r, float g, float b)
{
	printf("LIGHT origin %s %s %s radius %s rgb %s %s %s\n",
		fxf(org[0]), fxf(org[1]), fxf(org[2]), fxf(intensity), fxf(r), fxf(g), fxf(b));
}

static void fx_oracle_draw_stretch_pic(float x, float y, float w, float h,
	float s1, float t1, float s2, float t2, qhandle_t hShader)
{
	printf("STRETCHPIC x %s y %s w %s h %s shader %d\n",
		fxf(x), fxf(y), fxf(w), fxf(h), (int)hShader);
	(void)s1;
	(void)t1;
	(void)s2;
	(void)t2;
}

fxOracleRefExport_t re = {
	fx_oracle_register_model,
	fx_oracle_register_shader,
	fx_oracle_add_ref_entity,
	fx_oracle_add_mini_ref_entity,
	fx_oracle_add_poly,
	fx_oracle_add_decal,
	fx_oracle_add_light,
	fx_oracle_draw_stretch_pic,
};

// --- the cgame trap seam ----------------------------------------------------

// The FX code marshals every trap argument through this block.
// Source: `oracle/codemp/client/client.h:136`
static char s_sharedMemory[4096];
fxOracleClientActive_t cl = { s_sharedMemory };

// The FX code names the handle and never looks at it.
vm_t *cgvm = (vm_t *)1;

// --- scripted reply queues --------------------------------------------------

#define FX_ORACLE_MAX_REPLIES 64

struct FxOracleTraceReply {
	float fraction;
	vec3_t endpos;
	vec3_t normal;
	int startsolid;
	int allsolid;
	int surfaceFlags;
	int entityNum;
};

struct FxOracleBoltReply {
	int exists;
	vec3_t origin;
	vec3_t axis[3];
};

// `vec3_t` is a raw array, so a queue entry wraps it to stay assignable.
struct FxOracleVec3 {
	vec3_t v;
};

template <typename T>
struct FxOracleQueue {
	T items[FX_ORACLE_MAX_REPLIES];
	int count;
	int head;
};

// A queue is FIFO, and its last entry repeats forever once the queue drains.
template <typename T>
static void fx_oracle_queue_push(FxOracleQueue<T> *q, const T &item)
{
	if (q->count == FX_ORACLE_MAX_REPLIES) {
		fprintf(stderr, "fx-oracle: a scripted reply queue is full\n");
		exit(1);
	}
	q->items[q->count++] = item;
}

template <typename T>
static const T *fx_oracle_queue_pop(FxOracleQueue<T> *q)
{
	if (q->count == 0) {
		return NULL;
	}
	const T *item = &q->items[q->head];
	if (q->head < q->count - 1) {
		q->head++;
	}
	return item;
}

static FxOracleQueue<FxOracleTraceReply> s_traces;
static FxOracleQueue<int> s_pointContents;
static FxOracleQueue<FxOracleBoltReply> s_bolts;
static FxOracleQueue<FxOracleVec3> s_lerpOrigins;

void fx_oracle_push_trace(float fraction, const vec3_t endpos, const vec3_t normal,
	int startsolid, int allsolid, int surfaceFlags, int entityNum)
{
	FxOracleTraceReply reply;
	reply.fraction = fraction;
	VectorCopy(endpos, reply.endpos);
	VectorCopy(normal, reply.normal);
	reply.startsolid = startsolid;
	reply.allsolid = allsolid;
	reply.surfaceFlags = surfaceFlags;
	reply.entityNum = entityNum;
	fx_oracle_queue_push(&s_traces, reply);
}

void fx_oracle_push_pointcontents(int contents)
{
	fx_oracle_queue_push(&s_pointContents, contents);
}

void fx_oracle_push_bolt(int exists, const vec3_t origin, const vec3_t axis0,
	const vec3_t axis1, const vec3_t axis2)
{
	FxOracleBoltReply reply;
	reply.exists = exists;
	VectorCopy(origin, reply.origin);
	VectorCopy(axis0, reply.axis[0]);
	VectorCopy(axis1, reply.axis[1]);
	VectorCopy(axis2, reply.axis[2]);
	fx_oracle_queue_push(&s_bolts, reply);
}

void fx_oracle_push_lerporigin(const vec3_t origin)
{
	FxOracleVec3 copy;
	VectorCopy(origin, copy.v);
	fx_oracle_queue_push(&s_lerpOrigins, copy);
}

// --- the trap dispatcher ----------------------------------------------------

static void fx_oracle_answer_trace(int isG2)
{
	TCGTrace *td = (TCGTrace *)cl.mSharedMemory;

	printf("TRACE start %s %s %s mins %s %s %s maxs %s %s %s end %s %s %s"
		" skip %d mask %d g2 %d\n",
		fxf(td->mStart[0]), fxf(td->mStart[1]), fxf(td->mStart[2]),
		fxf(td->mMins[0]), fxf(td->mMins[1]), fxf(td->mMins[2]),
		fxf(td->mMaxs[0]), fxf(td->mMaxs[1]), fxf(td->mMaxs[2]),
		fxf(td->mEnd[0]), fxf(td->mEnd[1]), fxf(td->mEnd[2]),
		td->mSkipNumber, td->mMask, isG2);

	memset(&td->mResult, 0, sizeof(td->mResult));

	const FxOracleTraceReply *reply = fx_oracle_queue_pop(&s_traces);
	if (!reply) {
		// The miss reply: the trace ran clean to its endpoint.
		td->mResult.fraction = 1.0f;
		VectorCopy(td->mEnd, td->mResult.endpos);
		VectorSet(td->mResult.plane.normal, 0.0f, 0.0f, 1.0f);
		td->mResult.entityNum = ENTITYNUM_NONE;
		return;
	}

	td->mResult.fraction = reply->fraction;
	VectorCopy(reply->endpos, td->mResult.endpos);
	VectorCopy(reply->normal, td->mResult.plane.normal);
	td->mResult.startsolid = (byte)reply->startsolid;
	td->mResult.allsolid = (byte)reply->allsolid;
	td->mResult.surfaceFlags = reply->surfaceFlags;
	td->mResult.entityNum = (short)reply->entityNum;
}

static int fx_oracle_answer_point_contents(void)
{
	TCGPointContents *data = (TCGPointContents *)cl.mSharedMemory;

	const int *reply = fx_oracle_queue_pop(&s_pointContents);
	int contents = reply ? *reply : 0;

	printf("POINTCONTENTS point %s %s %s passent %d -> %d\n",
		fxf(data->mPoint[0]), fxf(data->mPoint[1]), fxf(data->mPoint[2]),
		data->mPassEntityNum, contents);
	return contents;
}

static void fx_oracle_answer_lerp_origin(void)
{
	TCGVectorData *data = (TCGVectorData *)cl.mSharedMemory;

	const FxOracleVec3 *reply = fx_oracle_queue_pop(&s_lerpOrigins);
	if (reply) {
		VectorCopy(reply->v, data->mPoint);
	} else {
		VectorClear(data->mPoint);
	}

	printf("LERPORIGIN ent %d -> %s %s %s\n", data->mEntityNum,
		fxf(data->mPoint[0]), fxf(data->mPoint[1]), fxf(data->mPoint[2]));
}

// CG_GET_LERP_DATA feeds the angles, origin and scale that
// `SFxHelper::GetOriginAxisFromBolt` hands to G2API_GetBoltMatrix. The harness
// answers the bolt question itself, so this arm only clears the block and the
// BOLT record carries the answer.
// Source: `oracle/codemp/client/FxSystem.cpp:96-108`
static void fx_oracle_answer_lerp_data(void)
{
	TCGGetBoltData *data = (TCGGetBoltData *)cl.mSharedMemory;
	int entityNum = data->mEntityNum;

	VectorClear(data->mOrigin);
	VectorClear(data->mAngles);
	VectorSet(data->mScale, 1.0f, 1.0f, 1.0f);
	data->mEntityNum = entityNum;
}

static void fx_oracle_answer_g2mark(void)
{
	TCGG2Mark *td = (TCGG2Mark *)cl.mSharedMemory;

	printf("G2DECAL shader %d start %s %s %s dir %s %s %s size %s\n", td->shader,
		fxf(td->start[0]), fxf(td->start[1]), fxf(td->start[2]),
		fxf(td->dir[0]), fxf(td->dir[1]), fxf(td->dir[2]), fxf(td->size));
}

static void fx_oracle_answer_camera_shake(void)
{
	TCGCameraShake *data = (TCGCameraShake *)cl.mSharedMemory;

	printf("SHAKE origin %s %s %s intensity %s radius %d time %d\n",
		fxf(data->mOrigin[0]), fxf(data->mOrigin[1]), fxf(data->mOrigin[2]),
		fxf(data->mIntensity), data->mRadius, data->mTime);
}

int VM_Call(vm_t *vm, int callNum, ...)
{
	(void)vm;

	switch (callNum) {
	case CG_TRACE:
		fx_oracle_answer_trace(0);
		return 0;
	case CG_G2TRACE:
		fx_oracle_answer_trace(1);
		return 0;
	case CG_POINT_CONTENTS:
		return fx_oracle_answer_point_contents();
	case CG_GET_LERP_ORIGIN:
		fx_oracle_answer_lerp_origin();
		return 0;
	case CG_GET_LERP_DATA:
		fx_oracle_answer_lerp_data();
		return 0;
	case CG_G2MARK:
		fx_oracle_answer_g2mark();
		return 0;
	case CG_FX_CAMERASHAKE:
		fx_oracle_answer_camera_shake();
		return 0;
	default:
		fprintf(stderr, "fx-oracle: the FX code made the uncovered trap %d\n", callNum);
		exit(1);
	}
	return 0;
}

// --- the ghoul2 seam --------------------------------------------------------

// `SFxHelper::GetOriginAxisFromBolt` reads the origin out of column three and
// the axes out of columns one, zero and two, in that order.
// Source: `oracle/codemp/client/FxSystem.cpp:110-127`
qboolean G2API_GetBoltMatrix(CGhoul2Info_v &ghoul2, const int modelIndex, const int boltIndex,
	mdxaBone_t *matrix, const vec3_t angles, const vec3_t position, const int frameNum,
	qhandle_t *modelList, vec3_t scale)
{
	(void)angles;
	(void)position;
	(void)frameNum;
	(void)modelList;
	(void)scale;

	const FxOracleBoltReply *reply = fx_oracle_queue_pop(&s_bolts);
	int exists = reply ? reply->exists : 0;

	// The caller wrote the entity number into the shared block for the
	// CG_GET_LERP_DATA call it makes one line earlier, and that arm preserves it.
	const TCGGetBoltData *lerp = (const TCGGetBoltData *)cl.mSharedMemory;
	printf("BOLT ent %d model %d bolt %d -> %d\n", lerp->mEntityNum, modelIndex, boltIndex, exists);
	(void)ghoul2;

	if (!exists) {
		return qfalse;
	}

	memset(matrix, 0, sizeof(*matrix));
	for (int i = 0; i < 3; i++) {
		matrix->matrix[i][3] = reply->origin[i];
		matrix->matrix[i][0] = reply->axis[1][i];
		matrix->matrix[i][1] = reply->axis[0][i];
		matrix->matrix[i][2] = reply->axis[2][i];
	}
	return qtrue;
}
