# Workers

Simple worker bots that poll the Orchestrator API and execute tasks.

## Quick start (Node)
```bash
cd workers/node-worker
npm install
ORCH_URL=http://localhost:3000 ORCH_API_KEY=dev_key node index.js
```

## LLM Orchestration
Use open‑source LLM CLIs to generate summaries:
- `LLM_PROVIDER=ollama` + `LLM_MODEL=llama3.1`
- `LLM_PROVIDER=llamacpp` + `LLM_BIN=./main` + `LLM_MODEL=./models/q4.gguf`

## Env
- `ORCH_URL` (default: http://localhost:3000)
- `ORCH_API_KEY` (default: dev_key)
- `WORKER_NAME` (default: worker-1)
- `POLL_INTERVAL_MS` (default: 5000)
- `LLM_PROVIDER` (default: echo)
- `LLM_MODEL` (optional)
- `LLM_BIN` (optional)
