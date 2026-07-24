# ADR 0006: Frontend Tauri Invocation Boundary and Validator Assurance Model

- Status: Accepted
- Date: 2026-07-22
- Human approval date: 2026-07-24
- Scope: Frontend ownership of the Phase 1B.1 storage status command and the
  assurance claims made by repository validation
- Implementation verification: Merged and fresh-main verified after independent exact-head R6

ADR numbers 0004 and 0005 are reserved by the accepted roadmap for signed
pairing and signed readiness. This ADR therefore uses the next unreserved
number, 0006.

## Context

At ADR decision time, Phase 1B.1 PR #27 was open, draft and unmerged at
`fdbb9c88c2ef8c46f4a2a0abb4defc68c00c361c`, based on
`eb0d7def94675e5668f8a061ecc9e74b493c48c3`. Its independent review history is:

```text
INDEPENDENT_REVIEW_R1 = REVIEW_BLOCKED_BY_FINDINGS
INDEPENDENT_REVIEW_R2 = REVIEW_BLOCKED_BY_FINDINGS
INDEPENDENT_REVIEW_R3 = R3_REVIEW_PASS_WITH_NONBLOCKING_FINDINGS
INDEPENDENT_REVIEW_R4 = R4_REVIEW_BLOCKED_BY_FINDINGS
R4_BLOCKER = ASSIGNMENT_ALIAS_FALSE_NEGATIVE
INDEPENDENT_REVIEW_R5 = R5_REVIEW_BLOCKED_BY_FINDINGS
R5_BLOCKER = OBJECT_LITERAL_INVOKE_ALIAS_FALSE_NEGATIVE
```

That block is historical review evidence. The accepted architecture correction
was subsequently implemented at reviewed head
`5d894f42a967c9360d86382c1aab9e603472e0c8`, passed independent R6 with
nonblocking inherited findings, merged as
`cd903fb18d1618bbe0787d2397948622849ef9d4` at
`2026-07-24T11:44:00Z`, and passed fresh-main verification.

## Implementation status

```text
ADR_0006 = ACCEPTED / IMPLEMENTED / MERGED / FRESH_MAIN_VERIFIED
PHASE_1B_1 = MERGED / FRESH_MAIN_VERIFIED
MERGED_REVIEWED_HEAD = 5d894f42a967c9360d86382c1aab9e603472e0c8
MERGE_COMMIT = cd903fb18d1618bbe0787d2397948622849ef9d4
MERGED_AT = 2026-07-24T11:44:00Z
PRIMARY_CONTROL = COMMAND_SCOPED_MODULE_BOUNDARY_AND_IMPORT_GRAPH_GATE
SECONDARY_CONTROL = LIMITED_AST_CHECKS / DEFENSE_IN_DEPTH
ARBITRARY_TYPESCRIPT_DATA_FLOW_PROOF = NOT_CLAIMED
CUSTOM_FULL_DATA_FLOW_ANALYZER = REJECTED
GLOBAL_FRONTEND_ADAPTER_MIGRATION = DEFERRED / SEPARATE_PHASE
STORAGE_BOOTSTRAP = IMPLEMENTED_AND_VERIFIED
STORAGE_RUNTIME_PROJECTION = IMPLEMENTED_AND_VERIFIED_IN_REPOSITORY
DURABLE_RUNTIME_STATE = PARTIALLY_IMPLEMENTED
PHASE_1B = NOT COMPLETE
PHASE_1B_2 = NOT AUTHORIZED
REAL_DESKTOP_RESTART_FLOW = NOT VERIFIED
CROSS_PLATFORM_RUNTIME = NOT VERIFIED
REMOTE_CI = NOT PRESENT / NOT CLAIMED
REMOTE_PRODUCTION_WRITES = 0
REAL_USER_PROFILE_WRITES = 0
DEPLOYMENTS = 0
```

The frozen grandfathered Tauri-core importer baseline is exactly:

- `src/App.tsx`;
- `src/components/EdgeActivation.tsx`;
- `src/components/GenesisWizard.tsx`;
- `src/components/LocalModelsPanel.tsx`;
- `src/components/MessagingPanel.tsx`;
- `src/components/PairingGate.tsx`;
- `src/lib/backendConfig.ts`;
- `src/lib/inferenceClient.ts`;
- `src/lib/storageRuntimeClient.ts`.

These paths are not global adapter approval. `storageRuntimeClient.ts` remains
the sole executable frontend owner of `get_storage_runtime_status`, its command
constant is private, and it exports no raw Tauri binding. Rust retains one
read-only status command with no user-deserialized argument. No Phase 1B.2
CRUD/API authority exists.

Fresh-main verification used pinned Rust 1.95.0 and passed 64/64 storage,
67/67 inference and 180/180 full Rust tests, Cargo check, Cargo Clippy, 29/29
primary boundary fixtures, 13/13 defense-in-depth fixtures, 46 structural
checks, a production build over 1,763 modules, and production npm audit with
zero vulnerabilities. Runtime-store warning locations were 0. Dev-inclusive
npm audit retained 11 inherited advisories outside the production dependency
set; inherited RustSec, warning and rustfmt debt were unchanged.

The implementation preserved:

```text
MIGRATION_SHA = 62341c5015e70605bc3d57f9152c9e6a571739beaec33a2a7574b3e9a482575d
STRUCTURAL_FINGERPRINT = 37f9060cd050c615e2576809266ad9535e05c105f57a0a168bcf488f1f14ed77
TABLES = 5
EXPLICIT_INDEXES = 7
SQLITE_AUTOINDEXES = 7
SQLITE_SEQUENCE = 0
MIGRATION_2 = ABSENT
```

R4 showed that a Tauri `invoke` binding assigned after declaration could reach
the storage command without being detected. The fifth PR commit added
TypeChecker symbol identity and fixed-point propagation for declarations and
assignments. R5 then showed that a function value placed in an object-literal
property still reached the same command without being detected:

```ts
const rawInvokeHolder = { call: tauriCore.invoke };
void rawInvokeHolder.call("get_storage_runtime_status");
```

This second result changes the architectural question. Arrays, destructuring,
spreads, parameters, returns, higher-order functions, closures, class fields,
getters, re-exports and dynamic properties are additional JavaScript/TypeScript
flow mechanisms. Extending a repository script one syntax form at a time would
turn it into a custom call-graph, control-flow, data-flow and eventually
interprocedural analysis engine.

The current repository also does not have a global one-adapter Tauri boundary.
`src/App.tsx` and several existing components and libraries legitimately import
`@tauri-apps/api/core` for non-storage commands. Therefore a Phase 1B.1
correction cannot truthfully require that `src/lib/storageRuntimeClient.ts` be
the only Tauri core importer in the entire frontend. The stable near-term
boundary must be command-scoped, while a repository-wide adapter migration is
deferred to a separately planned change.

## Problem statement

The repository needs a deterministic gate that prevents accidental bypass of
the approved storage adapter without claiming to prove arbitrary TypeScript
data flow. It also needs a clear route to higher assurance if deliberate
obfuscation becomes part of the threat model.

The current assurance claim must change from:

> The validator proves that raw Tauri invoke cannot flow through TypeScript.

to:

> The gate enforces approved frontend module and command choke points. Limited
> AST checks provide defense in depth and are not a completeness proof for
> arbitrary TypeScript data flow.

## Threat model

### Threat class A: accidental architectural violation

Examples include a storage UI component importing Tauri core, a developer
adding a second storage status invocation, duplicating the storage command
literal, or bypassing the adapter through an ordinary import or re-export.

This class is in scope for the required repository gate. A complete source-file
inventory, import/export graph, exact adapter export allowlist, executable
command-literal ownership and Rust command inventory are proportionate controls.

### Threat class B: intentionally obscured TypeScript data flow

Examples include aliases carried through objects or arrays, wrapper functions,
closures, higher-order calls, return values, cross-file re-exports and dynamic
property access.

This class is not fully controlled by a repository-local syntactic script.
Reliable detection requires a maintained analysis framework with explicit
source/sink modeling and data-flow support. CodeQL provides local and global
JavaScript/TypeScript data-flow libraries; Semgrep capabilities vary between
the per-file Community Edition and advanced cross-file engines; a type-aware
ESLint rule can enforce a narrow API boundary but is not automatically a full
interprocedural proof.

### Threat class C: attacker controls application code and the validator

An in-repository validator cannot defend against an actor who can modify both
the code and its gate. This class belongs to protected branches, required
independent review, required checks from trusted workflows, CODEOWNERS or an
equivalent ownership rule, protected workflow files, and signed or audited
release processes where applicable.

## Accepted architectural choke point

The primary invariant is storage-command ownership, not global removal
of every legacy Tauri import:

1. `src/lib/storageRuntimeClient.ts` is the sole executable frontend owner of
   `get_storage_runtime_status`.
2. The command literal may occur as executable frontend code only in that
   adapter. Validator fixtures may contain it only in their explicitly
   identified test-data scope.
3. `storageRuntimeCommand` is module-private in the correction package. It must
   not be re-exported or made available to UI consumers.
4. The adapter's runtime-value export allowlist contains the typed operation and
   controlled error only. DTOs may be type-only exports. Raw `invoke`, a Tauri
   namespace, a dynamic-import result or a function returning those bindings
   must not be exported.
5. `src/components/StorageRuntimeCard.tsx` imports and calls only the typed
   storage adapter operation. It has no Tauri core import and no storage command
   literal.
6. `src/App.tsx` mounts the card and may retain its existing non-storage Tauri
   calls, but it must not own, import or construct the storage command.
7. Import and re-export graph validation covers every executable file under
   `src/`, not only three named source strings. It fails on a second storage
   adapter, command-literal owner, raw-binding export or storage UI bypass.
8. The repository records the current Tauri core importer inventory. New direct
   import sites require explicit review; migration of existing non-storage
   sites to adapters is deferred.
9. Rust retains exactly one explicitly registered
   `runtime_store::commands::get_storage_runtime_status` command. It remains
   read-only, accepts only Tauri-injected state/handle parameters, and exposes
   no path, SQL or content authority.
10. Any retained alias analyzer is `DEFENSE_IN_DEPTH`, never
    `COMPLETE_SECURITY_PROOF`.

For Phase 1B.1 the adapter remains a single file. An adapter directory would
add unnecessary surface before a second approved storage operation exists.

## Considered options

### Option A: module-boundary and import-graph gate

The gate enumerates frontend source modules, parses static and dynamic imports,
parses exports/re-exports, enforces command-literal ownership, verifies the
adapter runtime export allowlist, checks the storage UI dependency path, and
retains the exact Rust command/registration inventory.

- Guarantees: accidental storage boundary violations represented in the module
  and executable-literal graph; unexpected adapter/export growth; exact command
  and Rust registration drift.
- Non-guarantees: arbitrary intentionally obscured in-module data flow or an
  attacker who also modifies the gate.
- Complexity: small to medium.
- Maintenance: low when the allowlist is explicit and reviewed.
- False positives: low, with intentional changes requiring allowlist review.
- False negatives: low for threat class A; expected for threat class B.
- Fit: best default for Phase 1B.1.

### Option B: established data-flow static analysis

Candidates for a later, bounded evaluation are:

- a CodeQL JavaScript/TypeScript custom query using maintained local/global
  data-flow libraries, runnable through the CLI and CI subject to repository
  and licensing conditions;
- a Semgrep rule, with explicit recognition that Community Edition is per-file
  while advanced cross-file analysis uses a different product capability;
- a type-aware custom typescript-eslint rule for import/API ownership, with the
  understanding that type information alone does not provide complete
  interprocedural flow analysis.

- Guarantees: query- and tool-model dependent; potentially broader than the
  repository script.
- Non-guarantees: dynamic behavior outside the selected model and malicious
  modification of both code and the check.
- Complexity: medium to high, including query tests and CI ownership.
- Maintenance: delegated analysis engine, repository-owned source/sink model.
- False positives/negatives: must be measured against an adversarial fixture
  corpus before becoming required.
- Fit: optional higher assurance after the primary boundary is stable.

No tool is selected or installed by this ADR.

### Option C: consciously limited current AST validator

The existing script may be retained after its claims are narrowed to the exact
documented direct and local alias forms it tests. It must not claim arbitrary
cross-container, cross-function or cross-module coverage. The module boundary
is authoritative; the alias logic is secondary detection only.

- Guarantees: exact positive/negative fixtures and structural checks currently
  exercised.
- Non-guarantees: every unmodeled language flow, including the confirmed R5
  object-literal path.
- Complexity: already high for a repository-local contract script.
- Maintenance: grows with every claimed syntax form.
- False positives: controlled partly by symbol identity and safe fixtures.
- False negatives: demonstrated by R4 and R5.
- Fit: only as reduced defense in depth.

### Option D: custom complete data-flow analyzer

Completeness would require at least control-flow/SSA modeling, alias and
container flow, functions and returns, closures, higher-order calls,
imports/re-exports, property sensitivity, dynamic-language conservatism and
interprocedural analysis.

- Complexity: very high.
- Maintenance: disproportionate to the Phase 1B.1 command boundary.
- False-positive/negative risk: high without a dedicated analysis-engine team
  and language conformance corpus.
- Fit: rejected.

```text
CUSTOM_FULL_DATA_FLOW_ANALYZER = REJECTED / NOT_PROPORTIONATE
```

## Guarantee matrix

The labels below are exact assurance classifications for the proposed control
models, not implementation status.

| Flow or attacker behavior | A: module/import gate | B: established analyzer | C: limited current AST | D: custom complete analyzer |
| --- | --- | --- | --- | --- |
| Direct raw storage invoke outside adapter | GUARANTEED | GUARANTEED | GUARANTEED | GUARANTEED |
| Renamed Tauri import outside adapter | GUARANTEED | GUARANTEED | GUARANTEED | GUARANTEED |
| Namespace Tauri import outside adapter | GUARANTEED | GUARANTEED | GUARANTEED | GUARANTEED |
| Variable alias | PARTIALLY_DETECTED | PARTIALLY_DETECTED | GUARANTEED | GUARANTEED |
| Later assignment alias | PARTIALLY_DETECTED | PARTIALLY_DETECTED | GUARANTEED | GUARANTEED |
| Object-property alias | PARTIALLY_DETECTED | PARTIALLY_DETECTED | NOT_GUARANTEED | GUARANTEED |
| Array/tuple alias | PARTIALLY_DETECTED | PARTIALLY_DETECTED | NOT_GUARANTEED | GUARANTEED |
| Destructuring | PARTIALLY_DETECTED | PARTIALLY_DETECTED | PARTIALLY_DETECTED | GUARANTEED |
| Wrapper function | PARTIALLY_DETECTED | PARTIALLY_DETECTED | NOT_GUARANTEED | GUARANTEED |
| Function return-value flow | PARTIALLY_DETECTED | PARTIALLY_DETECTED | NOT_GUARANTEED | GUARANTEED |
| Re-export/barrel indirection | GUARANTEED | PARTIALLY_DETECTED | NOT_GUARANTEED | GUARANTEED |
| Dynamic property access | NOT_GUARANTEED | PARTIALLY_DETECTED | PARTIALLY_DETECTED | PARTIALLY_DETECTED |
| Malicious validator modification | OUT_OF_SCOPE | OUT_OF_SCOPE | OUT_OF_SCOPE | OUT_OF_SCOPE |

`PARTIALLY_DETECTED` for Option A means the boundary catches creation or export
of an unauthorized raw capability but does not follow every value once code is
inside an allowlisted module. `PARTIALLY_DETECTED` for Option B means coverage
depends on the selected engine and explicit source/sink/propagator model. Option
D remains a theoretical closed-world result and is not proposed for execution.

## Decision

```text
PRIMARY_CONTROL = COMMAND_SCOPED_MODULE_BOUNDARY_AND_IMPORT_GRAPH_GATE
SECONDARY_CONTROL = LIMITED_AST_CHECKS / DEFENSE_IN_DEPTH
ARBITRARY_TYPESCRIPT_DATA_FLOW_PROOF = NOT_CLAIMED
OPTIONAL_HIGHER_ASSURANCE = DEFERRED_TOOL_EVALUATION
CUSTOM_FULL_DATA_FLOW_ANALYZER = REJECTED
GLOBAL_FRONTEND_ADAPTER_MIGRATION = DEFERRED / SEPARATE_PHASE
```

The primary control is command-scoped for Phase 1B.1. A broader rule that all UI
components use adapters is a desirable target but is not current repository
truth and is not authorized in this correction.

## Security claims

If implemented and verified, the primary gate may claim:

- one executable frontend owner for `get_storage_runtime_status`;
- no storage command literal, adapter bypass or raw-binding export outside the
  approved storage adapter and named fixtures;
- one typed storage UI route;
- one read-only, no-user-argument Rust command registered exactly once;
- explicit failure on unreviewed changes to those inventories.

## Explicit non-guarantees

The gate does not prove:

- absence of every possible TypeScript function-value flow;
- correctness of unmodeled dynamic property or runtime code generation;
- absence of malicious behavior when the application and gate are changed by
  the same untrusted actor;
- security of other existing Tauri commands;
- production deployment, desktop behavior or Phase 1B.2 authorization.

## PR #27 disposition

Four dispositions were considered:

1. Replace the overclaimed validator before merge with the command-scoped
   module/import/re-export gate, narrow the alias claims, add boundary fixtures,
   and obtain a fresh exact-head review. **Selected by human approval on
   2026-07-24.**
2. Retain the current AST logic as defense in depth, fix or remove the failed
   claim, make the module boundary authoritative, and obtain a fresh exact-head
   review. Acceptable only if the resulting implementation is simpler than
   disposition 1 and the R5 fixture is no longer represented as covered.
3. Add an established static-analysis tool before merge. Higher assurance but
   disproportionate unless a separate tooling evaluation demonstrates value.
4. Accept the current false-negative. **Not recommended**, because current gate
   output and completion evidence assert a stronger boundary than the script
   proves.

Human approval on 2026-07-24 selected disposition 1 and authorized one bounded
architecture-correction package in PR #27. That decision did not itself verify
the implementation; independent exact-head R6, controlled merge and fresh-main
verification subsequently supplied the required evidence recorded above.

## Migration plan

### Stage 1: declare the boundary — accepted

- approve the command-scoped adapter, prohibited locations, runtime export
  allowlist and executable command-literal ownership;
- decide whether `storageRuntimeCommand` becomes private;
- freeze the current non-storage Tauri importer inventory as migration debt,
  not as a storage exception.

### Stage 2: rebuild the gate

- enumerate all frontend source files;
- enforce imports, dynamic imports, exports and re-exports;
- enforce the storage command literal and typed adapter ownership;
- retain or reduce local alias checks as defense in depth;
- replace completeness claims with the assurance model in this ADR;
- add boundary-bypass fixtures, not a growing promise of language completeness.

### Stage 3: independent review

- confirm no runtime, product, schema, manifest or lockfile change;
- run the bounded frontend/Rust contract, TypeScript build and documentation
  checks;
- perform a fresh independent exact-head review;
- decide PR #27 ready/merge eligibility separately.

### Stage 4: optional higher assurance

- evaluate CodeQL, Semgrep or type-aware ESLint against a shared adversarial
  corpus if multiple Tauri commands justify broader enforcement;
- record engine, license, local/CI availability, determinism, performance and
  ownership in a separate ADR before adding a required dependency or workflow.

## Consequences

### Positive

- The security claim becomes small, testable and stable across TypeScript syntax
  growth.
- The repository stops reimplementing a language analysis engine.
- Accidental architectural drift is blocked at the authority boundary.
- Higher assurance remains available without coupling Phase 1B.1 to a new tool.

### Negative

- Threat class B remains outside the primary gate.
- Existing non-storage raw Tauri imports remain technical debt.
- A future adapter-wide frontend migration requires its own plan and review.
- Branch protection and trusted CI remain necessary for threat class C.

## Review and branch-protection considerations

Validator and workflow changes require independent review. If the repository
later makes the gate a required status check, the workflow and rule source must
be protected consistently with the application boundary. A passing check from
mutable code is evidence only for the reviewed exact head, not an independent
trust root.

## Deferred work

- repository-wide migration of all direct frontend Tauri imports to typed
  adapters;
- selection and implementation of a maintained data-flow analysis tool;
- broader Tauri command authorization policy;
- Phase 1B.2 content services;
- deployment, real user-profile writes and platform runtime proof.

## Implementation verification gate

The architecture decision is Accepted and its Phase 1B.1 implementation has
passed the complete local gate, independent exact-head R6, controlled merge,
and fresh-main verification:

```text
R5_REVIEW = R5_REVIEW_BLOCKED_BY_FINDINGS / HISTORICAL
SIXTH_POINT_CORRECTION = REJECTED
ARCHITECTURAL_CORRECTION = IMPLEMENTED / MERGED / FRESH_MAIN_VERIFIED
R5_BLOCKER_DISPOSITION = CLOSED_BY_ARCHITECTURAL_BOUNDARY_AFTER_R6_CONFIRMATION
R6_REVIEW = R6_REVIEW_PASS_WITH_NONBLOCKING_FINDINGS
PR_27 = MERGED
MERGED_REVIEWED_HEAD = 5d894f42a967c9360d86382c1aab9e603472e0c8
MERGE_COMMIT = cd903fb18d1618bbe0787d2397948622849ef9d4
MERGED_AT = 2026-07-24T11:44:00Z
PHASE_1B_2 = NOT AUTHORIZED
```

## References

- [Validator architecture audit](../audits/FRONTEND_TAURI_INVOCATION_VALIDATOR_ARCHITECTURE_AUDIT_2026-07-22.md)
- [CodeQL JavaScript/TypeScript data-flow guide](https://codeql.github.com/docs/codeql-language-guides/analyzing-data-flow-in-javascript-and-typescript/)
- [CodeQL CLI](https://docs.github.com/en/code-security/concepts/code-scanning/codeql/codeql-cli)
- [Semgrep analysis glossary](https://semgrep.dev/docs/writing-rules/glossary)
- [typescript-eslint custom rules](https://typescript-eslint.io/developers/custom-rules/)
- [typescript-eslint typed linting](https://typescript-eslint.io/getting-started/typed-linting/)
