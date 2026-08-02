// cin-oracle: the seam the driver shares with the host stubs.
#ifndef CIN_ORACLE_HOST_H
#define CIN_ORACLE_HOST_H

// Raven's renderer config block. `readQuadInfo` reads `maxTextureSize` from it,
// and the driver holds that above 256 so the Rage Pro clamp never runs.
extern glconfig_t glConfig;

// Stops the run and names the out-of-gate function that was reached.
void cin_oracle_unreachable(const char *name);

#endif // CIN_ORACLE_HOST_H
