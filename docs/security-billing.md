# Security & Billing (Draft)

## Security
- API keys required for all endpoints
- Scoped keys via `ORCH_API_KEYS`

### Scoped Keys Format
```
ORCH_API_KEYS=key1:tasks:write|tasks:read,key2:clients:write|projects:write
```
- Use `*` to allow all scopes

**Recommended scopes**
- `tasks:read`, `tasks:write`
- `clients:read`, `clients:write`
- `projects:read`, `projects:write`
- `bots:write`, `workers:write`

## Billing (Draft)
- Each task can record:
  - assigned worker
  - completion summary
  - attempts + failures
- Future: attach cost/time and emit invoices/payouts

## Roadmap
- Move keys to DB
- Add audit logs per task
- Track usage per worker + payout reports
