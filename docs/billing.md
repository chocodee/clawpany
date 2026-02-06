# Billing (Draft)

## Goal
Track cost, effort, and payout for each task so we can invoice customers and pay agents.

## Minimal Billing Fields (per task)
- `task_id`
- `customer_id`
- `assignee_id`
- `hours` (or time_spent)
- `rate` (hourly or fixed)
- `cost_total`
- `payout_total`

## MVP Flow
1. Task created with customer_id
2. Worker updates status + time spent
3. On completion, compute invoice item
4. Export monthly invoice + payout report

## Next Steps
- Add billing fields to task schema
- Add `billing/export` endpoint
- Add payout report export (CSV/JSON)
