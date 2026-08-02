// terrainmap-oracle: the differential dumper for cm_draw.cpp and
// cm_terrainmap.cpp.
//
// Both TUs are the UNMODIFIED Raven sources (porting-rules §F18); this file and
// src/host_stubs.cpp supply the stubbed environment. Every fixture is
// synthetic, from fixtures/gen_fixtures.py.
#include "build/codemp/qcommon/exe_headers.h"

#include <string>
#include <vector>

#include "build/codemp/qcommon/cm_draw.h"
#include "build/codemp/qcommon/cm_landscape.h"
#include "build/codemp/qcommon/cm_terrainmap.h"

#include "src/harness.h"

std::vector<byte> g_saved;
int g_savedWidth = 0;
int g_savedHeight = 0;

#ifndef FIXTURE_ROOT
#define FIXTURE_ROOT "fixtures"
#endif

std::vector<byte> harnessReadFixture(const char *name)
{
	std::string path = std::string(FIXTURE_ROOT) + "/" + name;
	FILE *f = fopen(path.c_str(), "rb");
	if (!f)
	{
		fprintf(stderr, "terrainmap-oracle: cannot open %s\n", path.c_str());
		exit(1);
	}
	fseek(f, 0, SEEK_END);
	long size = ftell(f);
	fseek(f, 0, SEEK_SET);
	std::vector<byte> out((size_t)size);
	if (size && fread(out.data(), 1, (size_t)size, f) != (size_t)size)
	{
		fprintf(stderr, "terrainmap-oracle: short read on %s\n", path.c_str());
		exit(1);
	}
	fclose(f);
	return out;
}

static unsigned int fnv1a(const byte *data, size_t len)
{
	unsigned int h = 2166136261u;
	for (size_t i = 0; i < len; i++)
	{
		h ^= data[i];
		h *= 16777619u;
	}
	return h;
}

// ---------------------------------------------------------------------------
// scenario: draw
// ---------------------------------------------------------------------------

static const int DW = 32;
static const int DH = 24;

static CPixel32 g_drawBuf[DW * DH];

static void resetPattern(void)
{
	for (int y = 0; y < DH; y++)
	{
		for (int x = 0; x < DW; x++)
		{
			g_drawBuf[y * DW + x] = CPixel32((byte)(x * 5), (byte)(y * 7), (byte)(x ^ y), 200);
		}
	}
}

static void dumpDrawBuf(const char *label)
{
	printf("-- %s\n", label);
	const byte *raw = (const byte *)g_drawBuf;
	for (int y = 0; y < DH; y++)
	{
		printf("row %02d ", y);
		for (int x = 0; x < DW; x++)
		{
			const CPixel32 &p = g_drawBuf[y * DW + x];
			printf("%02x%02x%02x%02x", p.r, p.g, p.b, p.a);
		}
		printf("\n");
	}
	printf("hash %08x\n", fnv1a(raw, sizeof(g_drawBuf)));
}

static void dumpClip(void)
{
	long a, b, c, d;
	CDraw32::GetClip(a, b, c, d);
	printf("clip %ld %ld %ld %ld\n", a, b, c, d);
}

static void scenarioDraw(void)
{
	CDraw32 draw;
	draw.SetBuffer(g_drawBuf);
	draw.SetBufferSize(DW, DH, DW);

	printf("== draw %dx%d\n", DW, DH);
	dumpClip();

	std::vector<byte> symBytes = harnessReadFixture("sym_start.rgba");
	CPixel32 *src = (CPixel32 *)symBytes.data();

	const CPixel32 solid(220, 40, 90, 255);
	const CPixel32 half(40, 220, 90, 128);
	const CPixel32 clear(0, 0, 0, 0);

	resetPattern();
	draw.ClearBuffer(CPixel32(10, 20, 30, 40));
	draw.SetAlphaLines(77, 4, 9);
	dumpDrawBuf("clear");

	resetPattern();
	draw.DrawLine(0, 0, DW - 1, DH - 1, solid);
	draw.DrawLine(DW - 1, 0, 0, DH - 1, solid);
	draw.DrawLine(2, 5, 29, 5, solid);
	draw.DrawLine(29, 7, 2, 7, solid);
	draw.DrawLine(10, 0, 10, DH - 1, solid);
	draw.DrawLine(12, DH - 1, 12, 0, solid);
	draw.DrawLine(-20, -10, 50, 30, solid);
	draw.DrawLine(-5, -5, -1, -1, solid);
	dumpDrawBuf("line_solid");

	resetPattern();
	draw.DrawLine(0, 0, DW - 1, DH - 1, half);
	draw.DrawLine(2, 5, 29, 5, half);
	draw.DrawLine(29, 7, 2, 7, half);
	draw.DrawLine(10, 0, 10, DH - 1, half);
	draw.DrawLine(-20, -10, 50, 30, half);
	dumpDrawBuf("line_alpha");

	resetPattern();
	draw.DrawLineAve(2, 3, 29, 3, solid);
	draw.DrawLineAve(29, 6, 2, 6, solid);
	draw.DrawLineAve(4, 0, 4, DH - 1, solid);
	draw.DrawLineAve(6, DH - 1, 6, 0, solid);
	draw.DrawLineAve(0, 0, DW - 1, DH - 1, half);
	dumpDrawBuf("line_ave");

	resetPattern();
	draw.DrawLineAA(1, 1, 30, 22, solid);
	draw.DrawLineAA(30, 1, 1, 22, half);
	draw.DrawLineAA(1, 3, 30, 3, solid);
	draw.DrawLineAA(5, 0, 5, DH - 1, solid);
	draw.DrawLineAA(0, 0, 22, 22, solid);
	draw.DrawLineAA(-10, 5, 40, 19, solid);
	dumpDrawBuf("line_aa");

	resetPattern();
	draw.DrawRect(3, 3, 10, 6, solid);
	draw.DrawRect(25, 18, 12, 10, half);
	draw.DrawRectNC(1, 15, 8, 4, solid);
	draw.DrawRectAve(14, 10, 9, 7, solid);
	draw.DrawRect(4, 4, 0, 5, solid);
	dumpDrawBuf("rect");

	resetPattern();
	draw.DrawBox(2, 2, 12, 9, solid);
	draw.DrawBox(-3, -3, 10, 10, half);
	draw.DrawBoxNC(20, 14, 10, 8, solid);
	draw.DrawBoxAve(6, 12, 14, 10, solid);
	dumpDrawBuf("box");

	resetPattern();
	draw.DrawCircle(16, 12, 7, solid, half);
	draw.DrawCircle(4, 4, 3, solid, clear);
	draw.DrawCircle(28, 20, 5, clear, solid);
	draw.DrawCircle(16, 12, 1, solid, half);
	draw.DrawCircle(16, 12, 0, solid, half);
	draw.DrawCircle(2, 20, 9, solid, half);
	dumpDrawBuf("circle");

	resetPattern();
	draw.DrawCircleAve(16, 12, 9, solid, half);
	draw.DrawCircleAve(2, 2, 4, solid, half);
	draw.DrawCircleAve(30, 22, 6, clear, solid);
	dumpDrawBuf("circle_ave");

	resetPattern();
	{
		POINT tri[3] = {{4, 2}, {28, 8}, {10, 21}};
		draw.DrawPolygon(3, tri, solid, half);
	}
	dumpDrawBuf("poly_tri");

	resetPattern();
	{
		// concave quad
		POINT quad[4] = {{2, 2}, {29, 4}, {14, 12}, {27, 21}};
		draw.DrawPolygon(4, quad, solid, half);
	}
	dumpDrawBuf("poly_concave");

	resetPattern();
	{
		// the AddPlayer arrowhead shape
		POINT arrow[4] = {{16, 12}, {11, 7}, {26, 12}, {11, 17}};
		draw.DrawPolygon(4, arrow, CPixel32(0, 0, 0, 128), CPixel32(0, 0, 0, 128));
		POINT arrow2[4] = {{15, 11}, {10, 6}, {25, 11}, {10, 16}};
		draw.DrawPolygon(4, arrow2, CPixel32(255, 255, 255, 255), CPixel32(255, 255, 255, 255));
	}
	dumpDrawBuf("poly_arrow");

	resetPattern();
	{
		// degenerate: a single scanline, and a zero-vertex call
		POINT flat[3] = {{3, 9}, {20, 9}, {12, 9}};
		draw.DrawPolygon(3, flat, solid, half);
		draw.DrawPolygon(0, flat, solid, half);
		// off-screen entirely
		POINT away[3] = {{-40, -40}, {-30, -35}, {-38, -28}};
		draw.DrawPolygon(3, away, solid, half);
	}
	dumpDrawBuf("poly_degenerate");

	resetPattern();
	draw.Blit(2, 2, 16, 16, src, 0, 0, 16);
	draw.Blit(20, 16, 16, 16, src, 0, 0, 16);
	draw.Blit(-4, -3, 16, 16, src, 0, 0, 16);
	dumpDrawBuf("blit");

	resetPattern();
	draw.BlitColor(2, 2, 16, 16, src, 0, 0, 16, solid);
	draw.BlitColor(20, 16, 16, 16, src, 0, 0, 16, half);
	draw.BlitColor(-4, -3, 16, 16, src, 0, 0, 16, solid);
	dumpDrawBuf("blit_color");

	resetPattern();
	draw.Emboss(4, 4, 16, 16, src, 0, 0, 16);
	dumpDrawBuf("emboss");

	resetPattern();
	draw.SetClip(5, 4, 20, 15);
	dumpClip();
	draw.DrawLine(0, 0, DW - 1, DH - 1, solid);
	draw.DrawLine(DW - 1, 0, 0, DH - 1, solid);
	draw.DrawLine(0, 9, DW - 1, 9, solid);
	draw.DrawRect(0, 0, DW, DH, half);
	draw.DrawCircle(12, 9, 8, solid, half);
	{
		POINT quad[4] = {{2, 2}, {29, 4}, {14, 12}, {27, 21}};
		draw.DrawPolygon(4, quad, solid, half);
	}
	draw.Blit(0, 0, 16, 16, src, 0, 0, 16);
	dumpDrawBuf("clipped");

	draw.SetClip(0, 0, DW - 1, DH - 1);
	dumpClip();
	CDraw32::CleanUp();
}

// ---------------------------------------------------------------------------
// scenario: terrainmap
// ---------------------------------------------------------------------------

static void dumpBuffer(const char *label, const std::vector<byte> &buf, int width, int height)
{
	printf("-- %s %d %d\n", label, width, height);
	if (buf.empty())
	{
		printf("empty\n");
		return;
	}
	for (int y = 0; y < height; y++)
	{
		printf("row %03d %08x\n", y, fnv1a(&buf[(size_t)y * (size_t)width * 4], (size_t)width * 4));
	}
	printf("hash %08x\n", fnv1a(buf.data(), buf.size()));
}

static void dumpCalls(void)
{
	for (size_t i = 0; i < g_calls.size(); i++)
	{
		printf("call %s\n", g_calls[i].c_str());
	}
	g_calls.clear();
}

static void scenarioTerrainMap(void)
{
	std::vector<byte> height = harnessReadFixture("heightmap.bin");
	// one trailing pad byte: see the note in stubs/qcommon/cm_landscape.h
	height.push_back(0);

	CCMLandScape land;
	land.mHeightMap = height.data();
	land.mRealWidth = 65;
	land.mRealHeight = 65;
	land.mBaseWaterHeight = 40;
	land.mMins[0] = -2048.0f;
	land.mMins[1] = -2048.0f;
	land.mMins[2] = -512.0f;
	land.mSize[0] = 4096.0f;
	land.mSize[1] = 4096.0f;
	land.mSize[2] = 1024.0f;

	printf("== terrainmap %d %d\n", TM_WIDTH, TM_HEIGHT);

	CM_TM_Create(&land);
	dumpCalls();

	CM_TM_SaveImageToDisk("t0", "m0", "s0");
	dumpCalls();
	dumpBuffer("image_after_ctor", g_saved, g_savedWidth, g_savedHeight);

	CM_TM_AddBuilding(-1000, 500, SIDE_BLUE);
	CM_TM_AddBuilding(1200, -800, SIDE_RED);
	CM_TM_AddBuilding(-3000, 3000, SIDE_NONE);
	CM_TM_AddStart(-1800, -1800, SIDE_BLUE);
	CM_TM_AddEnd(1800, 1800, SIDE_RED);
	CM_TM_AddObjective(0, 0, SIDE_NONE);
	CM_TM_AddNPC(300, -300, true);
	CM_TM_AddNPC(-300, 300, false);
	CM_TM_AddNode(700, 700);
	CM_TM_AddNode(-2040, -2040);
	CM_TM_AddWallRect(-500, -500, SIDE_BLUE);
	CM_TM_AddWallRect(-460, -500, SIDE_RED);
	CM_TM_AddWallRect(-420, -500, SIDE_NONE);

	CM_TM_SaveImageToDisk("t1", "m1", "s1");
	dumpCalls();
	dumpBuffer("image_after_symbols", g_saved, g_savedWidth, g_savedHeight);

	vec3_t origin = {100.0f, -200.0f, 64.0f};
	vec3_t angles = {0.0f, 37.5f, 0.0f};
	CM_TM_Upload(origin, angles);
	dumpCalls();
	dumpBuffer("upload_with_player", g_uploaded, g_uploadedWidth, g_uploadedHeight);

	CM_TM_Upload(NULL, angles);
	dumpCalls();
	dumpBuffer("upload_no_player", g_uploaded, g_uploadedWidth, g_uploadedHeight);

	static const int coords[][2] = {
		{0, 0}, {-2048, -2048}, {2047, 2047}, {-1000, 500}, {1200, -800}, {700, 700},
		{-3000, 3000}, {123, -456},
	};
	for (size_t i = 0; i < sizeof(coords) / sizeof(coords[0]); i++)
	{
		int x = coords[i][0];
		int y = coords[i][1];
		CM_TM_ConvertPosition(x, y, TM_WIDTH, TM_HEIGHT);
		printf("convert %d %d -> %d %d\n", coords[i][0], coords[i][1], x, y);
	}

	CM_TM_Free();
	dumpCalls();
}

int main(int argc, char **argv)
{
	if (argc < 2)
	{
		fprintf(stderr, "usage: terrainmap_dump <draw|terrainmap>\n");
		return 1;
	}
	if (!strcmp(argv[1], "draw"))
	{
		scenarioDraw();
		return 0;
	}
	if (!strcmp(argv[1], "terrainmap"))
	{
		scenarioTerrainMap();
		return 0;
	}
	fprintf(stderr, "terrainmap-oracle: unknown scenario %s\n", argv[1]);
	return 1;
}
