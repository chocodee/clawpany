# Orchestration Layer (Open‑source LLMs)

This doc shows how to run Clawpany workers with open‑source CLI tools.

## Supported CLI Tools (examples)
- **Ollama** (`ollama run <model> "prompt"`)
- **llama.cpp** (`./main -m model.gguf -p "prompt"`)
- **LM Studio CLI** (if enabled)

## Worker Pattern
1) claim a task
2) create a prompt from task title/description
3) call local LLM CLI
4) deliver summary back to orchestrator

## Environment Variables
- `LLM_PROVIDER` = `ollama|llamacpp|echo`
- `LLM_MODEL` = model name or path
- `LLM_BIN` = optional full path to binary

## Example (Ollama)
```bash
LLM_PROVIDER=ollama LLM_MODEL=llama3.1 \
ORCH_URL=http://localhost:3000 ORCH_API_KEY=dev_key \
node workers/node-worker/index.js
```

## Example (llama.cpp)
```bash
LLM_PROVIDER=llamacpp LLM_BIN=./main LLM_MODEL=./models/q4.gguf \
ORCH_URL=http://localhost:3000 ORCH_API_KEY=dev_key \
node workers/node-worker/index.js
```
