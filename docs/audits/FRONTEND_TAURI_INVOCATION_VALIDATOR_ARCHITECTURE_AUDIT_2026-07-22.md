# Frontend Tauri Invocation Validator Architecture Audit — 2026-07-22

Status: **COMPLETE / HUMAN DECISION RECORDED / IMPLEMENTATION R6-GATED**

## Scope and evidence boundary

This audit evaluates the frontend/Rust contract validator on Phase 1B.1 PR #27.
It does not correct the confirmed R5 object-literal bypass, change application
code, authorize merge, or authorize Phase 1B.2.

Repository state observed on 2026-07-22:

```text
REPOSITORY = DAARION-DAO/daarion-edge-client
PR_27 = OPEN / DRAFT / MERGEABLE / CLEAN / NOT_MERGED
BASE = eb0d7def94675e5668f8a061ecc9e74b493c48c3
HEAD = fdbb9c88c2ef8c46f4a2a0abb4defc68c00c361c
COMMITS = 5
R4_REVIEWED_HEAD = 7465673d0128e850d30f1b8f00c7c102d69b983a
R5_REVIEWED_HEAD = fdbb9c88c2ef8c46f4a2a0abb4defc68c00c361c
```

Final readback on 2026-07-24 confirmed that this PR state, base, head and commit
count were unchanged.

The implementation worktree and detached R1, R2, R3, R4 and R5 review
worktrees were clean at their expected commits. PR metadata, source and review
history were inspected read-only. The two documents in this package were
written only in a detached local worktree from exact `main`; they were not
committed or pushed.

## Executive conclusion

The current validator is a useful contract regression test but is not a stable
security proof for arbitrary TypeScript data flow. It loads three selected
frontend files independently, creates a TypeScript Program with `noResolve`,
tracks two manually maintained symbol sets, and propagates only explicitly
implemented syntax forms. R4 and R5 demonstrate that every newly supported
alias family can expose another unmodeled carrier.

```text
CUSTOM_FULL_DATA_FLOW_ANALYZER = NOT_RECOMMENDED
RECOMMENDED_PRIMARY_CONTROL = COMMAND_SCOPED_MODULE_BOUNDARY_AND_IMPORT_GRAPH_GATE
RECOMMENDED_SECONDARY_CONTROL = LIMITED_AST_CHECKS / DEFENSE_IN_DEPTH
ARBITRARY_TYPESCRIPT_DATA_FLOW_PROOF = NOT_CLAIMED
OPTIONAL_HIGHER_ASSURANCE = DEFERRED_TOOL_EVALUATION
CUSTOM_FULL_DATA_FLOW_ANALYZER = REJECTED
```

The primary boundary must be storage-command scoped. The current frontend has
nine direct `@tauri-apps/api/core` import sites for several unrelated commands,
so the repository cannot truthfully claim that the storage adapter is already
the only global Tauri importer. The stable Phase 1B.1 invariant is that the
storage adapter exclusively owns `get_storage_runtime_status`, exports no raw
capability or command constant, and is the only route used by the storage UI.

## Facts observed in current source

### Validator inputs and architecture

`scripts/validate-storage-runtime-contract.mjs`:

- reads only `src/lib/storageRuntimeClient.ts`,
  `src/components/StorageRuntimeCard.tsx`, `src/App.tsx`, selected Rust files and
  `package.json` into its main validation candidate;
- creates one in-memory TypeScript Program per source with `noLib: true`,
  `noResolve: true` and a host that exposes only that one file;
- seeds named and namespace symbols imported from `@tauri-apps/api/core`;
- recursively unwraps casts, parentheses, non-null and await expressions;
- manually recognizes selected conditionals and binary operators;
- uses a fixed-point over variable declarations and direct `=` assignments;
- has special handling for namespace destructuring assignments;
- checks the typed client, card, app mount, Rust command/registration, DTO
  parity, UI state labels and forbidden Rust command names;
- runs in-memory positive and negative fixtures;
- does not build the repository import graph or resolve imports/re-exports.

The installed TypeScript TypeChecker supplies symbol identity, but symbol
identity alone is not a call graph, control-flow graph, SSA engine, property
flow model, taint model or interprocedural analysis.

### Current frontend storage boundary

Observed executable route:

```text
src/components/StorageRuntimeCard.tsx
  -> getStorageRuntimeStatus()
src/lib/storageRuntimeClient.ts
  -> invoke<StorageRuntimeStatus>("get_storage_runtime_status")
src-tauri/src/runtime_store/commands.rs
  -> RuntimeStoreManager::read_status()
```

Facts:

- `src/lib/storageRuntimeClient.ts` directly imports `invoke`, exports the
  command constant, exports DTO types and errors, and exposes a zero-argument
  typed operation.
- `src/components/StorageRuntimeCard.tsx` imports the typed operation and DTO
  types, contains no Tauri core import, and calls the adapter during refresh.
- `src/App.tsx` mounts `StorageRuntimeCard` but also directly imports Tauri core
  and invokes unrelated commands.
- Eight additional existing frontend modules directly import Tauri core. The
  repository therefore has nine direct import sites in total.
- No frontend barrel or re-export of the storage adapter was found in the
  current `src/` tree.
- `src-tauri/src/runtime_store/commands.rs` defines one async read-status
  command whose only parameter is a Tauri `AppHandle`.
- `src-tauri/src/lib.rs` registers the command once in `generate_handler!`.
- The Rust command exposes no user path, SQL, content mutation or generic
  storage authority.

Capability classification:

```text
TYPED_STORAGE_ADAPTER = IMPLEMENTED_AND_VERIFIED
STORAGE_UI_ADAPTER_USAGE = IMPLEMENTED_AND_VERIFIED
RUST_READ_ONLY_STATUS_COMMAND = IMPLEMENTED_AND_VERIFIED
REPOSITORY_WIDE_TAURI_ADAPTER_BOUNDARY = MISSING
ARBITRARY_TYPESCRIPT_INVOKE_FLOW_PROOF = MISSING
```

### Preserved review history

```text
INDEPENDENT_REVIEW_R1 = REVIEW_BLOCKED_BY_FINDINGS
INDEPENDENT_REVIEW_R2 = REVIEW_BLOCKED_BY_FINDINGS
INDEPENDENT_REVIEW_R3 = R3_REVIEW_PASS_WITH_NONBLOCKING_FINDINGS
INDEPENDENT_REVIEW_R4 = R4_REVIEW_BLOCKED_BY_FINDINGS
R4_BLOCKER = ASSIGNMENT_ALIAS_FALSE_NEGATIVE
INDEPENDENT_REVIEW_R5 = R5_REVIEW_BLOCKED_BY_FINDINGS
R5_BLOCKER = OBJECT_LITERAL_INVOKE_ALIAS_FALSE_NEGATIVE
```

R4 showed that declaration-only alias discovery missed a later assignment. The
fifth commit added assignment propagation and verified 42 structural checks,
7/7 safe fixtures and 43/43 negative fixtures. R5 confirmed that correction but
showed that an object-literal property initializer remains a compiling
false-negative. The object-literal blocker is open.

## Current flow coverage

The status below describes only the current repository script and its exact
modeled files/fixtures. `IMPLEMENTED_AND_VERIFIED` does not imply repository-wide
or interprocedural completeness.

| Flow | Current status | Source-based reason |
| --- | --- | --- |
| Direct named import/call | IMPLEMENTED_AND_VERIFIED | Import symbols seed the invoke set; negative fixture exists |
| Renamed named import | IMPLEMENTED_AND_VERIFIED | Imported property name is compared with `invoke`; fixture exists |
| Namespace import/property call | IMPLEMENTED_AND_VERIFIED | Namespace symbols plus property/element checks; fixtures exist |
| Variable initializer alias | IMPLEMENTED_AND_VERIFIED | Identifier declarations propagate initializer symbols |
| Later assignment alias | IMPLEMENTED_AND_VERIFIED | R4 correction fixed-point visits direct assignments; fixtures exist |
| Chained/multi-step assignment | IMPLEMENTED_AND_VERIFIED | Fixed-point and binary flow recursion; fixtures exist |
| Object-literal property initializer | MISSING | R5 compiling false-negative; initializer is not propagated to property symbol |
| Array or tuple element | MISSING | No array-literal/element carrier model |
| Object destructuring | PARTIALLY_IMPLEMENTED | Direct namespace binding/assignment special cases only |
| Array destructuring | MISSING | No array binding/assignment model |
| Spread/rest | MISSING | No spread/rest propagation |
| Function parameter | MISSING | No argument-to-parameter flow model |
| Function return value | MISSING | No return-to-call-site flow model |
| Higher-order wrapper | MISSING | No interprocedural call/return graph |
| Closure capture | PARTIALLY_IMPLEMENTED | A previously resolved lexical symbol is found inside a closure; flow through closure construction/calls is not modeled |
| Conditional/ternary expression | PARTIALLY_IMPLEMENTED | Selected expression branches recurse without control-flow semantics |
| Class field initializer | MISSING | Only variable declarations and assignments seed target symbols |
| Getter/setter | MISSING | No property accessor or call semantics |
| Static property reassignment | PARTIALLY_IMPLEMENTED | Direct symbol-resolvable assignment targets propagate; object creation does not |
| Dynamic namespace property | PARTIALLY_IMPLEMENTED | Direct unknown namespace member is conservatively unsafe; arbitrary container properties are not modeled |
| Re-export/barrel module | MISSING | Programs use `noResolve` and validate no repository import graph |
| Indirect cross-file import graph | MISSING | Main frontend analysis is per named source file |

## Root-cause analysis

`resolveTauriInvokeAliases()` is a hand-built may-flow analysis over two finite
symbol sets. Its fixed point terminates, but completeness depends entirely on
which AST carriers were manually added. The R4 patch added assignment edges;
R5 exposed a missing object-literal property edge. Adding that edge would not
cover array elements, destructuring, spread, parameters, returns, closures,
higher-order calls, class properties, getters or cross-file exports.

Covering the requested flow inventory reliably would require:

- a call graph for invoked functions and methods;
- control-flow/SSA semantics for assignments and branches;
- property-sensitive container flow;
- call/return and parameter propagation;
- closure and higher-order function modeling;
- module resolution, re-exports and barrel graphs;
- conservative behavior for dynamic properties and JavaScript runtime features;
- a maintained adversarial conformance corpus.

That is a custom interprocedural TypeScript data-flow/taint analyzer. It is not
proportionate to one read-only storage status command and should not become a
Phase 1B.1 deliverable.

## Threat-model matrix

| Threat class | Example | Required control | Current result |
| --- | --- | --- | --- |
| A — accidental architectural violation | Second command literal, UI raw import, ordinary bypass/re-export | Command-scoped module/import/export graph, exact inventories, review | PARTIALLY_IMPLEMENTED |
| B — intentional obscured flow | Object/array alias, wrapper, closure, dynamic property | Established data-flow-capable analysis with explicit source/sink model | MISSING |
| C — attacker changes code and gate | Application bypass plus validator weakening | Protected branch/workflow, required independent review/checks, ownership policy, release integrity | OUTSIDE_REPOSITORY_SCRIPT |

Threat class A is the appropriate mandatory Phase 1B.1 scope. Threat class B
may justify optional higher assurance later. Threat class C cannot be closed by
code stored beside the code it validates.

## Architectural recommendations

### Primary control

Adopt a command-scoped module-boundary and import/re-export graph gate:

1. `src/lib/storageRuntimeClient.ts` is the only executable frontend owner of
   `get_storage_runtime_status`.
2. Make `storageRuntimeCommand` module-private so another existing raw Tauri
   importer cannot combine it with `invoke`.
3. Allow only the typed operation, controlled error and type-only DTO exports.
4. Forbid the adapter from exporting raw Tauri bindings, namespaces, dynamic
   import values or factories that return them.
5. Ensure `StorageRuntimeCard` reaches the command only through the typed
   operation.
6. Parse every frontend import/export/re-export edge and every executable
   occurrence of the storage command literal.
7. Preserve exact Rust status-command and registration checks.
8. Record existing non-storage Tauri import sites as a reviewed baseline and
   block new sites pending a broader adapter-migration plan.

### Secondary control

Retain only clearly documented local AST checks that remain simple and useful.
Their output must say what was checked and must not claim arbitrary data-flow
absence. The R5 object-literal behavior must not be reported as covered unless a
future selected control genuinely covers it.

### Optional higher assurance

If multiple Tauri commands later need deliberate-obfuscation resistance,
evaluate maintained tools against the same adversarial corpus:

- CodeQL: official JavaScript/TypeScript libraries distinguish AST nodes from
  data-flow nodes and provide local/global data flow and taint tracking; custom
  queries can run locally or in CI subject to repository/license conditions.
- Semgrep: syntactic and taint rules are available, but Community Edition is
  per-file; cross-file and advanced interprocedural claims must be matched to
  the exact selected engine.
- typescript-eslint: type-aware custom rules suit import/API ownership and can
  integrate with lint, but a custom rule is not automatically a full data-flow
  engine.

No tool should be installed before a separate plan records dependency,
licensing, CI, self-hosted execution, performance and maintenance evidence.

## Option comparison

| Option | Primary guarantee | Non-guarantee | Complexity | Maintenance | False-positive risk | False-negative risk | Recommendation |
| --- | --- | --- | --- | --- | --- | --- | --- |
| A — module/import graph | Storage authority remains at named choke points | Arbitrary in-module/cross-function value flow | S/M | Low | Low | Low for class A, expected for class B | SELECT |
| B — established analyzer | Broader modeled source-to-sink flow | Unmodeled dynamic behavior and mutable gate trust | M/H | Medium | Must be measured | Must be measured | OPTIONAL EVALUATION |
| C — limited AST | Exact documented local forms and fixtures | All unmodeled carriers | M and growing | High relative to value | Medium | Confirmed by R4/R5 | RETAIN ONLY IF SIMPLIFIED |
| D — custom full analyzer | Theoretical closed-world flow proof | Dynamic runtime and mutable gate trust | XL | Very high | High | High without language-engine rigor | REJECT |

## Guarantee matrix

| Flow or attacker behavior | A: module/import gate | B: established analyzer | C: limited current AST | D: custom complete analyzer |
| --- | --- | --- | --- | --- |
| Direct raw storage invoke | GUARANTEED | GUARANTEED | GUARANTEED | GUARANTEED |
| Renamed import | GUARANTEED | GUARANTEED | GUARANTEED | GUARANTEED |
| Namespace import | GUARANTEED | GUARANTEED | GUARANTEED | GUARANTEED |
| Variable alias | PARTIALLY_DETECTED | PARTIALLY_DETECTED | GUARANTEED | GUARANTEED |
| Assignment alias | PARTIALLY_DETECTED | PARTIALLY_DETECTED | GUARANTEED | GUARANTEED |
| Object-property alias | PARTIALLY_DETECTED | PARTIALLY_DETECTED | NOT_GUARANTEED | GUARANTEED |
| Array alias | PARTIALLY_DETECTED | PARTIALLY_DETECTED | NOT_GUARANTEED | GUARANTEED |
| Destructuring | PARTIALLY_DETECTED | PARTIALLY_DETECTED | PARTIALLY_DETECTED | GUARANTEED |
| Wrapper function | PARTIALLY_DETECTED | PARTIALLY_DETECTED | NOT_GUARANTEED | GUARANTEED |
| Return-value flow | PARTIALLY_DETECTED | PARTIALLY_DETECTED | NOT_GUARANTEED | GUARANTEED |
| Re-export | GUARANTEED | PARTIALLY_DETECTED | NOT_GUARANTEED | GUARANTEED |
| Dynamic property | NOT_GUARANTEED | PARTIALLY_DETECTED | PARTIALLY_DETECTED | PARTIALLY_DETECTED |
| Malicious validator modification | OUT_OF_SCOPE | OUT_OF_SCOPE | OUT_OF_SCOPE | OUT_OF_SCOPE |

## Selected PR #27 disposition

Human approval on 2026-07-24 selected Disposition 1 from accepted ADR 0006:

- replace the overclaimed validator portion with the storage command-scoped
  module/import/export graph gate;
- make the storage command constant private;
- reduce retained alias checks to explicit defense in depth;
- update completion evidence so it no longer claims complete raw-invoke flow
  proof;
- add boundary fixtures rather than another one-syntax alias patch;
- obtain a fresh independent exact-head review before a separate ready/merge
  decision.

This is an architecture correction package, not a sixth point correction. One
bounded implementation commit is separately authorized; ready and merge remain
forbidden pending independent exact-head R6 review and later authorization.

Accepting the current false-negative is not recommended because the current
completion evidence and validator output overstate the proven boundary.
Introducing CodeQL, Semgrep Pro or a new ESLint stack before merge is also not
recommended without a separate tooling decision.

## Bounded future implementation stages

### Stage 1 — boundary contract

- approve adapter ownership, prohibited locations, export allowlist and command
  literal ownership;
- decide the treatment of existing non-storage Tauri imports;
- define exact success/failure language for the gate.

### Stage 2 — gate implementation

- enumerate `src/` modules and static/dynamic imports;
- model exports/re-exports and executable command literals;
- enforce typed storage route and exact Rust inventory;
- simplify current alias logic and claims;
- add ordinary boundary-bypass and safe-change fixtures.

### Stage 3 — exact-head release review

- prove the correction changes no runtime/product/schema authority;
- run storage contract, build, docs and security checks;
- independently review the exact head;
- request separate ready/merge authorization only after a clean review.

### Stage 4 — optional analyzer evaluation

- compare CodeQL, Semgrep and type-aware ESLint with a fixed corpus;
- measure local/CI determinism, time, licensing, self-hosted support and upkeep;
- use a separate ADR before making a tool required.

## Human decisions recorded

1. ADR 0006 is Accepted.
2. PR #27 disposition 1 is selected.
3. `storageRuntimeCommand` becomes module-private.
4. The Phase 1B.1 gate is command-scoped; migration of the nine audited Tauri
   import sites is deferred to a separate phase.
5. Established static-analysis tooling remains an optional deferred evaluation.
6. One bounded architecture-correction commit is authorized. Individual
   object-literal, array, closure, parameter or return-flow patches are not.

## Documentation-only validation requirements

The local proposal must pass:

- changed-path allowlist: only this audit and accepted ADR 0006;
- `git diff --check`;
- Markdown relative-link/path validation;
- ADR numbering and reservation check;
- exact SHA and review-outcome consistency;
- terminology and false-implementation-claim review;
- secret/private-infrastructure scan;
- confirmation that PR #27 remains unmodified.

## Final audit status

```text
VALIDATOR_ARCHITECTURE_REVIEW = COMPLETE
REVIEW_TYPE = AUDIT / PLANNING / DOCUMENTATION_ONLY
PR_27 = OPEN / DRAFT / NOT_MERGED
PR_27_HEAD = fdbb9c88c2ef8c46f4a2a0abb4defc68c00c361c
R5_VERDICT = R5_REVIEW_BLOCKED_BY_FINDINGS
R5_BLOCKER = OBJECT_LITERAL_INVOKE_ALIAS_FALSE_NEGATIVE
SIXTH_POINT_CORRECTION = REJECTED
ARCHITECTURAL_CORRECTION = AUTHORIZED / IMPLEMENTATION_IN_PROGRESS
RECOMMENDED_PRIMARY_CONTROL = COMMAND_SCOPED_MODULE_BOUNDARY_AND_IMPORT_GRAPH_GATE
RECOMMENDED_SECONDARY_CONTROL = LIMITED_AST_CHECKS / DEFENSE_IN_DEPTH
ARBITRARY_TYPESCRIPT_DATA_FLOW_PROOF = NOT_CLAIMED
OPTIONAL_HIGHER_ASSURANCE = DEFERRED_TOOL_EVALUATION
CUSTOM_FULL_DATA_FLOW_ANALYZER = REJECTED
GLOBAL_FRONTEND_ADAPTER_MIGRATION = DEFERRED / SEPARATE_PHASE
ADR_0006_STATUS = ACCEPTED
HUMAN_APPROVAL_RECORDED = YES
R6_REVIEW = REQUIRED / NOT_PERFORMED
READY = NOT_PERFORMED
MERGE = NOT_PERFORMED
PHASE_1B_2 = NOT_AUTHORIZED
```

## References

- [Accepted ADR 0006](../adr/0006-frontend-tauri-invocation-boundary-and-validator-assurance-model.md)
- [Phase 1B.1 completion evidence at the reviewed PR head](https://github.com/DAARION-DAO/daarion-edge-client/blob/fdbb9c88c2ef8c46f4a2a0abb4defc68c00c361c/docs/planning/phases/phase-01b1-storage-runtime-vertical-slice-completion.md)
- [Security gates](../security/SECURITY_GATES.md)
- [Threat model](../security/THREAT_MODEL.md)
- [CodeQL JavaScript/TypeScript data-flow guide](https://codeql.github.com/docs/codeql-language-guides/analyzing-data-flow-in-javascript-and-typescript/)
- [CodeQL CLI](https://docs.github.com/en/code-security/concepts/code-scanning/codeql/codeql-cli)
- [Semgrep analysis glossary](https://semgrep.dev/docs/writing-rules/glossary)
- [typescript-eslint custom rules](https://typescript-eslint.io/developers/custom-rules/)
- [typescript-eslint typed linting](https://typescript-eslint.io/getting-started/typed-linting/)
