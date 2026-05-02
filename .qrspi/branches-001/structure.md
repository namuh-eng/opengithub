# branches-001 Structure

## Goal
Render a complete repository Branches page at `/{owner}/{repo}/branches` using live repository/ref data seams, Editorial primitives, client-side tab/search interactions, branch metadata, protection/status hints, and safe row actions.

## Vertical slice
1. Server route loads the authenticated session, repository overview, and repository refs through existing typed server/API seams.
2. Branch page component receives `RepositoryOverview` plus branch refs, derives deterministic branch metadata from real refs/repository state, and documents upgrade paths for future branch activity/check/protection tables.
3. Client UI provides Overview / Active / Stale / All tabs, case-insensitive branch search, dense branch rows, empty states, and non-destructive actions only.
4. Focused component tests cover tab classification, search, metadata, and action/link affordances.

## Data seam
- Real source today: `GET /api/repos/:owner/:repo/refs` via `getRepositoryRefs` / `RepositoryRefSummary`.
- Derived deterministic fields today: default branch marker, unprotected/default protection hint, unknown/no-check status hint, ahead/behind placeholders, PR discovery link.
- Upgrade path: replace derived fields with backend `branch_activity_snapshots`, `commit_status_summaries`, `pull_requests`, `branch_protection_rules`, and `repository_rulesets` without changing the component row contract.

## Validation
- Focused unit test: `npm --prefix web exec vitest run tests/repository-branches.test.tsx`.
- Mandatory banned scan over `web/src` excluding design tokens.
- Full validation before PRD flag update: `make check` and `make test`.
