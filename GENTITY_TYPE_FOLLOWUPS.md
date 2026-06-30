# gentity_t Type Follow-ups

After the MP `gentity_t` layout port in `src/common/mp/gentity.rs`, resolve these placeholder dependencies:

- `Vehicle_t`
  - Placeholder field: `gentity_t::m_pVehicle`
  - Raven source: `oracle/oracle/codemp/game/g_local.h:137`
  - Candidate definition source: `oracle/oracle/codemp/game/bg_vehicles.h:477-623`

- `gclient_s`
  - Placeholder field: `gentity_t::client`
  - Raven source: `oracle/oracle/codemp/game/g_local.h:173`
  - Candidate definition source: `oracle/oracle/codemp/game/g_local.h:535`

- `gNPC_t`
  - Placeholder field: `gentity_t::NPC`
  - Raven source: `oracle/oracle/codemp/game/g_local.h:175`
  - Candidate definition source: `oracle/oracle/codemp/game/b_public.h:264`

Also resolve the older `src/game/entity/mod.rs` references to `crate::abi::{..., NUM_TIDS}` so the stale entity module follows the newer `src/common/{mp,sp}` organization.
