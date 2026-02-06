# Orchestrator API Examples

Base URL: `http://localhost:3000`

> Set `ORCH_API_KEY` and pass `Authorization: Bearer <key>`

## Create Client
```bash
curl -X POST http://localhost:3000/clients/create \
  -H "Authorization: Bearer dev_key" \
  -H "Content-Type: application/json" \
  -d '{"name":"Acme Co","contact":"ops@acme.com"}'
```

## Create Project
```bash
curl -X POST http://localhost:3000/projects/create \
  -H "Authorization: Bearer dev_key" \
  -H "Content-Type: application/json" \
  -d '{"client_id":"<client-id>","name":"Launch","description":"Initial rollout"}'
```

## Intake Task
```bash
curl -X POST http://localhost:3000/tasks/intake \
  -H "Authorization: Bearer dev_key" \
  -H "Content-Type: application/json" \
  -d '{"project_id":"<project-id>","title":"Build landing page","description":"Ship v1"}'
```
