# commits-001 Vertical Slice Structure

## Goal
Render repository commit history at `/{owner}/{repo}/commits/{branch-or-tag}/[[...path]]` using real typed Rust API and Next.js server data seams.

## Data/API seam
- Extend `GET /api/repos/{owner}/{repo}/commits` with typed filters: `ref`, `path`, `author`, `since`, `until`, `page`, `pageSize`.
- Return an envelope that includes resolved ref metadata, active filters, branch/tag ref options, author options, and commit rows.
- Rows are backed by `commits`, `repository_git_refs`, `repository_files`, and `users`; no static UI-only mock data.

## UI slice
- Server route reads URL search params and passes filters through `getRepositoryCommitHistory`.
- Commit history page renders Editorial controls for branch/tag, author, date range, clear filters, and grouped rows by commit date.
- Each row links subject/short SHA to commit detail and Browse to the tree at that commit.
- Empty and unavailable states remain accessible and deterministic.

## Tests
- Rust API contract covers filter options and author/date/path filtering.
- React unit test covers controls, date grouping, row links, and empty state.
- Required validation: banned scan, focused tests, `make check`, `make test`.
