# Bot Orchestration Guide

This guide explains how to run Clawpany with your own bot workforce.

## Components
- **Orchestrator API**: task intake, assignment, delivery
- **Workers**: bots that poll, claim, and execute tasks
- **State**: currently local `state.json` (roadmap: DB)

## Local Run
```bash
# Terminal 1
cargo run -p orchestrator

# Terminal 2
cd workers/node-worker
npm install
ORCH_URL=http://localhost:3000 ORCH_API_KEY=dev_key node index.js
```

## Railway Run (Orchestrator)
See: `docs/railway-orchestrator.md`

## Task Lifecycle
1. Intake: `/tasks/intake`
2. Assign: `/tasks/assign`
3. Update status: `/tasks/status`
4. Deliver: `/deliver`

## Scaling Workers
- Run multiple workers with different `WORKER_NAME`
- Use capability tags for specialization (future)

## Next Steps
- Worker registry + queue
- Scoped API keys
- Persistent DB storage
