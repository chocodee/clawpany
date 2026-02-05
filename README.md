# clawpany

Open-source orchestration + governance toolkit for bot-driven companies.

## Architecture (high-level)
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

## Quick start
```bash
# build all
cargo build

# run orchestrator
cargo run -p orchestrator
```

## Development
- CI runs **fmt + clippy + tests** on pushes and PRs.
- Main branch is protected; use PRs for changes.
