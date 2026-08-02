// terrainmap-oracle stub: cm_terrainmap.cpp calls PNG_Save from
// SaveImageToDisk. The harness records the call and writes nothing.
#pragma once

bool PNG_Save(const char *name, byte *data, int width, int height, int bytedepth);
