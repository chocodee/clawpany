# Governance & Tokenomics

This doc explains how Clawpany models governance, roles, and token incentives.

---

## Governance Model
Clawpany uses **roles + votes** to grant permissions and protect critical actions.

**Key ideas:**
- Roles define authority (CEO, Board, President, etc.)
- Votes are required for sensitive role changes
- Guardrails limit risk (LOC caps, review flows)

---

## Roles
Roles are defined in `bnet` and assigned to holders. Examples:
- CEO
- President / Co‑President
- Board Seat
- Operators

---

## Votes
Votes are created for sensitive actions like role assignment.
- Each vote has a target role + holder
- Voters are holders with authority
- Resolution uses a threshold (e.g., 2/3)

---

## Tokenomics
Tokenomics track supply, allocations, and work payouts.

**Core fields:**
- Total supply cap
- Minted supply
- Allocation templates (marketing, engineering, ops)

---

## Early Joiner Rewards
The onboarding flow supports early joiner rewards and limits.

---

## CLI Examples
```bash
# Onboard new holder
cargo run -p bnet -- onboard --id alice --name "Alice" --cash 0 --early-limit 50 --early-reward 100

# Set tokenomics
cargo run -p bnet -- set-tokenomics --cap 1000000 --minted 100000 --allocations marketing:lead:5,writer:3,designer:2

# List holders
cargo run -p bnet -- list-holders
```

---

## Roadmap
- Add on-chain payout export
- Add audit logs for votes
- Role-based permissions in API
