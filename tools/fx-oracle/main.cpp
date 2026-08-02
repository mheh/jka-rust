// fx-oracle - the scripted driver over the unmodified Raven FX TUs.
//
//   fx_dump <script.fxs>
//
// The driver reads a scenario script, calls the FX system in the scripted
// order, and writes the canonical emission stream to stdout. README.md holds
// the command schema and the golden format.
//
// The Rust FX port (wayfinder ticket gh#27) must reproduce every golden byte
// for byte.
// Every standard header the FX headers pull in lands first, so the access
// relaxation below cannot reach the standard library.
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <algorithm>
#include <list>
#include <map>
#include <memory>
#include <string>
#include <vector>

#include "../qcommon/exe_headers.h"
#include "client.h"

// `dumpstate` reads `mNextFree2DEffect`, and `dumptemplate` reads
// `mEffectTemplates` and the ordered handle list inside `CMediaHandles`. All
// three are private and the oracle offers no ordered read accessor
// (`CMediaHandles::GetHandle` picks at random). The relaxation runs in the
// driver only, changes no oracle file, and changes no layout: clang lays fields
// out in declaration order whatever their access.
// Source: `oracle/codemp/client/FxScheduler.h:66-79,443-451`
#define private public
#define protected public
#include "FxScheduler.h"
#undef private
#undef protected

#include "FXExport.h"
#include "host.h"

// The saber-trail entry point. `cl_cgame.cpp` forward-declares it the same way,
// because FxPrimitives.h does not.
// Source: `oracle/codemp/client/cl_cgame.cpp:67`
void FX_FeedTrail(effectTrailArgStruct_t *a);

// The FX pool counters live at file scope in FxUtil.cpp, which declares them in
// no header. `dumpstate` reads them rather than patching the TU.
// Source: `oracle/codemp/client/FxUtil.cpp:26-27`
extern int activeFx;
extern int drawnFx;

// The view the FX system culls and projects against. `refdef` fills it and
// `init` hands its address to FX_InitSystem, which is what the cgame does.
static refdef_t s_refdef;

// --- script parsing ---------------------------------------------------------

static char *fx_oracle_next(char **cursor)
{
	char *p = *cursor;
	while (*p == ' ' || *p == '\t') {
		p++;
	}
	if (!*p) {
		*cursor = p;
		return NULL;
	}
	char *start = p;
	while (*p && *p != ' ' && *p != '\t') {
		p++;
	}
	if (*p) {
		*p++ = 0;
	}
	*cursor = p;
	return start;
}

static const char *fx_oracle_word(char **cursor, const char *command)
{
	const char *w = fx_oracle_next(cursor);
	if (!w) {
		fprintf(stderr, "fx-oracle: %s wants more arguments\n", command);
		exit(1);
	}
	return w;
}

static float fx_oracle_float(char **cursor, const char *command)
{
	return (float)atof(fx_oracle_word(cursor, command));
}

static int fx_oracle_int(char **cursor, const char *command)
{
	return atoi(fx_oracle_word(cursor, command));
}

static void fx_oracle_vec(char **cursor, const char *command, vec3_t out)
{
	out[0] = fx_oracle_float(cursor, command);
	out[1] = fx_oracle_float(cursor, command);
	out[2] = fx_oracle_float(cursor, command);
}

static void fx_oracle_axis(char **cursor, const char *command, vec3_t out[3])
{
	fx_oracle_vec(cursor, command, out[0]);
	fx_oracle_vec(cursor, command, out[1]);
	fx_oracle_vec(cursor, command, out[2]);
}

// --- template and state dumps -----------------------------------------------

// One PRIMRANGE line per CFxRange field, in the declaration order of
// CPrimitiveTemplate.
// Source: `oracle/codemp/client/FxScheduler.h:167-254`
struct FxOracleRangeField {
	const char *name;
	size_t offset;
};

#define FX_ORACLE_RANGE(label, field) { label, offsetof(CPrimitiveTemplate, field) }

static const FxOracleRangeField s_rangeFields[] = {
	FX_ORACLE_RANGE("spawnDelay", mSpawnDelay),
	FX_ORACLE_RANGE("spawnCount", mSpawnCount),
	FX_ORACLE_RANGE("life", mLife),
	FX_ORACLE_RANGE("origin1X", mOrigin1X),
	FX_ORACLE_RANGE("origin1Y", mOrigin1Y),
	FX_ORACLE_RANGE("origin1Z", mOrigin1Z),
	FX_ORACLE_RANGE("origin2X", mOrigin2X),
	FX_ORACLE_RANGE("origin2Y", mOrigin2Y),
	FX_ORACLE_RANGE("origin2Z", mOrigin2Z),
	FX_ORACLE_RANGE("radius", mRadius),
	FX_ORACLE_RANGE("height", mHeight),
	FX_ORACLE_RANGE("windModifier", mWindModifier),
	FX_ORACLE_RANGE("rotation", mRotation),
	FX_ORACLE_RANGE("rotationDelta", mRotationDelta),
	FX_ORACLE_RANGE("angle1", mAngle1),
	FX_ORACLE_RANGE("angle2", mAngle2),
	FX_ORACLE_RANGE("angle3", mAngle3),
	FX_ORACLE_RANGE("angle1Delta", mAngle1Delta),
	FX_ORACLE_RANGE("angle2Delta", mAngle2Delta),
	FX_ORACLE_RANGE("angle3Delta", mAngle3Delta),
	FX_ORACLE_RANGE("velX", mVelX),
	FX_ORACLE_RANGE("velY", mVelY),
	FX_ORACLE_RANGE("velZ", mVelZ),
	FX_ORACLE_RANGE("accelX", mAccelX),
	FX_ORACLE_RANGE("accelY", mAccelY),
	FX_ORACLE_RANGE("accelZ", mAccelZ),
	FX_ORACLE_RANGE("gravity", mGravity),
	FX_ORACLE_RANGE("density", mDensity),
	FX_ORACLE_RANGE("variance", mVariance),
	FX_ORACLE_RANGE("redStart", mRedStart),
	FX_ORACLE_RANGE("greenStart", mGreenStart),
	FX_ORACLE_RANGE("blueStart", mBlueStart),
	FX_ORACLE_RANGE("redEnd", mRedEnd),
	FX_ORACLE_RANGE("greenEnd", mGreenEnd),
	FX_ORACLE_RANGE("blueEnd", mBlueEnd),
	FX_ORACLE_RANGE("rgbParm", mRGBParm),
	FX_ORACLE_RANGE("alphaStart", mAlphaStart),
	FX_ORACLE_RANGE("alphaEnd", mAlphaEnd),
	FX_ORACLE_RANGE("alphaParm", mAlphaParm),
	FX_ORACLE_RANGE("sizeStart", mSizeStart),
	FX_ORACLE_RANGE("sizeEnd", mSizeEnd),
	FX_ORACLE_RANGE("sizeParm", mSizeParm),
	FX_ORACLE_RANGE("size2Start", mSize2Start),
	FX_ORACLE_RANGE("size2End", mSize2End),
	FX_ORACLE_RANGE("size2Parm", mSize2Parm),
	FX_ORACLE_RANGE("lengthStart", mLengthStart),
	FX_ORACLE_RANGE("lengthEnd", mLengthEnd),
	FX_ORACLE_RANGE("lengthParm", mLengthParm),
	FX_ORACLE_RANGE("texCoordS", mTexCoordS),
	FX_ORACLE_RANGE("texCoordT", mTexCoordT),
	FX_ORACLE_RANGE("elasticity", mElasticity),
};

// The five handle lists, in declaration order.
// Source: `oracle/codemp/client/FxScheduler.h:172-176`
struct FxOracleMediaField {
	const char *name;
	size_t offset;
};

static const FxOracleMediaField s_mediaFields[] = {
	{ "mediaHandles", offsetof(CPrimitiveTemplate, mMediaHandles) },
	{ "impactFxHandles", offsetof(CPrimitiveTemplate, mImpactFxHandles) },
	{ "deathFxHandles", offsetof(CPrimitiveTemplate, mDeathFxHandles) },
	{ "emitterFxHandles", offsetof(CPrimitiveTemplate, mEmitterFxHandles) },
	{ "playFxHandles", offsetof(CPrimitiveTemplate, mPlayFxHandles) },
};

static void fx_oracle_dump_template(int handle)
{
	if (handle < 1 || handle >= FX_MAX_EFFECTS || !theFxScheduler.mEffectTemplates[handle].mInUse) {
		printf("TEMPLATE %d MISSING\n", handle);
		return;
	}
	const SEffectTemplate *effect = &theFxScheduler.mEffectTemplates[handle];

	printf("TEMPLATE %d name %s repeatDelay %d primitiveCount %d\n", handle,
		effect->mEffectName, effect->mRepeatDelay, effect->mPrimitiveCount);

	for (int i = 0; i < effect->mPrimitiveCount; i++) {
		const CPrimitiveTemplate *prim = effect->mPrimitives[i];

		printf("PRIM %d name %s type %d flags %d spawnFlags %d matImpactFX %d"
			" cullRange %d soundRadius %d soundVolume %d\n",
			i, prim->mName, (int)prim->mType, prim->mFlags, prim->mSpawnFlags,
			(int)prim->mMatImpactFX, prim->mCullRange, prim->mSoundRadius, prim->mSoundVolume);

		for (size_t f = 0; f < sizeof(s_rangeFields) / sizeof(s_rangeFields[0]); f++) {
			const CFxRange *range =
				(const CFxRange *)((const char *)prim + s_rangeFields[f].offset);
			printf("PRIMRANGE %d %s %s %s\n", i, s_rangeFields[f].name,
				fxf(range->GetMin()), fxf(range->GetMax()));
		}

		printf("PRIMVEC %d min %s %s %s max %s %s %s\n", i,
			fxf(prim->mMin[0]), fxf(prim->mMin[1]), fxf(prim->mMin[2]),
			fxf(prim->mMax[0]), fxf(prim->mMax[1]), fxf(prim->mMax[2]));

		for (size_t m = 0; m < sizeof(s_mediaFields) / sizeof(s_mediaFields[0]); m++) {
			const CMediaHandles *media =
				(const CMediaHandles *)((const char *)prim + s_mediaFields[m].offset);
			printf("PRIMMEDIA %d %s %d", i, s_mediaFields[m].name, (int)media->mMediaList.size());
			for (size_t h = 0; h < media->mMediaList.size(); h++) {
				printf(" %d", media->mMediaList[h]);
			}
			printf("\n");
		}
	}
}

static void fx_oracle_dump_state(void)
{
	printf("STATE activeFx %d drawnFx %d scheduledFx %d nextFree2DEffect %d\n",
		activeFx, drawnFx, theFxScheduler.NumScheduledFx(),
		theFxScheduler.mNextFree2DEffect);
}

// --- the interpreter --------------------------------------------------------

int main(int argc, char **argv)
{
	if (argc != 2) {
		fprintf(stderr, "usage: %s <script.fxs>\n", argv[0]);
		return 2;
	}

	FILE *script = fopen(argv[1], "rb");
	if (!script) {
		fprintf(stderr, "fx-oracle: cannot open %s\n", argv[1]);
		return 2;
	}

	// The header carries the scenario stem, so a golden names itself.
	const char *stem = strrchr(argv[1], '/');
	stem = stem ? stem + 1 : argv[1];
	char stemBuf[128];
	Q_strncpyz(stemBuf, stem, sizeof(stemBuf));
	char *dot = strrchr(stemBuf, '.');
	if (dot) {
		*dot = 0;
	}
	printf("== fx-oracle %s ==\n", stemBuf);

	// The default view: at the origin, looking down +X.
	memset(&s_refdef, 0, sizeof(s_refdef));
	VectorSet(s_refdef.viewaxis[0], 1.0f, 0.0f, 0.0f);
	VectorSet(s_refdef.viewaxis[1], 0.0f, 1.0f, 0.0f);
	VectorSet(s_refdef.viewaxis[2], 0.0f, 0.0f, 1.0f);
	s_refdef.fov_x = 90.0f;
	s_refdef.fov_y = 73.739792f;

	char line[1024];
	while (fgets(line, sizeof(line), script)) {
		char *nl = strpbrk(line, "\r\n");
		if (nl) {
			*nl = 0;
		}
		char *cursor = line;
		const char *cmd = fx_oracle_next(&cursor);
		if (!cmd || cmd[0] == '#') {
			continue;
		}

		if (strcmp(cmd, "seed") == 0) {
			Rand_Init(fx_oracle_int(&cursor, cmd));

		} else if (strcmp(cmd, "refdef") == 0) {
			fx_oracle_vec(&cursor, cmd, s_refdef.vieworg);
			fx_oracle_vec(&cursor, cmd, s_refdef.viewangles);
			fx_oracle_axis(&cursor, cmd, s_refdef.viewaxis);
			s_refdef.fov_x = fx_oracle_float(&cursor, cmd);
			s_refdef.fov_y = fx_oracle_float(&cursor, cmd);

		} else if (strcmp(cmd, "cvar") == 0) {
			const char *name = fx_oracle_word(&cursor, cmd);
			const char *value = fx_oracle_word(&cursor, cmd);
			fx_oracle_cvar_set(name, value);

		} else if (strcmp(cmd, "init") == 0) {
			FX_InitSystem(&s_refdef);

		} else if (strcmp(cmd, "register") == 0) {
			const char *path = fx_oracle_word(&cursor, cmd);
			int handle = FX_RegisterEffect(path);
			printf("REGISTER %s -> %d\n", path, handle);

		} else if (strcmp(cmd, "dumptemplate") == 0) {
			fx_oracle_dump_template(fx_oracle_int(&cursor, cmd));

		} else if (strcmp(cmd, "trace") == 0) {
			float fraction = fx_oracle_float(&cursor, cmd);
			vec3_t endpos;
			vec3_t normal;
			fx_oracle_vec(&cursor, cmd, endpos);
			fx_oracle_vec(&cursor, cmd, normal);
			int startsolid = fx_oracle_int(&cursor, cmd);
			int allsolid = fx_oracle_int(&cursor, cmd);
			int surfaceFlags = fx_oracle_int(&cursor, cmd);
			int entityNum = fx_oracle_int(&cursor, cmd);
			fx_oracle_push_trace(fraction, endpos, normal, startsolid, allsolid,
				surfaceFlags, entityNum);

		} else if (strcmp(cmd, "pointcontents") == 0) {
			fx_oracle_push_pointcontents(fx_oracle_int(&cursor, cmd));

		} else if (strcmp(cmd, "bolt") == 0) {
			int exists = fx_oracle_int(&cursor, cmd);
			vec3_t origin;
			vec3_t axis[3];
			fx_oracle_vec(&cursor, cmd, origin);
			fx_oracle_axis(&cursor, cmd, axis);
			fx_oracle_push_bolt(exists, origin, axis[0], axis[1], axis[2]);

		} else if (strcmp(cmd, "lerporigin") == 0) {
			vec3_t origin;
			fx_oracle_vec(&cursor, cmd, origin);
			fx_oracle_push_lerporigin(origin);

		} else if (strcmp(cmd, "playid") == 0) {
			int id = fx_oracle_int(&cursor, cmd);
			vec3_t org;
			vec3_t fwd;
			fx_oracle_vec(&cursor, cmd, org);
			fx_oracle_vec(&cursor, cmd, fwd);
			int vol = fx_oracle_int(&cursor, cmd);
			int rad = fx_oracle_int(&cursor, cmd);
			int portal = fx_oracle_int(&cursor, cmd);
			FX_PlayEffectID(id, org, fwd, vol, rad, (qboolean)!!portal);

		} else if (strcmp(cmd, "play") == 0) {
			char path[MAX_QPATH];
			Q_strncpyz(path, fx_oracle_word(&cursor, cmd), sizeof(path));
			vec3_t org;
			vec3_t fwd;
			fx_oracle_vec(&cursor, cmd, org);
			fx_oracle_vec(&cursor, cmd, fwd);
			int vol = fx_oracle_int(&cursor, cmd);
			int rad = fx_oracle_int(&cursor, cmd);
			FX_PlayEffect(path, org, fwd, vol, rad);

		} else if (strcmp(cmd, "playbolted") == 0) {
			int id = fx_oracle_int(&cursor, cmd);
			vec3_t org;
			fx_oracle_vec(&cursor, cmd, org);
			int boltInfo = fx_oracle_int(&cursor, cmd);
			int iGhoul2 = fx_oracle_int(&cursor, cmd);
			int iLoopTime = fx_oracle_int(&cursor, cmd);
			int isRelative = fx_oracle_int(&cursor, cmd);
			FX_PlayBoltedEffectID(id, org, boltInfo, iGhoul2, iLoopTime, (qboolean)!!isRelative);

		} else if (strcmp(cmd, "playentity") == 0) {
			int id = fx_oracle_int(&cursor, cmd);
			vec3_t org;
			vec3_t axis[3];
			fx_oracle_vec(&cursor, cmd, org);
			fx_oracle_axis(&cursor, cmd, axis);
			int boltInfo = fx_oracle_int(&cursor, cmd);
			int entNum = fx_oracle_int(&cursor, cmd);
			int vol = fx_oracle_int(&cursor, cmd);
			int rad = fx_oracle_int(&cursor, cmd);
			FX_PlayEntityEffectID(id, org, axis, boltInfo, entNum, vol, rad);

		} else if (strcmp(cmd, "stop") == 0) {
			char path[MAX_QPATH];
			Q_strncpyz(path, fx_oracle_word(&cursor, cmd), sizeof(path));
			int boltInfo = fx_oracle_int(&cursor, cmd);
			int portal = fx_oracle_int(&cursor, cmd);
			theFxScheduler.StopEffect(path, boltInfo, !!portal);

		} else if (strcmp(cmd, "addline") == 0) {
			vec3_t start;
			vec3_t end;
			vec3_t sRGB;
			vec3_t eRGB;
			fx_oracle_vec(&cursor, cmd, start);
			fx_oracle_vec(&cursor, cmd, end);
			float size1 = fx_oracle_float(&cursor, cmd);
			float size2 = fx_oracle_float(&cursor, cmd);
			float sizeParm = fx_oracle_float(&cursor, cmd);
			float a1 = fx_oracle_float(&cursor, cmd);
			float a2 = fx_oracle_float(&cursor, cmd);
			float aParm = fx_oracle_float(&cursor, cmd);
			fx_oracle_vec(&cursor, cmd, sRGB);
			fx_oracle_vec(&cursor, cmd, eRGB);
			float rgbParm = fx_oracle_float(&cursor, cmd);
			int killTime = fx_oracle_int(&cursor, cmd);
			int shader = fx_oracle_int(&cursor, cmd);
			int flags = fx_oracle_int(&cursor, cmd);
			FX_AddLine(start, end, size1, size2, sizeParm, a1, a2, aParm,
				sRGB, eRGB, rgbParm, killTime, shader, flags);

		} else if (strcmp(cmd, "addelectricity") == 0) {
			vec3_t start;
			vec3_t end;
			vec3_t sRGB;
			vec3_t eRGB;
			fx_oracle_vec(&cursor, cmd, start);
			fx_oracle_vec(&cursor, cmd, end);
			float size1 = fx_oracle_float(&cursor, cmd);
			float size2 = fx_oracle_float(&cursor, cmd);
			float sizeParm = fx_oracle_float(&cursor, cmd);
			float a1 = fx_oracle_float(&cursor, cmd);
			float a2 = fx_oracle_float(&cursor, cmd);
			float aParm = fx_oracle_float(&cursor, cmd);
			fx_oracle_vec(&cursor, cmd, sRGB);
			fx_oracle_vec(&cursor, cmd, eRGB);
			float rgbParm = fx_oracle_float(&cursor, cmd);
			float chaos = fx_oracle_float(&cursor, cmd);
			int killTime = fx_oracle_int(&cursor, cmd);
			int shader = fx_oracle_int(&cursor, cmd);
			int flags = fx_oracle_int(&cursor, cmd);
			FX_AddElectricity(start, end, size1, size2, sizeParm, a1, a2, aParm,
				sRGB, eRGB, rgbParm, chaos, killTime, shader, flags);

		} else if (strcmp(cmd, "addbezier") == 0) {
			vec3_t start;
			vec3_t end;
			vec3_t c1;
			vec3_t c1vel;
			vec3_t c2;
			vec3_t c2vel;
			vec3_t sRGB;
			vec3_t eRGB;
			fx_oracle_vec(&cursor, cmd, start);
			fx_oracle_vec(&cursor, cmd, end);
			fx_oracle_vec(&cursor, cmd, c1);
			fx_oracle_vec(&cursor, cmd, c1vel);
			fx_oracle_vec(&cursor, cmd, c2);
			fx_oracle_vec(&cursor, cmd, c2vel);
			float size1 = fx_oracle_float(&cursor, cmd);
			float size2 = fx_oracle_float(&cursor, cmd);
			float sizeParm = fx_oracle_float(&cursor, cmd);
			float a1 = fx_oracle_float(&cursor, cmd);
			float a2 = fx_oracle_float(&cursor, cmd);
			float aParm = fx_oracle_float(&cursor, cmd);
			fx_oracle_vec(&cursor, cmd, sRGB);
			fx_oracle_vec(&cursor, cmd, eRGB);
			float rgbParm = fx_oracle_float(&cursor, cmd);
			int killTime = fx_oracle_int(&cursor, cmd);
			int shader = fx_oracle_int(&cursor, cmd);
			int flags = fx_oracle_int(&cursor, cmd);
			FX_AddBezier(start, end, c1, c1vel, c2, c2vel, size1, size2, sizeParm,
				a1, a2, aParm, sRGB, eRGB, rgbParm, killTime, shader, flags);

		} else if (strcmp(cmd, "addpoly") == 0) {
			// Exactly three verts, which keeps the command line finite.
			int numVerts = fx_oracle_int(&cursor, cmd);
			vec3_t verts[3];
			vec2_t st[3];
			for (int i = 0; i < 3; i++) {
				fx_oracle_vec(&cursor, cmd, verts[i]);
			}
			for (int i = 0; i < 3; i++) {
				st[i][0] = fx_oracle_float(&cursor, cmd);
				st[i][1] = fx_oracle_float(&cursor, cmd);
			}
			vec3_t vel;
			vec3_t accel;
			vec3_t rgb1;
			vec3_t rgb2;
			vec3_t rotDelta;
			fx_oracle_vec(&cursor, cmd, vel);
			fx_oracle_vec(&cursor, cmd, accel);
			float a1 = fx_oracle_float(&cursor, cmd);
			float a2 = fx_oracle_float(&cursor, cmd);
			float aParm = fx_oracle_float(&cursor, cmd);
			fx_oracle_vec(&cursor, cmd, rgb1);
			fx_oracle_vec(&cursor, cmd, rgb2);
			float rgbParm = fx_oracle_float(&cursor, cmd);
			fx_oracle_vec(&cursor, cmd, rotDelta);
			float bounce = fx_oracle_float(&cursor, cmd);
			int motionDelay = fx_oracle_int(&cursor, cmd);
			int killTime = fx_oracle_int(&cursor, cmd);
			int shader = fx_oracle_int(&cursor, cmd);
			int flags = fx_oracle_int(&cursor, cmd);
			FX_AddPoly(verts, st, numVerts, vel, accel, a1, a2, aParm,
				rgb1, rgb2, rgbParm, rotDelta, bounce, motionDelay, killTime, shader, flags);

		} else if (strcmp(cmd, "addsprite") == 0) {
			// The CG_FX_ADDSPRITE arm: FX_AddParticle with rgb = 1,1,1.
			// Source: `oracle/codemp/client/cl_cgame.cpp:1210-1229`
			vec3_t org;
			vec3_t vel;
			vec3_t accel;
			fx_oracle_vec(&cursor, cmd, org);
			fx_oracle_vec(&cursor, cmd, vel);
			fx_oracle_vec(&cursor, cmd, accel);
			float scale = fx_oracle_float(&cursor, cmd);
			float dscale = fx_oracle_float(&cursor, cmd);
			float sAlpha = fx_oracle_float(&cursor, cmd);
			float eAlpha = fx_oracle_float(&cursor, cmd);
			float rotation = fx_oracle_float(&cursor, cmd);
			float bounce = fx_oracle_float(&cursor, cmd);
			int life = fx_oracle_int(&cursor, cmd);
			int shader = fx_oracle_int(&cursor, cmd);
			int flags = fx_oracle_int(&cursor, cmd);
			vec3_t rgb;
			rgb[0] = 1;
			rgb[1] = 1;
			rgb[2] = 1;
			FX_AddParticle(org, vel, accel, scale, dscale, 0, sAlpha, eAlpha, 0,
				rgb, rgb, 0, rotation, 0, vec3_origin, vec3_origin, bounce, 0, 0, life,
				shader, flags);

		} else if (strcmp(cmd, "addtrail") == 0) {
			// A simple quad with unit colour, alpha and ST, which keeps the
			// command finite while still driving CTrail::Update and Draw.
			effectTrailArgStruct_t a;
			memset(&a, 0, sizeof(a));
			for (int i = 0; i < 4; i++) {
				fx_oracle_vec(&cursor, cmd, a.mVerts[i].origin);
				VectorSet(a.mVerts[i].rgb, 1.0f, 1.0f, 1.0f);
				VectorSet(a.mVerts[i].destrgb, 1.0f, 1.0f, 1.0f);
				VectorSet(a.mVerts[i].curRGB, 1.0f, 1.0f, 1.0f);
				a.mVerts[i].alpha = 1.0f;
				a.mVerts[i].destAlpha = 1.0f;
				a.mVerts[i].curAlpha = 1.0f;
				a.mVerts[i].ST[0] = 1.0f;
				a.mVerts[i].ST[1] = 1.0f;
				a.mVerts[i].destST[0] = 1.0f;
				a.mVerts[i].destST[1] = 1.0f;
				a.mVerts[i].curST[0] = 1.0f;
				a.mVerts[i].curST[1] = 1.0f;
			}
			a.mShader = fx_oracle_int(&cursor, cmd);
			a.mSetFlags = fx_oracle_int(&cursor, cmd);
			a.mKillTime = fx_oracle_int(&cursor, cmd);
			FX_FeedTrail(&a);

		} else if (strcmp(cmd, "advance") == 0) {
			// AdjustTime takes an absolute time, so the harness accumulates.
			// Source: `oracle/codemp/client/FxSystem.cpp:53-80`
			fx_oracle_clock_ms += fx_oracle_int(&cursor, cmd);
			FX_AdjustTime(fx_oracle_clock_ms);
			printf("TIME %d\n", fx_oracle_clock_ms);

		} else if (strcmp(cmd, "addscheduled") == 0) {
			FX_AddScheduledEffects((qboolean)!!fx_oracle_int(&cursor, cmd));

		} else if (strcmp(cmd, "draw2d") == 0) {
			float xscale = fx_oracle_float(&cursor, cmd);
			float yscale = fx_oracle_float(&cursor, cmd);
			FX_Draw2DEffects(xscale, yscale);

		} else if (strcmp(cmd, "dumpstate") == 0) {
			fx_oracle_dump_state();

		} else if (strcmp(cmd, "free") == 0) {
			FX_FreeSystem();

		} else if (strcmp(cmd, "reset") == 0) {
			FX_Free(false);

		} else {
			fprintf(stderr, "fx-oracle: unknown command '%s'\n", cmd);
			return 1;
		}
	}
	fclose(script);

	printf("== end ==\n");
	return 0;
}
