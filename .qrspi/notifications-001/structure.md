# notifications-001 structure

## Goal
Render the signed-in notifications inbox with working folders, default filters, repository buckets, query search, sorting, grouping, and row states using OpenGitHub Editorial UI tokens.

## Vertical phases

### Phase 1 — Screen-ready API contract
- Add authenticated `GET /api/notifications` backed by `notifications` and `repositories` tables.
- Support `q`, `folder`, `tab`, `sort`, `group`, `repo`, `page`, and `pageSize` query inputs.
- Parse supported notification qualifiers: `is:unread`, `is:read`, `is:saved`, `is:done`, `reason:*`, `repo:owner/name`, and `type:*`.
- Return folders, default filters, repository buckets, groups, rows, active query, sort/group choices, and empty-state text.
- Add `PATCH /api/notifications/:id/read` for last-read updates when opening rows.
- Focused Rust contract test verifies auth, query filters, grouping, sorting, repository buckets, and mark-read behavior.

### Phase 2 — Next data bridge and inbox route
- Add typed notification view models and `getNotificationsFromCookie` to `web/src/lib/api.ts`.
- Replace `/notifications` placeholder with a signed-in app-shell inbox using `AppShellFrame`.
- Add `/notifications/[notificationId]/open` route that marks read then redirects to the notification source target.

### Phase 3 — Editorial UI and controls
- Left rail: folders, default filters, repository buckets, manage notifications link.
- Main panel: All/Unread tabs, query-builder search, Sort and Group controls, cleanup notice, group headers, select-all checkbox, compact rows with unread/saved/done/subscription states.
- Controls are real links/forms that preserve/update URL query state; no inert controls.

### Phase 4 — Verification and project state
- Add focused Vitest coverage for the rendered inbox page and query href behavior.
- Run banned-value scan, focused tests, `make check`, and `make test`.
- Update `build-progress.txt`, `qa-hints.json`, and `prd.json` only after passing mandatory build verification.
