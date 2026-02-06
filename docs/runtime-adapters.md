# Runtime Adapters (OpenClaw + AutoGen)

This doc defines how multiple agent runtimes plug into Clawpany.

## Adapter Contract
Each runtime adapter should implement:
- **run(prompt)** → string summary/result
- **health()** → availability

## OpenClaw Adapter
- REST call to `/agent/run`
- Returns `message` string

## AutoGen Adapter
- Runs a multi‑agent chat flow
- Returns last assistant message

## Roadmap
- Add LangChain and CrewAI adapters
- Add runtime selection per task
