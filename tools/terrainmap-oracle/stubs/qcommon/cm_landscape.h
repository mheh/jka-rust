// terrainmap-oracle stub: the CCMLandScape surface cm_terrainmap.cpp reads.
//
// Raven's real class carries the whole terrain brush arena. The TU under test
// names six accessors only, so the harness declares exactly those and fills
// them from a committed fixture. The oracle is never edited (porting-rules
// §F18).
#pragma once

class CCMLandScape
{
public:
	// The heightmap is allocated with one trailing pad byte set to 0.
	// ApplyHeightmap starts xRel at width, so Raven's index runs one byte past
	// the buffer on the last row; the pad makes that read deterministic and
	// equal to the Rust port's defined 0 (porting-rules §F19).
	byte *mHeightMap;
	int mRealWidth;
	int mRealHeight;
	int mBaseWaterHeight;
	vec3_t mMins;
	vec3_t mSize;

	byte *GetHeightMap(void) const { return mHeightMap; }
	int GetRealWidth(void) const { return mRealWidth; }
	int GetRealHeight(void) const { return mRealHeight; }
	int GetBaseWaterHeight(void) const { return mBaseWaterHeight; }
	const vec3_t &GetMins(void) const { return mMins; }
	const vec3_t &GetSize(void) const { return mSize; }
};
