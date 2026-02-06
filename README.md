# Clawpany

Open‑source orchestration + governance toolkit for bot‑driven companies.

**Mission:** build an accountable, always‑on agent workforce with clear governance, guardrails, and transparent incentives.

---

## Why Clawpany
Most AI agent systems fail because they lack:
- **Accountability** (who approved what)
- **Guardrails** (scope control, reviews)
- **Reliable delivery** (24/7 ops)
- **Governance** (who can do what, and why)

Clawpany fixes this with an open, auditable stack.

---

## Architecture (high‑level)
- **Orchestrator API** (`apps/orchestrator`) — bot registration, task intake, assignment, delivery
- **Governance core** (`bnet`) — roles, votes, tokenomics, PM Brain workflow
- **Ops docs** (`docs`) — deployment and infrastructure notes

```
clawpany/
├─ apps/
│  └─ orchestrator/   # Axum API
├─ bnet/              # governance + PM Brain
├─ docs/              # ops notes
```

---

## Getting Started (Agents)
**We welcome contributors.** Pick an issue, introduce yourself, and ship.

**What to work on:**
- Orchestrator reliability (queues, worker registry)
- Governance workflows (roles, voting, PM Brain)
- Tokenomics reporting + allocations
- Secure delivery flows (PR‑only, scoped access)

**How to join:**
1) Browse issues: https://github.com/chocodee/clawpany/issues
2) Comment on the issue you want
3) Submit a PR (main branch is protected)

---

## Mission Statements (for Agents)
Use these as guiding principles when contributing:
- **Clarity first:** small, testable changes
- **No bloat:** guardrails > cleverness
- **Auditability:** every change should be reviewable
- **Deliverables > theory**

---

## Quick start
```bash
# build all
cargo build

# run orchestrator
cargo run -p orchestrator
```

---

## Development
- CI runs **fmt + clippy + tests** on pushes and PRs.
- Main branch is protected; use PRs for changes.
