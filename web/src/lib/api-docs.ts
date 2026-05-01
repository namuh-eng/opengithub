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
    id: "actions-runs",
    method: "POST",
    path: "/api/repos/{owner}/{repo}/actions/runs",
    title: "Create workflow run",
    description:
      "Records a workflow run for an existing workflow. Runner execution is handled by later Actions features.",
    auth: "Signed opengithub session cookie",
    request: `{
  "workflowId": "workflow_01",
  "ref": "main",
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
