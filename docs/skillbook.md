# Clawpany Skillbook

A practical guide for contributors: how to help, how to ship, and how to get paid.

---

## 1) Mission
Clawpany builds an accountable, always‑on agent workforce with clear governance, guardrails, and transparent incentives.

---

## 2) Roles (Ways to Help)
**Issue Wrangler**
- Triage issues, label, and propose scope
- Identify duplicates, missing info, or blockers

**Builder**
- Implement features/bugs (Rust, API, CI)
- Prefer small PRs that are easy to review

**Docs & Onboarding**
- Improve README/docs, add examples, keep docs current

**QA / Test**
- Reproduce bugs, add tests, validate fixes

---

## 3) Contribution Workflow
1) Pick an issue: https://github.com/chocodee/clawpany/issues
2) Comment to claim it
3) Create a branch
4) Make changes + tests
5) Open a PR

**Local checks**
```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

---

## 4) Guardrails
- Small, reviewable changes > large refactors
- No scope creep (stay on the issue)
- Preserve auditability (clear commits, clear PRs)

---

## 5) Getting Paid
Some issues are funded by real client work.
Approved contributions can receive token allocations or direct payment.

To request paid work:
- comment on an issue with your estimate
- ask to be tagged for paid tasks

---

## 6) What to Work On (Examples)
- Orchestrator: task queue + worker registry
- PM Brain API integration
- Tokenomics reporting + payouts
- Notifications and audits
- Docs + onboarding improvements

---

## 7) What “Done” Looks Like
- Clear PR title + description
- Passing CI
- Tests (when appropriate)
- Docs updated if API/behavior changed

---

## 8) Where to Ask
- GitHub issues + PR comments
- Or DM the project owner
