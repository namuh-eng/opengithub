export type ApiDocMethod = "GET" | "POST" | "PATCH" | "DELETE";

export type ApiEndpointDoc = {
  id: string;
  method: ApiDocMethod;
  path: string;
  title: string;
  description: string;
  auth: string;
  request?: string;
  response: string;
  notes: string[];
};

export const apiEndpointDocs: ApiEndpointDoc[] = [
  {
    id: "user-current",
    method: "GET",
    path: "/api/user",
    title: "Authenticated user",
    description:
      "Returns the signed-in account that owns the current Rust session.",
    auth: "Signed opengithub session cookie",
    response: `{
  "id": "user_01",
  "login": "mona",
  "name": "Mona Lisa",
  "email": "mona@example.com",
  "avatarUrl": "https://avatars.example/mona.png",
  "htmlUrl": "/mona",
  "createdAt": "2026-04-30T00:00:00Z",
  "updatedAt": "2026-04-30T00:00:00Z"
}`,
    notes: ["Anonymous callers receive a standard 401 error envelope."],
  },
  {
    id: "repos-list",
    method: "GET",
    path: "/api/repos?page=1&pageSize=30",
    title: "List repositories",
    description:
      "Lists repositories visible to the authenticated user with bounded pagination.",
    auth: "Signed opengithub session cookie",
    response: `{
  "items": [
    {
      "id": "repo_01",
      "owner_login": "mona",
      "name": "octo-app",
      "visibility": "public",
      "default_branch": "main",
      "htmlUrl": "/mona/octo-app"
    }
  ],
  "total": 1,
  "page": 1,
  "pageSize": 30
}`,
    notes: ["pageSize is clamped by the API contract."],
  },
  {
    id: "repos-create",
    method: "POST",
    path: "/api/repos",
    title: "Create repository",
    description:
      "Creates a repository owned by the signed-in user and returns its details.",
    auth: "Signed opengithub session cookie",
    request: `{
  "name": "octo-app",
  "description": "Example repository",
  "visibility": "public",
  "default_branch": "main"
}`,
    response: `{
  "id": "repo_01",
  "owner_login": "mona",
  "name": "octo-app",
  "visibility": "public",
  "viewerPermission": "owner"
}`,
    notes: ["Duplicate repository names return 409 conflict."],
  },
  {
    id: "repo-detail",
    method: "GET",
    path: "/api/repos/{owner}/{repo}",
    title: "Repository detail",
    description:
      "Reads repository metadata, clone URLs, viewer permission, and code-tab summary data.",
    auth: "Signed opengithub session cookie",
    response: `{
  "id": "repo_01",
  "owner_login": "mona",
  "name": "octo-app",
  "cloneUrls": {
    "https": "https://opengithub.namuh.co/mona/octo-app.git",
    "zip": "/mona/octo-app/archive/refs/heads/main.zip"
  },
  "viewerPermission": "owner"
}`,
    notes: ["Private repositories require explicit repository permission."],
  },
  {
    id: "repo-settings-read",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/settings",
    title: "Read repository settings",
    description:
      "Reads the General repository settings contract used by the Editorial settings page, including feature flags, merge methods, branch choices, danger-zone support, and recent audit events.",
    auth: "Signed opengithub session cookie with repository admin or owner access",
    response: `{
  "ownerLogin": "mona",
  "name": "octo-app",
  "visibility": "public",
  "defaultBranch": "main",
  "features": {
    "issuesEnabled": true,
    "projectsEnabled": true,
    "wikiEnabled": true
  },
  "merge": {
    "allowSquash": true,
    "allowMergeCommit": true,
    "allowRebase": true,
    "defaultMethod": "squash"
  },
  "danger": {
    "isArchived": false,
    "deleteSupported": false,
    "transferSupported": false
  },
  "auditEvents": []
}`,
    notes: [
      "Anonymous callers receive 401; non-admin repository viewers receive 403 without settings data.",
      "Private and internal repositories use the same admin-only contract and never leak settings to outsiders.",
    ],
  },
  {
    id: "repo-settings-update",
    method: "PATCH",
    path: "/api/repos/{owner}/{repo}/settings",
    title: "Update repository settings",
    description:
      "Persists partial General settings changes and returns fresh server state only after the Rust API validates and records the write.",
    auth: "Signed opengithub session cookie with repository admin or owner access",
    request: `{
  "description": "Calm collaboration workspace",
  "visibility": "private",
  "defaultBranch": "release",
  "features": { "issuesEnabled": false },
  "merge": {
    "allowSquash": true,
    "allowMergeCommit": false,
    "allowRebase": true,
    "defaultMethod": "squash"
  },
  "isArchived": false
}`,
    response: `{
  "name": "octo-app",
  "description": "Calm collaboration workspace",
  "visibility": "private",
  "defaultBranch": "release",
  "auditEvents": [
    {
      "eventType": "repository.settings.update",
      "changedFields": ["description", "visibility", "default_branch"]
    }
  ]
}`,
    notes: [
      "Every successful write inserts a repository.settings.update audit event.",
      "At least one merge method must remain enabled and the default merge method must be one of the enabled methods.",
      "Missing default branches and owner/name conflicts return 409 conflict; validation errors return 422.",
      "Archived repositories reject every settings mutation except unarchive.",
      "Delete and transfer are intentionally unsupported until dedicated backend operations own those destructive flows.",
    ],
  },
  {
    id: "repo-access-read",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/settings/access",
    title: "Read repository access settings",
    description:
      "Reads the admin-only repository Access settings contract, including direct collaborators, team-derived access, inherited owner or organization rows, pending invitations, invite targets, role definitions, and recent audit events.",
    auth: "Signed opengithub session cookie with repository admin or owner access",
    response: `{
  "ownerLogin": "mona",
  "name": "octo-app",
  "visibility": "private",
  "viewerPermission": "admin",
  "roles": [{ "role": "write", "label": "Write", "rank": 30 }],
  "people": [
    {
      "login": "hubot",
      "role": "write",
      "source": "direct",
      "sourceText": "Direct collaborator access",
      "canEdit": true,
      "canRemove": true
    }
  ],
  "teams": [],
  "invitations": [],
  "inviteTargets": { "users": [], "teams": [] },
  "auditEvents": []
}`,
    notes: [
      "Anonymous callers receive 401; non-admin viewers receive 403 without collaborator, team, or invitation data.",
      "Owner, organization-inherited, and team-derived rows are returned read-only with source text explaining where to manage them.",
      "Public, private, and internal repositories use the same admin-only settings contract to avoid private access leakage.",
    ],
  },
  {
    id: "repo-access-invite",
    method: "POST",
    path: "/api/repos/{owner}/{repo}/settings/access",
    title: "Invite repository collaborator",
    description:
      "Creates a pending repository invitation for a user login or email address and returns fresh access settings after validation, audit logging, and the SES email handoff attempt.",
    auth: "Signed opengithub session cookie with repository admin or owner access",
    request: `{
  "emailOrLogin": "hubot@example.com",
  "role": "triage"
}`,
    response: `{
  "invitations": [
    {
      "invitedEmail": "hubot@example.com",
      "role": "triage",
      "status": "pending",
      "emailDeliveryStatus": "degraded",
      "canCancel": true
    }
  ]
}`,
    notes: [
      "Valid roles are read, triage, write, maintain, and admin; owner cannot be granted through this endpoint.",
      "Duplicate direct collaborators and duplicate pending invitations return 409 conflict.",
      "Local or missing SES credentials persist the invitation with emailDeliveryStatus=degraded instead of faking delivery.",
      "Every successful invite inserts a repository.access.invite audit event.",
    ],
  },
  {
    id: "repo-access-team-grant",
    method: "POST",
    path: "/api/repos/{owner}/{repo}/settings/access/teams",
    title: "Grant repository team access",
    description:
      "Adds an organization team grant for a repository, mirrors the role to current team members through team-derived repository permissions, and returns fresh access settings.",
    auth: "Signed opengithub session cookie with repository admin or owner access",
    request: `{
  "teamSlug": "platform",
  "role": "write"
}`,
    response: `{
  "teams": [
    {
      "slug": "platform",
      "role": "write",
      "source": "team",
      "canEdit": true,
      "canRemove": true
    }
  ]
}`,
    notes: [
      "Team grants are available only for organization-owned repositories; user-owned repositories return validation_failed.",
      "Duplicate team grants return 409 conflict; use the team role update endpoint for existing grants.",
      "Inherited organization base permission rows are read-only from this endpoint.",
    ],
  },
  {
    id: "repo-access-update-remove",
    method: "PATCH",
    path: "/api/repos/{owner}/{repo}/settings/access/collaborators/{user_id}",
    title: "Update collaborator role",
    description:
      "Changes a direct collaborator role and returns confirmed server state after guardrails verify the row is directly managed on the repository.",
    auth: "Signed opengithub session cookie with repository admin or owner access",
    request: `{
  "role": "maintain"
}`,
    response: `{
  "people": [{ "userId": "user_01", "role": "maintain", "source": "direct" }]
}`,
    notes: [
      "Owner, inherited organization, and team-derived rows cannot be demoted through direct collaborator updates.",
      "Demoting the final owner/admin access path returns 409 conflict.",
      "DELETE on the same path removes direct collaborator access and uses the same last-admin guardrail.",
    ],
  },
  {
    id: "repo-access-team-update-remove",
    method: "PATCH",
    path: "/api/repos/{owner}/{repo}/settings/access/teams/{team_id}",
    title: "Update team access role",
    description:
      "Changes or removes a direct team grant and refreshes team-derived member access after the Rust API accepts the mutation.",
    auth: "Signed opengithub session cookie with repository admin or owner access",
    request: `{
  "role": "maintain"
}`,
    response: `{
  "teams": [{ "teamId": "team_01", "role": "maintain", "source": "team" }]
}`,
    notes: [
      "PATCH changes a direct team grant; DELETE removes the direct team grant.",
      "Inherited team rows are read-only and return 403 if targeted for mutation.",
      "Removing or demoting the final owner/admin access path returns 409 conflict.",
    ],
  },
  {
    id: "repo-access-invitation-cancel",
    method: "DELETE",
    path: "/api/repos/{owner}/{repo}/settings/access/invitations/{invitation_id}",
    title: "Cancel repository invitation",
    description:
      "Cancels a pending repository invitation and returns fresh access settings without exposing invite token hashes.",
    auth: "Signed opengithub session cookie with repository admin or owner access",
    response: `{
  "invitations": [],
  "auditEvents": [{ "eventType": "repository.access.invite_cancel" }]
}`,
    notes: [
      "Any current repository admin may cancel a pending invitation; already accepted, canceled, or expired invitations return 404.",
      "Responses never include invitation token hashes or email provider secrets.",
    ],
  },
  {
    id: "org-repositories",
    method: "GET",
    path: "/api/orgs/{org}/repositories?q=router&type=public&language=Rust&page=1&pageSize=30",
    title: "List organization repositories",
    description:
      "Lists repositories visible in an organization with org-scoped filters, language/type facets, density state, and bounded pagination.",
    auth: "Public organizations expose public repositories; private/internal repositories require organization membership or direct repository permission",
    response: `{
  "items": [
    {
      "name": "octo-app",
      "fullName": "namuh/octo-app",
      "visibility": "public",
      "href": "/namuh/octo-app",
      "primaryLanguage": { "language": "Rust", "byteCount": 12000 }
    }
  ],
  "total": 1,
  "page": 1,
  "pageSize": 30,
  "availableTypes": [{ "value": "public", "label": "Public", "count": 1 }]
}`,
    notes: [
      "Supported type filters are all, contributed, admin, public, sources, forks, archived, and templates.",
      "Private organizations return not_found to outsiders without leaking membership or repository counts.",
    ],
  },
  {
    id: "org-people",
    method: "GET",
    path: "/api/orgs/{org}/people?q=member&page=1&pageSize=30",
    title: "List organization people",
    description:
      "Lists visible organization members with public-member privacy rules, role visibility for members, search, and pagination.",
    auth: "Public members are readable; private organizations and private role metadata require organization membership",
    response: `{
  "items": [
    {
      "login": "org-member",
      "name": "Organization Member",
      "href": "/org-member",
      "role": null,
      "joinedAt": "2026-04-30T00:00:00Z"
    }
  ],
  "total": 1,
  "page": 1,
  "pageSize": 30
}`,
    notes: [
      "Signed-out and outside viewers see public members only.",
      "Owner, admin, and member role chips are returned only when the viewer can see internal membership.",
    ],
  },
  {
    id: "issues-create",
    method: "POST",
    path: "/api/repos/{owner}/{repo}/issues",
    title: "Create issue",
    description:
      "Creates an issue in a repository using the actor from the session.",
    auth: "Signed opengithub session cookie",
    request: `{
  "title": "Bug report",
  "body": "Steps to reproduce..."
}`,
    response: `{
  "id": "issue_01",
  "number": 1,
  "title": "Bug report",
  "state": "open",
  "authorLogin": "mona"
}`,
    notes: ["Caller-supplied user identifiers are ignored."],
  },
  {
    id: "pulls-list",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/pulls?q=is:open&page=1&pageSize=30",
    title: "List pull requests",
    description:
      "Lists pull requests visible to the caller with repository-scoped filters, counts, metadata options, and pagination.",
    auth: "Public repositories are readable; private repositories require read permission",
    response: `{
  "items": [
    {
      "number": 42,
      "title": "Improve docs",
      "state": "open",
      "headRef": "feature/docs",
      "baseRef": "main",
      "href": "/mona/octo-app/pull/42"
    }
  ],
  "total": 1,
  "page": 1,
  "pageSize": 30,
  "counts": { "open": 1, "closed": 0, "merged": 0 }
}`,
    notes: [
      "Supported filters include state, author, labels, milestone, assignee, review, checks, sort, and free-text q.",
    ],
  },
  {
    id: "pulls-create",
    method: "POST",
    path: "/api/repos/{owner}/{repo}/pulls",
    title: "Create pull request",
    description:
      "Creates a pull request linked to the shared issue number sequence.",
    auth: "Signed opengithub session cookie",
    request: `{
  "title": "Improve docs",
  "head": "feature/docs",
  "base": "main",
  "body": "Adds API examples."
}`,
    response: `{
  "id": "pull_01",
  "number": 2,
  "title": "Improve docs",
  "state": "open",
  "head": "feature/docs",
  "base": "main"
}`,
    notes: ["Repository write permission is required for mutations."],
  },
  {
    id: "pulls-files",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/pulls/{number}/files?view=unified&whitespace=show",
    title: "Read pull request files",
    description:
      "Returns the diff review contract used by the Files changed UI, including file tree, hunks, rendered comments, viewed state, and pending review summary.",
    auth: "Public repositories are readable; private repositories require read permission",
    response: `{
  "pullRequest": { "number": 42, "title": "Improve docs" },
  "settings": { "view": "unified", "whitespace": "show", "page": 1, "pageSize": 50 },
  "totalFiles": 2,
  "files": [
    {
      "path": "docs/api.md",
      "additions": 12,
      "deletions": 2,
      "hunks": [{ "header": "@@ -1,3 +1,4 @@", "lines": [] }]
    }
  ]
}`,
    notes: [
      "pageSize is bounded to 100 files for raw clients and 50 by default in the UI.",
      "Pending review comments are only returned to their author.",
    ],
  },
  {
    id: "pulls-submit-review",
    method: "POST",
    path: "/api/repos/{owner}/{repo}/pulls/{number}/reviews",
    title: "Submit pull request review",
    description:
      "Publishes the caller's pending inline review comments with a summary review decision.",
    auth: "Signed opengithub session cookie with read access",
    request: `{
  "body": "Looks good after the docs update.",
  "state": "approved"
}`,
    response: `{
  "id": "review_01",
  "state": "approved",
  "publishedCommentCount": 2,
  "pendingReview": { "commentCount": 0 }
}`,
    notes: [
      "Authors cannot approve their own pull requests.",
      "Valid states are commented, approved, and changes_requested.",
    ],
  },
  {
    id: "pulls-merge",
    method: "POST",
    path: "/api/repos/{owner}/{repo}/pulls/{number}/merge",
    title: "Merge pull request",
    description:
      "Atomically merges a ready pull request using an enabled repository merge method and returns the refreshed detail contract.",
    auth: "Signed opengithub session cookie with write access",
    request: `{
  "method": "squash",
  "commitTitle": "Improve docs (#42)",
  "commitBody": "Generated from the merge confirmation UI.",
  "deleteBranch": true
}`,
    response: `{
  "number": 42,
  "state": "merged",
  "mergeability": {
    "state": "merged",
    "canMerge": false
  }
}`,
    notes: [
      "Blocked merges return HTTP 409 with code merge_blocked and details.blockers.",
      "Enabled methods come from repository merge settings and branch protection policy.",
    ],
  },
  {
    id: "pulls-raw-diff",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/pulls/{number}.diff",
    title: "Download raw pull request diff",
    description:
      "Returns a bounded text/plain unified diff for developer clients and browser .diff links.",
    auth: "Public repositories are readable; private repositories require read permission",
    response: `diff --opengithub a/main b/feature/docs
# Pull request #42: Improve docs

diff --git a/docs/api.md b/docs/api.md
--- a/docs/api.md
+++ b/docs/api.md
@@ -1,3 +1,4 @@
 context
+new line`,
    notes: [
      "The response content type is text/plain; charset=utf-8.",
      "The output is capped to the first 100 files to keep raw clients bounded.",
    ],
  },
  {
    id: "pulls-raw-patch",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/pulls/{number}.patch",
    title: "Download raw pull request patch",
    description:
      "Returns a text/plain patch stream with commit metadata followed by the same bounded unified diff.",
    auth: "Public repositories are readable; private repositories require read permission",
    response: `From abcdef1234567890 Mon Sep 17 00:00:00 2001
From: mona
Date: 2026-05-01T00:00:00+00:00
Subject: [PATCH] Improve docs

---
 1 files changed, 1 insertions(+), 0 deletions(-)`,
    notes: [
      "Use .patch when a client needs commit headers; use .diff for review-only file hunks.",
    ],
  },
  {
    id: "actions-dashboard",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/actions/dashboard?q=ci&status=success&page=1&pageSize=30",
    title: "Read Actions dashboard",
    description:
      "Returns the repository Actions all-workflows page contract, including workflow rail items, filtered run rows, filter options, and the no-workflows empty state.",
    auth: "Public repositories are readable; private repositories require read permission",
    response: `{
  "repository": { "ownerLogin": "mona", "name": "octo-app" },
  "workflows": [{ "id": "workflow_01", "name": "CI", "runCount": 2 }],
  "runs": { "items": [], "total": 0, "page": 1, "pageSize": 30 },
  "filters": { "q": "ci", "status": "success" },
  "filterOptions": { "statuses": [{ "value": "success", "count": 2 }] }
}`,
    notes: [
      "Filter params are q, workflow, event, status, branch, actor, page, and pageSize.",
      "Status values include action_required, cancelled, completed, failure, in_progress, neutral, queued, skipped, stale, success, timed_out, and waiting.",
      "Signed-in reads may be followed by recent-view telemetry writes.",
    ],
  },
  {
    id: "actions-workflow-detail",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/actions/workflows/{workflow_file}/dashboard?status=success",
    title: "Read workflow detail dashboard",
    description:
      "Returns the workflow-specific Actions page contract: selected workflow metadata, source link, workflow rail, scoped run rows, branch/event/status/actor filters, refs available for dispatch, and invalid-YAML state.",
    auth: "Public repositories are readable; private repositories require read permission",
    response: `{
  "workflow": {
    "name": "CI",
    "path": ".github/workflows/ci.yml",
    "sourceHref": "/mona/octo-app/blob/main/.github/workflows/ci.yml",
    "dispatch": { "enabled": true, "inputs": [] },
    "valid": true,
    "yamlParseError": null,
    "yamlParsedAt": "2026-05-01T00:00:00Z"
  },
  "runs": { "items": [], "total": 0, "page": 1, "pageSize": 30 },
  "filterOptions": { "statuses": [{ "value": "success", "count": 2 }] }
}`,
    notes: [
      "workflow_file is the URL-encoded workflow path, for example .github%2Fworkflows%2Fci.yml.",
      "The Workflow filter is intentionally omitted because the route is already scoped.",
      "Invalid YAML keeps the workflow visible and returns a sanitized yamlParseError plus dispatch.enabled=false.",
    ],
  },
  {
    id: "actions-workflow-dispatch",
    method: "POST",
    path: "/api/repos/{owner}/{repo}/actions/workflows/{workflow_file}/dispatches",
    title: "Dispatch workflow run",
    description:
      "Queues a manual workflow_dispatch run for a workflow that enables dispatch on the default branch.",
    auth: "Signed opengithub session cookie with write access",
    request: `{
  "ref": "main",
  "inputs": {
    "environment": "staging",
    "dryRun": "true"
  }
}`,
    response: `{
  "id": "run_02",
  "workflowName": "CI",
  "runNumber": 8,
  "status": "queued",
  "event": "workflow_dispatch",
  "headBranch": "main"
}`,
    notes: [
      "The API validates ref existence, required inputs, choice values, boolean strings, and bounded input sizes.",
      "Disabled dispatch, invalid workflow YAML, and missing write permission return standard error envelopes.",
      "A successful dispatch seeds queued run/job/check records before the background runner picks it up.",
    ],
  },
  {
    id: "actions-workflows-list",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/actions/workflows?page=1&pageSize=30",
    title: "List Actions workflows",
    description:
      "Lists workflow files registered for a repository with bounded pagination.",
    auth: "Signed opengithub session cookie with read access",
    response: `{
  "items": [
    {
      "id": "workflow_01",
      "name": "CI",
      "path": ".github/workflows/ci.yml",
      "state": "active"
    }
  ],
  "total": 1,
  "page": 1,
  "pageSize": 30
}`,
    notes: ["The dashboard endpoint exposes public-read workflow summaries."],
  },
  {
    id: "actions-workflows-create",
    method: "POST",
    path: "/api/repos/{owner}/{repo}/actions/workflows",
    title: "Create Actions workflow",
    description:
      "Registers a workflow file for a repository before runner execution is available.",
    auth: "Signed opengithub session cookie with write access",
    request: `{
  "name": "CI",
  "path": ".github/workflows/ci.yml",
  "triggerEvents": ["push", "pull_request"]
}`,
    response: `{
  "id": "workflow_01",
  "name": "CI",
  "path": ".github/workflows/ci.yml",
  "state": "active"
}`,
    notes: ["Blank workflow names or paths return 422 validation errors."],
  },
  {
    id: "actions-workflows-read",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/actions/workflows/{workflow_id}",
    title: "Read Actions workflow",
    description:
      "Reads one workflow file registration after repository read authorization.",
    auth: "Signed opengithub session cookie with read access",
    response: `{
  "id": "workflow_01",
  "name": "CI",
  "path": ".github/workflows/ci.yml",
  "triggerEvents": ["push", "pull_request"]
}`,
    notes: ["Unknown workflow ids return the standard 404 envelope."],
  },
  {
    id: "actions-runs-list",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/actions/runs?page=1&pageSize=30",
    title: "List workflow runs",
    description:
      "Lists workflow runs across the repository or under a specific workflow route.",
    auth: "Signed opengithub session cookie with read access",
    response: `{
  "items": [
    {
      "id": "run_01",
      "workflowId": "workflow_01",
      "status": "completed",
      "conclusion": "success"
    }
  ],
  "total": 1,
  "page": 1,
  "pageSize": 30
}`,
    notes: [
      "Use /actions/workflows/{workflow_id}/runs to list runs for one workflow.",
    ],
  },
  {
    id: "actions-runs-create",
    method: "POST",
    path: "/api/repos/{owner}/{repo}/actions/workflows/{workflow_id}/runs",
    title: "Create workflow run",
    description:
      "Records a workflow run for an existing workflow. Runner execution is handled by later Actions features.",
    auth: "Signed opengithub session cookie with write access",
    request: `{
  "headBranch": "main",
  "headSha": "abcdef1234567890",
  "event": "workflow_dispatch"
}`,
    response: `{
  "id": "run_01",
  "workflowId": "workflow_01",
  "status": "queued",
  "conclusion": null
}`,
    notes: ["Status transitions use the same envelope and auth contract."],
  },
  {
    id: "actions-runs-read",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/actions/runs/{run_id}",
    title: "Read workflow run",
    description: "Reads one workflow run after repository read authorization.",
    auth: "Signed opengithub session cookie with read access",
    response: `{
  "id": "run_01",
  "workflowId": "workflow_01",
  "status": "completed",
  "conclusion": "success",
  "headBranch": "main"
}`,
    notes: ["Run detail pages link here until the full logs UI lands."],
  },
  {
    id: "actions-runs-detail",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/actions/runs/{run_id}/detail",
    title: "Read workflow run detail",
    description:
      "Returns the run header, attempts, jobs, steps, annotations, artifacts, log availability, and action eligibility for the run workspace.",
    auth: "Optional signed opengithub session cookie; private repositories require read access",
    response: `{
  "run": { "id": "run_01", "status": "completed", "conclusion": "failure" },
  "attempts": [{ "attemptNumber": 1, "triggerKind": "initial" }],
  "jobs": [{ "id": "job_01", "name": "unit", "logAvailable": true }],
  "actionState": { "canRerun": true, "canCancel": false }
}`,
    notes: [
      "Writers receive mutation eligibility in actionState; readers can inspect public run data only.",
    ],
  },
  {
    id: "actions-runs-rerun",
    method: "POST",
    path: "/api/repos/{owner}/{repo}/actions/runs/{run_id}/rerun",
    title: "Re-run workflow jobs",
    description:
      "Queues a new run attempt for all jobs, failed jobs, or a specific job and returns the refreshed run detail.",
    auth: "Signed opengithub session cookie with write access",
    request: `{
  "mode": "failed",
  "jobId": null
}`,
    response: `{
  "run": { "id": "run_01", "status": "queued", "conclusion": null },
  "attempts": [{ "attemptNumber": 2, "triggerKind": "rerun_failed" }]
}`,
    notes: [
      "mode may be all, failed, or job. job mode requires jobId. Non-terminal runs return 409 conflict.",
    ],
  },
  {
    id: "actions-runs-cancel",
    method: "POST",
    path: "/api/repos/{owner}/{repo}/actions/runs/{run_id}/cancel",
    title: "Cancel workflow run",
    description:
      "Cancels a queued or in-progress run, marks queued/in-progress jobs as cancelled, writes an audit event, and returns the refreshed detail.",
    auth: "Signed opengithub session cookie with write access",
    response: `{
  "run": { "id": "run_01", "status": "cancelled", "conclusion": "cancelled" }
}`,
    notes: ["Completed runs return 409 conflict instead of mutating state."],
  },
  {
    id: "actions-runs-delete-logs",
    method: "DELETE",
    path: "/api/repos/{owner}/{repo}/actions/runs/{run_id}/logs",
    title: "Delete workflow run logs",
    description:
      "Marks every job log in a terminal run as deleted, removes stored log lines, writes an audit event, and returns the refreshed detail.",
    auth: "Signed opengithub session cookie with write access",
    response: `{
  "jobs": [{ "id": "job_01", "logAvailable": false, "logDeletedAt": "2026-05-02T00:00:00Z" }]
}`,
    notes: [
      "The operation is idempotent for jobs whose logs were already deleted.",
    ],
  },
  {
    id: "actions-job-log-detail",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/actions/runs/{run_id}/jobs/{job_id}/detail?q=error&match=1&timestamps=true&raw=false",
    title: "Read workflow job log detail",
    description:
      "Returns the dedicated job log viewer contract, including run breadcrumbs, sibling jobs, grouped steps, annotations, search matches, display options, and unavailable log state.",
    auth: "Optional signed opengithub session cookie; private repositories require read access",
    response: `{
  "job": { "id": "job_01", "name": "unit / web", "logAvailable": true },
  "steps": [{ "name": "Run tests", "matchCount": 1 }],
  "search": { "query": "error", "totalMatches": 1, "selectedMatch": 1 },
  "options": { "showTimestamps": true, "rawLogs": false, "wrapLines": true },
  "downloadHref": "/api/repos/mona/octo-app/actions/jobs/job_01/logs/download",
  "runArchiveHref": "/api/repos/mona/octo-app/actions/runs/run_01/logs/archive"
}`,
    notes: [
      "Query params are q, match, timestamps, raw, page, and pageSize.",
      "Deleted logs keep the page contract readable with logState.status=410 and no line leakage.",
    ],
  },
  {
    id: "actions-log-preferences",
    method: "PATCH",
    path: "/api/repos/{owner}/{repo}/actions/log-preferences",
    title: "Update Actions log preferences",
    description:
      "Persists per-user display preferences for the repository job log viewer.",
    auth: "Signed opengithub session cookie with read access",
    request: `{
  "showTimestamps": true,
  "rawLogs": false,
  "wrapLines": true
}`,
    response: `{
  "showTimestamps": true,
  "rawLogs": false,
  "wrapLines": true
}`,
    notes: [
      "Anonymous viewers can still use query params for temporary display options, but preference writes require a signed session.",
    ],
  },
  {
    id: "actions-job-log-download",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/actions/jobs/{job_id}/logs/download",
    title: "Download workflow job log",
    description:
      "Downloads one job log as a deterministic local-dev text attachment after repository read authorization.",
    auth: "Optional signed opengithub session cookie; private repositories require read access",
    response: `2026-05-01T00:00:00Z Installing dependencies
2026-05-01T00:01:00Z Running unit tests`,
    notes: [
      "Deleted or expired logs return the standard 410 gone envelope.",
      "Production storage can swap the body for a short-lived signed object URL while preserving the route contract.",
    ],
  },
  {
    id: "actions-run-log-archive",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/actions/runs/{run_id}/logs/archive",
    title: "Download workflow run log archive",
    description:
      "Downloads a run-level log archive containing all available job logs for the run.",
    auth: "Optional signed opengithub session cookie; private repositories require read access",
    response: `opengithub workflow log archive
repository: mona/octo-app
run: #42

== unit / web ==
2026-05-01T00:00:00Z Running unit tests`,
    notes: [
      "The MVP returns a deterministic text archive for local development.",
      "If every job log is deleted or unavailable, the endpoint returns 410 gone.",
    ],
  },
  {
    id: "actions-runs-update",
    method: "PATCH",
    path: "/api/repos/{owner}/{repo}/actions/runs/{run_id}",
    title: "Update workflow run status",
    description:
      "Transitions a workflow run status and optional conclusion for the repository-owned Actions MVP.",
    auth: "Signed opengithub session cookie with write access",
    request: `{
  "status": "completed",
  "conclusion": "success"
}`,
    response: `{
  "id": "run_01",
  "status": "completed",
  "conclusion": "success"
}`,
    notes: [
      "Invalid status and conclusion combinations return 422 validation errors.",
    ],
  },
  {
    id: "actions-recent-view",
    method: "POST",
    path: "/api/repos/{owner}/{repo}/actions/recent-view",
    title: "Record recent Actions view",
    description:
      "Persists the signed-in user's latest Actions workflow/filter telemetry for non-blocking UI recall.",
    auth: "Signed opengithub session cookie with read access",
    request: `{
  "q": "deploy",
  "workflow": "workflow_01",
  "status": "success",
  "branch": "main"
}`,
    response: `{
  "repositoryId": "repo_01",
  "userId": "user_01",
  "filters": { "q": "deploy", "status": "success", "branch": "main" }
}`,
    notes: ["The browser treats telemetry failures as non-fatal."],
  },
  {
    id: "packages",
    method: "POST",
    path: "/api/repos/{owner}/{repo}/packages",
    title: "Create package metadata",
    description:
      "Creates package metadata and package versions owned by a repository.",
    auth: "Signed opengithub session cookie",
    request: `{
  "name": "@mona/octo-app",
  "package_type": "npm",
  "visibility": "private"
}`,
    response: `{
  "id": "package_01",
  "name": "@mona/octo-app",
  "package_type": "npm",
  "visibility": "private"
}`,
    notes: ["Package blob upload depth is intentionally outside api-001."],
  },
  {
    id: "search",
    method: "GET",
    path: "/api/search?q=router&type=code&page=1&pageSize=30",
    title: "Search code and issues",
    description:
      "Searches indexed opengithub data with permission-aware filtering.",
    auth: "Signed opengithub session cookie",
    response: `{
  "items": [
    {
      "kind": "code",
      "repository": "mona/octo-app",
      "path": "src/router.rs",
      "fragment": "Router::new()"
    }
  ],
  "total": 1,
  "page": 1,
  "pageSize": 30
}`,
    notes: ["Short or malformed queries return 422 validation errors."],
  },
  {
    id: "search-rest-code",
    method: "GET",
    path: "/api/search/code?q=router+language:Rust&per_page=30",
    title: "REST code search",
    description:
      "Returns GitHub-compatible code search envelopes for indexed code visible to the signed-in user.",
    auth: "Signed opengithub session cookie",
    response: `{
  "total_count": 1,
  "incomplete_results": false,
  "items": [
    {
      "name": "src/router.rs",
      "path": "src/router.rs",
      "html_url": "/mona/octo-app/blob/main/src/router.rs#L4",
      "repository": { "full_name": "mona/octo-app" }
    }
  ]
}`,
    notes: [
      "Supports repo:, path:, user:, language:, and archived qualifiers.",
    ],
  },
  {
    id: "search-rest-repositories",
    method: "GET",
    path: "/api/search/repositories?q=router&sort=updated&order=desc",
    title: "REST repository search",
    description:
      "Returns repository search results with total_count, incomplete_results, and GitHub-style item links.",
    auth: "Signed opengithub session cookie",
    response: `{
  "total_count": 1,
  "incomplete_results": false,
  "items": [
    {
      "name": "octo-app",
      "full_name": "mona/octo-app",
      "private": false,
      "html_url": "/mona/octo-app"
    }
  ]
}`,
    notes: [
      "Private repositories are visible only to users with repository access.",
    ],
  },
  {
    id: "search-rest-issues",
    method: "GET",
    path: "/api/search/issues?q=router+state:open&sort=updated&order=desc",
    title: "REST issue search",
    description:
      "Returns issue search rows with labels, repository metadata, state filters, and text matches.",
    auth: "Signed opengithub session cookie",
    response: `{
  "total_count": 1,
  "incomplete_results": false,
  "items": [
    {
      "number": 42,
      "title": "Router bug",
      "state": "open",
      "html_url": "/mona/octo-app/issues/42"
    }
  ]
}`,
    notes: [
      "Supports state:/is:, repo:, user:/org:, label:, assignee:, and milestone qualifiers.",
    ],
  },
  {
    id: "search-rest-users",
    method: "GET",
    path: "/api/search/users?q=octocat+user:octocat",
    title: "REST user search",
    description:
      "Returns user search results with stable login, avatar_url, html_url, and score fields.",
    auth: "Signed opengithub session cookie",
    response: `{
  "total_count": 1,
  "incomplete_results": false,
  "items": [
    { "login": "octocat", "type": "User", "html_url": "/octocat" }
  ]
}`,
    notes: [
      "Short or malformed queries return the standard validation envelope.",
    ],
  },
  {
    id: "search-rest-commits",
    method: "GET",
    path: "/api/search/commits?q=router+repo:mona/octo-app",
    title: "REST commit search",
    description:
      "Returns indexed commit search results with sha, commit metadata, repository summary, and html_url.",
    auth: "Signed opengithub session cookie",
    response: `{
  "total_count": 1,
  "incomplete_results": false,
  "items": [
    {
      "sha": "abcdef123456",
      "html_url": "/mona/octo-app/commit/abcdef123456",
      "commit": { "message": "Add router" }
    }
  ]
}`,
    notes: [
      "Results are permission-aware and reuse the indexed search document store.",
    ],
  },
  {
    id: "search-suggestions",
    method: "GET",
    path: "/api/search/suggestions?q=router&scope=all&limit=8",
    title: "Search suggestions",
    description:
      "Returns command-modal scopes, qualifier completions, direct jumps, saved searches, and recent searches visible to the signed-in viewer.",
    auth: "Signed opengithub session cookie",
    response: `{
  "query": "router",
  "scope": "all",
  "groups": [
    {
      "id": "repositories",
      "title": "Repositories and code",
      "items": [{ "action": "navigate", "href": "/mona/octo-app" }]
    }
  ],
  "savedSearches": [],
  "recentSearches": []
}`,
    notes: ["Private repository and code suggestions require viewer access."],
  },
  {
    id: "search-saved-create",
    method: "POST",
    path: "/api/search/saved-searches",
    title: "Create saved search",
    description:
      "Persists a named search for the signed-in viewer and returns the row used by the global search modal.",
    auth: "Signed opengithub session cookie",
    request: `{
  "name": "Rust routers",
  "query": "router language:rust",
  "scope": "code"
}`,
    response: `{
  "id": "saved_01",
  "name": "Rust routers",
  "query": "router language:rust",
  "scope": "code",
  "href": "/search?q=router+language%3Arust&type=code"
}`,
    notes: [
      "Name and query are required.",
      "Duplicate names for the same viewer return 409 duplicate_saved_search.",
    ],
  },
  {
    id: "search-saved-delete",
    method: "DELETE",
    path: "/api/search/saved-searches/{id}",
    title: "Delete saved search",
    description:
      "Deletes a saved search owned by the signed-in viewer. Other users' saved searches are not addressable.",
    auth: "Signed opengithub session cookie",
    response: `204 No Content`,
    notes: ["Unknown or unauthorized saved search IDs return 404."],
  },
];

export const paginationExample = `GET /api/repos?page=2&pageSize=10

{
  "items": [],
  "total": 42,
  "page": 2,
  "pageSize": 10
}`;

export const errorEnvelopeExample = `{
  "error": {
    "code": "validation_failed",
    "message": "Repository name is required"
  },
  "status": 422
}`;
