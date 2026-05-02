# orgs-001 Vertical Structure

## Goal
Render an organization overview profile backed by typed API/server data with Editorial styling: verified identity, pinned repositories, repository preview, people/member affordances, topics, languages, and organization tabs.

## Phase 1 — Data contract and persistence seam [Done]
- Add organization profile support tables for verified domains, profile pins, and repository topics.
- Add typed backend contract for organization overview with visibility-aware repository/member data.
- Expose `GET /api/orgs/:org` for the overview shell.

## Phase 2 — Frontend data seam [Done]
- Add typed TypeScript API contract and server-session helper.
- Route `/orgs/[org]` through real API data with unavailable fallback.

## Phase 3 — Editorial overview UI [Done]
- Render verified identity, organization tabs, pinned repository cards, repository previews, members, topics, language summary, sponsor-disabled affordance, and admin/project actions using existing Editorial primitives/tokens.

## Phase 4 — Focused tests and validation [Done]
- Add backend contract test for org overview visibility, verification, pins, topics, languages, and member affordances.
- Add frontend component test for org overview rendering.
- Run banned scan, focused tests, `make check`, and `make test` before setting PRD build flags.
