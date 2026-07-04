# Repository Role

Status: public sanitized repository role card

This repository is part of the DAARION.city / DAGI / MicroDAO ecosystem.

The full canonical repository ownership map lives in the private operations
repository: `IvanTytar/microdao-daarion`.

This public repository contains only a sanitized local role summary.

## Role

`DAARION-DAO/daarion-edge-client` is the public user-installed Edge Client.

It is the Tauri/Rust local device runtime for local identity, pairing, capability
detection, backend health checks, and future gated worker sandbox behavior.

## Owns

- installer/runtime;
- local device identity;
- pairing state;
- backend health client;
- local health checks;
- local models and capabilities;
- future local worker sandbox only after a separate security/governance gate.

## Does Not Own

- public backend uptime;
- DNS/TLS/firewall;
- NODA inventory;
- production backend profile rows;
- Edge Backend server runtime;
- live node-network operations truth.

## Related Repositories

- `DAARION-DAO/daarion-ai-city` - public city frontend.
- `DAARION-DAO/loval-echoes` - MicroDAO web app and Connect Device UI.
- `DAARION-DAO/daarion-edge-backend` - backend health/API contract.
- `IvanTytar/microdao-daarion` - private operations truth.

## Public / Private Boundary

Public repositories may contain source code, public contracts, sanitized docs,
generic examples, and non-sensitive roadmap notes.

This repository must not contain live NODA/IP/DNS/firewall/Octelium/deployment
truth, private production runbooks, incidents, secrets, operator access details,
or private infrastructure evidence.

Live node-network operations truth belongs in the private
`IvanTytar/microdao-daarion` repository.
