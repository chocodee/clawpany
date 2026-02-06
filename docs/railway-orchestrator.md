# Railway Deployment (Orchestrator)

## 1) Create project
- New project → Deploy from GitHub
- Select `chocodee/clawpany`

## 2) Set env vars
- `ORCH_API_KEY` (required)

## 3) Deploy
Railway will build from `Dockerfile` and start `/app/orchestrator`.

## 4) Test
```bash
curl -X POST $URL/clients/create \
  -H "Authorization: Bearer <key>" \
  -H "Content-Type: application/json" \
  -d '{"name":"Acme","contact":"ops@acme.com"}'
```
