// terrainmap-oracle: the shared surface between the dumper and the host stubs.
#pragma once

#include <cstdarg>
#include <string>
#include <vector>

std::vector<byte> harnessReadFixture(const char *name);

extern std::vector<std::string> g_calls;
extern std::vector<byte> g_uploaded;
extern int g_uploadedWidth;
extern int g_uploadedHeight;

// PNG_Save is handed `mImage` itself, which is the only read-back of that
// private member the class offers.
extern std::vector<byte> g_saved;
extern int g_savedWidth;
extern int g_savedHeight;
