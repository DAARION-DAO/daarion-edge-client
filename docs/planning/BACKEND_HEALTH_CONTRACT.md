# DAARION Edge Backend Health Contract

This document defines the public backend health and compatibility contract for
DAARION Edge clients. It is intentionally limited to unauthenticated liveness
and version compatibility. It does not define signed identity, pairing proof,
operator authorization, worker relay, payments, federation, marketplace, or
Hermes-style catalog behavior.

## Canonical Endpoint

```text
GET /api/v1/edge/health
```

The endpoint must be safe, read-only, unauthenticated, and cache-resistant for
interactive diagnostics. Backends should return `Cache-Control: no-store`.

Clients must resolve the backend base URL through the existing pairing-aware
backend resolver before calling this endpoint. The health contract must never
mutate `pairing.json`, identity metadata, enrollment state, or worker state.

## Contract Stability

`GET /api/v1/edge/health` is the canonical backend health and compatibility
endpoint for DAARION Edge clients.

Future backend implementations must preserve backwards compatibility for
`schema_version: 1` or increment `schema_version`. Clients must reject
unsupported schema versions instead of guessing.

Unknown response fields are allowed for forward-compatible additions. Missing
required fields are not allowed.

## Response Schema

Successful responses must use HTTP `200` with JSON content:

```json
{
  "schema_version": 1,
  "status": "ok",
  "service": "daarion-edge-backend",
  "environment": "production",
  "backend_version": "0.1.0",
  "edge_protocol_version": "1.0.0",
  "min_edge_client_version": "0.2.2-3",
  "server_time": "2026-06-17T00:00:00Z",
  "capabilities": {
    "genesis": true,
    "registry": true,
    "model_registry": true,
    "voice_ceremony": false,
    "worker_relay": false
  }
}
```

Required fields:

| Field | Type | Rules |
|---|---|---|
| `schema_version` | integer | Must be `1` for this contract. |
| `status` | string | One of `ok`, `degraded`, `maintenance`. |
| `service` | string | Must be `daarion-edge-backend`. |
| `environment` | string | One of `production`, `staging`, `development`. |
| `backend_version` | string | Non-empty backend build/version identifier. |
| `edge_protocol_version` | string | SemVer-compatible protocol version. |
| `min_edge_client_version` | string | SemVer-compatible minimum client version. |
| `server_time` | string | RFC 3339 timestamp. |
| `capabilities` | object | Feature flags for backend-supported client surfaces. |

Known capability keys:

| Capability | Meaning |
|---|---|
| `genesis` | Backend supports Genesis registration endpoints. |
| `registry` | Backend supports node enrollment, heartbeat, and capability sync. |
| `model_registry` | Backend supports model registry fetches. |
| `voice_ceremony` | Backend supports voice imprint ceremony endpoints. |
| `worker_relay` | Backend advertises a worker relay surface. This does not imply worker authorization. |

Clients must tolerate unknown capability keys and treat missing known keys as
`false`.

## Client Interpretation Policy

The next connectivity implementation must map health checks to these client
states:

| Condition | Client state |
|---|---|
| No paired or effective backend exists | `pairing_required` |
| Request times out or network is unreachable | `offline` |
| HTTP `401` or `403` | `contract_invalid` |
| Any other non-2xx HTTP response | `offline` |
| Response is not valid JSON | `contract_invalid` |
| Required field is missing or has the wrong type | `contract_invalid` |
| `schema_version` is unsupported | `version_mismatch` |
| `service` is not `daarion-edge-backend` | `contract_invalid` |
| `edge_protocol_version` is incompatible | `version_mismatch` |
| Client version is lower than `min_edge_client_version` | `version_mismatch` |
| Valid response with `status: ok` | `online` |
| Valid response with `status: degraded` | `online_degraded` |
| Valid response with `status: maintenance` | `maintenance` |

HTTP `401` or `403` is a backend misconfiguration for this endpoint. Public
health must not require identity auth.

Timeout policy:

- Interactive health checks should use a 5 second timeout.
- Manual health checks should not retry automatically.
- Background checks, if introduced later, must use a conservative interval and
  must not block Genesis, enrollment, or dashboard rendering.

## Deferred Contracts

The following contracts are explicitly out of scope for this document:

- signed identity or pairing verification;
- operator approval diagnostics;
- worker relay liveness;
- OAuth, wallet, governance, payment, federation, or marketplace checks;
- backend discovery or short-code lookup.

Signed identity and pairing verification must be defined in a later trust/auth
contract before the client treats a backend as authorized for privileged actions.
