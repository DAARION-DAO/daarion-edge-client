---
name: daarion-security-review
description: "Use before and after every DAARION implementation phase to threat-model changed boundaries, review the diff, classify findings, and enforce security blockers."
---

# DAARION Security Review

Perform a scoped pre-implementation threat model and a post-implementation diff review. Consider prompt injection, tool escalation, malicious pairing, compromised accounts/backends/models/agents, replay, forged capabilities, IPC spoofing, sidecar replacement, poisoned memory, wallet abuse, SSRF, command injection, path traversal, sandbox escape, local database theft, supply chain, auto-update, deep links, and secret logging as relevant.

For each finding record severity (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`, or `INFO`), affected repository/path/symbol, sanitized evidence, realistic impact, remediation, verification, and whether it blocks the phase. Clearly label hypotheses; do not report speculative vulnerabilities as confirmed.

Review trust boundaries, authorization, input/output validation, data egress, key handling, dependencies, logs, failure defaults, race/replay behavior, and test gaps. Scan the complete diff for credentials and private infrastructure evidence without printing discovered values.

Unresolved critical or high-severity findings block completion unless the authorized human explicitly accepts the specific risk in writing. Such acceptance results in at most `CONDITIONAL_PASS`, never silent `PASS`.
