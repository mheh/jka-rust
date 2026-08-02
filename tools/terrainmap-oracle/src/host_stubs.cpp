// terrainmap-oracle: the engine services the two TUs under test call out to.
//
// Every stub is deterministic and side-effect free apart from the recorded
// call log, so the dump is reproducible.
#include "../build/codemp/qcommon/exe_headers.h"

#include <string>
#include <vector>

#include "harness.h"

void *Z_Malloc(int size, int tag, qboolean zeroit)
{
	(void)tag;
	void *p = malloc((size_t)size);
	if (zeroit && p)
	{
		memset(p, 0, (size_t)size);
	}
	return p;
}

void Z_Free(void *ptr)
{
	free(ptr);
}

char *va(const char *format, ...)
{
	static char buf[4096];
	va_list argptr;
	va_start(argptr, format);
	vsnprintf(buf, sizeof(buf), format, argptr);
	va_end(argptr);
	return buf;
}

// q_math.c's rotation, transcribed here so the harness needs no q_math TU.
// Source: oracle/codemp/qcommon/q_math.c (RotatePointAroundVector)
static void CrossProduct(const vec3_t v1, const vec3_t v2, vec3_t cross)
{
	cross[0] = v1[1] * v2[2] - v1[2] * v2[1];
	cross[1] = v1[2] * v2[0] - v1[0] * v2[2];
	cross[2] = v1[0] * v2[1] - v1[1] * v2[0];
}

static vec_t VectorLength(const vec3_t v)
{
	return (vec_t)sqrt(v[0] * v[0] + v[1] * v[1] + v[2] * v[2]);
}

static vec_t VectorNormalize(vec3_t v)
{
	float length = (float)sqrt(v[0] * v[0] + v[1] * v[1] + v[2] * v[2]);
	if (length)
	{
		float ilength = 1 / length;
		v[0] *= ilength;
		v[1] *= ilength;
		v[2] *= ilength;
	}
	return length;
}

void RotatePointAroundVector(vec3_t dst, const vec3_t dir, const vec3_t point, float degrees)
{
	float m[3][3];
	float im[3][3];
	float zrot[3][3];
	float tmpmat[3][3];
	float rot[3][3];
	int i;
	vec3_t vr, vup, vf;
	float rad;

	vf[0] = dir[0];
	vf[1] = dir[1];
	vf[2] = dir[2];

	// PerpendicularVector( vr, dir );
	{
		int pos = 0;
		float minelem = 1.0F;
		vec3_t tempvec;
		for (i = 0; i < 3; i++)
		{
			if (fabs(dir[i]) < minelem)
			{
				pos = i;
				minelem = (float)fabs(dir[i]);
			}
		}
		tempvec[0] = tempvec[1] = tempvec[2] = 0.0F;
		tempvec[pos] = 1.0F;
		// ProjectPointOnPlane( dst, tempvec, dir )
		{
			float d;
			vec3_t n;
			float inv_denom = 1.0F / (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]);
			d = (tempvec[0] * dir[0] + tempvec[1] * dir[1] + tempvec[2] * dir[2]) * inv_denom;
			n[0] = dir[0] * inv_denom;
			n[1] = dir[1] * inv_denom;
			n[2] = dir[2] * inv_denom;
			vr[0] = tempvec[0] - d * n[0];
			vr[1] = tempvec[1] - d * n[1];
			vr[2] = tempvec[2] - d * n[2];
		}
		VectorNormalize(vr);
	}

	CrossProduct(vr, vf, vup);

	m[0][0] = vr[0];
	m[1][0] = vr[1];
	m[2][0] = vr[2];

	m[0][1] = vup[0];
	m[1][1] = vup[1];
	m[2][1] = vup[2];

	m[0][2] = vf[0];
	m[1][2] = vf[1];
	m[2][2] = vf[2];

	memcpy(im, m, sizeof(im));

	im[0][1] = m[1][0];
	im[0][2] = m[2][0];
	im[1][0] = m[0][1];
	im[1][2] = m[2][1];
	im[2][0] = m[0][2];
	im[2][1] = m[1][2];

	memset(zrot, 0, sizeof(zrot));
	zrot[0][0] = zrot[1][1] = zrot[2][2] = 1.0F;

	rad = (float)(degrees * (M_PI * 2 / 360));
	zrot[0][0] = (float)cos(rad);
	zrot[0][1] = (float)sin(rad);
	zrot[1][0] = (float)-sin(rad);
	zrot[1][1] = (float)cos(rad);

	// MatrixMultiply( m, zrot, tmpmat );
	for (i = 0; i < 3; i++)
	{
		for (int j = 0; j < 3; j++)
		{
			tmpmat[i][j] = m[i][0] * zrot[0][j] + m[i][1] * zrot[1][j] + m[i][2] * zrot[2][j];
		}
	}
	// MatrixMultiply( tmpmat, im, rot );
	for (i = 0; i < 3; i++)
	{
		for (int j = 0; j < 3; j++)
		{
			rot[i][j] = tmpmat[i][0] * im[0][j] + tmpmat[i][1] * im[1][j] + tmpmat[i][2] * im[2][j];
		}
	}

	for (i = 0; i < 3; i++)
	{
		dst[i] = rot[i][0] * point[0] + rot[i][1] * point[1] + rot[i][2] * point[2];
	}

	(void)VectorLength;
}

// --- the renderer half -------------------------------------------------------

std::vector<std::string> g_calls;
std::vector<byte> g_uploaded;
int g_uploadedWidth = 0;
int g_uploadedHeight = 0;

typedef unsigned int GLenum;

void R_LoadImage(const char *name, byte **pic, int *width, int *height, GLenum *format)
{
	*format = 0x1908; // GL_RGBA
	*pic = NULL;

	const char *file = NULL;
	int w = 0, h = 0;
	if (!strcmp(name, "gfx\\menus\\rmg\\01_bg"))
	{
		file = "bg.rgba";
		w = h = 64;
	}
	else if (!strcmp(name, "gfx/menus/rmg/start"))
	{
		file = "sym_start.rgba";
		w = h = 16;
	}
	else if (!strcmp(name, "gfx/menus/rmg/end"))
	{
		file = "sym_end.rgba";
		w = h = 16;
	}
	else if (!strcmp(name, "gfx/menus/rmg/objective"))
	{
		file = "sym_objective.rgba";
		w = h = 16;
	}
	else if (!strcmp(name, "gfx/menus/rmg/building"))
	{
		file = "sym_bld.rgba";
		w = h = 16;
	}

	g_calls.push_back(std::string("R_LoadImage ") + name);
	if (!file)
	{
		return;
	}

	std::vector<byte> bytes = harnessReadFixture(file);
	byte *out = (byte *)Z_Malloc((int)bytes.size(), 0, qfalse);
	memcpy(out, bytes.data(), bytes.size());
	*pic = out;
	*width = w;
	*height = h;
}

void R_CreateAutomapImage(const char *name, const byte *pic, int width, int height,
						  qboolean mipmap, qboolean allowPicmip, qboolean allowTC, int glWrapClampMode)
{
	char line[256];
	snprintf(line, sizeof(line), "R_CreateAutomapImage %s %d %d %d %d %d %d",
			 name, width, height, (int)mipmap, (int)allowPicmip, (int)allowTC, glWrapClampMode);
	g_calls.push_back(line);

	g_uploaded.assign(pic, pic + (size_t)width * (size_t)height * 4);
	g_uploadedWidth = width;
	g_uploadedHeight = height;
}

bool PNG_Save(const char *name, byte *data, int width, int height, int bytedepth)
{
	char line[256];
	snprintf(line, sizeof(line), "PNG_Save %s %d %d %d", name, width, height, bytedepth);
	g_calls.push_back(line);

	g_saved.assign(data, data + (size_t)width * (size_t)height * (size_t)bytedepth);
	g_savedWidth = width;
	g_savedHeight = height;
	return true;
}
