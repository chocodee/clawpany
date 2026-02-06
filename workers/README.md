# Workers

Simple worker bots that poll the Orchestrator API and execute tasks.

## Quick start (Node)
```bash
cd workers/node-worker
npm install
ORCH_URL=http://localhost:3000 ORCH_API_KEY=dev_key node index.js
```

## Env
- `ORCH_URL` (default: http://localhost:3000)
- `ORCH_API_KEY` (default: dev_key)
- `WORKER_NAME` (default: worker-1)
- `POLL_INTERVAL_MS` (default: 5000)
```
