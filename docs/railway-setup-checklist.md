# Railway Project Setup Checklist (Client Project)

Use this checklist per client project. Keep access least-privilege.

## 1) Create project
- Create a new Railway project named for the client.
- Add a Postgres service.
- Add a Redis service (if using queues).

## 2) Connect repo
- Connect the client’s private GitHub repo to the Railway project.
- Set the deploy branch (e.g., main).

## 3) Configure project env vars
- Add only client‑specific secrets.
- Use the template in `bnet/config/railway.env.example`.
- Do not reuse secrets across clients.

## 4) Set access controls
- Invite client members with **Viewer** or **Developer** role.
- Avoid **Admin** unless absolutely necessary.

## 5) Restrict bot permissions
- Use repo‑scoped GitHub deploy keys or GitHub App tokens per client.
- Ensure bots have no access to other client repos.

## 6) Enable logging + monitoring
- Keep Railway logs enabled.
- Export logs or set alerts if needed.

## 7) Configure GitHub Actions secrets
- RAILWAY_TOKEN
- RAILWAY_PROJECT_ID
- RAILWAY_ENVIRONMENT_ID

## 8) Validate deployment
- Run a test deploy from GitHub Actions.
- Confirm health check endpoints (if any).

## 9) Handoff
- Provide client with:
  - Project URL
  - Access role
  - Contact for support
