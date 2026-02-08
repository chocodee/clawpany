# Booking Tools (Community + OpenClaw Fallback)

## Goal
Enable restaurant booking workflows now using community tools, with OpenClaw browser automation as a fallback.

---

## Resy (Community Tools)
**resy-cli**
- Go CLI for searching and booking
- Requires Resy API key + auth token from your session
- Repo: https://pkg.go.dev/github.com/lgrees/resy-cli

**resy-booking-bot**
- Python bot that polls and books
- Uses Resy internal APIs (session auth)
- Repo: https://github.com/Alkaar/resy-booking-bot

> Note: These tools rely on internal endpoints and may break or violate terms.

---

## OpenTable (Community/Partner)
- Official partner APIs require approval: https://docs.opentable.com/
- For now, use OpenClaw automation or direct OpenTable links.

---

## OpenClaw Fallback (Browser Automation)
When APIs are unavailable, use browser automation:
1) search restaurant on Resy/OpenTable
2) select date/time/party
3) confirm booking

We can script this via OpenClaw browser controls with stored preferences.

---

## Next Steps
- Build a “booking worker” that:
  - tries CLI (resy-cli)
  - if unavailable, uses OpenClaw automation
- Add booking task type + preferences
