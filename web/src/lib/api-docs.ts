export type ApiDocMethod = "GET" | "POST" | "PATCH" | "PUT" | "DELETE";

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
    id: "gists-list",
    method: "GET",
    path: "/api/gists?scope=mine",
    title: "List gists",
    description:
      "Lists public gists plus the caller's own secret gists, or only the caller's gists with scope=mine.",
    auth: "Optional signed opengithub session cookie",
    response: `{
  "items": [{
    "id": "gist_01",
    "description": "Release helper",
    "isPublic": false,
    "files": [{ "filename": "release.ts", "language": "TypeScript" }],
    "cloneUrl": "https://opengithub.namuh.co/gist/gist_01.git",
    "embedUrl": "https://opengithub.namuh.co/api/gists/gist_01/embed.js"
  }],
  "total": 1,
  "page": 1,
  "pageSize": 30,
  "scope": "mine"
}`,
    notes: [
      "Secret gists are unlisted and visible by URL only unless the owner is signed in.",
      "User profiles call /api/users/{username}/gists for the public gists widget.",
    ],
  },
  {
    id: "gists-create",
    method: "POST",
    path: "/api/gists",
    title: "Create gist",
    description:
      "Creates a public or secret single-file or multi-file gist with detected languages, content hashes, and initial revision history.",
    auth: "Signed opengithub session cookie",
    request: `{
  "description": "Release helper",
  "isPublic": false,
  "files": [{ "filename": "release.ts", "content": "export const release = true;" }]
}`,
    response: `{
  "id": "gist_01",
  "description": "Release helper",
  "isPublic": false,
  "files": [{ "filename": "release.ts", "language": "TypeScript" }],
  "viewer": { "canEdit": true, "isStarred": false }
}`,
    notes: [
      "PATCH /api/gists/{gist_id} replaces the file set and appends a new revision.",
      "PUT/DELETE /api/gists/{gist_id}/star and POST /api/gists/{gist_id}/fork back the browser controls.",
      "GET /api/gists/{gist_id}/embed.js returns a self-contained script that writes token-compatible HTML.",
    ],
  },
  {
    id: "ai-repository-summary",
    method: "GET",
    path: "/api/ai/repos/{owner}/{repo}/summary",
    title: "AI repository summary",
    description:
      "Returns or generates a cached OpenAI-backed repository summary from README, top-level files, and recent commits.",
    auth: "Optional signed opengithub session cookie; private repositories require read access and repository AI opt-in",
    response: `{
  "enabled": true,
  "reason": null,
  "output": {
    "kind": "repo_summary",
    "model": "gpt-4o-mini",
    "output": "Three concise bullets...",
    "cached": true
  }
}`,
    notes: [
      "POST to the same path regenerates the cache entry.",
      "Private repository content never leaves the cluster unless repository ai_features_enabled is true.",
      "Outputs are cached in ai_outputs by content_hash, prompt_version, and model.",
    ],
  },
  {
    id: "ai-pull-request-summary",
    method: "GET",
    path: "/api/ai/repos/{owner}/{repo}/pulls/{number}/summary",
    title: "AI pull request summary",
    description:
      "Builds the PR AI tab contract: TL;DR output, files of interest, reviewer suggestions, and an author-only inline comment seed.",
    auth: "Optional signed opengithub session cookie; private repositories require read access and repository AI opt-in",
    response: `{
  "enabled": true,
  "filesOfInterest": [{ "path": "src/main.rs", "note": "modified +12 -3" }],
  "suggestedReviewers": [{ "login": "mona", "reason": "recent committer" }],
  "inlineCommentSeed": "Check whether this needs an integration test."
}`,
    notes: [
      "POST to the same path forces a fresh OpenAI call when OPENAI_API_KEY is configured.",
      "Reviewer suggestions prefer requested reviewers and fall back to recent committers.",
    ],
  },
  {
    id: "ai-release-changelog",
    method: "POST",
    path: "/api/ai/repos/{owner}/{repo}/releases/changelog",
    title: "AI release changelog",
    description:
      "Generates grouped Added/Changed/Fixed/Deprecated Markdown bullets from repository commit history for edit-before-publish release notes.",
    auth: "Optional signed opengithub session cookie; private repositories require read access and repository AI opt-in",
    request: `{
  "previousTag": "v1.9.0",
  "targetTag": "v2.0.0"
}`,
    response: `{
  "enabled": true,
  "targetTag": "v2.0.0",
  "output": { "kind": "changelog", "model": "gpt-4o" }
}`,
    notes: [
      "Uses the same ai_outputs cache contract as repository and PR summaries.",
      "Provider failures return ai_provider_failed without exposing prompts or secrets.",
    ],
  },
  {
    id: "organization-slug-availability",
    method: "GET",
    path: "/api/organizations/slug-availability?name=Acme%20Labs",
    title: "Check organization slug availability",
    description:
      "Normalizes a proposed organization name and reports whether the resulting URL slug can be used by the signed-in user before submitting the create form.",
    auth: "Signed opengithub session cookie",
    response: `{
  "normalizedSlug": "acme-labs",
  "available": true,
  "reason": null,
  "existingKind": null
}`,
    notes: [
      "The same normalizer is used by the browser preview and POST /api/organizations.",
      "Reserved slugs and existing user or organization logins return available=false without leaking private account metadata.",
      "Validation errors use the standard validation_failed envelope and never include stack traces.",
    ],
  },
  {
    id: "organization-create",
    method: "POST",
    path: "/api/organizations",
    title: "Create organization",
    description:
      "Creates a Free organization from the onboarding flow with owner membership, default policy settings, and a redacted audit event.",
    auth: "Signed opengithub session cookie",
    request: `{
  "name": "Acme Labs",
  "contactEmail": "admin@example.com",
  "ownershipType": "business",
  "companyName": "Acme Labs Inc.",
  "termsAccepted": true
}`,
    response: `{
  "id": "org_01",
  "slug": "acme-labs",
  "displayName": "Acme Labs",
  "contactEmail": "admin@example.com",
  "ownershipType": "business",
  "termsOfServiceType": "free_organization_terms",
  "companyName": "Acme Labs Inc.",
  "href": "/orgs/acme-labs"
}`,
    notes: [
      "Only the Free plan is provisioned in the MVP; paid plan cards remain informational browser states.",
      "The creator receives the owner role in organization_memberships in the same transaction.",
      "New rows receive default organization_policy_settings before the API returns success.",
      "Duplicate, reserved, invalid email, missing terms, and rate-limit failures return stable error envelopes for inline form rendering.",
      "organization_audit_events store redacted create metadata only; contact email and company fields are not written to audit payloads.",
    ],
  },
  {
    id: "organization-profile-settings-read",
    method: "GET",
    path: "/api/orgs/{org}/settings/profile",
    title: "Read organization profile settings",
    description:
      "Returns the owner-only organization settings contract used by the Editorial profile settings shell.",
    auth: "Signed opengithub session cookie with organization owner role",
    response: `{
  "organization": {
    "id": "org_01",
    "slug": "acme-labs",
    "displayName": "Acme Labs",
    "settingsHref": "/organizations/acme-labs/settings/profile"
  },
  "profile": {
    "displayName": "Acme Labs",
    "description": "Open collaboration tools",
    "websiteUrl": "https://opengithub.namuh.co",
    "location": "Seoul, KR",
    "publicEmail": "public@example.com",
    "contactEmail": "owners@example.com",
    "billingEmail": "finance@example.com",
    "avatar": { "state": "unavailable" }
  },
  "socialAccounts": [
    { "provider": "x", "value": "@opengithub" },
    { "provider": "mastodon", "value": "https://social.example/@opengithub" }
  ],
  "danger": {
    "archiveSupported": false,
    "deleteSupported": false
  }
}`,
    notes: [
      "Anonymous callers receive 401; organization members without owner role receive 403 without settings-only contact or billing fields.",
      "The response includes four bounded social account providers and omits raw S3 bucket/object metadata for avatars.",
      "Archive and delete eligibility is explicit so the browser can render disabled non-destructive guardrails while billing and retention policies remain out of scope.",
    ],
  },
  {
    id: "organization-profile-settings-update",
    method: "PATCH",
    path: "/api/orgs/{org}/settings/profile",
    title: "Update organization profile settings",
    description:
      "Persists independent profile, contact, billing-email, location, and social-account updates for organization owners.",
    auth: "Signed opengithub session cookie with organization owner role",
    request: `{
  "displayName": "Acme Labs",
  "description": "Open collaboration tools",
  "websiteUrl": "https://opengithub.namuh.co",
  "location": "Seoul, KR",
  "publicEmail": "public@example.com",
  "contactEmail": "owners@example.com",
  "billingEmail": "finance@example.com",
  "socialAccounts": [
    { "provider": "x", "value": "@opengithub" },
    { "provider": "mastodon", "value": "https://social.example/@opengithub" }
  ]
}`,
    response: `{
  "organization": {
    "slug": "acme-labs",
    "displayName": "Acme Labs"
  },
  "profile": {
    "publicEmail": "public@example.com",
    "contactEmail": "owners@example.com",
    "billingEmail": "finance@example.com"
  },
  "socialAccounts": [
    { "provider": "x", "value": "@opengithub" }
  ]
}`,
    notes: [
      "Partial patches preserve fields that are omitted by a section-specific Save button.",
      "Validation rejects blank display names, non-HTTP(S) URLs, invalid emails, oversized values, and unsupported social providers with validation_failed envelopes.",
      "Every successful write records an organization.profile_settings.update audit event with redacted metadata; contact and billing emails are not copied into audit payloads.",
    ],
  },
  {
    id: "organization-profile-settings-rename",
    method: "POST",
    path: "/api/orgs/{org}/settings/profile/rename",
    title: "Rename organization slug",
    description:
      "Renames an organization after owner confirmation, slug normalization, and availability checks.",
    auth: "Signed opengithub session cookie with organization owner role",
    request: `{
  "newSlug": "acme-platform",
  "confirmation": "acme-labs"
}`,
    response: `{
  "organization": {
    "slug": "acme-platform",
    "displayName": "Acme Labs",
    "settingsHref": "/organizations/acme-platform/settings/profile"
  },
  "profile": {
    "displayName": "Acme Labs"
  }
}`,
    notes: [
      "Reserved, duplicate user, and duplicate organization slugs return the same slug_unavailable envelope without leaking private account details or reserved-source metadata.",
      "The old slug returns not_found after a successful rename, and the browser replaces the URL with the returned settingsHref.",
      "Renames write organization.rename audit events; archive and delete execution remain unsupported and non-destructive until retention and recovery policies are implemented.",
    ],
  },
  {
    id: "organization-member-privileges",
    method: "GET",
    path: "/api/orgs/{org}/settings/member-privileges",
    title: "Read organization member privileges",
    description:
      "Returns the owner-only organization policy settings that drive base repository permissions, repository creation choices, Pages publishing, team creation, app access, discussions, forking, and destructive repository affordances.",
    auth: "Signed opengithub session cookie with organization owner role",
    response: `{
  "organization": {
    "slug": "acme-labs",
    "settingsHref": "/organizations/acme-labs/settings/member_privileges"
  },
  "policies": {
    "baseRepositoryPermission": "read",
    "membersCanCreatePublicRepositories": true,
    "membersCanCreatePrivateRepositories": true,
    "membersCanCreateInternalRepositories": false,
    "membersCanForkPrivateRepositories": true,
    "repositoryDiscussionsEnabled": true,
    "projectsBasePermission": "write",
    "pagesPublicPublishing": true,
    "pagesPrivatePublishing": true,
    "appAccessRequestPolicy": "owners_and_members",
    "membersCanChangeRepositoryVisibility": false,
    "membersCanDeleteRepositories": false,
    "membersCanTransferRepositories": false,
    "membersCanDeleteIssues": false,
    "membersCanCreateTeams": true
  },
  "capabilities": {
    "canUpdate": true,
    "requiresConfirmationFields": [
      "baseRepositoryPermission",
      "projectsBasePermission"
    ],
    "locks": []
  }
}`,
    notes: [
      "Anonymous callers receive 401; organization members, admins, and private-organization outsiders without owner role cannot read this settings-only policy surface.",
      "The API creates a default organization_policy_settings row before returning, so every owner receives a complete policy object.",
      "Base repository permission is inherited by organization members for repository authorization, while explicit direct and team permissions preserve the strongest role.",
      "Repository creation, team creation, Pages publishing, discussions, forking, app-access, visibility, transfer, delete, and issue-delete UI surfaces consume the policy lock fields instead of rendering dead controls.",
    ],
  },
  {
    id: "organization-member-privileges-update",
    method: "PATCH",
    path: "/api/orgs/{org}/settings/member-privileges",
    title: "Update organization member privileges",
    description:
      "Persists partial organization policy changes and returns the refreshed member-privileges contract for the long Editorial settings page.",
    auth: "Signed opengithub session cookie with organization owner role",
    request: `{
  "baseRepositoryPermission": "none",
  "membersCanCreatePublicRepositories": false,
  "pagesPrivatePublishing": false,
  "membersCanCreateTeams": false,
  "confirmation": "confirm"
}`,
    response: `{
  "policies": {
    "baseRepositoryPermission": "none",
    "membersCanCreatePublicRepositories": false,
    "pagesPrivatePublishing": false,
    "membersCanCreateTeams": false
  },
  "capabilities": {
    "requiresConfirmationFields": [
      "baseRepositoryPermission",
      "projectsBasePermission"
    ],
    "locks": []
  }
}`,
    notes: [
      "Partial patches preserve omitted policy fields and validate enum values for baseRepositoryPermission, projectsBasePermission, and appAccessRequestPolicy.",
      "Base repository permission and Projects base permission changes return confirmation_required until the browser resubmits with confirmation=confirm.",
      "Successful writes record redacted organization.policy.update audit events with old/new JSON values and no session secrets, OAuth data, or repository private metadata.",
      "Policy-denied repository creation, Pages source updates, team creation, and repository settings mutations return policy_locked envelopes with a member-privileges settings link for owners.",
    ],
  },
  {
    id: "organization-teams",
    method: "GET",
    path: "/api/orgs/{org}/teams?q=platform&visibility=all&page=1&pageSize=30",
    title: "List organization teams",
    description:
      "Returns the signed-in viewer's organization teams directory with visibility-aware rows, parent options, counts, and the Editorial empty-state contract.",
    auth: "Signed opengithub session cookie with organization membership",
    response: `{
  "items": [
    {
      "slug": "platform",
      "name": "Platform",
      "visibility": "visible",
      "mentionable": true,
      "notificationsEnabled": true,
      "memberCount": 4,
      "repositoryCount": 3,
      "childTeamCount": 1,
      "parent": null,
      "viewerCapabilities": {
        "canView": true,
        "canManage": false,
        "canMention": true,
        "isMember": true
      }
    }
  ],
  "total": 1,
  "page": 1,
  "pageSize": 30,
  "counts": { "total": 1, "visible": 1, "secret": 0, "memberTeams": 1 },
  "emptyState": {
    "newTeamHref": "/orgs/acme-labs/teams/new",
    "learnMoreHref": "/docs/api#organization-teams"
  }
}`,
    notes: [
      "Anonymous callers receive 401; private organizations return not_found for outside viewers without leaking team counts.",
      "Supported visibility filters are all, visible, and secret; invalid filters return validation_failed.",
      "Visible teams are discoverable and @mentionable by organization members; secret teams are returned only to owners/admins or direct members.",
      "Responses include Flexible repository access, Request-to-join teams, and Team mentions empty-state columns, but never invitation tokens or private member records.",
    ],
  },
  {
    id: "profile-followers",
    method: "GET",
    path: "/api/users/{username}/followers?page=1&pageSize=30",
    title: "List profile followers",
    description:
      "Returns a paginated social graph list for /{user}/followers, including profile links, public bios, follow timestamps, and viewer follow/block state.",
    auth: "Optional signed opengithub session cookie; private profiles require the owner",
    response: `{
  "owner": { "login": "mona", "name": "Mona", "href": "/mona" },
  "mode": "followers",
  "items": [
    {
      "id": "user-1",
      "login": "ashley",
      "name": "Ashley",
      "avatarUrl": null,
      "bio": "Maintainer",
      "href": "/ashley",
      "followedAt": "2026-05-07T00:00:00Z",
      "viewerState": {
        "authenticated": true,
        "isSelf": false,
        "isFollowing": false,
        "isBlocking": false,
        "canFollow": true,
        "canBlock": true,
        "canReport": true
      }
    }
  ],
  "total": 1,
  "page": 1,
  "pageSize": 30
}`,
    notes: [
      "GET /api/users/{username}/following returns the same shape with mode following.",
      "Private profiles return forbidden for outside viewers and never expose hidden follower counts, emails, session data, or OAuth metadata.",
    ],
  },
  {
    id: "projects-user-list",
    method: "GET",
    path: "/api/users/{username}/projects?q=is%3Aopen&state=open&tab=projects&sort=recently_updated&page=1&pageSize=30",
    title: "List user Projects",
    description:
      "Returns the Editorial Projects v2 list contract for a user-owned profile tab, including visible project rows, templates, counts, URL-backed filters, and viewer copy permissions.",
    auth: "Optional signed opengithub session cookie; private projects require owner or explicit project access",
    response: `{
  "items": [
    {
      "id": "project_01",
      "number": 12,
      "title": "Roadmap planning",
      "description": "Tracks repository work across the next milestone.",
      "state": "open",
      "visibility": "public",
      "workspaceHref": "/mona/projects/12/views/1",
      "status": { "status": "on_track", "label": "On track" },
      "counts": { "total": 8, "open": 6, "closed": 1, "draft": 1 },
      "viewerRole": "write",
      "viewerCanCopy": true
    }
  ],
  "total": 1,
  "page": 1,
  "pageSize": 30,
  "scope": { "kind": "user", "login": "mona", "repository": null },
  "filters": {
    "query": "is:open",
    "state": "open",
    "tab": "projects",
    "sort": "recently_updated"
  },
  "counts": { "open": 1, "closed": 1, "templates": 1, "total": 2 },
  "templates": { "items": [], "total": 0, "page": 1, "pageSize": 30 },
  "viewerPermissions": { "authenticated": true, "viewerRole": "write", "canCreate": true, "canCopy": true },
  "unavailableReason": null
}`,
    notes: [
      "Supported tab values are projects and templates; supported state values are open and closed.",
      "Supported sort values are recently_updated, name_asc, name_desc, created_desc, and created_asc; invalid filters return validation_failed.",
      "Search matches title, description, status labels, and the is:open or is:closed qualifier used by the Projects search box.",
      "Private user-owned projects are omitted for anonymous or unauthorized viewers instead of leaking counts or titles.",
    ],
  },
  {
    id: "projects-org-list",
    method: "GET",
    path: "/api/orgs/{org}/projects?q=roadmap&state=open&tab=projects&sort=name_asc&page=1&pageSize=30",
    title: "List organization Projects",
    description:
      "Returns an organization Projects v2 directory for the org profile shell with search, open/closed counts, templates, status badges, and policy-aware viewer permissions.",
    auth: "Optional signed opengithub session cookie; private organizations require membership",
    response: `{
  "items": [
    {
      "id": "project_02",
      "number": 4,
      "title": "Platform roadmap",
      "href": "/orgs/acme-labs/projects/4",
      "workspaceHref": "/orgs/acme-labs/projects/4/views/1",
      "owner": "acme-labs",
      "isTemplate": false,
      "linkedRepositoriesCount": 3,
      "status": { "status": "at_risk", "label": "At risk" },
      "viewerCanCopy": true
    }
  ],
  "scope": { "kind": "organization", "login": "acme-labs", "repository": null },
  "counts": { "open": 3, "closed": 1, "templates": 2, "total": 6 },
  "viewerPermissions": { "authenticated": true, "viewerRole": "admin", "canCreate": true, "canCopy": true },
  "unavailableReason": null
}`,
    notes: [
      "Organization policy-disabled Projects return a readable unavailableReason so the browser can render disabled controls without pretending creation is available.",
      "Outside viewers see public organization projects only; private organization metadata and project counts are not exposed.",
      "Templates are returned in a separate templates envelope while retaining the same search, sort, page, and pageSize validation.",
    ],
  },
  {
    id: "projects-repo-list",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/projects?q=release&state=open&tab=projects&sort=created_desc&page=1&pageSize=30",
    title: "List repository Projects",
    description:
      "Returns Projects linked to a repository by default repository or explicit project_repositories rows for the repository Projects tab.",
    auth: "Optional signed opengithub session cookie; private repositories require read access",
    response: `{
  "items": [
    {
      "id": "project_03",
      "number": 9,
      "title": "Release train",
      "workspaceHref": "/orgs/acme-labs/projects/9/views/1",
      "defaultRepository": {
        "owner": "acme-labs",
        "name": "opengithub",
        "fullName": "acme-labs/opengithub",
        "href": "/acme-labs/opengithub"
      },
      "linkedRepositoriesCount": 1,
      "viewerRole": "read",
      "viewerCanCopy": false
    }
  ],
  "scope": {
    "kind": "repository",
    "login": "acme-labs",
    "repository": { "owner": "acme-labs", "name": "opengithub", "fullName": "acme-labs/opengithub" }
  },
  "viewerPermissions": { "authenticated": true, "viewerRole": "read", "canCreate": false, "canCopy": false }
}`,
    notes: [
      "Repository project lists include projects where the repository is the default repository or appears in project_repositories.",
      "Private repositories return not_found to unauthorized viewers, matching repository privacy behavior elsewhere in the API.",
      "Copy affordances are permission-derived per project row and are false for read-only repository viewers.",
    ],
  },
  {
    id: "projects-copy",
    method: "POST",
    path: "/api/projects/{project_id}/copies",
    title: "Copy Project",
    description:
      "Creates a permissioned copy of a Projects v2 project, cloning metadata, repository links, views, fields, workflows, and optional draft issue items.",
    auth: "Signed opengithub session cookie with project write/admin access",
    request: `{
  "title": "[COPY] Roadmap planning",
  "includeDraftIssues": true
}`,
    response: `{
  "id": "project_copy_01",
  "number": 13,
  "title": "[COPY] Roadmap planning",
  "href": "/orgs/acme-labs/projects/13",
  "workspaceHref": "/orgs/acme-labs/projects/13/views/1",
  "owner": "acme-labs",
  "copiedViews": 2,
  "copiedFields": 5,
  "copiedWorkflows": 3,
  "copiedDraftItems": 4
}`,
    notes: [
      "Copies are transactional and preserve source visibility, scope, repository links, view layout, field options, and workflow configuration where the actor has access.",
      "Linked issues and pull requests are not duplicated in this feature; includeDraftIssues copies only draft project items.",
      "Successful writes append audit_events and project_recent_visits rows; permission, policy, closed-source, and missing-source failures use standard error envelopes.",
      "Title is required and bounded; validation failures return validation_failed without stack traces or session details.",
    ],
  },
  {
    id: "projects-workspace-read",
    method: "GET",
    path: "/api/projects/{project_id}/workspace?view=1&q=is%3Aopen&sort=manual&group=Status&slice=Priority&page=1&pageSize=50",
    title: "Read Project workspace",
    description:
      "Returns the screen-ready Projects v2 workspace contract used by user and organization project routes, including table, board, and roadmap saved views, visible fields, grouped rows, slice rail options, URL filters, unsaved-view metadata, layout choices, and viewer capabilities.",
    auth: "Optional signed opengithub session cookie; private projects and hidden linked repositories require project or repository read access",
    response: `{
  "project": {
    "id": "project_02",
    "number": 4,
    "title": "Platform roadmap",
    "owner": "acme-labs",
    "scope": "organization",
    "workspaceHref": "/orgs/acme-labs/projects/4/views/1"
  },
  "selectedView": {
    "id": "view_table",
    "number": 1,
    "name": "Table",
    "layout": "table",
    "updatedAt": "2026-05-06T09:00:00Z",
    "configuration": { "hiddenFieldIds": ["field_cost"] }
  },
  "views": [{ "id": "view_table", "number": 1, "name": "Table", "layout": "table" }],
  "layoutChoices": [
    { "layout": "table", "label": "Table", "keyboardHint": "t", "active": true, "canSelect": true },
    { "layout": "board", "label": "Board", "keyboardHint": "b", "active": false, "canSelect": true },
    { "layout": "roadmap", "label": "Roadmap", "keyboardHint": "r", "active": false, "canSelect": true }
  ],
  "fields": [
    { "id": "field_title", "name": "Title", "type": "title", "hidden": false },
    { "id": "field_status", "name": "Status", "type": "single_select", "hidden": false }
  ],
  "boardConfig": {
    "columnField": { "id": "field_status", "name": "Status", "type": "single_select" },
    "swimlaneField": { "id": "field_priority", "name": "Priority", "type": "single_select" },
    "columns": [{ "id": "todo", "label": "Todo", "value": "todo", "count": 4, "limit": 5, "overLimit": false }]
  },
  "roadmapConfig": {
    "startDateField": { "id": "field_start", "name": "Start date", "type": "date" },
    "targetDateField": { "id": "field_target", "name": "Target date", "type": "date" },
    "markerFields": [{ "id": "field_milestone", "name": "Milestone", "type": "milestone" }],
    "zoom": "month",
    "zoomOptions": ["month", "quarter", "year"]
  },
  "items": [
    {
      "id": "item_issue_01",
      "type": "issue",
      "title": "Ship table workspace",
      "href": "/acme-labs/opengithub/issues/42",
      "repository": { "fullName": "acme-labs/opengithub" },
      "fieldValues": [{ "fieldId": "field_status", "displayValue": "In progress" }]
    }
  ],
  "groups": [{ "key": "in_progress", "label": "In progress", "count": 1 }],
  "slices": [{ "key": "p1", "label": "Priority 1", "count": 1 }],
  "filters": { "query": "is:open", "sort": "manual", "group": "Status", "slice": "Priority", "tokens": ["is:open"] },
  "unsavedView": { "active": true, "reasons": ["query", "group"] },
  "viewerPermissions": { "canEdit": true, "canManageViews": true, "canAddItems": true },
  "total": 1
}`,
    notes: [
      "Supported filter qualifiers include text terms, is:open, is:closed, is:issue, is:pr, is:draft, repo:owner/name, assignee, label, no:assignee, no:label, and supported field equality filters.",
      "layoutChoices exposes Table, Board, and Roadmap with keyboard hints t, b, and r; unsupported layouts include unavailableReason copy instead of dead controls.",
      "boardConfig computes eligible column and swimlane fields, column counts, limits, over-limit warnings, and empty columns from project_board_column_settings plus visible item values.",
      "roadmapConfig returns compatible start, target, marker fields, Month/Quarter/Year zoom choices, and missing-date warning metadata while preserving filter, sort, group, and slice state.",
      "Linked issue and pull request rows are omitted when the viewer can read the project but cannot read the backing private repository.",
      "Invalid view numbers, unsupported grouping or slicing fields, malformed pagination, and unknown sort values return standard validation_failed envelopes.",
      "The response intentionally carries viewer capability flags so the browser can disable write controls instead of rendering dead actions.",
    ],
  },
  {
    id: "projects-view-layout-update",
    method: "PATCH",
    path: "/api/projects/{project_id}/views/{view_id}/layout",
    title: "Update Project view layout",
    description:
      "Persists the active Projects view layout as table, board, or roadmap while preserving the saved view's existing filter, sort, group, and slice state.",
    auth: "Signed opengithub session cookie with project write/admin access",
    request: `{
  "expectedUpdatedAt": "2026-05-06T09:00:00Z",
  "layout": "board",
  "columnFieldId": "field_status",
  "swimlaneFieldId": "field_priority"
}`,
    response: `{
  "selectedView": {
    "id": "view_board",
    "layout": "board",
    "configuration": {
      "columnFieldId": "field_status",
      "swimlaneFieldId": "field_priority"
    }
  },
  "layoutChoices": [
    { "layout": "board", "label": "Board", "keyboardHint": "b", "active": true, "canSelect": true }
  ]
}`,
    notes: [
      "layout must be table, board, or roadmap; board layouts require a status, single-select, or iteration column field.",
      "Roadmap layouts require compatible start and target date or iteration fields; incompatible field ids return validation_failed without changing the saved view.",
      "expectedUpdatedAt protects against stale layout saves; conflicts return conflict and keep the newer view configuration.",
      "Successful saves append audit_events, refresh the workspace response, and keep URL filter, sort, group, and slice state intact.",
    ],
  },
  {
    id: "projects-view-state-update",
    method: "PATCH",
    path: "/api/projects/{project_id}/views/{view_id}/state",
    title: "Update Project view state",
    description:
      "Persists Projects table view filters, sort, grouping, slicing, and visible field configuration after the browser marks a saved view as unsaved.",
    auth: "Signed opengithub session cookie with project write/admin access",
    request: `{
  "expectedUpdatedAt": "2026-05-06T09:00:00Z",
  "query": "is:open label:api",
  "sort": "updated_desc",
  "group": "Status",
  "slice": "Priority",
  "hiddenFieldIds": ["field_cost", "field_risk"]
}`,
    response: `{
  "selectedView": {
    "id": "view_table",
    "configuration": {
      "query": "is:open label:api",
      "sort": "updated_desc",
      "group": "Status",
      "slice": "Priority",
      "hiddenFieldIds": ["field_cost", "field_risk"]
    }
  },
  "unsavedView": { "active": false, "reasons": [] }
}`,
    notes: [
      "expectedUpdatedAt protects against stale view edits; conflicts return conflict without overwriting a newer saved view.",
      "Only table views can be updated by this endpoint; board, roadmap, insight, automation, and field settings administration are outside projects-002.",
      "Successful saves append audit_events and return the refreshed workspace response.",
      "Unsupported filters, incompatible group or slice fields, and hidden field ids outside the project return validation_failed without exposing session or SQL details.",
    ],
  },
  {
    id: "projects-item-field-update",
    method: "PATCH",
    path: "/api/projects/{project_id}/items/{item_id}/fields/{field_id}",
    title: "Update Project item field",
    description:
      "Updates one editable Projects table cell and synchronizes mapped linked issue or pull request metadata when the field represents native repository data.",
    auth: "Signed opengithub session cookie with project write/admin access; linked issue and pull request metadata edits also require write access to the backing repository",
    request: `{
  "expectedUpdatedAt": "2026-05-06T09:15:00Z",
  "value": { "kind": "single_select", "optionId": "status_in_progress" }
}`,
    response: `{
  "items": [
    {
      "id": "item_issue_01",
      "fieldValues": [{ "fieldId": "field_status", "displayValue": "In progress" }]
    }
  ]
}`,
    notes: [
      "Supported values cover title, status and single-select options, iteration, date, text, number, labels, assignees, and milestone where the project field type allows it.",
      "Custom project fields update project_item_field_values; native title, status, labels, assignees, and milestone fields also sync to linked issues or pull requests when permitted.",
      "Archived items, stale expectedUpdatedAt values, invalid option ids, unreadable repositories, and read-only viewers return standard error envelopes.",
      "Successful edits create project item events plus timeline, audit, and notification side effects for linked resources.",
    ],
  },
  {
    id: "projects-item-add",
    method: "POST",
    path: "/api/projects/{project_id}/items",
    title: "Add Project item",
    description:
      "Adds a linked issue, linked pull request, or draft issue from the Projects table omnibar and returns the refreshed workspace.",
    auth: "Signed opengithub session cookie with project write/admin access and read access to linked repositories",
    request: `{
  "kind": "linked",
  "resourceUrl": "/acme-labs/opengithub/issues/42",
  "positionAfterItemId": "item_previous"
}`,
    response: `{
  "items": [
    { "id": "item_issue_01", "type": "issue", "title": "Ship table workspace" }
  ],
  "total": 9
}`,
    notes: [
      "For draft issues, submit kind=draft with title, body, and optional field defaults; linked items can be supplied by app-relative URL or known resource id.",
      "Duplicate linked issues or pull requests are rejected with conflict instead of silently creating two rows for the same resource.",
      "Filtered metadata defaults are applied only when valid for the target field; invalid defaults return validation_failed.",
      "Successful adds write project item events, audit_events, timeline events for linked resources, and notification fanout where configured.",
    ],
  },
  {
    id: "projects-items-bulk-add",
    method: "POST",
    path: "/api/projects/{project_id}/items/bulk",
    title: "Bulk add Project items",
    description:
      "Adds multiple readable linked issues or pull requests from the Projects table bulk-add dialog.",
    auth: "Signed opengithub session cookie with project write/admin access and read access to every linked repository",
    request: `{
  "items": [
    { "kind": "linked", "resourceUrl": "/acme-labs/opengithub/issues/42" },
    { "kind": "linked", "resourceUrl": "/acme-labs/opengithub/pull/7" }
  ],
  "positionAfterItemId": "item_previous"
}`,
    response: `{
  "items": [
    { "id": "item_issue_01", "type": "issue" },
    { "id": "item_pr_01", "type": "pull_request" }
  ],
  "total": 10
}`,
    notes: [
      "Bulk requests validate every requested item before writing, so private or duplicate resources do not produce partial success.",
      "Readable repository checks happen before item creation to avoid leaking private issue or pull request titles.",
      "Successful bulk adds preserve manual order and append project_item_events plus audit evidence for each inserted item.",
    ],
  },
  {
    id: "projects-item-position",
    method: "PATCH",
    path: "/api/projects/{project_id}/items/{item_id}/position",
    title: "Reorder Project item",
    description:
      "Persists manual Projects ordering after table row move controls or board column move interactions.",
    auth: "Signed opengithub session cookie with project write/admin access",
    request: `{
  "positionBeforeItemId": "item_next",
  "positionAfterItemId": "item_previous",
  "expectedUpdatedAt": "2026-05-06T09:20:00Z",
  "groupFieldId": "field_status",
  "groupValue": "done"
}`,
    response: `{
  "items": [
    { "id": "item_previous", "position": "a1" },
    { "id": "item_issue_01", "position": "a2" },
    { "id": "item_next", "position": "a3" }
  ]
}`,
    notes: [
      "Board moves can include groupFieldId and groupValue so the backing status, single-select, or iteration field changes in the same mutation as the manual position.",
      "Column value validation reuses the item field update rules; invalid target columns, incompatible swimlane values, archived items, stale rows, and read-only viewers return standard errors.",
      "Successful moves write project item events and audit_events.",
    ],
  },
  {
    id: "projects-roadmap-settings-update",
    method: "PATCH",
    path: "/api/projects/{project_id}/views/{view_id}/roadmap-settings",
    title: "Update Project roadmap settings",
    description:
      "Persists Roadmap start and target date fields, marker fields, and timeline zoom for a roadmap view.",
    auth: "Signed opengithub session cookie with project write/admin access",
    request: `{
  "expectedUpdatedAt": "2026-05-06T09:25:00Z",
  "startFieldId": "field_start",
  "targetFieldId": "field_target",
  "markerFieldIds": ["field_milestone", "field_iteration"],
  "zoom": "quarter"
}`,
    response: `{
  "selectedView": {
    "id": "view_roadmap",
    "layout": "roadmap"
  },
  "roadmapConfig": {
    "startDateField": { "id": "field_start", "name": "Start date", "type": "date" },
    "targetDateField": { "id": "field_target", "name": "Target date", "type": "date" },
    "markerFields": [
      { "id": "field_milestone", "name": "Milestone", "type": "milestone" }
    ],
    "zoom": "quarter"
  }
}`,
    notes: [
      "Only roadmap views can be updated; table and board views return validation_failed for roadmap settings writes.",
      "startFieldId, targetFieldId, and markerFieldIds must reference compatible date, iteration, milestone, or item-date fields from the selected project.",
      "zoom must be month, quarter, or year; duplicate marker ids are deduplicated before persistence.",
      "Successful saves upsert project_roadmap_settings, synchronize project_views.configuration, append audit_events, and return the refreshed workspace.",
    ],
  },
  {
    id: "projects-item-remove",
    method: "DELETE",
    path: "/api/projects/{project_id}/items/{item_id}",
    title: "Remove Project item",
    description:
      "Removes an item from a Projects table without deleting the linked issue, pull request, or draft issue body.",
    auth: "Signed opengithub session cookie with project write/admin access",
    response: `{
  "items": [],
  "total": 0
}`,
    notes: [
      "Removal archives the project item relationship and leaves linked repository content intact.",
      "Archived or missing items return not_found; private projects return not_found or forbidden according to the caller's visibility.",
      "Successful removes write project_item_events, audit_events, and linked timeline evidence when the item points at an issue or pull request.",
    ],
  },
  {
    id: "projects-item-detail",
    method: "GET",
    path: "/api/projects/{project_id}/items/{item_id}",
    title: "Read Project item detail",
    description:
      "Returns the side-panel contract for a draft, linked issue, or linked pull request Project item, including source metadata, draft body, comments, activity, field values, archive state, and viewer actions.",
    auth: "Signed opengithub session cookie when required by project visibility; hidden linked repositories require repository read access",
    response: `{
  "project": { "id": "project_01", "number": 1, "title": "Release tracking" },
  "item": {
    "id": "item_01",
    "itemType": "draft_issue",
    "title": "Write upgrade notes",
    "body": "Project-only planning notes.",
    "href": "/orgs/acme-labs/projects/1/items/item_01",
    "fieldValues": [{ "fieldId": "field_status", "displayValue": "Ready" }]
  },
  "source": null,
  "draft": { "editable": true, "updatedAt": "2026-05-06T10:20:00Z" },
  "comments": [{ "id": "comment_01", "body": "Keep project-only until conversion." }],
  "activity": [{ "id": "event_01", "eventType": "draft_created" }],
  "archive": { "archived": false, "archivedAt": null, "restoredAt": null },
  "viewerPermissions": {
    "canEdit": true,
    "canComment": true,
    "canConvert": true,
    "canArchive": true,
    "canRestore": false,
    "canRemove": true
  }
}`,
    notes: [
      "Private linked issue and pull request details return not_found to project readers who cannot read the backing repository, without leaking the source title.",
      "Draft comments and activity are project-only until conversion and do not create repository notifications.",
      "The browser uses viewerPermissions to render real enabled controls or explicit disabled controls without placeholder actions.",
    ],
  },
  {
    id: "projects-archived-items",
    method: "GET",
    path: "/api/projects/{project_id}/items/archived?itemType=draft_issue&page=1&pageSize=30",
    title: "List archived Project items",
    description:
      "Returns archived Project items for the archive page with filters, pagination, source summaries, archived actor metadata, and restore capability flags.",
    auth: "Signed opengithub session cookie with project read access",
    response: `{
  "items": [
    {
      "item": { "id": "item_01", "itemType": "draft_issue", "title": "Write upgrade notes" },
      "archivedAt": "2026-05-06T10:30:00Z",
      "archivedBy": { "login": "mona" },
      "source": null,
      "viewerPermissions": { "canRestore": true, "canRemove": true }
    }
  ],
  "total": 1,
  "page": 1,
  "pageSize": 30
}`,
    notes: [
      "Archived list filters support draft_issue, issue, and pull_request item types while preserving private linked repository filtering.",
      "Restored items are removed from the archived response and return to the end of the active project ordering.",
      "The archive page uses this endpoint for desktop and mobile smoke coverage of restore controls.",
    ],
  },
  {
    id: "projects-draft-update",
    method: "PATCH",
    path: "/api/projects/{project_id}/items/{item_id}/draft",
    title: "Update Project draft item",
    description:
      "Updates a project-only draft issue title and body from the item side panel without notifying repository subscribers.",
    auth: "Signed opengithub session cookie with project write/admin access",
    request: `{
  "title": "Write upgrade notes",
  "body": "Project-only planning notes.",
  "expectedUpdatedAt": "2026-05-06T10:20:00Z"
}`,
    response: `{
  "item": { "id": "item_01", "title": "Write upgrade notes", "body": "Project-only planning notes." },
  "activity": [{ "eventType": "project.draft.update" }]
}`,
    notes: [
      "Only draft items can be edited through this route; linked issues and pull requests return validation_failed and must be edited through their repository routes.",
      "Archived items reject draft edits, stale expectedUpdatedAt values return conflict, and successful saves write project_item_events plus audit_events.",
      "Mention text in draft bodies is not fanned out to repository notifications before conversion.",
    ],
  },
  {
    id: "projects-item-comments",
    method: "POST",
    path: "/api/projects/{project_id}/items/{item_id}/comments",
    title: "Create Project item comment",
    description:
      "Adds a project-only side-panel comment. Existing comments use PATCH and DELETE on /api/projects/{project_id}/items/{item_id}/comments/{comment_id}.",
    auth: "Signed opengithub session cookie with project write/admin access",
    request: `{
  "body": "Coordinate in the project before opening the issue.",
  "expectedUpdatedAt": "2026-05-06T10:20:00Z"
}`,
    response: `{
  "comments": [
    { "id": "comment_02", "body": "Coordinate in the project before opening the issue." }
  ],
  "activity": [{ "eventType": "project.comment.create" }]
}`,
    notes: [
      "PATCH comment writes preserve project-only scope; DELETE marks the comment deleted without removing linked repository content.",
      "Comment create, update, and delete reject archived items and read-only viewers.",
      "Project item comment mutations write project_item_events and audit_events but do not create repository timeline events.",
    ],
  },
  {
    id: "projects-conversion-targets",
    method: "GET",
    path: "/api/projects/{project_id}/conversion-targets",
    title: "List Project draft conversion targets",
    description:
      "Returns repositories, labels, assignees, and milestones that the signed-in viewer can use when converting a draft Project item to an issue.",
    auth: "Signed opengithub session cookie with project write access and backing repository write access",
    response: `{
  "repositories": [
    {
      "id": "repo_01",
      "fullName": "acme-labs/platform",
      "labels": [{ "id": "label_bug", "name": "bug" }],
      "assignees": [{ "id": "user_01", "login": "mona" }],
      "milestones": [{ "id": "milestone_01", "title": "May" }]
    }
  ]
}`,
    notes: [
      "Private repositories are omitted unless the viewer can write issues in that repository.",
      "The response is intentionally bounded to conversion metadata so the browser does not need a fake repository picker.",
      "Empty repository arrays render a disabled conversion form with explanatory copy.",
    ],
  },
  {
    id: "projects-draft-convert",
    method: "POST",
    path: "/api/projects/{project_id}/items/{item_id}/convert-to-issue",
    title: "Convert Project draft to issue",
    description:
      "Converts a draft Project item into a repository issue, retains project field values, and refreshes the item side panel as a normal linked issue.",
    auth: "Signed opengithub session cookie with project write access and selected repository issue write access",
    request: `{
  "repositoryId": "repo_01",
  "labelIds": ["label_bug"],
  "assigneeUserIds": ["user_01"],
  "milestoneId": "milestone_01",
  "expectedUpdatedAt": "2026-05-06T10:20:00Z"
}`,
    response: `{
  "item": {
    "id": "item_01",
    "itemType": "issue",
    "number": 42,
    "href": "/acme-labs/platform/issues/42"
  },
  "source": { "type": "issue", "number": 42, "repository": { "fullName": "acme-labs/platform" } }
}`,
    notes: [
      "Conversion validates repository, label, assignee, and milestone ownership before allocating an issue number.",
      "Duplicate submits are idempotent for already-converted items and return the linked issue state instead of creating another issue.",
      "Successful conversion writes issue timeline events, project_item_events, audit_events, and assignee notifications.",
    ],
  },
  {
    id: "projects-item-archive",
    method: "PATCH",
    path: "/api/projects/{project_id}/items/{item_id}/archive",
    title: "Archive Project item",
    description:
      "Archives an active Project item so it disappears from table, board, and roadmap views while preserving history and linked repository content.",
    auth: "Signed opengithub session cookie with project write/admin access",
    response: `{
  "item": { "id": "item_01", "title": "Write upgrade notes" },
  "archive": {
    "archived": true,
    "archivedAt": "2026-05-06T10:30:00Z",
    "archivedBy": { "login": "mona" }
  }
}`,
    notes: [
      "Archiving a linked issue or pull request never closes, deletes, or mutates the backing repository resource.",
      "Archived items reject active-view field edits, draft edits, comments, conversion, and second archive requests.",
      "Successful archive writes project_item_events and audit_events and the active workspace read no longer returns the item.",
    ],
  },
  {
    id: "projects-item-restore",
    method: "PATCH",
    path: "/api/projects/{project_id}/items/{item_id}/restore",
    title: "Restore Project item",
    description:
      "Restores an archived Project item to the active workspace ordering and records restored actor metadata.",
    auth: "Signed opengithub session cookie with project write/admin access",
    response: `{
  "item": { "id": "item_01", "title": "Write upgrade notes" },
  "archive": {
    "archived": false,
    "restoredAt": "2026-05-06T10:35:00Z",
    "restoredBy": { "login": "mona" }
  }
}`,
    notes: [
      "Restore places the item at the end of active manual ordering so table, board, and roadmap views can show it consistently.",
      "Restoring preserves archived actor history while clearing stale removed-state metadata.",
      "Successful restore writes project_item_events and audit_events and removes the row from archived list responses.",
    ],
  },
  {
    id: "projects-field-settings-read",
    method: "GET",
    path: "/api/projects/{project_id}/settings/fields",
    title: "Read Project field settings",
    description:
      "Returns the Projects v2 field administration contract used by the settings Fields page, including built-in fields, custom fields, single-select options, iteration cycles, breaks, field limits, usage counts, and viewer capabilities.",
    auth: "Signed opengithub session cookie when required by project visibility; private projects require project read access",
    response: `{
  "project": { "id": "project_01", "number": 1, "title": "Release tracking" },
  "fields": [
    { "id": "field_title", "name": "Title", "fieldType": "title", "builtIn": true, "editable": false },
    {
      "id": "field_status",
      "name": "Status",
      "fieldType": "single_select",
      "builtIn": false,
      "options": [{ "id": "option_ready", "name": "Ready", "color": "green", "position": 1 }]
    },
    {
      "id": "field_cycle",
      "name": "Cycle",
      "fieldType": "iteration",
      "iterations": [{ "id": "iteration_01", "name": "Sprint 1", "startDate": "2026-05-04", "durationDays": 14 }],
      "breaks": [{ "id": "break_01", "name": "Planning break", "startDate": "2026-05-18", "durationDays": 2 }]
    }
  ],
  "limits": { "maxFields": 50, "remainingFields": 12, "maxOptionsPerField": 50, "maxIterationsPerField": 100 },
  "viewerPermissions": { "canCreateFields": true, "canManageOptions": true, "canManageIterations": true }
}`,
    notes: [
      "Built-in fields are returned with editable=false and deletable=false so the browser can render honest disabled controls.",
      "Private linked issue and pull request counts are not exposed to viewers who can read the project but cannot read the backing repository.",
      "Field settings reads use the standard not_found, forbidden, and validation_failed envelopes without session secrets or SQL details.",
      "User and organization project settings pages resolve project numbers before calling this project-id endpoint.",
    ],
  },
  {
    id: "projects-field-lifecycle",
    method: "POST",
    path: "/api/projects/{project_id}/settings/fields",
    title: "Create Project field",
    description:
      "Creates a custom date, text, number, single-select, or iteration field from the Projects settings page.",
    auth: "Signed opengithub session cookie with project write/admin access",
    request: `{
  "name": "Target date",
  "fieldType": "date"
}`,
    response: `{
  "fields": [
    { "id": "field_target", "name": "Target date", "fieldType": "date", "builtIn": false }
  ],
  "limits": { "remainingFields": 11 }
}`,
    notes: [
      "Supported fieldType values are single_select, iteration, date, text, and number; invalid or blank names return validation_failed.",
      "Field names are normalized for uniqueness within the project, so case-only and whitespace-only duplicates return conflict.",
      "Creating an iteration field seeds three default cycles using the iteration settings defaults.",
      "Successful creates append audit_events, increment the project field cache version, and invalidate project view/filter caches.",
    ],
  },
  {
    id: "projects-field-update-delete",
    method: "PATCH",
    path: "/api/projects/{project_id}/fields/{field_id}",
    title: "Rename Project field",
    description:
      "Renames a custom Projects field with stale-update protection. The same resource supports DELETE for confirmed field deletion.",
    auth: "Signed opengithub session cookie with project write/admin access",
    request: `{
  "name": "Launch target",
  "expectedUpdatedAt": "2026-05-06T10:10:00Z"
}`,
    response: `{
  "fields": [
    { "id": "field_target", "name": "Launch target", "cacheVersion": 3 }
  ]
}`,
    notes: [
      "Built-in fields cannot be renamed or deleted; custom field writes require expectedUpdatedAt when stale protection is available.",
      "DELETE /api/projects/{project_id}/fields/{field_id} removes the custom field and its project_item_field_values but never deletes linked issues or pull requests.",
      "Deletes write project.field.delete audit events, project item events for cleaned values, and view/filter cache invalidation evidence.",
      "Stale timestamps return conflict so the browser can ask users to refresh before overwriting newer field settings.",
    ],
  },
  {
    id: "projects-field-options",
    method: "POST",
    path: "/api/projects/{project_id}/fields/{field_id}/options",
    title: "Create Project field option",
    description:
      "Adds a single-select option. Existing options can be edited, reordered, or deleted through the sibling option routes.",
    auth: "Signed opengithub session cookie with project write/admin access",
    request: `{
  "name": "Ready",
  "color": "green",
  "description": "Prepared for review"
}`,
    response: `{
  "fields": [
    {
      "id": "field_status",
      "options": [
        { "id": "option_ready", "name": "Ready", "color": "green", "position": 1 }
      ]
    }
  ]
}`,
    notes: [
      "Only single-select and status-compatible fields accept options; date, text, number, and iteration fields return validation_failed.",
      "PATCH /api/projects/{project_id}/fields/{field_id}/options/{option_id} renames, recolors, and updates the description while syncing matching project_item_field_values and board column settings.",
      "PATCH /api/projects/{project_id}/fields/{field_id}/options/reorder accepts an ordered optionIds array and persists positions without drag-only assumptions.",
      "DELETE /api/projects/{project_id}/fields/{field_id}/options/{option_id} removes matching item field values and board columns without deleting linked issues or pull requests.",
      "Option writes validate color tokens, duplicate names, max-option limits, permissions, audit_events, and project view/filter cache invalidation.",
    ],
  },
  {
    id: "projects-field-options-reorder",
    method: "PATCH",
    path: "/api/projects/{project_id}/fields/{field_id}/options/reorder",
    title: "Reorder Project field options",
    description:
      "Persists a full ordered list of single-select option ids after keyboard-safe Up and Down controls reorder options in field settings.",
    auth: "Signed opengithub session cookie with project write/admin access",
    request: `{
  "optionIds": ["option_ready", "option_review", "option_done"]
}`,
    response: `{
  "fields": [
    {
      "id": "field_status",
      "options": [
        { "id": "option_ready", "position": 1 },
        { "id": "option_review", "position": 2 },
        { "id": "option_done", "position": 3 }
      ]
    }
  ]
}`,
    notes: [
      "The submitted optionIds array must contain exactly the active options for that field; missing, duplicate, or foreign ids return validation_failed.",
      "Reordering touches the field cache version, writes project.field_option.reorder audit evidence, and invalidates project view/filter caches.",
      "This endpoint deliberately supports button-driven reorder controls; full pointer drag-and-drop is not required for projects-004.",
    ],
  },
  {
    id: "projects-field-iterations",
    method: "PATCH",
    path: "/api/projects/{project_id}/fields/{field_id}/iterations/settings",
    title: "Update Project iteration settings",
    description:
      "Regenerates iteration cycles for an iteration field, and pairs with iteration and break endpoints for manual schedule administration.",
    auth: "Signed opengithub session cookie with project write/admin access",
    request: `{
  "startDate": "2026-05-04",
  "duration": 2,
  "durationUnit": "weeks",
  "generatedIterations": 6,
  "expectedUpdatedAt": "2026-05-06T10:20:00Z"
}`,
    response: `{
  "fields": [
    {
      "id": "field_cycle",
      "iterations": [
        { "id": "iteration_01", "name": "Iteration 1", "startDate": "2026-05-04", "durationDays": 14 }
      ],
      "breaks": []
    }
  ]
}`,
    notes: [
      "Iteration settings writes are accepted only for iteration fields; duration can be expressed in days or weeks and generatedIterations is bounded by the field limit.",
      "POST /api/projects/{project_id}/fields/{field_id}/iterations appends a future cycle; PATCH /api/projects/{project_id}/fields/{field_id}/iterations/{iteration_id} edits a single cycle name, start date, and duration.",
      "POST /api/projects/{project_id}/fields/{field_id}/iteration-breaks inserts a non-overlapping break; DELETE /api/projects/{project_id}/fields/{field_id}/iteration-breaks/{break_id} removes it.",
      "Iteration ranges and breaks reject overlaps, stale schedule saves return conflict, and successful writes append audit_events plus view/filter cache invalidation.",
      "Workspace filters understand iteration values with @current, @previous, @next, comparison operators, and date ranges.",
    ],
  },
  {
    id: "projects-field-iteration-break-create",
    method: "POST",
    path: "/api/projects/{project_id}/fields/{field_id}/iteration-breaks",
    title: "Create Project iteration break",
    description:
      "Inserts a non-overlapping break into an iteration field schedule from the field settings page.",
    auth: "Signed opengithub session cookie with project write/admin access",
    request: `{
  "name": "Planning break",
  "startDate": "2026-06-15",
  "durationDays": 2
}`,
    response: `{
  "fields": [
    {
      "id": "field_cycle",
      "breaks": [
        { "id": "break_01", "name": "Planning break", "startDate": "2026-06-15", "durationDays": 2 }
      ]
    }
  ]
}`,
    notes: [
      "Breaks are valid only for iteration fields, must have positive duration, and cannot overlap existing iterations or breaks.",
      "DELETE /api/projects/{project_id}/fields/{field_id}/iteration-breaks/{break_id} removes a break without changing linked issues or pull requests.",
      "Break create and delete writes audit_events, touches the field cache version, and invalidates project view/filter caches.",
    ],
  },
  {
    id: "projects-settings-read",
    method: "GET",
    path: "/api/projects/{project_id}/settings",
    title: "Read Project settings",
    description:
      "Returns the Project settings contract used by General, Access, Templates, and Danger Zone pages, including metadata, README, policy locks, linked repositories, access grants, status updates, template state, and lifecycle capabilities.",
    auth: "Signed opengithub session cookie when required by project visibility; private projects require project read access",
    response: `{
  "project": { "id": "project_01", "number": 12, "title": "Release tracking" },
  "general": {
    "title": "Release tracking",
    "shortDescription": "Plan the next train",
    "readmeMarkdown": "## Release process",
    "visibility": "private",
    "defaultRepositoryId": "repo_01",
    "updatedAt": "2026-05-06T10:20:00Z"
  },
  "policy": {
    "projectsEnabled": true,
    "visibilityChangesAllowed": false,
    "basePermission": "write"
  },
  "repositories": [
    { "id": "repo_01", "fullName": "acme-labs/platform", "default": true, "viewerCanWrite": true }
  ],
  "accessGrants": [
    { "id": "grant_01", "subjectType": "user", "login": "mona", "role": "admin", "inherited": false }
  ],
  "teamGrants": [
    { "id": "grant_02", "team": { "slug": "platform" }, "role": "write", "inherited": false }
  ],
  "statusUpdates": [
    { "id": "status_01", "state": "on_track", "messageMarkdown": "Ready for review" }
  ],
  "template": { "isTemplate": true, "copySourceProjectId": null },
  "dangerState": { "closed": false, "deleted": false },
  "viewerPermissions": { "canUpdateSettings": true, "canManageAccess": true, "canDeleteProject": true }
}`,
    notes: [
      "GET /api/projects/{project_id}/settings/access returns the same contract for Access pages so the browser can share permission, grant, and policy state.",
      "User and organization settings routes resolve project numbers before calling this project-id endpoint.",
      "Linked repositories are filtered by repository visibility, and policy locks are explicit instead of relying on browser-only disabled controls.",
      "Deleted projects return privacy-preserving not_found responses through normal settings reads.",
    ],
  },
  {
    id: "projects-settings-update",
    method: "PATCH",
    path: "/api/projects/{project_id}/settings",
    title: "Update Project settings",
    description:
      "Persists General settings edits for title, short description, README Markdown, visibility, and default repository.",
    auth: "Signed opengithub session cookie with project admin access",
    request: `{
  "title": "Release train",
  "shortDescription": "Plan the next train",
  "readmeMarkdown": "## Release process",
  "visibility": "private",
  "defaultRepositoryId": "repo_01",
  "expectedUpdatedAt": "2026-05-06T10:20:00Z"
}`,
    response: `{
  "general": {
    "title": "Release train",
    "visibility": "private",
    "defaultRepositoryId": "repo_01",
    "updatedAt": "2026-05-06T10:25:00Z"
  }
}`,
    notes: [
      "Organization policy can deny visibility changes while still allowing title, README, and default repository edits.",
      "Default repositories must be linked to the project and writable by the actor so new issue routing cannot target inaccessible repositories.",
      "README edits create project_readme revision rows; stale expectedUpdatedAt values return conflict.",
      "Successful updates write project.settings.update audit events and return a refreshed settings contract.",
    ],
  },
  {
    id: "projects-settings-access-read",
    method: "GET",
    path: "/api/projects/{project_id}/settings/access",
    title: "Read Project access settings",
    description:
      "Returns the shared settings contract scoped for the Access page, including direct grants, team grants, eligible teams, base permission inheritance, and viewer access capabilities.",
    auth: "Signed opengithub session cookie when required by project visibility; private projects require project read access",
    response: `{
  "accessGrants": [
    { "id": "grant_01", "subjectType": "user", "login": "mona", "role": "admin" }
  ],
  "teamGrants": [
    { "id": "grant_02", "team": { "slug": "platform" }, "role": "write" }
  ],
  "eligibleTeams": [
    { "id": "team_01", "slug": "platform", "name": "Platform" }
  ],
  "policy": { "basePermission": "write" },
  "viewerPermissions": { "canManageAccess": true }
}`,
    notes: [
      "This endpoint shares the same privacy, deleted-project, and no-secret behavior as GET /api/projects/{project_id}/settings.",
      "Inherited organization base permissions and team grants are explicit so the browser can distinguish removable direct grants from policy-derived access.",
    ],
  },
  {
    id: "projects-settings-repositories",
    method: "POST",
    path: "/api/projects/{project_id}/repositories/{repository_id}",
    title: "Link Project repository",
    description:
      "Links a repository to a Project so repository Projects tabs and default issue routing can point back to the Project. The same route supports DELETE to remove a link.",
    auth: "Signed opengithub session cookie with project admin access and repository write access",
    request: `{
  "expectedUpdatedAt": "2026-05-06T10:20:00Z"
}`,
    response: `{
  "repositories": [
    { "id": "repo_01", "fullName": "acme-labs/platform", "default": true }
  ]
}`,
    notes: [
      "Duplicate links return conflict, foreign-owner or inaccessible repositories return forbidden, and removing the default repository clears defaultRepositoryId.",
      "DELETE /api/projects/{project_id}/repositories/{repository_id} removes only the project link; it never deletes the repository.",
      "Repository link changes write project.repository.link or project.repository.unlink audit events.",
    ],
  },
  {
    id: "projects-settings-status-update",
    method: "POST",
    path: "/api/projects/{project_id}/status-updates",
    title: "Publish Project status update",
    description:
      "Publishes an on-track, at-risk, off-track, or complete status update for a Project settings status panel.",
    auth: "Signed opengithub session cookie with project write/admin access",
    request: `{
  "state": "at_risk",
  "messageMarkdown": "Waiting on security review",
  "startDate": "2026-05-01",
  "targetDate": "2026-05-31",
  "expectedUpdatedAt": "2026-05-06T10:20:00Z"
}`,
    response: `{
  "statusUpdates": [
    { "id": "status_02", "state": "at_risk", "messageMarkdown": "Waiting on security review" }
  ]
}`,
    notes: [
      "State must be on_track, at_risk, off_track, or complete; blank messages and inverted date ranges return validation_failed.",
      "Closed or deleted projects reject new status updates.",
      "Successful publishes append project_status_updates and project.status_update.create audit events.",
    ],
  },
  {
    id: "projects-settings-template",
    method: "PATCH",
    path: "/api/projects/{project_id}/template",
    title: "Update Project template state",
    description:
      "Toggles whether a Project can be copied as a template and records optional copy-source metadata.",
    auth: "Signed opengithub session cookie with project admin access",
    request: `{
  "isTemplate": true,
  "expectedUpdatedAt": "2026-05-06T10:20:00Z"
}`,
    response: `{
  "template": { "isTemplate": true, "copySourceProjectId": null }
}`,
    notes: [
      "Template toggles are disabled for closed or deleted projects and honor stale expectedUpdatedAt conflicts.",
      "Successful writes update project_templates and write project.template.update audit events.",
    ],
  },
  {
    id: "projects-settings-access-grants",
    method: "POST",
    path: "/api/projects/{project_id}/access-grants",
    title: "Create Project access grant",
    description:
      "Adds a direct user or team grant to a Project Access settings page. Existing grants can be changed or removed through the grant-id route.",
    auth: "Signed opengithub session cookie with project admin access",
    request: `{
  "subjectType": "team",
  "subjectId": "team_01",
  "role": "write",
  "expectedUpdatedAt": "2026-05-06T10:20:00Z"
}`,
    response: `{
  "teamGrants": [
    { "id": "grant_02", "team": { "slug": "platform" }, "role": "write" }
  ]
}`,
    notes: [
      "Roles are read, write, or admin; users must belong to the owning organization and teams must be owned by the same organization.",
      "PATCH /api/projects/{project_id}/access-grants/{grant_id} changes role with stale-update protection.",
      "DELETE /api/projects/{project_id}/access-grants/{grant_id} removes the grant but protects the last admin.",
      "Access mutations write project.access_grant.create, update, or delete audit events.",
    ],
  },
  {
    id: "projects-settings-access-grant-update",
    method: "PATCH",
    path: "/api/projects/{project_id}/access-grants/{grant_id}",
    title: "Update Project access grant",
    description:
      "Changes a direct user or team grant role. The same grant-id resource supports DELETE to remove a grant.",
    auth: "Signed opengithub session cookie with project admin access",
    request: `{
  "role": "admin",
  "expectedUpdatedAt": "2026-05-06T10:20:00Z"
}`,
    response: `{
  "accessGrants": [
    { "id": "grant_01", "subjectType": "user", "login": "mona", "role": "admin" }
  ]
}`,
    notes: [
      "Role updates require read, write, or admin and reject stale expectedUpdatedAt values.",
      "DELETE /api/projects/{project_id}/access-grants/{grant_id} removes a direct grant but protects the last admin.",
      "Inherited organization base permissions cannot be removed through direct grant deletion.",
    ],
  },
  {
    id: "projects-settings-close",
    method: "POST",
    path: "/api/projects/{project_id}/close",
    title: "Close Project",
    description:
      "Closes an active Project after admin confirmation, leaving it readable while mutation controls become disabled.",
    auth: "Signed opengithub session cookie with project admin access",
    request: `{
  "expectedUpdatedAt": "2026-05-06T10:20:00Z"
}`,
    response: `{
  "dangerState": {
    "closed": true,
    "closedAt": "2026-05-06T10:30:00Z",
    "deleted": false
  },
  "viewerPermissions": { "canUpdateSettings": false, "canManageAccess": false }
}`,
    notes: [
      "POST /api/projects/{project_id}/reopen reverses a closed project when the actor still has admin access.",
      "Closed projects remain readable from settings and workspace reads but reject unrelated field, access, and item mutations.",
      "Close and reopen writes project.close and project.reopen audit events.",
    ],
  },
  {
    id: "projects-settings-reopen",
    method: "POST",
    path: "/api/projects/{project_id}/reopen",
    title: "Reopen Project",
    description:
      "Reopens a closed Project when the actor still has project admin access.",
    auth: "Signed opengithub session cookie with project admin access",
    request: `{
  "expectedUpdatedAt": "2026-05-06T10:30:00Z"
}`,
    response: `{
  "dangerState": {
    "closed": false,
    "closedAt": null,
    "deleted": false
  },
  "viewerPermissions": { "canUpdateSettings": true, "canManageAccess": true }
}`,
    notes: [
      "Reopen rejects deleted projects and stale expectedUpdatedAt values.",
      "Successful reopens write project.reopen audit events and return refreshed mutation capabilities.",
    ],
  },
  {
    id: "projects-settings-delete",
    method: "DELETE",
    path: "/api/projects/{project_id}",
    title: "Delete Project",
    description:
      "Soft-deletes a Project after typed title confirmation and returns the owner or organization Projects list destination.",
    auth: "Signed opengithub session cookie with project admin access",
    request: `{
  "confirmation": "Release tracking",
  "expectedUpdatedAt": "2026-05-06T10:20:00Z"
}`,
    response: `{
  "deleted": true,
  "redirectHref": "/orgs/acme-labs/projects"
}`,
    notes: [
      "Confirmation must match the current title; casing or whitespace mistakes return validation_failed.",
      "Deleted projects disappear from normal list, workspace, and settings reads without leaking private metadata.",
      "Soft delete preserves audit, item, workflow, and archive rows for retention while hiding the Project from product surfaces.",
      "Successful deletes write project.delete audit events.",
    ],
  },
  {
    id: "projects-workflow-settings-read",
    method: "GET",
    path: "/api/projects/{project_id}/workflows",
    title: "Read Project workflow settings",
    description:
      "Returns the built-in Projects workflow settings contract used by the Workflows settings page, including default workflow cards, eligible status fields, repository targets, recent automation logs, and viewer capabilities.",
    auth: "Signed opengithub session cookie when required by project visibility; private projects require project read access",
    response: `{
  "project": { "id": "project_01", "number": 1, "title": "Release tracking" },
  "automationActor": "@opengithub-project-automation",
  "workflows": [
    {
      "id": "workflow_01",
      "workflowKey": "closed_item_to_done",
      "name": "Set status to Done when closed",
      "enabled": true,
      "triggerEvent": "issue_closed",
      "configuration": {
        "target": { "fieldId": "field_status", "optionId": "option_done" },
        "condition": "state:closed"
      },
      "repositoryTargetIds": ["repo_01"],
      "lastRunStatus": "success"
    }
  ],
  "eligibleFields": [
    {
      "id": "field_status",
      "name": "Status",
      "supportsStatusTarget": true,
      "supportsArchiveCriteria": true,
      "options": [{ "id": "option_done", "name": "Done", "color": "green" }]
    }
  ],
  "repositoryTargets": [
    { "id": "repo_01", "fullName": "acme-labs/platform", "permission": "write" }
  ],
  "recentLogs": [
    { "workflowKey": "closed_item_to_done", "source": "system", "status": "success" }
  ],
  "viewerPermissions": { "canManageWorkflows": true, "canViewLogs": true }
}`,
    notes: [
      "Default workflow rows are seeded on first read for closed issue/PR to Done, merged PR to Done, item-added default status, and auto-archive completed items.",
      "Repository targets are filtered by repository permission, so private repositories are omitted from readers who can inspect the project but cannot access that repository.",
      "Missing Status or Done fields are represented through eligibleFields and unavailable workflow configuration rather than browser-only assumptions.",
      "User and organization workflow settings pages resolve project numbers before calling this project-id endpoint.",
    ],
  },
  {
    id: "projects-workflow-update",
    method: "PATCH",
    path: "/api/projects/{project_id}/workflows/{workflow_id}",
    title: "Update Project workflow",
    description:
      "Persists built-in Project workflow enablement, condition filters, target status selection, repository auto-add targets, archive criteria, and close-on-status behavior.",
    auth: "Signed opengithub session cookie with project write/admin access; selected repository targets require repository write access",
    request: `{
  "enabled": true,
  "condition": "state:closed label:ready",
  "statusFieldId": "field_status",
  "statusOptionId": "option_done",
  "repositoryTargetIds": ["repo_01"],
  "archiveAfterDays": 14,
  "closeOnStatus": false,
  "expectedUpdatedAt": "2026-05-06T10:40:00Z"
}`,
    response: `{
  "workflows": [
    {
      "id": "workflow_01",
      "enabled": true,
      "source": "ui",
      "lastRunMessage": "Workflow configuration saved."
    }
  ],
  "recentLogs": [
    { "workflowKey": "closed_item_to_done", "source": "ui", "status": "success" }
  ]
}`,
    notes: [
      "expectedUpdatedAt protects concurrent workflow edits; stale saves return conflict without overwriting newer configuration.",
      "Status field and option ids must belong to the same project and support status targeting; invalid ids return validation_failed.",
      "Repository target ids must be linked to the project and writable by the actor because later auto-add and close-on-status side effects touch repository resources.",
      "Successful saves write workflow_execution_logs and audit_events before later item-state automation runs.",
    ],
  },
  {
    id: "projects-automation-invocation",
    method: "POST",
    path: "/api/projects/{project_id}/automation/invocations",
    title: "Invoke Project automation",
    description:
      "Applies a bounded Actions or GraphQL-style Project automation invocation through the same workflow engine used by built-in item-state events.",
    auth: "Signed opengithub session cookie or Bearer personal access token with project write access",
    request: `{
  "source": "actions",
  "itemId": "item_01",
  "workflowKey": "closed_item_to_done",
  "actionsWorkflowRunId": "run_01",
  "idempotencyKey": "run_01:item_01:status",
  "fieldUpdates": [
    { "fieldId": "field_status", "value": { "kind": "single_select", "optionId": "option_done" } }
  ]
}`,
    response: `{
  "projectId": "project_01",
  "itemId": "item_01",
  "workflowKey": "closed_item_to_done",
  "source": "actions",
  "status": "success",
  "message": "Applied 1 project automation update.",
  "appliedUpdates": [{ "fieldId": "field_status", "displayValue": "Done" }],
  "idempotencyKey": "run_01:item_01:status"
}`,
    notes: [
      "Supported source values are actions and graphql; ui and system are reserved for first-party workflow settings and built-in event execution.",
      "Duplicate idempotency keys return a skipped response and do not write a second success log or duplicate field value event.",
      "Actions run attribution requires write access to the run's repository, and linked issue or pull request updates require backing repository write access.",
      "Successful invocations update project_item_field_values, project_item_events, workflow_execution_logs, project_workflows last-run metadata, and audit_events.",
    ],
  },
  {
    id: "projects-insights-read",
    method: "GET",
    path: "/api/projects/{project_id}/insights?chart=burn-up&range=1m&filter=is%3Aopen&table=true",
    title: "Read Project Insights charts",
    description:
      "Returns the Projects v2 Insights contract for default Burn up charts, custom chart navigation, filter/range exploration, accessible chart data rows, latest status, share metadata, and cache snapshot evidence.",
    auth: "Signed opengithub session cookie when required by project visibility; private projects require project read access",
    response: `{
  "project": { "id": "project_01", "number": 12, "title": "Roadmap" },
  "navigation": {
    "returnHref": "/orgs/acme/projects/12/views/1",
    "insightsHref": "/orgs/acme/projects/12/insights",
    "selectedItem": "insights"
  },
  "selectedChart": {
    "id": "burn-up",
    "title": "Burn up",
    "chartType": "burn_up",
    "shareHref": "/orgs/acme/projects/12/insights?chart=burn-up&range=1m",
    "sharedWithViewers": true,
    "configuration": { "table": true }
  },
  "matchingItemCount": 24,
  "series": [{ "id": "completed", "name": "Completed", "points": [] }],
  "dataRows": [{ "itemId": "item_01", "title": "Ship chart sharing" }],
  "cache": { "cacheKey": "project_01:burn-up:1m", "version": 3, "stale": false }
}`,
    notes: [
      "Supported range values are 2w, 1m, 3m, max, and custom; custom requires start and end dates and rejects excessive windows.",
      "Supported filter tokens are bounded Projects item qualifiers such as is:open, is:closed, type:issue, type:pull_request, and type:draft_issue; unsupported tokens return invalid_filter.",
      "table=true returns the same permission-filtered chart data as accessible rows so browser users can switch from the chart region to a data table.",
      "Private linked repository items are omitted when the project viewer cannot read the backing repository.",
      "Cache metadata comes from project_chart_series_cache snapshots and is refreshed when Insights recomputes a chart response.",
      "Readers can view shared project charts and status context but cannot mutate private charts or create/edit/delete chart configuration.",
    ],
  },
  {
    id: "projects-insights-chart-create",
    method: "POST",
    path: "/api/projects/{project_id}/charts",
    title: "Create Project Insights chart",
    description:
      "Creates a custom Project Insights chart visible either to the creator or to all project viewers.",
    auth: "Signed opengithub session cookie with project write/admin access",
    request: `{
  "title": "Open bugs by team",
  "description": "Tracks unresolved bugs by team.",
  "chartType": "bar",
  "filter": "is:open type:issue",
  "xFieldId": "field_status",
  "yFieldId": "",
  "groupFieldId": "field_team",
  "visibility": "project"
}`,
    response: `{
  "selectedChart": {
    "id": "chart_01",
    "title": "Open bugs by team",
    "visibility": "project",
    "shareHref": "/orgs/acme/projects/12/insights?chart=chart_01&range=1m"
  },
  "customCharts": [{ "id": "chart_01", "title": "Open bugs by team" }]
}`,
    notes: [
      "Chart title, type, filter, field ids, and visibility are validated before insert; duplicate titles return conflict.",
      "visibility=project generates a stable share slug and allows project viewers to open the shared chart through the Insights read endpoint.",
      "Successful creates write project_chart_revisions, invalidate or refresh project_chart_series_cache, and append audit_events.",
    ],
  },
  {
    id: "projects-insights-chart-update",
    method: "PATCH",
    path: "/api/projects/{project_id}/charts/{chart_id}",
    title: "Update Project Insights chart",
    description:
      "Updates a custom Project Insights chart configuration and returns the refreshed Insights contract.",
    auth: "Signed opengithub session cookie with project write/admin access",
    request: `{
  "title": "Closed issue trend",
  "description": "Tracks completed work.",
  "chartType": "line",
  "filter": "is:closed",
  "visibility": "project",
  "expectedUpdatedAt": "2026-05-06T10:40:00Z"
}`,
    response: `{
  "selectedChart": {
    "id": "chart_01",
    "title": "Closed issue trend",
    "chartType": "line",
    "sharedWithViewers": true
  }
}`,
    notes: [
      "expectedUpdatedAt protects concurrent chart edits; stale saves return conflict without overwriting newer chart configuration.",
      "Default charts such as burn-up are not mutable through this endpoint.",
      "Successful updates write a new project_chart_revisions row, refresh cache snapshot metadata, and append audit_events.",
    ],
  },
  {
    id: "projects-insights-chart-delete",
    method: "DELETE",
    path: "/api/projects/{project_id}/charts/{chart_id}",
    title: "Delete Project Insights chart",
    description:
      "Deletes a custom Project Insights chart and returns the default Insights chart response.",
    auth: "Signed opengithub session cookie with project write/admin access",
    request: `{
  "expectedUpdatedAt": "2026-05-06T10:40:00Z"
}`,
    response: `{
  "selectedChart": { "id": "burn-up", "title": "Burn up" },
  "customCharts": []
}`,
    notes: [
      "Default charts cannot be deleted; private chart ids that the actor cannot edit return not_found or forbidden without leaking chart configuration.",
      "Deletes remove the chart row and cascade revisions/cache rows while preserving project audit history.",
      "The refreshed response keeps the user in the same project/range/filter context where possible.",
    ],
  },
  {
    id: "organization-team-create",
    method: "POST",
    path: "/api/orgs/{org}/teams",
    title: "Create organization team",
    description:
      "Creates a visible or secret organization team from the New team form after policy, slug, parent, nesting, and notification validation.",
    auth: "Signed opengithub session cookie with organization owner/admin role, or member role when team creation policy allows it",
    request: `{
  "name": "Release Infrastructure",
  "description": "Owns release trains.",
  "parentTeamId": "team_platform",
  "visibility": "visible",
  "notificationsEnabled": false
}`,
    response: `{
  "team": {
    "slug": "release-infrastructure",
    "name": "Release Infrastructure",
    "visibility": "visible",
    "notificationsEnabled": false,
    "parent": { "slug": "platform", "name": "Platform" }
  },
  "href": "/orgs/acme-labs/teams/release-infrastructure"
}`,
    notes: [
      "Team names are slugified with the same Rust normalizer used by the browser preview; duplicate slugs return 409 conflict.",
      "Secret teams cannot be nested under any parent, visible child teams cannot use a secret parent, and parent cycles are rejected with validation_failed.",
      "The notificationsEnabled flag controls team-mention fanout while keeping mention indexing available for allowed visible teams.",
      "Successful creates write redacted organization.team.create audit events and never copy submitted descriptions into sensitive logs.",
    ],
  },
  {
    id: "organization-team-detail",
    method: "GET",
    path: "/api/orgs/{org}/teams/{team_slug}",
    title: "Read organization team detail",
    description:
      "Returns one team detail surface with member rows, child teams, direct and inherited repository grants, hierarchy, mentionability, and notification state.",
    auth: "Signed opengithub session cookie with organization membership and visibility to the requested team",
    response: `{
  "team": { "slug": "frontend", "visibility": "visible" },
  "hierarchy": {
    "parentChain": [{ "slug": "platform", "name": "Platform" }],
    "inheritedRepositoryCount": 1,
    "directRepositoryCount": 1,
    "childTeamCount": 0
  },
  "members": [{ "login": "mona", "role": "maintainer" }],
  "repositories": [
    {
      "fullName": "acme-labs/runtime",
      "role": "write",
      "sourceTeamSlug": "platform",
      "inherited": true
    }
  ],
  "mentionState": {
    "mentionable": true,
    "notificationsEnabled": false,
    "fanoutState": "team mentions stay indexed, but member fanout is suppressed"
  }
}`,
    notes: [
      "Secret team detail returns not_found to non-members unless the viewer is an organization owner/admin.",
      "Parent team repository permissions cascade to children for repository authorization and review-request lookups.",
      "Repository rows identify direct versus inherited team grants so settings pages remain the source of mutation.",
      "Notification fanout de-dupes team mention subscribers with direct mentions, participation, and review-request recipients.",
    ],
  },
  {
    id: "appearance-settings",
    method: "PATCH",
    path: "/api/user/settings/appearance",
    title: "Read or update appearance settings",
    description:
      "Persists site-wide Editorial theme and text-scale preferences for the signed-in user.",
    auth: "Signed opengithub session cookie",
    request: `{
  "theme": "dark_dimmed",
  "fontSize": "large"
}`,
    response: `{
  "userId": "user_01",
  "theme": "dark_dimmed",
  "fontSize": "large",
  "updatedAt": "2026-05-07T00:00:00Z"
}`,
    notes: [
      "Supported themes are light, dark, system, dark_dimmed, and dark_high_contrast.",
      "The Next.js appearance action mirrors saved values into color_mode and font_size cookies for first paint.",
      "The root layout exposes data-color-mode, data-light-theme, data-dark-theme, and data-font-size on html while keeping the locked Editorial token system.",
    ],
  },
  {
    id: "personal-access-tokens-list",
    method: "GET",
    path: "/api/settings/tokens",
    title: "List personal access tokens",
    description:
      "Returns the signed-in user's personal access tokens with prefix-only metadata for Developer Settings.",
    auth: "Signed opengithub session cookie",
    response: `{
  "sudo": {
    "active": true,
    "expiresAt": "2026-05-04T12:30:00Z",
    "requiredFor": ["create_personal_access_token", "revoke_personal_access_token"]
  },
  "tokens": [
    {
      "id": "token_01",
      "name": "Deploy token",
      "type": "fine_grained",
      "prefix": "oghp_12345678",
      "scopes": ["repo:read", "packages:write"],
      "repositoryAccess": "selected",
      "selectedRepositories": [
        { "fullName": "mona/octo-app", "visibility": "private" }
      ],
      "status": "active",
      "lastUsedAt": null,
      "expiresAt": "2026-06-04T00:00:00Z"
    }
  ]
}`,
    notes: [
      "The response never includes token_hash or plaintext token material.",
      "Revoked and expired tokens are included with status metadata so users can audit stale automation credentials.",
      "Successful REST, Git, and package-registry token use updates lastUsedAt after scope and repository checks pass.",
    ],
  },
  {
    id: "personal-access-token-context",
    method: "GET",
    path: "/api/settings/tokens/new",
    title: "Read token creation context",
    description:
      "Returns resource owners, visible repositories, permission choices, expiration bounds, and sudo state for the new-token form.",
    auth: "Signed opengithub session cookie",
    response: `{
  "sudo": { "active": false, "expiresAt": null },
  "resourceOwners": [
    { "kind": "user", "login": "mona", "displayName": "Mona Lisa" },
    { "kind": "organization", "login": "namuh", "displayName": "Namuh" }
  ],
  "repositories": [
    { "id": "repo_01", "fullName": "mona/octo-app", "visibility": "private" }
  ],
  "permissionGroups": [
    {
      "key": "repositories",
      "permissions": [
        { "key": "contents", "levels": ["none", "read", "write"] }
      ]
    }
  ],
  "defaultExpirationDays": 30,
  "maxExpirationDays": 366
}`,
    notes: [
      "Only repositories visible to the current user are returned.",
      "Organization owners require owner/admin membership before they appear as token resource owners.",
      "Query parameters on the browser page prefill the form, but the Rust API validates the submitted owner, repositories, scopes, and expiration.",
    ],
  },
  {
    id: "sudo-grant",
    method: "POST",
    path: "/api/settings/sudo",
    title: "Create sudo grant",
    description:
      "Creates a short-lived session-bound sudo grant required before sensitive token creation or revocation.",
    auth: "Signed opengithub session cookie",
    request: `{
  "email": "mona@example.com"
}`,
    response: `{
  "active": true,
  "expiresAt": "2026-05-04T12:30:00Z",
  "requiredFor": ["create_personal_access_token", "revoke_personal_access_token"]
}`,
    notes: [
      "Local development confirms the current account email; production should replace this with the Google reauthentication ceremony.",
      "Invalid confirmation returns validation_failed without creating a sudo grant.",
      "Sudo grants are tied to the current Rust session and expire automatically.",
    ],
  },
  {
    id: "account-security-settings",
    method: "GET",
    path: "/api/settings/security",
    title: "Read account security settings",
    description:
      "Returns linked Google sign-in methods, session-bound sudo state, and the disabled 2FA placeholder for Personal Settings.",
    auth: "Signed opengithub session cookie",
    response: `{
  "signInMethods": [
    {
      "id": "oauth_01",
      "provider": "google",
      "displayLabel": "Google",
      "email": "mona@example.com",
      "canUnlink": false,
      "linkedAt": "2026-05-04T00:00:00Z"
    }
  ],
  "sudo": {
    "active": false,
    "expiresAt": null,
    "requiredFor": ["link_google_account", "unlink_sign_in_method"]
  },
  "twoFactor": { "enabled": false, "available": false }
}`,
    notes: [
      "Raw Google provider subject IDs are never returned to the browser.",
      "canUnlink is false when the account has only one sign-in method.",
      "2FA remains visible but disabled while Google-only auth is the supported sign-in method.",
    ],
  },
  {
    id: "account-security-sudo",
    method: "POST",
    path: "/api/settings/security/sudo",
    title: "Create account-security sudo grant",
    description:
      "Confirms the current account email and enables a 30-minute sudo window for sign-in method changes.",
    auth: "Signed opengithub session cookie",
    request: `{
  "confirmation": "mona@example.com"
}`,
    response: `{
  "sudo": {
    "active": true,
    "expiresAt": "2026-05-04T12:30:00Z",
    "requiredFor": ["link_google_account", "unlink_sign_in_method"]
  },
  "signInMethods": []
}`,
    notes: [
      "The grant is session-bound and also mirrors the expiry to sessions.elevated_until for account-security auditability.",
      "Invalid confirmation returns sudo_confirmation_failed.",
    ],
  },
  {
    id: "account-security-unlink-method",
    method: "DELETE",
    path: "/api/settings/security/sign-in-methods/{account_id}",
    title: "Unlink sign-in method",
    description:
      "Removes one linked Google OAuth account from the signed-in user after sudo confirmation.",
    auth: "Signed opengithub session cookie with active sudo grant",
    response: `{
  "removedId": "oauth_02",
  "settings": {
    "signInMethods": [
      { "id": "oauth_01", "provider": "google", "canUnlink": false }
    ]
  }
}`,
    notes: [
      "The last remaining sign-in method is protected with a last_identity conflict.",
      "Unlinking writes a redacted sign_in_method.unlink security audit event.",
    ],
  },
  {
    id: "account-security-link-google",
    method: "GET",
    path: "/api/settings/security/google/link",
    title: "Start linked Google account flow",
    description:
      "Starts a sudo-protected Google OAuth flow from Account Security for adding another sign-in method.",
    auth: "Signed opengithub session cookie with active sudo grant",
    response: "302 Found to Google OAuth",
    notes: [
      "The endpoint refuses to redirect without active sudo mode.",
      "The redirect uses the same Google OAuth provider as normal sign-in and returns to /settings/security.",
    ],
  },
  {
    id: "account-security-log",
    method: "GET",
    path: "/api/settings/security-log?action=session.revoke&page=1&pageSize=50",
    title: "Read account security log",
    description:
      "Returns the signed-in user's immutable security events with action filtering and 50-row pagination.",
    auth: "Signed opengithub session cookie",
    response: `{
  "events": [
    {
      "id": "event_01",
      "action": "session.revoke",
      "ipAddress": "127.0.0.1",
      "location": "Localhost",
      "userAgent": "Chrome on macOS",
      "createdAt": "2026-05-07T01:10:00Z"
    }
  ],
  "actions": ["session.revoke", "sign_in_method.unlink"],
  "pagination": { "total": 1, "page": 1, "pageSize": 50 }
}`,
    notes: [
      "Only rows whose actor_user_id matches the current account are returned.",
      "Rows are append-only in security_audit_events; this endpoint exposes read and export surfaces only.",
      "Metadata is redacted by event writers; session secrets and OAuth provider subjects are not returned.",
    ],
  },
  {
    id: "account-security-log-export",
    method: "GET",
    path: "/api/settings/security-log/export?format=csv&action=session.revoke",
    title: "Export account security log",
    description:
      "Streams the filtered security log as CSV or JSON with a download filename header.",
    auth: "Signed opengithub session cookie",
    response: "timestamp,action,ip,location,user_agent\\n...",
    notes: [
      "JSON exports return application/json; CSV exports return text/csv.",
      "The action filter is preserved so exports match the visible table.",
      "The export is generated server-side rather than as a browser data URL.",
    ],
  },
  {
    id: "personal-access-token-create",
    method: "POST",
    path: "/api/settings/tokens",
    title: "Create personal access token",
    description:
      "Creates fine-grained or classic personal access tokens and returns the plaintext secret exactly once.",
    auth: "Signed opengithub session cookie with active sudo grant",
    request: `{
  "type": "fine_grained",
  "name": "Deploy token",
  "description": "Release automation",
  "resourceOwnerId": "owner_01",
  "repositoryAccess": "selected",
  "repositoryIds": ["repo_01"],
  "expiresInDays": 30,
  "permissions": {
    "contents": "read",
    "packages": "write",
    "api": "read"
  }
}`,
    response: `{
  "plainTextToken": "oghp_generated_secret",
  "token": {
    "id": "token_01",
    "name": "Deploy token",
    "prefix": "oghp_generated_s",
    "type": "fine_grained",
    "scopes": ["repo:read", "packages:write", "api:read"]
  }
}`,
    notes: [
      "The Rust API stores only a sha256-prefixed hash and a collision-resistant display prefix.",
      "Classic tokens use broad legacy repository access; fine-grained tokens can be limited to selected repositories.",
      "Validation rejects invalid expirations, invisible repositories, unauthorized resource owners, and empty permission matrices.",
      "Security audit events store redacted token metadata only.",
    ],
  },
  {
    id: "personal-access-token-revoke",
    method: "DELETE",
    path: "/api/settings/tokens/{token_id}",
    title: "Revoke personal access token",
    description:
      "Revokes one owned token so REST, Git over HTTPS, and package registry authentication fail immediately.",
    auth: "Signed opengithub session cookie with active sudo grant",
    response: `{
  "revokedAt": "2026-05-04T13:00:00Z",
  "token": {
    "id": "token_01",
    "name": "Deploy token",
    "status": "revoked",
    "prefix": "oghp_12345678"
  }
}`,
    notes: [
      "Unknown, already-revoked, or cross-user token IDs return stable error envelopes without token material.",
      "The browser requires typed confirmation before forwarding the delete request.",
      "Revocation writes a redacted security audit event and preserves historical prefix/status rows for user review.",
    ],
  },
  {
    id: "signing-keys-list",
    method: "GET",
    path: "/api/settings/keys",
    title: "List SSH and GPG keys",
    description:
      "Returns the signed-in user's SSH keys, GPG keys, vigilant-mode preference, and sudo state for Developer Settings.",
    auth: "Signed opengithub session cookie",
    response: `{
  "sshKeys": [
    {
      "id": "ssh_key_01",
      "title": "Work laptop",
      "keyType": "ssh-ed25519",
      "fingerprintSha256": "SHA256:abc123",
      "accessMode": "read_write",
      "source": "browser",
      "lastUsedAt": null,
      "revokedAt": null,
      "createdAt": "2026-05-04T00:00:00Z"
    }
  ],
  "gpgKeys": [
    {
      "id": "gpg_key_01",
      "title": "Release signing",
      "primaryFingerprint": "0F1E2D3C4B5A6978",
      "keyId": "4B5A6978",
      "emails": ["mona@example.com"],
      "revokedAt": null
    }
  ],
  "vigilantMode": true,
  "sudo": { "active": false, "requiredFor": ["revoke_signing_key"] }
}`,
    notes: [
      "Responses expose fingerprints and metadata only; raw SSH public keys and armored GPG blocks are not serialized.",
      "Revoked keys remain visible with revokedAt for audit history and cannot authenticate or verify future signatures.",
      "Anonymous callers receive 401; the browser renders an explicit sign-in state without leaking key metadata.",
    ],
  },
  {
    id: "ssh-key-create",
    method: "POST",
    path: "/api/settings/keys/ssh",
    title: "Add SSH key",
    description:
      "Validates a public SSH key, derives its SHA256 fingerprint, enforces active per-user uniqueness, and returns the metadata row.",
    auth: "Signed opengithub session cookie",
    request: `{
  "title": "Work laptop",
  "keyType": "ssh-ed25519",
  "publicKey": "ssh-ed25519 AAAAC3Nza... mona@laptop",
  "accessMode": "read_write"
}`,
    response: `{
  "sshKey": {
    "id": "ssh_key_01",
    "title": "Work laptop",
    "keyType": "ssh-ed25519",
    "fingerprintSha256": "SHA256:abc123",
    "accessMode": "read_write",
    "revokedAt": null
  }
}`,
    notes: [
      "Validation checks the declared key type, wire key type, base64 body, bounded title, and allowed read_write/read_only access mode.",
      "Duplicate active fingerprints return validation_failed without exposing the existing key row.",
      "Security audit events store redacted key metadata only.",
    ],
  },
  {
    id: "ssh-key-revoke",
    method: "DELETE",
    path: "/api/settings/keys/ssh/{key_id}",
    title: "Revoke SSH key",
    description:
      "Revokes one owned SSH key behind sudo mode while preserving the key row for account security history.",
    auth: "Signed opengithub session cookie with active sudo grant",
    response: `{
  "revokedAt": "2026-05-04T13:00:00Z",
  "sshKey": {
    "id": "ssh_key_01",
    "title": "Work laptop",
    "fingerprintSha256": "SHA256:abc123",
    "revokedAt": "2026-05-04T13:00:00Z"
  }
}`,
    notes: [
      "Sudo mode uses the same short-lived session grant as token revocation.",
      "Unknown, already-revoked, or cross-user key IDs return stable error envelopes.",
      "Future SSH authentication helpers ignore revoked keys and never expose public key material.",
    ],
  },
  {
    id: "gpg-key-create",
    method: "POST",
    path: "/api/settings/keys/gpg",
    title: "Add GPG key",
    description:
      "Validates an armored public GPG key, extracts signing metadata, stores the public-key fingerprint, and returns redacted summary rows.",
    auth: "Signed opengithub session cookie",
    request: `{
  "title": "Release signing",
  "armoredPublicKey": "-----BEGIN PGP PUBLIC KEY BLOCK-----..."
}`,
    response: `{
  "gpgKey": {
    "id": "gpg_key_01",
    "title": "Release signing",
    "primaryFingerprint": "0F1E2D3C4B5A6978",
    "keyId": "4B5A6978",
    "emails": ["mona@example.com"],
    "revokedAt": null
  }
}`,
    notes: [
      "The armored public key is accepted only on create and is not returned by list or mutation responses.",
      "Active GPG fingerprints drive commit and tag signature presentation for commits attributed to the user.",
      "Malformed armor, duplicate active fingerprints, and oversized titles return validation_failed.",
    ],
  },
  {
    id: "gpg-key-revoke",
    method: "DELETE",
    path: "/api/settings/keys/gpg/{key_id}",
    title: "Revoke GPG key",
    description:
      "Revokes one owned GPG key behind sudo mode and stops it from verifying future commit or tag signatures.",
    auth: "Signed opengithub session cookie with active sudo grant",
    response: `{
  "revokedAt": "2026-05-04T13:00:00Z",
  "gpgKey": {
    "id": "gpg_key_01",
    "title": "Release signing",
    "primaryFingerprint": "0F1E2D3C4B5A6978",
    "revokedAt": "2026-05-04T13:00:00Z"
  }
}`,
    notes: [
      "Revoked GPG keys stay in settings history but are excluded from verified signature classification.",
      "Typed browser confirmation plus sudo mode protects destructive signing-key changes.",
      "Security audit events retain key IDs, fingerprints, and action metadata without raw armor.",
    ],
  },
  {
    id: "vigilant-mode-update",
    method: "PATCH",
    path: "/api/settings/keys/vigilant-mode",
    title: "Update vigilant mode",
    description:
      "Persists whether unsigned or untrusted commits attributed to the user should be presented as unverified.",
    auth: "Signed opengithub session cookie",
    request: `{
  "enabled": true
}`,
    response: `{
  "vigilantMode": true
}`,
    notes: [
      "The preference is stored on users.vigilant_mode and writes a vigilant_mode.update security audit event when it changes.",
      "Commit and tag presentation uses active GPG fingerprints first, then applies vigilant-mode unverified messaging for unsigned or untrusted user-attributed work.",
      "The browser rolls back the checkbox if the Rust API rejects the update.",
    ],
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
    id: "repo-file-finder",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/find?ref=main",
    title: "Repository file finder",
    description:
      "Returns the cached file path list for a repository ref so the browser can run instant Sublime-style fuzzy matching locally.",
    auth: "Signed opengithub session cookie with repository read access",
    response: `{
  "items": [
    {
      "path": "src/app/page.tsx",
      "name": "page.tsx",
      "kind": "file",
      "href": "/mona/octo-app/blob/main/src/app/page.tsx",
      "byteSize": 2048,
      "language": "TypeScript"
    }
  ],
  "total": 1,
  "page": 1,
  "pageSize": 100,
  "resolvedRef": {
    "shortName": "main",
    "qualifiedName": "refs/heads/main"
  }
}`,
    notes: [
      "The endpoint also remains available at /api/repos/{owner}/{repo}/file-finder for older toolbar clients.",
      "The Rust API refreshes repository_ref_files for the resolved ref when serving this contract.",
      "The dedicated /{owner}/{repo}/find/{ref} page fetches this list once and performs fuzzy scoring, keyboard navigation, and highlighted matches in the browser.",
    ],
  },
  {
    id: "repo-commit-history",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/commits?ref=main&path=src&author=mona&until=2026-04-30T23:59:59Z&page=1&pageSize=30",
    title: "Repository commit history",
    description:
      "Returns the screen-ready commit history contract for repository commit-list pages, including resolved ref metadata, URL-backed filters, grouped rows, status summaries, signature state, and pagination.",
    auth: "Signed opengithub session cookie with repository read access",
    response: `{
  "repository": {
    "ownerLogin": "mona",
    "name": "octo-app",
    "defaultBranch": "main",
    "visibility": "public"
  },
  "resolvedRef": {
    "shortName": "main",
    "qualifiedName": "refs/heads/main",
    "kind": "branch",
    "targetOid": "abcdef1234567890",
    "href": "/mona/octo-app/tree/main"
  },
  "filters": {
    "path": "src",
    "author": "mona",
    "until": "2026-04-30T23:59:59Z"
  },
  "groups": [
    {
      "date": "2026-04-30",
      "commits": [
        {
          "oid": "abcdef1234567890",
          "shortOid": "abcdef1",
          "subject": "Refactor router",
          "href": "/mona/octo-app/commit/abcdef1234567890",
          "browseHref": "/mona/octo-app/tree/abcdef1234567890/src",
          "pullRequests": [{ "number": 12, "href": "/mona/octo-app/pull/12" }],
          "status": {
            "status": "completed",
            "conclusion": "success",
            "totalCount": 3,
            "completedCount": 3,
            "failedCount": 0,
            "href": "/mona/octo-app/actions?commit=abcdef1234567890"
          },
          "verification": {
            "verified": true,
            "signatureState": "verified",
            "signatureSummary": "Verified signature from an active GPG key."
          }
        }
      ]
    }
  ],
  "authorOptions": [{ "login": "mona", "count": 4, "active": true }],
  "total": 4,
  "page": 1,
  "pageSize": 30,
  "hasNextPage": false,
  "hasPreviousPage": false
}`,
    notes: [
      "ref resolves against repository_git_refs and accepts branches or tags; missing refs return ref_not_found without leaking private commit OIDs.",
      "path scopes history to commits touching the requested file or directory; missing paths return path_not_found.",
      "author, until, before, page, and pageSize are URL-backed filters; page is normalized to 1 and pageSize is clamped by the API contract.",
      "Private repositories require read access. Anonymous callers receive 401, unauthorized signed-in callers receive 403, and error envelopes never include stack traces, tokens, session secrets, or private ref names.",
      "Status and verification summaries are presentation metadata only; raw check logs, signing keys, and secret material are never included in the list response.",
    ],
  },
  {
    id: "repo-commit-detail",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/commits/{sha}",
    title: "Repository commit detail",
    description:
      "Returns the screen-ready commit detail contract for the commit summary, parent/branch/PR links, status and signature chips, file tree, bounded unified diffs, Raw/View file actions, and binary or large-file placeholders.",
    auth: "Signed opengithub session cookie with repository read access",
    response: `{
  "repository": {
    "ownerLogin": "mona",
    "name": "octo-app",
    "defaultBranch": "main",
    "visibility": "public",
    "href": "/mona/octo-app",
    "commitHistoryHref": "/mona/octo-app/commits/main"
  },
  "commit": {
    "oid": "abcdef1234567890",
    "shortOid": "abcdef1",
    "subject": "Refactor router",
    "body": "Move repository routes behind typed handlers.",
    "href": "/mona/octo-app/commit/abcdef1234567890",
    "browseHref": "/mona/octo-app/tree/abcdef1234567890",
    "committedAt": "2026-04-30T00:00:00Z",
    "authorLogin": "mona"
  },
  "parents": [{ "oid": "1234567890abcdef", "href": "/mona/octo-app/commit/1234567890abcdef" }],
  "branches": [{ "name": "main", "href": "/mona/octo-app/commits/main" }],
  "pullRequests": [{ "number": 12, "href": "/mona/octo-app/pull/12", "state": "merged" }],
  "status": {
    "status": "completed",
    "conclusion": "success",
    "totalCount": 3,
    "completedCount": 3,
    "failedCount": 0,
    "href": "/mona/octo-app/actions?commit=abcdef1234567890"
  },
  "verification": {
    "verified": true,
    "signatureState": "verified",
    "signatureSummary": "Verified signature from an active GPG key."
  },
  "diffSummary": { "totalFiles": 2, "additions": 12, "deletions": 4 },
  "fileTree": [
    {
      "path": "src/router.rs",
      "status": "modified",
      "additions": 8,
      "deletions": 2,
      "href": "#diff-src-router-rs"
    }
  ],
  "files": [
    {
      "path": "src/router.rs",
      "status": "modified",
      "rawHref": "/mona/octo-app/raw/abcdef1234567890/src/router.rs",
      "viewHref": "/mona/octo-app/blob/abcdef1234567890/src/router.rs",
      "isBinary": false,
      "isLarge": false,
      "hunks": [
        {
          "id": "diff-src-router-rs-hunk-1",
          "header": "@@ -1,2 +1,2 @@ src/router.rs",
          "lines": [{ "kind": "context", "oldLine": 1, "newLine": 1, "content": "pub fn routes() {" }]
        }
      ]
    }
  ]
}`,
    notes: [
      "sha accepts an exact OID or an unambiguous abbreviation; malformed, missing, or ambiguous SHAs return stable validation/not_found envelopes without leaking private commit OIDs.",
      "Private repositories require read access. Anonymous callers receive 401, unauthorized signed-in callers receive 403, and private-repository not-found responses are redacted.",
      "Root commits return an empty parents array; merge commits return every parent link in commit order.",
      "Binary and large files keep concrete Raw/View file actions while omitting inline hunks behind truthful bounded placeholders.",
      "Status, signature, linked pull request, and branch/tag joins are presentation data only; raw check logs, signing keys, session rows, tokens, and stack traces are never included.",
      "Successful reads may record repository_commit_recent_visits for the signed-in viewer without exposing that telemetry in the public response.",
    ],
  },
  {
    id: "repo-commit-detail-context",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/commits/{sha}/context?path=src/router.rs&hunkId=diff-src-router-rs-hunk-1&contextLines=80",
    title: "Repository commit diff context",
    description:
      "Expands one commit-detail diff hunk with a bounded context window for the browser Expand all lines control.",
    auth: "Signed opengithub session cookie with repository read access",
    response: `{
  "path": "src/router.rs",
  "hunkId": "diff-src-router-rs-hunk-1",
  "expanded": true,
  "message": "Expanded context lines loaded.",
  "lines": [
    { "kind": "context", "oldLine": 1, "newLine": 1, "content": "pub fn routes() {" },
    { "kind": "removed", "oldLine": 2, "newLine": null, "content": "  todo!()" },
    { "kind": "added", "oldLine": null, "newLine": 2, "content": "  commit_detail()" }
  ]
}`,
    notes: [
      "path and hunkId must address a hunk already visible in the commit-detail response; invalid combinations return validation_failed.",
      "contextLines is clamped server-side to a bounded window so large or generated files cannot force an unbounded diff response.",
      "The same repository visibility, SHA resolution, private-access redaction, and no-secret error-envelope rules as the commit-detail endpoint apply.",
      "The Next.js same-origin proxy forwards the current Rust session cookie so browser expansion never relies on client-side auth libraries.",
    ],
  },
  {
    id: "repo-branches-directory",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/branches?tab=stale&q=release&page=1&pageSize=30",
    title: "Repository branches directory",
    description:
      "Returns the screen-ready branch directory contract for the Overview, Active, Stale, and All tabs, including search, pagination, default branch metadata, latest commits, check summaries, ahead/behind counts, linked pull requests, protection summaries, and row action capabilities.",
    auth: "Signed opengithub session cookie with repository read access",
    response: `{
  "repository": {
    "ownerLogin": "mona",
    "name": "octo-app",
    "defaultBranch": "main",
    "visibility": "public",
    "viewerPermission": "write"
  },
  "tabs": { "overview": 4, "active": 2, "stale": 1, "all": 4, "default": 1 },
  "filters": { "tab": "stale", "query": "release", "staleCutoffDays": 90 },
  "defaultBranch": {
    "name": "main",
    "qualifiedName": "refs/heads/main",
    "isDefault": true,
    "href": "/mona/octo-app/tree/main",
    "commitsHref": "/mona/octo-app/commits/main",
    "activityHref": "/mona/octo-app/branches/main",
    "protection": {
      "protected": true,
      "matchingRuleCount": 1,
      "matchingRulesetCount": 1,
      "requiredStatusChecks": ["ci"],
      "href": "/mona/octo-app/settings/branches?branch=main"
    }
  },
  "branches": [
    {
      "name": "release/old-tree",
      "classification": "stale",
      "href": "/mona/octo-app/tree/release%2Fold-tree",
      "commitsHref": "/mona/octo-app/commits/release%2Fold-tree",
      "activityHref": "/mona/octo-app/branches/release%2Fold-tree",
      "latestCommit": { "shortOid": "abcdef1", "subject": "Prepare release branch" },
      "checks": { "status": "completed", "conclusion": "success", "totalCount": 2 },
      "ahead": 1,
      "behind": 4,
      "pullRequest": { "number": 42, "state": "open", "draft": false, "href": "/mona/octo-app/pull/42" },
      "capabilities": { "canCopy": true, "canViewActivity": true, "canViewRules": true, "canDelete": false }
    }
  ],
  "total": 1,
  "page": 1,
  "pageSize": 30,
  "hasNextPage": false,
  "hasPreviousPage": false
}`,
    notes: [
      "tab accepts overview, active, stale, or all; invalid tabs return validation_failed and page/pageSize are normalized by the API.",
      "Search is case-insensitive over branch names and records bounded branch directory recent-visit telemetry for signed-in viewers.",
      "Branch names with slashes are encoded as a single route segment in href fields so tree, commits, activity, and rules destinations stay reversible.",
      "Private repositories require read access. Anonymous callers receive 401, unauthorized signed-in callers receive 403, and private-repository not-found responses are redacted.",
      "Responses include presentation summaries only; raw check logs, private refs the viewer cannot read, session rows, tokens, and stack traces are never included.",
    ],
  },
  {
    id: "repo-branch-activity",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/branches/activity?branch=release%2Fold-tree",
    title: "Repository branch activity",
    description:
      "Returns the branch activity drill-down contract used by branch row Activity links, including recent commits, recent pull requests, matching branch rules, check summaries, recovery links, and compare/tree/history destinations.",
    auth: "Signed opengithub session cookie with repository read access",
    response: `{
  "repository": {
    "ownerLogin": "mona",
    "name": "octo-app",
    "defaultBranch": "main",
    "visibility": "public"
  },
  "branch": {
    "name": "release/old-tree",
    "qualifiedName": "refs/heads/release/old-tree",
    "isDefault": false,
    "updatedAt": "2026-05-01T00:00:00Z",
    "ahead": 1,
    "behind": 4,
    "checks": { "status": "completed", "conclusion": "success", "totalCount": 2 },
    "protection": { "protected": true, "matchingRuleCount": 1, "matchingRulesetCount": 0 }
  },
  "recentCommits": [
    {
      "oid": "abcdef1234567890",
      "shortOid": "abcdef1",
      "subject": "Prepare release branch",
      "href": "/mona/octo-app/commit/abcdef1234567890",
      "status": { "conclusion": "success", "totalCount": 2, "href": "/mona/octo-app/actions?commit=abcdef1234567890" }
    }
  ],
  "recentPullRequests": [{ "number": 42, "title": "Release branch", "href": "/mona/octo-app/pull/42" }],
  "protectionEvents": [
    {
      "sourceType": "branch_rule",
      "name": "Release branches",
      "enforcement": "active",
      "href": "/mona/octo-app/settings/branches?branch=release%2Fold-tree",
      "requiredStatusChecks": ["ci"]
    }
  ],
  "links": {
    "branchesHref": "/mona/octo-app/branches",
    "treeHref": "/mona/octo-app/tree/release%2Fold-tree",
    "commitsHref": "/mona/octo-app/commits/release%2Fold-tree",
    "compareHref": "/mona/octo-app/compare/main...release%2Fold-tree",
    "rulesHref": "/mona/octo-app/settings/branches?branch=release%2Fold-tree"
  }
}`,
    notes: [
      "branch is required and accepts slash-containing branch names; malformed or missing branch values return validation_failed.",
      "Missing branches return a non-leaky recovery payload with a Branches link instead of exposing private ref names or target OIDs.",
      "The same repository privacy rules as the branch directory apply: 401 for anonymous callers, 403 for unauthorized signed-in callers, and redacted private not-found responses.",
      "Rules and check data are summaries for navigation and presentation only; raw rule bypass actors, check logs, tokens, session rows, and stack traces are never included.",
    ],
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
    id: "repo-wiki-read",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/wiki",
    title: "Read repository wiki Home",
    description:
      "Returns the repository wiki reader contract for the Home page or deterministic first page, including sanitized Markdown, revision metadata, page list, custom sidebar/footer blocks, viewer capabilities, and clone URL metadata.",
    auth: "Anonymous for public repositories; signed opengithub session cookie with repository read access for private repositories",
    response: `{
  "repository": {
    "ownerLogin": "mona",
    "name": "octo-app",
    "visibility": "public",
    "wikiEnabled": true
  },
  "viewer": {
    "permission": "read",
    "canRead": true,
    "canEditWiki": false
  },
  "state": { "kind": "ready", "message": "Wiki page is ready." },
  "page": {
    "title": "Home",
    "slug": "Home",
    "href": "/mona/octo-app/wiki",
    "revision": {
      "shortOid": "abc1234",
      "message": "Publish wiki home",
      "href": "/mona/octo-app/wiki/Home/_history/abcdef1234567890"
    },
    "html": "<h1 id=\\"home\\">Home</h1>",
    "outline": [{ "id": "home", "level": 1, "text": "Home", "href": "#home" }]
  },
  "pages": [{ "title": "Home", "slug": "Home", "active": true, "hasOutline": true }],
  "sidebar": null,
  "footer": null,
  "clone": { "httpsUrl": "https://opengithub.namuh.co/mona/octo-app.wiki.git" },
  "links": { "homeHref": "/mona/octo-app/wiki", "newPageHref": null }
}`,
    notes: [
      "Opening /wiki resolves Home first, then falls back to the first non-special page when Home is absent.",
      "Disabled visible repositories return state.kind=disabled with page=null; empty visible wikis return state.kind=empty.",
      "Private repositories never leak existence to unauthorized readers: anonymous callers receive 401 and unauthorized signed-in callers receive repository-safe not_found/forbidden envelopes according to the repository visibility contract.",
      "Markdown HTML is rendered and sanitized by the Rust API. Script tags, unsafe links, session rows, OAuth secrets, storage paths, and stack traces are never included.",
      "_Sidebar and _Footer pages are resolved as rendered blocks but are omitted from the normal page list.",
    ],
  },
  {
    id: "repo-wiki-page-read",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/wiki/{slug}",
    title: "Read repository wiki page",
    description:
      "Returns the selected wiki slug with the same reader contract as Home, preserving active page state and outline data for page-list table-of-contents expansion.",
    auth: "Anonymous for public repositories; signed opengithub session cookie with repository read access for private repositories",
    response: `{
  "state": { "kind": "ready", "message": "Wiki page is ready." },
  "page": {
    "title": "Architecture Guide",
    "slug": "Architecture Guide",
    "href": "/mona/octo-app/wiki/Architecture%20Guide",
    "historyHref": "/mona/octo-app/wiki/Architecture%20Guide/_history",
    "editHref": null,
    "outline": [
      { "id": "architecture-guide", "level": 1, "text": "Architecture Guide", "href": "#architecture-guide" },
      { "id": "services", "level": 2, "text": "Services", "href": "#services" }
    ]
  },
  "pages": [
    { "title": "Home", "slug": "Home", "active": false, "hasOutline": true },
    { "title": "Architecture Guide", "slug": "Architecture Guide", "active": true, "hasOutline": true }
  ]
}`,
    notes: [
      "Slugs may include spaces and nested path segments. Empty slugs and traversal-like segments are rejected by the same-origin TOC proxy before it forwards to Rust.",
      "Unknown readable pages return state.kind=missing_page with page=null and a permissioned New Page link only when the viewer can edit the wiki.",
      "The browser lazy-loads TOC expansion through /api/repos/{owner}/{repo}/wiki-toc/{slug}; that proxy returns only the selected page summary and outline from this Rust read contract.",
      "Clone URL metadata is HTTPS-only and derived from the configured public app host; full wiki Git transport is intentionally outside wiki-001.",
    ],
  },
  {
    id: "repo-wiki-pages-index",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/wiki/_pages",
    title: "List repository wiki pages",
    description:
      "Returns the editor-facing pages index with all non-special wiki pages, supported markup formats, viewer write capability, clone URL metadata, and concrete links for page detail, edit, and New Page routes.",
    auth: "Signed opengithub session cookie with repository read access; write controls require repository write or admin permission",
    response: `{
  "repository": { "ownerLogin": "mona", "name": "octo-app", "wikiEnabled": true },
  "viewer": { "permission": "write", "canEditWiki": true },
  "pages": [
    {
      "id": "page_01",
      "title": "Architecture Guide",
      "slug": "Architecture Guide",
      "href": "/mona/octo-app/wiki/Architecture%20Guide",
      "editHref": "/mona/octo-app/wiki/Architecture%20Guide/_edit",
      "updatedAt": "2026-05-06T00:00:00Z"
    }
  ],
  "supportedMarkupFormats": [{ "mode": "markdown", "label": "Markdown", "extension": ".md" }]
}`,
    notes: [
      "The browser sorts rows by title and hides _Sidebar/_Footer from the normal pages list while still allowing those special pages to be edited directly.",
      "Readers receive the same list data without New Page or Edit affordances; mutation endpoints remain protected by Rust session and permission checks.",
      "Disabled wikis and unauthorized private repositories return the standard repository-safe error envelope and do not expose storage paths or session metadata.",
    ],
  },
  {
    id: "repo-wiki-page-edit-read",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/wiki/{slug}/edit",
    title: "Read repository wiki edit draft",
    description:
      "Returns the latest editable source for a selected wiki page, including title, slug, Markdown, latest revision id, edit mode, revision metadata, and save/preview links.",
    auth: "Signed opengithub session cookie with repository write or admin permission",
    response: `{
  "page": {
    "id": "page_01",
    "title": "Architecture Guide",
    "slug": "Architecture Guide",
    "markdown": "# Architecture Guide\\n\\n## Services\\n",
    "editMode": "markdown",
    "latestRevisionId": "rev_01"
  },
  "supportedMarkupFormats": [{ "mode": "markdown", "label": "Markdown", "extension": ".md" }]
}`,
    notes: [
      "Readers and anonymous callers cannot fetch editable source; private repository existence follows the same safe 401/403/404 boundaries as repository reads.",
      "The latestRevisionId is required by PATCH for stale-edit conflict detection.",
      "The response contains source Markdown only, never rendered cache storage paths, OAuth secrets, session rows, or local Git filesystem paths.",
    ],
  },
  {
    id: "repo-wiki-preview",
    method: "POST",
    path: "/api/repos/{owner}/{repo}/wiki/preview",
    title: "Preview repository wiki Markdown",
    description:
      "Renders a wiki draft through the Rust Markdown sanitizer without persisting a revision, cache row, image asset row, activity event, audit event, or local Git commit.",
    auth: "Signed opengithub session cookie with repository write or admin permission",
    request: `{
  "markdown": "## Preview\\n\\n![Diagram](https://images.example/diagram.png)",
  "editMode": "markdown"
}`,
    response: `{
  "html": "<h2 id=\\"preview\\">Preview</h2><p><img src=\\"https://images.example/diagram.png\\" alt=\\"Diagram\\"></p>",
  "outline": [{ "id": "preview", "level": 2, "text": "Preview", "href": "#preview" }]
}`,
    notes: [
      "Only supported markup modes are accepted; unsupported modes return validation_failed with a stable 422 error envelope.",
      "Preview output is sanitized by Rust and is the only HTML the browser renders; the client-side editor is not trusted as the sanitizer.",
      "Preview does not write wiki_page_revisions, wiki_assets, repository_activity_events, audit_events, rendered_markdown_cache, or wiki_git_commits.",
    ],
  },
  {
    id: "repo-wiki-page-create",
    method: "POST",
    path: "/api/repos/{owner}/{repo}/wiki/pages",
    title: "Create repository wiki page",
    description:
      "Creates a wiki page from an editor draft, validates title/body/message/mode, appends a revision, records linked image references, refreshes rendered Markdown caches, writes audit/activity rows, commits to the local bare wiki repository, and returns the rendered page redirect.",
    auth: "Signed opengithub session cookie with repository write or admin permission",
    request: `{
  "title": "Operations Guide",
  "markdown": "# Operations\\n\\n![Deploy map](https://images.example/deploy.png)",
  "message": "Create operations guide",
  "editMode": "markdown"
}`,
    response: `{
  "page": {
    "title": "Operations Guide",
    "slug": "Operations Guide",
    "href": "/mona/octo-app/wiki/Operations%20Guide",
    "revision": { "id": "rev_02", "shortOid": "f00ba47" }
  },
  "gitCommit": { "shortOid": "f00ba47", "branch": "master" },
  "redirectHref": "/mona/octo-app/wiki/Operations%20Guide"
}`,
    notes: [
      "Duplicate normalized slugs return conflict instead of overwriting another page.",
      "Linked Markdown images are recorded in wiki_assets with storage_kind=remote_url; binary upload storage is intentionally outside this feature.",
      "Successful saves write repository.wiki_page.save audit events and repository activity rows; failed local Git publishing aborts the live page update.",
    ],
  },
  {
    id: "repo-wiki-page-update",
    method: "PATCH",
    path: "/api/repos/{owner}/{repo}/wiki/{slug}",
    title: "Update repository wiki page",
    description:
      "Updates an existing wiki page through the same publishing path as create, including rename validation, expected revision conflict checks, image reference refresh, cache invalidation, audit/activity rows, and local bare Git commit metadata.",
    auth: "Signed opengithub session cookie with repository write or admin permission",
    request: `{
  "title": "Architecture Guide",
  "markdown": "# Architecture Guide\\n\\nUpdated services.",
  "message": "Update architecture guide",
  "editMode": "markdown",
  "expectedRevisionId": "rev_01"
}`,
    response: `{
  "page": {
    "title": "Architecture Guide",
    "slug": "Architecture Guide",
    "href": "/mona/octo-app/wiki/Architecture%20Guide",
    "revision": { "id": "rev_03", "shortOid": "c0ffee1" }
  },
  "gitCommit": { "shortOid": "c0ffee1", "branch": "master" },
  "redirectHref": "/mona/octo-app/wiki/Architecture%20Guide"
}`,
    notes: [
      "Stale expectedRevisionId values return 409 conflict so the browser can preserve the draft and ask the editor to refresh.",
      "Renames are allowed only when the normalized target slug does not collide with another page; _Sidebar and _Footer keep their special rendering semantics.",
      "Only commits that update the wiki default branch become live pages; the API does not expose local filesystem paths, credentials, or raw session data.",
    ],
  },
  {
    id: "repo-wiki-history",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/wiki/_history?page=1&pageSize=30",
    title: "List repository wiki revision history",
    description:
      "Returns the screen-ready wiki revision history contract for all pages or a page-scoped history route, including author metadata, commit messages, short SHA links, revision snapshot links, pagination, and viewer permissions.",
    auth: "Anonymous for public repositories; signed opengithub session cookie with repository read access for private repositories",
    response: `{
  "repository": { "ownerLogin": "mona", "name": "octo-app", "wikiEnabled": true },
  "viewer": { "permission": "write", "canEditWiki": true },
  "scope": { "kind": "all_pages", "page": null },
  "revisions": [
    {
      "id": "rev_02",
      "pageTitle": "Home",
      "pageSlug": "Home",
      "message": "Refresh wiki home",
      "shortOid": "bcdef12",
      "href": "/mona/octo-app/wiki/Home/_history/bcdef1234567890",
      "revisionHref": "/mona/octo-app/wiki/Home/_history/bcdef1234567890"
    }
  ],
  "pagination": {
    "page": 1,
    "pageSize": 30,
    "hasNewer": false,
    "hasOlder": true,
    "olderHref": "/mona/octo-app/wiki/_history?page=2"
  },
  "links": { "historyHref": "/mona/octo-app/wiki/_history" }
}`,
    notes: [
      "Page-scoped history is available at /api/repos/{owner}/{repo}/wiki/{slug}/_history and preserves page, pageSize, and page scope in Newer/Older links.",
      "page is clamped to at least 1 and pageSize is bounded so large histories cannot force unbounded reads.",
      "Short SHA links target immutable revision snapshots; the response never includes raw session rows, OAuth secrets, local wiki storage paths, or stack traces.",
      "Readers can select revisions for compare, but write-only controls such as revert are exposed later by the compare contract only when canEditWiki is true.",
    ],
  },
  {
    id: "repo-wiki-revision-read",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/wiki/{slug}/_history/{revision}",
    title: "Read repository wiki historical revision",
    description:
      "Renders one historical wiki revision by revision id, full commit OID, or unique short OID using the same Rust Markdown sanitizer as the live reader while keeping the browser route read-only.",
    auth: "Anonymous for public repositories; signed opengithub session cookie with repository read access for private repositories",
    response: `{
  "page": {
    "title": "Home",
    "slug": "Home",
    "html": "<h1 id=\\"home\\">Home</h1><p>Historical body.</p>",
    "revision": {
      "id": "rev_01",
      "message": "Publish wiki home",
      "shortOid": "abcdef1"
    },
    "editHref": null
  },
  "revisionContext": {
    "selectedRevision": { "id": "rev_01", "shortOid": "abcdef1" },
    "latestHref": "/mona/octo-app/wiki",
    "historyHref": "/mona/octo-app/wiki/Home/_history",
    "previousRevisionHref": null,
    "nextRevisionHref": "/mona/octo-app/wiki/Home/_history/bcdef12",
    "isLatest": false
  }
}`,
    notes: [
      "Ambiguous short OIDs, unknown revisions, disabled wikis, and private repositories outside the viewer boundary return structured error envelopes.",
      "Historical snapshots never return editHref or save endpoints; reverting is intentionally available only from compare after permission checks.",
      "HTML is sanitized from the stored historical Markdown, not from the latest page content or client-side rendering.",
    ],
  },
  {
    id: "repo-wiki-compare",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/wiki/_compare?base={revision}&head={revision}&page={slug}",
    title: "Compare repository wiki revisions",
    description:
      "Returns a bounded line diff between two wiki revisions, normalizing base/head order, resolving cached diffs when available, and exposing permissioned revert capability through viewer.canEditWiki.",
    auth: "Anonymous for public repositories; signed opengithub session cookie with repository read access for private repositories",
    response: `{
  "page": { "title": "Home", "slug": "Home", "href": "/mona/octo-app/wiki" },
  "base": { "id": "rev_01", "shortOid": "abcdef1", "message": "Publish wiki home" },
  "head": { "id": "rev_02", "shortOid": "bcdef12", "message": "Refresh wiki home" },
  "stats": { "additions": 1, "deletions": 1, "totalLines": 2, "truncated": false },
  "files": [
    {
      "path": "Home.md",
      "additions": 1,
      "deletions": 1,
      "hunks": [
        {
          "header": "@@ -1,2 +1,2 @@",
          "lines": [
            { "kind": "deletion", "oldNumber": 2, "newNumber": null, "content": "Old body." },
            { "kind": "addition", "oldNumber": null, "newNumber": 2, "content": "New body." }
          ]
        }
      ]
    }
  ],
  "links": { "historyHref": "/mona/octo-app/wiki/Home/_history" }
}`,
    notes: [
      "base and head may be revision ids, full commit OIDs, or unique short OIDs; identical, unrelated, cross-page, and unavailable revisions return validation or not_found envelopes.",
      "wiki_diff_cache stores bounded structured diff results and is refreshed when source revisions change or revert invalidates stale comparisons.",
      "Huge diffs are truncated for inline viewing and still preserve line numbers, paths, stats, and raw revision links for QA inspection.",
    ],
  },
  {
    id: "repo-wiki-revert",
    method: "POST",
    path: "/api/repos/{owner}/{repo}/wiki/reverts",
    title: "Revert repository wiki revisions",
    description:
      "Creates a new wiki revision that restores the base revision from a compare view, records a revert event, publishes a local bare Git commit, invalidates render/diff caches, and redirects to page history.",
    auth: "Signed opengithub session cookie with repository write or admin permission",
    request: `{
  "pageSlug": "Home",
  "baseRevisionId": "rev_01",
  "expectedHeadRevisionId": "rev_02"
}`,
    response: `{
  "page": {
    "title": "Home",
    "slug": "Home",
    "href": "/mona/octo-app/wiki",
    "revision": { "id": "rev_03", "shortOid": "cafe123" }
  },
  "gitCommit": {
    "shortOid": "cafe123",
    "message": "Revert wiki page Home to abcdef1"
  },
  "revertEventId": "revert_01",
  "restoredRevisionId": "rev_03",
  "redirectHref": "/mona/octo-app/wiki/Home/_history"
}`,
    notes: [
      "The mutation validates edit permission, enabled wiki state, non-archived repository state, same-page revision compatibility, and expectedHeadRevisionId freshness.",
      "Successful reverts write wiki_revert_events, wiki_page_revisions, wiki_git_commits, rendered_markdown_cache invalidation, repository activity, and audit_events in one publishing flow.",
      "Reader, stale, disabled, archived, cross-page, and unavailable revisions return 401, 403, 404, 409, or 422 without stack traces, secrets, local Git paths, or raw submitted cookies.",
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
    id: "repo-branch-settings-read",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/settings/branches",
    title: "Read repository branch policies",
    description:
      "Reads the Branches settings contract for default branch state, branch refs, branch protection rules, repository rulesets, matching branch previews, status-check suggestions, bypass actors, and viewer edit capability.",
    auth: "Signed opengithub session cookie; repository admins receive editable state, non-admin readers receive active/evaluate policy explanations only",
    response: `{
  "repository": {
    "ownerLogin": "mona",
    "name": "octo-app",
    "viewerPermission": "admin",
    "canEdit": true
  },
  "defaultBranch": {
    "name": "main",
    "protected": true,
    "matchingRuleCount": 1,
    "matchingRulesetCount": 1
  },
  "rules": [
    {
      "id": "rule_01",
      "pattern": "main",
      "enforcement": "active",
      "requirements": {
        "requiredApprovingReviewCount": 2,
        "requiredStatusChecks": ["ci", "biome"],
        "requiresLinearHistory": true
      },
      "bypassActors": []
    }
  ],
  "rulesets": [],
  "auditEvents": []
}`,
    notes: [
      "Anonymous callers receive 401; private repository outsiders receive 404 without branch policy counts.",
      "Non-admin readers can see active and evaluate policy explanations, but mutation controls are omitted by viewer.canEdit=false.",
      "Matching previews use the same bounded fnmatch-style pattern matcher as PR mergeability and Git push enforcement.",
    ],
  },
  {
    id: "repo-branch-rule-create",
    method: "POST",
    path: "/api/repos/{owner}/{repo}/settings/branches/rules",
    title: "Create branch protection rule",
    description:
      "Creates a branch protection rule and returns fresh Branches settings after validation and audit logging.",
    auth: "Signed opengithub session cookie with repository admin or owner access",
    request: `{
  "pattern": "main",
  "description": "Protect the release branch",
  "enforcement": "active",
  "requirements": {
    "requiredApprovingReviewCount": 2,
    "requiredStatusChecks": ["ci", "biome"],
    "requiresConversationResolution": true,
    "requiresSignedCommits": true,
    "requiresLinearHistory": true,
    "requiresMergeQueue": false,
    "requiresDeployments": false,
    "locked": false,
    "restrictsPushes": true,
    "allowsForcePushes": false,
    "allowsDeletions": false
  },
  "bypassActors": []
}`,
    response: `{
  "rules": [{ "pattern": "main", "enforcement": "active" }],
  "auditEvents": [{ "eventType": "repository.branch_rule.create" }]
}`,
    notes: [
      "PATCH /api/repos/{owner}/{repo}/settings/branches/rules/{rule_id} updates the same shape; DELETE removes the rule.",
      "Blank or invalid patterns, duplicate exact patterns, negative review counts, blank status checks, invalid bypass actors, and unsafe default-branch deletion allowances return validation errors.",
      "Every successful create, update, or delete writes repository_settings_audit_events with before/after branch policy context.",
    ],
  },
  {
    id: "repo-branch-ruleset-create",
    method: "POST",
    path: "/api/repos/{owner}/{repo}/settings/branches/rulesets",
    title: "Create repository ruleset",
    description:
      "Creates an active, evaluate, or disabled repository ruleset with branch target patterns, requirements, bypass actors, and audit logging.",
    auth: "Signed opengithub session cookie with repository admin or owner access",
    request: `{
  "name": "Release branches",
  "enforcement": "evaluate",
  "patterns": ["release/*"],
  "requirements": {
    "requiredApprovingReviewCount": 1,
    "requiredStatusChecks": ["release-smoke"],
    "requiresDeployments": true,
    "requiredDeploymentEnvironments": ["production"]
  },
  "bypassActors": [
    { "actorType": "role", "actorId": "admin", "label": "Repository admins" }
  ]
}`,
    response: `{
  "rulesets": [
    {
      "name": "Release branches",
      "enforcement": "evaluate",
      "matchingBranches": ["release/2026-05"]
    }
  ]
}`,
    notes: [
      "PATCH /api/repos/{owner}/{repo}/settings/branches/rulesets/{ruleset_id} updates the same shape; DELETE removes the ruleset.",
      "Active rulesets block PR merges and Git pushes when requirements fail; evaluate rulesets insert repository_rule_evaluations without blocking.",
      "Push enforcement returns branch_policy_blocked for locked branches, restricted pushes, force pushes, deletions, or missing bypass permissions.",
    ],
  },
  {
    id: "repo-webhooks-list",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/settings/hooks",
    title: "List repository webhooks",
    description:
      "Reads the admin-only repository Webhooks settings contract, including configured endpoints, event subscriptions, write-only secret state, recent delivery summaries, supported event definitions, and audit events.",
    auth: "Signed opengithub session cookie with repository admin or owner access",
    response: `{
  "repository": {
    "ownerLogin": "mona",
    "name": "octo-app",
    "viewerPermission": "admin",
    "canEdit": true
  },
  "eventDefinitions": [
    { "name": "push", "label": "Pushes", "category": "Code" }
  ],
  "hooks": [
    {
      "id": "hook_01",
      "payloadUrl": "https://receiver.example.com/hooks/opengithub",
      "contentType": "json",
      "active": true,
      "sslVerify": true,
      "eventSelection": "selected",
      "events": ["push", "pull_request"],
      "secretConfigured": true,
      "latestDelivery": {
        "id": "delivery_01",
        "guid": "9a8b7c",
        "event": "push",
        "status": "delivered",
        "responseStatus": 202,
        "attemptCount": 1,
        "durationMs": 88
      }
    }
  ],
  "auditEvents": []
}`,
    notes: [
      "Anonymous callers receive 401; non-admin viewers receive 403 without hook URLs, secrets, or delivery history.",
      "Plaintext webhook secrets are never returned; secretConfigured is the only readable secret state.",
      "Delivery statuses are queued, delivered, and failed with attempt counts, duration, and redelivery lineage.",
    ],
  },
  {
    id: "repo-webhooks-create",
    method: "POST",
    path: "/api/repos/{owner}/{repo}/settings/hooks",
    title: "Create repository webhook",
    description:
      "Creates a webhook endpoint, stores the secret write-only, queues an initial ping delivery, records an audit event, and returns fresh settings with the queued delivery summary.",
    auth: "Signed opengithub session cookie with repository admin or owner access",
    request: `{
  "payloadUrl": "https://receiver.example.com/hooks/opengithub",
  "contentType": "json",
  "secret": "shown-only-on-submit",
  "sslVerify": true,
  "eventSelection": "selected",
  "events": ["push", "issues"],
  "active": true
}`,
    response: `{
  "settings": {
    "hooks": [{ "payloadUrl": "https://receiver.example.com/hooks/opengithub" }]
  },
  "delivery": {
    "event": "ping",
    "status": "queued",
    "attemptCount": 0
  }
}`,
    notes: [
      "Payload URLs must use HTTPS and contentType must be json or form.",
      "Selected event mode requires at least one supported event; everything mode subscribes to all supported events.",
      "Validation errors return the standard validation_failed envelope without echoing secrets.",
    ],
  },
  {
    id: "repo-webhooks-detail",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/settings/hooks/{hook_id}",
    title: "Read repository webhook detail",
    description:
      "Reads one webhook configuration plus its recent delivery history for the Editorial hook detail and Recent deliveries panels.",
    auth: "Signed opengithub session cookie with repository admin or owner access",
    response: `{
  "hook": {
    "id": "hook_01",
    "payloadUrl": "https://receiver.example.com/hooks/opengithub",
    "secretConfigured": true
  },
  "deliveries": [
    {
      "id": "delivery_01",
      "guid": "9a8b7c",
      "event": "push",
      "status": "failed",
      "responseStatus": 503,
      "attemptCount": 2,
      "durationMs": 240,
      "redeliveryOfId": null
    }
  ]
}`,
    notes: [
      "Missing hooks and private repository outsiders return not_found without leaking endpoint metadata.",
      "Recent delivery rows include retry attempt counts and redeliveryOfId for manual redelivery lineage.",
    ],
  },
  {
    id: "repo-webhooks-update-delete",
    method: "PATCH",
    path: "/api/repos/{owner}/{repo}/settings/hooks/{hook_id}",
    title: "Update repository webhook",
    description:
      "Updates endpoint configuration and returns fresh settings only after validation, secret-retention handling, and audit logging succeed.",
    auth: "Signed opengithub session cookie with repository admin or owner access",
    request: `{
  "payloadUrl": "https://receiver.example.com/hooks/opengithub-v2",
  "contentType": "form",
  "secret": "",
  "sslVerify": true,
  "eventSelection": "push",
  "events": ["push"],
  "active": false
}`,
    response: `{
  "hooks": [
    {
      "payloadUrl": "https://receiver.example.com/hooks/opengithub-v2",
      "contentType": "form",
      "active": false,
      "secretConfigured": true
    }
  ],
  "auditEvents": [{ "eventType": "repository.webhook.update" }]
}`,
    notes: [
      "Blank secret on edit retains the existing secret hash; a non-empty secret rotates it.",
      "DELETE /api/repos/{owner}/{repo}/settings/hooks/{hook_id} removes the hook and writes repository.webhook.delete.",
      "Deleted hooks cannot be pinged or redelivered.",
    ],
  },
  {
    id: "repo-webhooks-ping",
    method: "POST",
    path: "/api/repos/{owner}/{repo}/settings/hooks/{hook_id}/ping",
    title: "Ping repository webhook",
    description:
      "Queues a manual ping delivery for an active or inactive webhook so admins can test receiver connectivity without changing subscriptions.",
    auth: "Signed opengithub session cookie with repository admin or owner access",
    response: `{
  "settings": { "hooks": [{ "id": "hook_01" }] },
  "delivery": {
    "id": "delivery_ping",
    "event": "ping",
    "status": "queued",
    "attemptCount": 0
  }
}`,
    notes: [
      "The delivery worker signs ping payloads with the configured secret using x-hub-signature-256.",
      "Worker writes response status, bounded headers/body excerpts or storage keys, duration, and retry metadata.",
    ],
  },
  {
    id: "repo-webhooks-delivery",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/settings/hooks/{hook_id}/deliveries/{delivery_id}",
    title: "Read webhook delivery detail",
    description:
      "Reads one delivery request/response panel with redacted headers, bounded body excerpts, attempt metadata, and retry timing.",
    auth: "Signed opengithub session cookie with repository admin or owner access",
    response: `{
  "summary": {
    "id": "delivery_01",
    "guid": "9a8b7c",
    "event": "push",
    "status": "failed",
    "attemptCount": 2,
    "responseStatus": 503,
    "durationMs": 240
  },
  "requestHeaders": { "x-github-event": "push" },
  "requestBody": "{\\"zen\\":\\"Keep it logically awesome.\\"}",
  "responseHeaders": { "content-type": "application/json" },
  "responseBody": "{\\"ok\\":false}",
  "nextAttemptAt": "2026-05-03T02:05:00Z"
}`,
    notes: [
      "Secret headers and authorization-like receiver headers are redacted before storage or display.",
      "Oversized request and response bodies are represented by storage keys instead of unbounded inline strings.",
    ],
  },
  {
    id: "repo-webhooks-redeliver",
    method: "POST",
    path: "/api/repos/{owner}/{repo}/settings/hooks/{hook_id}/deliveries/{delivery_id}/redeliver",
    title: "Redeliver webhook event",
    description:
      "Queues a new delivery from an existing delivery payload and links it to the original delivery for audit and timeline clarity.",
    auth: "Signed opengithub session cookie with repository admin or owner access",
    response: `{
  "settings": { "hooks": [{ "id": "hook_01" }] },
  "delivery": {
    "id": "delivery_redelivered",
    "event": "redelivery",
    "status": "queued",
    "redeliveryOfId": "delivery_01"
  }
}`,
    notes: [
      "Redelivery preserves original hook ownership checks and fails closed if the delivery does not belong to the hook.",
      "Every successful redelivery writes repository.webhook.redeliver without storing plaintext secrets.",
    ],
  },
  {
    id: "org-webhooks",
    method: "GET",
    path: "/api/orgs/{org}/settings/hooks",
    title: "Manage organization webhooks",
    description:
      "Reads the owner-only organization webhook settings contract. Organization hooks share the repository hook schema and receive matching repository events from repositories owned by the organization.",
    auth: "Signed opengithub session cookie with organization owner access",
    request: `{
  "payloadUrl": "https://receiver.example.com/hooks/org",
  "contentType": "json",
  "secret": "shown-only-on-submit",
  "eventSelection": "selected",
  "events": ["push", "workflow_run"],
  "active": true
}`,
    response: `{
  "organizationId": "org_01",
  "slug": "namuh",
  "viewerRole": "owner",
  "hooks": [
    {
      "payloadUrl": "https://receiver.example.com/hooks/org",
      "secretConfigured": true,
      "latestDelivery": { "event": "ping", "status": "queued" }
    }
  ]
}`,
    notes: [
      "POST creates a hook and queues ping; PATCH/DELETE use /api/orgs/{org}/settings/hooks/{hook_id}.",
      "POST /api/orgs/{org}/settings/hooks/{hook_id}/ping queues a signed ping delivery.",
      "GET /api/orgs/{org}/settings/hooks/{hook_id}/deliveries/{delivery_id} and the redeliver subroute mirror repository delivery detail behavior.",
      "Organization admins and members receive 403; private organization outsiders receive not_found without endpoint metadata.",
    ],
  },
  {
    id: "repo-actions-secrets-list",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/settings/secrets",
    title: "Read Actions secrets and variables",
    description:
      "Reads the admin-only Actions secrets and variables settings contract, including repository-scoped metadata, inherited organization or environment metadata when visible, audit-safe update actors, and workflow availability diagnostics.",
    auth: "Signed opengithub session cookie with repository admin or owner access",
    response: `{
  "repositoryId": "repo_01",
  "ownerLogin": "mona",
  "name": "octo-app",
  "canEdit": true,
  "secrets": [
    {
      "id": "secret_01",
      "name": "DEPLOY_KEY",
      "scope": { "kind": "repository", "name": null },
      "secretConfigured": true,
      "storageKind": "local_envelope",
      "updatedAt": "2026-05-03T00:00:00Z"
    }
  ],
  "variables": [
    {
      "id": "var_01",
      "name": "PUBLIC_BASE_URL",
      "value": "https://opengithub.namuh.co"
    }
  ],
  "inheritedSecrets": [],
  "inheritedVariables": []
}`,
    notes: [
      "Anonymous callers receive 401; non-admin viewers receive 403 without secret names, variable values, inherited metadata, or audit details.",
      "Secret responses expose only metadata and secretConfigured; plaintext, ciphertext, fingerprints, encrypted refs, nonce material, and storage envelopes are never serialized.",
      "Inherited organization and environment rows follow their visibility policy and remain metadata-only for secrets.",
      "Workflow runtime resolution uses these rows but blocks secrets for untrusted fork pull_request events, environment-gated jobs before approval, and reusable workflows without explicit secret allow-lists.",
    ],
  },
  {
    id: "repo-actions-secret-create",
    method: "POST",
    path: "/api/repos/{owner}/{repo}/settings/secrets/secrets",
    title: "Create Actions secret",
    description:
      "Creates a repository Actions secret, encrypts the submitted value with the server-side envelope abstraction, writes a redacted audit event, and returns fresh settings metadata.",
    auth: "Signed opengithub session cookie with repository admin or owner access",
    request: `{
  "name": "DEPLOY_KEY",
  "value": "shown-only-on-submit"
}`,
    response: `{
  "secrets": [
    {
      "name": "DEPLOY_KEY",
      "secretConfigured": true,
      "storageKind": "local_envelope"
    }
  ],
  "variables": []
}`,
    notes: [
      "PATCH /api/repos/{owner}/{repo}/settings/secrets/secrets/{name} rotates or renames a secret and also requires a fresh nonblank value.",
      "DELETE /api/repos/{owner}/{repo}/settings/secrets/secrets/{name} removes the repository-scoped secret and writes repository.actions_secret.delete.",
      "Names are normalized to identifier-like uppercase names; reserved runtime names such as GITHUB_TOKEN return validation_failed.",
      "Validation and conflict responses never echo submitted secret values.",
    ],
  },
  {
    id: "repo-actions-variable-create",
    method: "POST",
    path: "/api/repos/{owner}/{repo}/settings/secrets/variables",
    title: "Create Actions variable",
    description:
      "Creates a repository Actions variable whose value may be displayed to repository admins and resolved into future workflow runtime environments.",
    auth: "Signed opengithub session cookie with repository admin or owner access",
    request: `{
  "name": "PUBLIC_BASE_URL",
  "value": "https://opengithub.namuh.co"
}`,
    response: `{
  "variables": [
    {
      "name": "PUBLIC_BASE_URL",
      "value": "https://opengithub.namuh.co",
      "visibilityPolicy": "repository"
    }
  ]
}`,
    notes: [
      "PATCH /api/repos/{owner}/{repo}/settings/secrets/variables/{name} updates or renames the variable and returns server-confirmed settings.",
      "DELETE /api/repos/{owner}/{repo}/settings/secrets/variables/{name} removes the repository variable and writes repository.actions_variable.delete.",
      "Duplicate names, invalid identifiers, reserved runtime names, blank values, and archived repositories return standard error envelopes.",
      "Variable values can be serialized to workflow context; secret values cannot and are masked from job logs, annotations, log downloads, and run archives.",
    ],
  },
  {
    id: "repo-pages-read",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/settings/pages",
    title: "Read repository Pages settings",
    description:
      "Reads the Pages settings contract for source selection, domain verification, HTTPS state, cloud provisioning metadata, deployment history, workflow suggestions, and audit events.",
    auth: "Signed opengithub session cookie with repository read access; admin-only challenge and cloud metadata are redacted for non-admin readers",
    response: `{
  "repositoryId": "repo_01",
  "ownerLogin": "mona",
  "name": "octo-app",
  "canEdit": true,
  "site": {
    "source": { "kind": "branch", "branch": "main", "folder": "/docs" },
    "defaultSiteUrl": "https://mona.opengithub.namuh.co/octo-app",
    "customDomain": "docs.example.com",
    "domain": {
      "status": "pending",
      "challenge": {
        "recordType": "TXT",
        "name": "_opengithub-pages.docs.example.com",
        "value": "og-pages-token"
      }
    },
    "httpsEnforced": false,
    "certificateStatus": "pending",
    "provisioningStatus": "queued"
  },
  "deployments": []
}`,
    notes: [
      "Anonymous callers receive 401 and private repository outsiders receive not_found without leaking Pages metadata.",
      "Non-admin readers can inspect public live status but never receive DNS challenge values, CloudFront aliases, or S3 artifact storage keys.",
      "Local development can report degraded provisioning while preserving the same S3, CloudFront, and Cloudflare metadata shape used in production.",
    ],
  },
  {
    id: "repo-pages-source-update",
    method: "PATCH",
    path: "/api/repos/{owner}/{repo}/settings/pages/source",
    title: "Update Pages publishing source",
    description:
      "Configures no source, branch publishing, or Actions artifact publishing and returns fresh server-confirmed Pages settings.",
    auth: "Signed opengithub session cookie with repository admin or owner access",
    request: `{
  "kind": "branch",
  "branch": "main",
  "folder": "/docs",
  "workflowId": null,
  "workflowArtifactName": null
}`,
    response: `{
  "site": {
    "source": { "kind": "branch", "branch": "main", "folder": "/docs" },
    "provisioningStatus": "queued"
  },
  "deployments": [{ "status": "queued", "source": { "kind": "branch" } }]
}`,
    notes: [
      "Branch sources require an existing repository ref and either / or /docs at the selected commit.",
      "Actions sources require a compatible workflow and artifact name; unrelated workflow artifacts cannot create Pages deployments.",
      "Every successful source change writes repository.pages.source.update and queues deployment work when publishing remains enabled.",
    ],
  },
  {
    id: "repo-pages-domain-save",
    method: "POST",
    path: "/api/repos/{owner}/{repo}/settings/pages/domain",
    title: "Save Pages custom domain",
    description:
      "Normalizes and reserves a custom domain, creates a DNS TXT challenge, disables HTTPS until verification succeeds, and returns updated Pages settings.",
    auth: "Signed opengithub session cookie with repository admin or owner access",
    request: `{
  "domain": "Docs.Example.COM."
}`,
    response: `{
  "site": {
    "customDomain": "docs.example.com",
    "domain": {
      "status": "pending",
      "challenge": { "name": "_opengithub-pages.docs.example.com" }
    },
    "httpsEnforced": false,
    "certificateStatus": "pending"
  }
}`,
    notes: [
      "DELETE /api/repos/{owner}/{repo}/settings/pages/domain removes the domain, challenge, certificate state, and HTTPS enforcement.",
      "Wildcard, apex-conflict, duplicate active-domain, and unsupported domain inputs return validation or conflict envelopes.",
      "Domain writes are audited with repository.pages.domain.save or repository.pages.domain.remove.",
    ],
  },
  {
    id: "repo-pages-dns-recheck",
    method: "POST",
    path: "/api/repos/{owner}/{repo}/settings/pages/domain/recheck",
    title: "Recheck Pages DNS verification",
    description:
      "Checks the latest custom-domain challenge through the configured DNS/provider path and advances certificate readiness only after verification succeeds.",
    auth: "Signed opengithub session cookie with repository admin or owner access",
    response: `{
  "site": {
    "domain": { "status": "verified", "lastCheckedAt": "2026-05-03T00:00:00Z" },
    "certificateStatus": "issued"
  }
}`,
    notes: [
      "Local mode records pending or misconfigured verification rather than faking Cloudflare success.",
      "CloudFront alias activation remains gated on verified DNS and issued certificate state.",
      "Failed checks retain bounded error text without exposing provider credentials or environment values.",
    ],
  },
  {
    id: "repo-pages-https-update",
    method: "PATCH",
    path: "/api/repos/{owner}/{repo}/settings/pages/https",
    title: "Update Pages HTTPS enforcement",
    description:
      "Toggles HTTPS enforcement for a verified custom domain once DNS and certificate prerequisites are satisfied.",
    auth: "Signed opengithub session cookie with repository admin or owner access",
    request: `{
  "enforced": true
}`,
    response: `{
  "site": {
    "customDomain": "docs.example.com",
    "httpsEnforced": true,
    "certificateStatus": "issued"
  }
}`,
    notes: [
      "Requests before domain verification or certificate issuance return validation_failed and do not update local UI state.",
      "Every successful toggle writes repository.pages.https.update.",
    ],
  },
  {
    id: "repo-pages-deployments-create",
    method: "POST",
    path: "/api/repos/{owner}/{repo}/settings/pages/deployments",
    title: "Request Pages deployment",
    description:
      "Queues a deployment from the saved branch source and returns Pages settings with the new deployment row.",
    auth: "Signed opengithub session cookie with repository admin or owner access",
    response: `{
  "deployments": [
    {
      "source": { "kind": "branch", "branch": "main", "folder": "/docs" },
      "status": "queued",
      "artifactManifest": {}
    }
  ]
}`,
    notes: [
      "POST /api/repos/{owner}/{repo}/settings/pages/actions-deployments links a confirmed Actions artifact deployment using the same response shape.",
      "The Pages worker records build logs, artifact manifests, storage keys, status transitions, and page_build webhook deliveries.",
      "Production deployments publish to S3 and CloudFront; local tests can use local_metadata storage with degraded cloud health notes.",
    ],
  },
  {
    id: "repo-pages-unpublish",
    method: "POST",
    path: "/api/repos/{owner}/{repo}/settings/pages/unpublish",
    title: "Unpublish Pages site",
    description:
      "Disables serving and deployment state while preserving repository source files, historical deployments, and audit history.",
    auth: "Signed opengithub session cookie with repository admin or owner access",
    response: `{
  "site": {
    "source": { "kind": "none", "branch": null, "folder": null },
    "provisioningStatus": "unpublished",
    "unpublishedAt": "2026-05-03T00:00:00Z"
  }
}`,
    notes: [
      "Unpublish never deletes repository Git objects, branch files, or Actions artifacts.",
      "CloudFront/S3 publication metadata is cleared or marked inactive, and repository.pages.unpublish is audited.",
    ],
  },
  {
    id: "repo-pulse",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/pulse?period=1w",
    title: "Repository Pulse insights",
    description:
      "Computes the repository Pulse activity snapshot for a bounded period, including overview metrics, top committers, release activity, merged pull requests, issue activity, linked metric destinations, and cache freshness metadata.",
    auth: "Public repositories are readable; private repositories require read permission; anonymous callers receive 401",
    response: `{
  "repository": {
    "ownerLogin": "mona",
    "name": "octo-app",
    "defaultBranch": "main",
    "viewerPermission": "write",
    "href": "/mona/octo-app"
  },
  "period": {
    "key": "1w",
    "label": "Last week",
    "startedAt": "2026-05-01T00:00:00Z",
    "endedAt": "2026-05-07T00:00:00Z"
  },
  "summary": {
    "sentence": "2 authors pushed 12 commits touching 18 files with 420 additions and 90 deletions in the 1w window.",
    "commits": 12,
    "filesChanged": 18,
    "additions": 420,
    "deletions": 90,
    "authors": 2,
    "mergedPullRequests": 4,
    "openPullRequests": 2,
    "closedIssues": 8,
    "newIssues": 3,
    "releases": 1
  },
  "metrics": [
    {
      "key": "merged_pull_requests",
      "label": "Merged pull requests",
      "count": 4,
      "href": "/mona/octo-app/pulls?state=merged&from=2026-05-01T00%3A00%3A00Z&until=2026-05-07T00%3A00%3A00Z"
    }
  ],
  "topCommitters": [
    {
      "login": "mona",
      "authorStatus": "active",
      "isBot": false,
      "commits": 9,
      "filesChanged": 12,
      "additions": 320,
      "deletions": 45,
      "profileHref": "/mona",
      "commitsHref": "/mona/octo-app/commits/main?author=mona&until=2026-05-07T00%3A00%3A00Z"
    }
  ],
  "releases": [],
  "mergedPullRequests": [],
  "issueActivity": [],
  "snapshot": {
    "cacheKey": "1w:202605010000:202605070000",
    "computedAt": "2026-05-07T00:00:00Z",
    "expiresAt": "2026-05-07T00:10:00Z",
    "stale": false
  }
}`,
    notes: [
      "Supported period values are 24h, 3d, 1w, and 1m; unsupported values return validation_failed without stack traces.",
      "Date bounds are normalized server-side and included in metric hrefs so browser cards navigate to filtered pull request or issue lists.",
      "Top committers include authorStatus and isBot metadata; unmatched or deleted authors are represented without exposing private user rows.",
      "repository_insight_snapshots stores the bounded snapshot payload and recent_insight_views records read telemetry; responses never expose storage keys, raw sessions, tokens, or environment secrets.",
      "Private repository outsiders receive not_found without leaking Pulse counts or cache metadata.",
    ],
  },
  {
    id: "repo-contributors",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/graphs/contributors?period=1w&start=2026-05-01T00:00:00Z&end=2026-05-07T00:00:00Z",
    title: "Repository Contributors insights",
    description:
      "Returns the screen-ready Contributors analytics contract for the default branch, including repository-wide weekly commit buckets, top contributor rows, period and range metadata, line-count threshold state, CSV-ready table values, profile and commit-history links, and cache freshness metadata.",
    auth: "Public repositories are readable by signed-in users; private repositories require read permission; anonymous callers receive 401",
    response: `{
  "repository": {
    "ownerLogin": "mona",
    "name": "octo-app",
    "defaultBranch": "main",
    "visibility": "public",
    "viewerPermission": "write",
    "href": "/mona/octo-app"
  },
  "period": {
    "key": "1w",
    "label": "Last week",
    "startedAt": "2026-05-01T00:00:00Z",
    "endedAt": "2026-05-07T00:00:00Z",
    "bucketCount": 2
  },
  "threshold": {
    "commitLimit": 5000,
    "commitsConsidered": 12,
    "lineCountsOmitted": false,
    "message": "Line additions and deletions are included for this bounded commit range."
  },
  "totals": {
    "commits": 12,
    "authors": 2,
    "additions": 420,
    "deletions": 90
  },
  "weeks": [
    {
      "weekStart": "2026-05-01T00:00:00Z",
      "weekEnd": "2026-05-04T00:00:00Z",
      "commits": 4,
      "additions": 120,
      "deletions": 30
    }
  ],
  "contributors": [
    {
      "userId": "user_01",
      "login": "mona",
      "authorStatus": "active",
      "isBot": false,
      "avatarUrl": null,
      "totalCommits": 9,
      "totalAdditions": 320,
      "totalDeletions": 45,
      "profileHref": "/mona",
      "commitsHref": "/mona/octo-app/commits/main?author=mona&since=2026-05-01T00%3A00%3A00Z&until=2026-05-07T00%3A00%3A00Z",
      "weeks": [
        {
          "weekStart": "2026-05-04T00:00:00Z",
          "commits": 6,
          "additions": 220,
          "deletions": 35
        }
      ]
    }
  ],
  "snapshot": {
    "cacheKey": "contributors:main:1w:202605010000:202605070000",
    "computedAt": "2026-05-07T00:00:00Z",
    "expiresAt": "2026-05-07T00:10:00Z",
    "stale": false
  }
}`,
    notes: [
      "The endpoint resolves analytics from the repository default branch through repository_git_refs; branch names with slashes are encoded in commit-history hrefs as a single reversible route segment.",
      "Supported period values are 24h, 3d, 1w, and 1m. Optional start and end range bounds are parsed as RFC3339 timestamps or dates, clipped to the selected period, and invalid ranges return validation_failed.",
      "Merge commits and empty commits are excluded. Repositories over the commitLimit keep commit counts but omit additions/deletions with null table values and a truthful threshold message.",
      "Contributor rows include authorStatus and isBot metadata for active, bot, and unmatched/deleted authors; unmatched authors link back to a repository-safe destination rather than leaking private user records.",
      "repository_contributors_weekly stores bounded rollups and repository_insight_snapshots stores cache freshness keyed by repository, default branch, period, and range. recent_insight_views records signed-in viewer telemetry only.",
      "Private repository outsiders receive not_found, anonymous callers receive 401, and error envelopes never include actor emails, OAuth data, session rows, tokens, storage keys, stack traces, or private commit OIDs.",
    ],
  },
  {
    id: "repo-traffic",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/graphs/traffic",
    title: "Repository Traffic insights",
    description:
      "Returns the permissioned Traffic analytics contract for repository users with push access, including the 14-day UTC clone and visitor series, summary totals, referrer rows, popular content rows, cache freshness metadata, and repository-safe permission errors.",
    auth: "Signed opengithub session cookie with repository write, admin, or owner access; read-only users receive 403 without traffic counts",
    response: `{
  "repository": {
    "ownerLogin": "mona",
    "name": "octo-app",
    "defaultBranch": "main",
    "visibility": "private",
    "viewerPermission": "write",
    "href": "/mona/octo-app"
  },
  "window": {
    "key": "14d",
    "label": "Last 14 days",
    "startedOn": "2026-04-24",
    "endedOn": "2026-05-07",
    "timezone": "UTC",
    "dayCount": 14,
    "clonesUpdateCadence": "hourly",
    "visitorsUpdateCadence": "hourly",
    "referrersUpdateCadence": "daily",
    "popularContentUpdateCadence": "daily",
    "internalTrafficExcluded": true
  },
  "summaries": {
    "clonesTotal": 42,
    "clonesUnique": 12,
    "visitorsTotal": 220,
    "visitorsUnique": 87,
    "referrersTotal": 2,
    "popularContentTotal": 2,
    "activeDays": 3,
    "hasTraffic": true
  },
  "clones": [
    { "date": "2026-05-07", "total": 18, "unique": 4 }
  ],
  "visitors": [
    { "date": "2026-05-07", "total": 70, "unique": 24 }
  ],
  "referrers": [
    {
      "referrer": "https://search.opengithub.local/results?q=traffic",
      "href": "https://search.opengithub.local/results?q=traffic",
      "totalViews": 120,
      "uniqueVisitors": 44
    }
  ],
  "popularContent": [
    {
      "path": "docs/traffic report.md",
      "title": "Traffic report",
      "href": "/mona/octo-app/blob/main/docs/traffic%20report.md",
      "totalViews": 16,
      "uniqueVisitors": 7
    }
  ],
  "snapshot": {
    "cacheKey": "traffic:repo-1:20260424:20260507",
    "computedAt": "2026-05-07T01:00:00Z",
    "expiresAt": "2026-05-07T02:00:00Z",
    "stale": false
  }
}`,
    notes: [
      "The endpoint always returns a 14-day UTC window. Clone and visitor series update hourly; referrers and popular content update daily.",
      "Traffic is visible only to users with push access. Anonymous callers receive 401, private outsiders receive not_found, and signed-in read-only users receive traffic_access_required with countsVisible=false.",
      "Clone and visitor arrays include zero-filled sparse days so browser charts and data-table fallbacks can render stable keyboard-focusable points for exact date, total, and unique values.",
      "Referrer rows are sorted by total views, unique visitors, then referrer label; external hrefs are normalized for safe browser anchors with noopener noreferrer.",
      "Popular content rows link to repository blob paths on the default branch. Long paths and unsafe-looking text are returned as text, not HTML.",
      "repository_traffic_daily, repository_referrers_daily, and repository_popular_content_daily store bounded rollups; repository_insight_snapshots stores the cache payload and recent_insight_views records signed-in viewer telemetry.",
      "Error envelopes never include traffic counts, actor emails, OAuth data, session rows, tokens, storage keys, stack traces, environment secrets, or private commit OIDs.",
    ],
  },
  {
    id: "repo-security-overview",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/security",
    title: "Repository Security overview",
    description:
      "Returns the screen-ready Security and quality overview, including sanitized SECURITY.md preview content, repository security feature cards, published advisory rows, viewer permissions, and private-count redaction metadata.",
    auth: "Signed opengithub session cookie with repository read permission; private outsiders receive not_found",
    response: `{
  "repository": {
    "ownerLogin": "mona",
    "name": "octo-app",
    "visibility": "private",
    "defaultBranch": "main",
    "securityHref": "/mona/octo-app/security",
    "policyHref": "/mona/octo-app/security/policy",
    "advisoriesHref": "/mona/octo-app/security/advisories"
  },
  "viewer": {
    "permission": "read",
    "canRead": true,
    "canWrite": false,
    "canEditPolicy": false,
    "canViewPrivateAlertCounts": false
  },
  "policy": {
    "exists": true,
    "path": "SECURITY.md",
    "ref": "main",
    "contentSha": "sha256:policy",
    "html": "<h1 id=\\"security-policy\\">Security policy</h1>",
    "sourceHref": "/mona/octo-app/blob/main/SECURITY.md",
    "rawHref": "/mona/octo-app/raw/main/SECURITY.md",
    "historyHref": "/mona/octo-app/commits/main/SECURITY.md",
    "editHref": null
  },
  "features": [
    {
      "key": "dependabot",
      "label": "Dependabot",
      "status": "enabled",
      "summary": "Dependency alerts are monitored.",
      "alertCount": null,
      "privateCount": null,
      "href": "/mona/octo-app/security/dependabot"
    }
  ],
  "advisories": [
    {
      "identifier": "GHSA-demo-2026",
      "severity": "high",
      "status": "published",
      "title": "Demo package vulnerability",
      "href": "/mona/octo-app/security/advisories/GHSA-demo-2026"
    }
  ]
}`,
    notes: [
      "Anonymous callers receive 401; private repository outsiders receive not_found without policy, feature, advisory, alert-count, or cache metadata.",
      "Published advisories are returned newest first. Draft advisories remain hidden from overview readers until published.",
      "Read-only viewers receive public policy and published advisory data, but alertCount and privateCount are null. Maintainers receive concrete private counts and policy edit hrefs.",
      "Policy Markdown is rendered through the Rust sanitizer. Script tags, unsafe URLs, raw session rows, OAuth data, storage keys, environment secrets, and stack traces are never returned.",
    ],
  },
  {
    id: "repo-security-advisories-list",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/security/advisories?state=published&severity=high&page=1&page_size=10",
    title: "List repository security advisories",
    description:
      "Returns the repository advisory list used by the Security and quality Advisories tab, including URL-backed filters, pagination, draft privacy, package metadata, author summaries, and viewer write affordances.",
    auth: "Signed opengithub session cookie with repository read permission; draft rows require maintainer or advisory collaborator access",
    response: `{
  "repository": {
    "ownerLogin": "mona",
    "name": "octo-app",
    "advisoriesHref": "/mona/octo-app/security/advisories"
  },
  "viewer": {
    "canRead": true,
    "canWrite": true,
    "canCreate": true
  },
  "filters": {
    "state": "published",
    "severity": "high",
    "query": "",
    "sort": "recently_updated",
    "page": 1,
    "pageSize": 10
  },
  "advisories": [
    {
      "id": "adv_01",
      "ghsaId": "GHSA-demo-2026",
      "cveId": "CVE-2026-1234",
      "state": "published",
      "severity": "high",
      "title": "Token scope bypass",
      "summary": "Repository imports could retain a broad token scope.",
      "href": "/mona/octo-app/security/advisories/GHSA-demo-2026",
      "author": { "login": "mona", "profileHref": "/mona" },
      "package": {
        "ecosystem": "cargo",
        "name": "opengithub-import",
        "affectedVersions": "< 1.2.3",
        "patchedVersions": ">= 1.2.3"
      }
    }
  ],
  "pagination": { "total": 1, "page": 1, "pageSize": 10, "hasNext": false }
}`,
    notes: [
      "Supported states are published, draft, and withdrawn; supported severities are critical, high, moderate, low, and unknown. Invalid filters return validation_failed with field-level messages for inline rendering.",
      "Readers only receive published advisories. Draft advisories, draft counts, collaborators, and private repository metadata stay hidden from unauthorised viewers.",
      "Rows are sorted by recently_updated by default. Search matches title, GHSA id, CVE id, package ecosystem/name, and summary without reflecting raw HTML.",
      "Error envelopes never include session cookies, OAuth profile payloads, package storage keys, private fork names, environment variables, or stack traces.",
    ],
  },
  {
    id: "repo-security-advisory-detail",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/security/advisories/{ghsa_id}",
    title: "Get repository security advisory detail",
    description:
      "Returns the full advisory detail contract with sanitized Markdown, CVSS score and base metrics, CVE/CWE disclosures, package version ranges, credits, collaborators, timeline rows, and maintainer action affordances.",
    auth: "Signed opengithub session cookie with repository read permission; draft detail requires maintainer or advisory collaborator access",
    response: `{
  "advisory": {
    "id": "adv_01",
    "ghsaId": "GHSA-demo-2026",
    "cveId": "CVE-2026-1234",
    "state": "published",
    "severity": "high",
    "title": "Token scope bypass",
    "summary": "Repository imports could retain a broad token scope.",
    "html": "<p>Upgrade before running imports.</p>",
    "cvss": {
      "vector": "CVSS:3.1/AV:N/AC:L/PR:L/UI:N/S:U/C:H/I:H/A:N",
      "score": 8.1,
      "metrics": { "attackVector": "network", "privilegesRequired": "low" }
    },
    "cwes": [{ "id": "CWE-284", "name": "Improper Access Control" }],
    "credits": [{ "login": "security-reporter", "creditType": "reporter" }],
    "collaborators": [{ "login": "mona", "role": "author" }],
    "timeline": [{ "eventType": "published", "message": "Published advisory GHSA-demo-2026" }]
  },
  "viewer": { "canEdit": true, "canPublish": false }
}`,
    notes: [
      "Private repository outsiders and unauthorised draft readers receive not_found without confirming whether the advisory exists.",
      "Markdown is sanitized on write and returned as safe HTML plus raw Markdown for authorised editors. Unsafe tags, event handlers, javascript URLs, and raw HTML reflections are stripped.",
      "CVSS, CWE, CVE, package ecosystem/name, affected version, patched version, credit, and collaborator fields are normalised for the browser detail and edit flows.",
      "Response payloads never expose session rows, OAuth data, notification internals, audit redaction payloads, private fork storage identifiers, or environment secrets.",
    ],
  },
  {
    id: "repo-security-advisory-create",
    method: "POST",
    path: "/api/repos/{owner}/{repo}/security/advisories",
    title: "Create draft repository security advisory",
    description:
      "Creates a private draft advisory with a generated GHSA-local identifier, optional package/CVE/CVSS/CWE metadata, initial author collaboration, sanitized Markdown, notifications, and audit events.",
    auth: "Signed opengithub session cookie with maintainer permission",
    request: `{
  "title": "Token scope bypass",
  "summary": "Repository imports could retain a broad token scope.",
  "details": "Upgrade before running imports.",
  "severity": "high",
  "cveId": "CVE-2026-1234",
  "packageEcosystem": "cargo",
  "packageName": "opengithub-import",
  "affectedVersions": "< 1.2.3",
  "patchedVersions": ">= 1.2.3",
  "cvssVector": "CVSS:3.1/AV:N/AC:L/PR:L/UI:N/S:U/C:H/I:H/A:N",
  "cvssScore": 8.1,
  "cwes": [{ "id": "CWE-284", "name": "Improper Access Control" }],
  "credits": [{ "login": "security-reporter", "creditType": "reporter" }]
}`,
    response: `{
  "advisory": {
    "ghsaId": "GHSA-local-01",
    "state": "draft",
    "title": "Token scope bypass",
    "href": "/mona/octo-app/security/advisories/GHSA-local-01"
  }
}`,
    notes: [
      "Title is required. Invalid CVE, CVSS, CWE, package, credit, collaborator, and version-range fields return validation_failed with stable field keys.",
      "Drafts remain private to maintainers and advisory collaborators until published; readers cannot infer draft count changes from public list responses.",
      "Creation writes a redacted security_audit_events row and timeline entry and notifies advisory collaborators when present.",
      "The endpoint does not call GitHub APIs and never returns raw session cookies, OAuth profile payloads, notification secrets, storage keys, or stack traces.",
    ],
  },
  {
    id: "repo-security-advisory-update",
    method: "PATCH",
    path: "/api/repos/{owner}/{repo}/security/advisories/{ghsa_id}",
    title: "Update repository security advisory metadata",
    description:
      "Updates draft or published advisory metadata and collaboration fields with maintainer permission, server-side validation, sanitized Markdown refresh, timeline entries, notifications, and redacted audit events.",
    auth: "Signed opengithub session cookie with maintainer permission",
    request: `{
  "title": "Updated token scope bypass",
  "severity": "critical",
  "cveId": "CVE-2026-20002",
  "packageEcosystem": "cargo",
  "packageName": "opengithub-import",
  "affectedVersions": "< 1.2.4",
  "patchedVersions": ">= 1.2.4",
  "cvssScore": 9.1,
  "cwes": [{ "id": "CWE-862", "name": "Missing Authorization" }],
  "collaborators": [{ "login": "mona", "role": "author" }]
}`,
    response: `{
  "advisory": {
    "ghsaId": "GHSA-demo-2026",
    "state": "published",
    "severity": "critical",
    "title": "Updated token scope bypass"
  }
}`,
    notes: [
      "Read-only users receive 403 and archived repositories reject mutation attempts. Missing advisories and private draft misses return not_found.",
      "Validation errors use the standard envelope; stale or duplicate GHSA/CVE conflicts are reported without leaking private advisory titles.",
      "Credits and collaborators are replaced from the validated request in one transaction so the browser never sees a partially updated advisory.",
      "Audit and notification rows redact Markdown bodies, session identifiers, OAuth data, environment values, and storage details.",
    ],
  },
  {
    id: "repo-security-advisory-publish",
    method: "POST",
    path: "/api/repos/{owner}/{repo}/security/advisories/{ghsa_id}/publish",
    title: "Publish repository security advisory",
    description:
      "Publishes a draft advisory to the public repository advisory list after readiness validation, records immutable publish metadata, sends collaborator notifications, writes audit/timeline rows, and links dependency advisory feed data when package metadata is present.",
    auth: "Signed opengithub session cookie with maintainer permission",
    response: `{
  "advisory": {
    "ghsaId": "GHSA-local-01",
    "state": "published",
    "publishedAt": "2026-05-05T12:00:00Z",
    "href": "/mona/octo-app/security/advisories/GHSA-local-01"
  }
}`,
    notes: [
      "Only draft advisories can be published. Re-publish attempts, missing titles, invalid package metadata, and package advisories without patched-version guidance return validation_failed.",
      "After publication, readers can view the advisory from the list and detail endpoints; draft collaborator-only fields remain redacted.",
      "Publishing creates security_audit_events, advisory timeline rows, notification fanout for collaborators/watchers, and dependency advisory feed rows when package metadata is present.",
      "The publish response never exposes private fork references, raw notification payloads, session cookies, OAuth data, storage keys, environment secrets, or stack traces.",
    ],
  },
  {
    id: "repo-security-policy",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/security/policy",
    title: "Repository Security policy",
    description:
      "Returns the dedicated SECURITY.md reader contract with sanitized Markdown HTML, heading outline anchors, source/raw/history/edit destinations, latest commit metadata, and reader or maintainer empty states.",
    auth: "Signed opengithub session cookie with repository read permission; private outsiders receive not_found",
    response: `{
  "policy": {
    "exists": true,
    "path": "SECURITY.md",
    "ref": "main",
    "blobOid": "blob_01",
    "contentSha": "sha256:policy",
    "markdown": "# Security policy",
    "html": "<h1 id=\\"security-policy\\">Security policy</h1>",
    "outline": [
      { "id": "security-policy", "level": 1, "text": "Security policy", "href": "#security-policy" }
    ],
    "sourceHref": "/mona/octo-app/blob/main/SECURITY.md",
    "rawHref": "/mona/octo-app/raw/main/SECURITY.md",
    "historyHref": "/mona/octo-app/commits/main/SECURITY.md",
    "editHref": null,
    "latestCommit": {
      "shortOid": "abcdef1",
      "message": "Publish security policy",
      "href": "/mona/octo-app/commit/abcdef1234567890"
    }
  }
}`,
    notes: [
      "The API discovers SECURITY.md from the default branch and preserves supported policy path precedence: SECURITY.md, .github/SECURITY.md, then docs/SECURITY.md.",
      "Relative Markdown links are rewritten to repository blob destinations on the same ref; mailto links remain mailto; unsafe HTML is stripped before the browser receives it.",
      "Maintainers see editHref for setup or editing. Readers see a read-only missing-policy message when no policy exists.",
      "Responses never include private alert counts, raw Git object storage locations, session cookies, OAuth tokens, environment secrets, or stack traces.",
    ],
  },
  {
    id: "repo-security-policy-create",
    method: "POST",
    path: "/api/repos/{owner}/{repo}/security/policy",
    title: "Create repository Security policy",
    description:
      "Creates SECURITY.md through the repository file materialization path, writes a commit, advances the target branch ref, refreshes repository_security_policies, and records a redacted security audit event.",
    auth: "Signed opengithub session cookie with repository write, admin, or owner access",
    request: `{
  "markdown": "# Security policy\\n\\nReport issues privately.",
  "commitMessage": "Create security policy",
  "path": "SECURITY.md",
  "ref": "main"
}`,
    response: `{
  "policy": {
    "exists": true,
    "path": "SECURITY.md",
    "ref": "main",
    "contentSha": "sha256:new-policy",
    "sourceHref": "/mona/octo-app/blob/main/SECURITY.md",
    "rawHref": "/mona/octo-app/raw/main/SECURITY.md",
    "latestCommit": {
      "message": "Create security policy",
      "href": "/mona/octo-app/commit/abcdef1234567890"
    }
  }
}`,
    notes: [
      "Allowed policy paths are SECURITY.md, .github/SECURITY.md, and docs/SECURITY.md. Blank Markdown, blank commit messages, invalid refs, and archived repositories return validation_failed.",
      "The write updates repository_files, git_objects, commits, repository_git_refs, repository_security_policies, and security_audit_events atomically so Code, blob, raw, history, and policy pages reflect the same file.",
      "Users without write access receive 403. The MVP does not create propose-change branches for read-only users.",
      "Validation and error envelopes never echo submitted secrets or include session rows, OAuth data, storage keys, environment secrets, or stack traces.",
    ],
  },
  {
    id: "repo-security-policy-update",
    method: "PATCH",
    path: "/api/repos/{owner}/{repo}/security/policy",
    title: "Update repository Security policy",
    description:
      "Updates an existing SECURITY.md file through the same repository write path, enforcing content-SHA freshness before writing the commit and branch ref.",
    auth: "Signed opengithub session cookie with repository write, admin, or owner access",
    request: `{
  "markdown": "# Security policy\\n\\nEmail security@example.com.",
  "commitMessage": "Update security policy",
  "expectedContentSha": "sha256:previous-policy"
}`,
    response: `{
  "policy": {
    "exists": true,
    "contentSha": "sha256:updated-policy",
    "markdown": "# Security policy\\n\\nEmail security@example.com.",
    "html": "<h1 id=\\"security-policy\\">Security policy</h1>",
    "latestCommit": {
      "message": "Update security policy",
      "href": "/mona/octo-app/commit/fedcba9876543210"
    }
  }
}`,
    notes: [
      "expectedContentSha protects concurrent edits. Stale updates return conflict without writing a commit or moving the branch ref.",
      "Archived repositories reject updates with validation_failed; invalid path/ref/Markdown/commit-message inputs use stable validation_failed envelopes.",
      "Successful updates write repository.security_policy.upsert audit events with redacted metadata only.",
      "The response is the same sanitized policy view used by GET /api/repos/{owner}/{repo}/security/policy and never leaks secrets, raw storage details, private alert counts, or stack traces.",
    ],
  },
  {
    id: "repo-network",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/network",
    title: "Repository Network insights",
    description:
      "Returns the screen-ready repository Network graph contract for the 50 most recently pushed readable forks, including source repository metadata, bounded projection freshness, hidden private fork counts, and concrete fork, tree, issues, pulls, and network hrefs.",
    auth: "Public repositories are readable by signed-in users; private repositories require read permission; anonymous callers receive 401",
    response: `{
  "repository": {
    "ownerLogin": "mona",
    "name": "octo-app",
    "defaultBranch": "release/main",
    "visibility": "private",
    "viewerPermission": "read",
    "href": "/mona/octo-app",
    "treeHref": "/mona/octo-app/tree/release%2Fmain"
  },
  "summary": {
    "totalReadableForks": 12,
    "projectedForks": 12,
    "hiddenPrivateForks": 2,
    "copy": "Network graph shows the most recently pushed readable forks in this repository network.",
    "updateNote": "Repository network projections refresh daily from fork and branch activity."
  },
  "forks": [
    {
      "repositoryId": "repo_fork_01",
      "ownerLogin": "ashley",
      "name": "octo-app-labs",
      "visibility": "public",
      "defaultBranch": "release/main",
      "isArchived": false,
      "isStarredByActor": true,
      "starsCount": 3,
      "forksCount": 1,
      "openIssuesCount": 2,
      "openPullRequestsCount": 1,
      "createdAt": "2026-04-30T00:00:00Z",
      "updatedAt": "2026-05-06T00:00:00Z",
      "pushedAt": "2026-05-06T12:00:00Z",
      "href": "/ashley/octo-app-labs",
      "ownerHref": "/ashley",
      "treeHref": "/ashley/octo-app-labs/tree/release%2Fmain",
      "networkHref": "/ashley/octo-app-labs/network"
    }
  ],
  "freshness": {
    "computedAt": "2026-05-07T01:00:00Z",
    "expiresAt": "2026-05-08T01:00:00Z",
    "stale": false,
    "cadence": "daily"
  },
  "links": {
    "forksHref": "/mona/octo-app/forks",
    "treeHref": "/mona/octo-app/tree/release%2Fmain"
  }
}`,
    notes: [
      "The Network graph is limited to the 50 most recently pushed readable forks so private forks, inaccessible fork names, and private owner metadata are never leaked.",
      "repository_network_forks stores bounded daily projection rows derived from repository_forks, repository_git_refs, commits, stars, issues, and pull_requests; reads may refresh stale projections before responding.",
      "Branch names with slashes are encoded as single reversible route segments in treeHref values, preserving links such as release/main without mutating upstream data.",
      "Private repository outsiders receive not_found, anonymous callers receive 401, and error envelopes never include actor emails, OAuth data, session rows, tokens, storage keys, stack traces, environment secrets, or private commit OIDs.",
    ],
  },
  {
    id: "repo-forks",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/forks?period=1m&type=starred&sort=most_starred",
    title: "Repository Forks list",
    description:
      "Returns the filterable Forks list contract for a repository network, including period/type/sort state, actor-scoped saved-default metadata, dense fork rows, hidden private fork counts, and daily projection freshness.",
    auth: "Public repositories are readable by signed-in users; private repositories require read permission; anonymous callers receive 401",
    response: `{
  "repository": {
    "ownerLogin": "mona",
    "name": "octo-app",
    "defaultBranch": "release/main",
    "visibility": "private",
    "viewerPermission": "read",
    "href": "/mona/octo-app",
    "treeHref": "/mona/octo-app/tree/release%2Fmain"
  },
  "filters": {
    "period": {
      "key": "1m",
      "label": "Last month",
      "startedAt": "2026-04-07T00:00:00Z",
      "endedAt": "2026-05-07T00:00:00Z"
    },
    "repositoryType": "starred",
    "sort": "most_starred"
  },
  "defaults": {
    "saved": true,
    "matchesCurrent": true,
    "periodKey": "1m",
    "repositoryType": "starred",
    "sortKey": "most_starred",
    "savedAt": "2026-05-07T00:30:00Z"
  },
  "total": 1,
  "hiddenPrivateForks": 2,
  "forks": [],
  "freshness": {
    "computedAt": "2026-05-07T01:00:00Z",
    "expiresAt": "2026-05-08T01:00:00Z",
    "stale": false,
    "cadence": "daily"
  },
  "links": {
    "forksHref": "/mona/octo-app/forks",
    "treeHref": "/mona/octo-app/tree/release%2Fmain"
  }
}`,
    notes: [
      "Supported period values are 24h, 3d, 1w, 1m, and all. Supported type filters are all, active, inactive, archived, and starred.",
      "Supported sort values are most_starred, recently_pushed, recently_created, recently_updated, and name_asc. Invalid filter values return validation_failed without stack traces.",
      "Rows include archived, inactive, and starred badges plus created, updated, and pushed timestamps. Metric links point to repository-safe destinations and never mutate fork or upstream data.",
      "Readable-only filtering is enforced before totals and row serialization. hiddenPrivateForks reports omitted forks without exposing private fork names, owner logins, branches, or metric counts.",
    ],
  },
  {
    id: "repo-stargazers",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/stargazers?page=1&pageSize=30",
    title: "Repository stargazers",
    description:
      "Returns the paginated users who starred a readable repository, powering /{owner}/{repo}/stargazers and repository star-count links.",
    auth: "Public repositories are readable anonymously; private repositories require a signed session with read permission",
    response: `{
  "repository": {
    "ownerLogin": "mona",
    "name": "octo-app",
    "href": "/mona/octo-app"
  },
  "items": [
    {
      "id": "user-1",
      "login": "ashley",
      "name": "Ashley",
      "avatarUrl": null,
      "bio": "Maintainer",
      "href": "/ashley",
      "starredAt": "2026-05-07T00:00:00Z"
    }
  ],
  "total": 1,
  "page": 1,
  "pageSize": 30
}`,
    notes: [
      "Rows are ordered by newest star first and include only public profile metadata needed by the social list UI.",
      "Private repository outsiders receive not_found, and responses never expose email, session, OAuth, token, or storage metadata.",
    ],
  },
  {
    id: "repo-forks-defaults",
    method: "PUT",
    path: "/api/repos/{owner}/{repo}/forks/defaults",
    title: "Save repository Forks defaults",
    description:
      "Persists the signed-in actor's default Forks period, repository type, and sort choice for one source repository, then returns the refreshed Forks response metadata used by the browser Save defaults control.",
    auth: "Signed opengithub session cookie with repository read permission",
    request: `{
  "period": "all",
  "repositoryType": "starred",
  "sort": "recently_pushed"
}`,
    response: `{
  "defaults": {
    "saved": true,
    "matchesCurrent": true,
    "periodKey": "all",
    "repositoryType": "starred",
    "sortKey": "recently_pushed",
    "savedAt": "2026-05-07T00:45:00Z"
  }
}`,
    notes: [
      "The write is actor-scoped in saved_fork_filter_defaults and never mutates the upstream repository, fork repositories, or repository_network_forks projection rows.",
      "Validation uses the same period, repository type, and sort enums as GET /api/repos/{owner}/{repo}/forks.",
      "Anonymous callers receive 401, private outsiders receive not_found, and successful writes do not expose session cookies, OAuth provider data, token hashes, storage keys, stack traces, or private fork metadata.",
    ],
  },
  {
    id: "repo-dependency-graph-dependencies",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/network/dependencies?q=sqlx&ecosystem=cargo&relationship=direct",
    title: "Repository Dependency graph dependencies",
    description:
      "Returns the screen-ready Dependency graph Dependencies tab contract from supported default-branch manifests and lockfiles, including package rows, direct and transitive relationship state, advisory summaries, manifest links, filter state, freshness metadata, and SBOM export affordances.",
    auth: "Public repositories are readable by signed-in users; private repositories require read permission; anonymous callers receive 401",
    response: `{
  "repository": {
    "ownerLogin": "mona",
    "name": "octo-app",
    "defaultBranch": "main",
    "visibility": "private",
    "viewerPermission": "read",
    "href": "/mona/octo-app",
    "treeHref": "/mona/octo-app/tree/main"
  },
  "filters": {
    "query": "sqlx",
    "ecosystem": "cargo",
    "relationship": "direct"
  },
  "summary": {
    "total": 1,
    "directCount": 1,
    "transitiveCount": 0,
    "ecosystemCounts": [{ "ecosystem": "cargo", "count": 1 }],
    "manifestCount": 2,
    "advisoryCount": 0
  },
  "manifests": [
    {
      "path": "crates/api/Cargo.toml",
      "ecosystem": "cargo",
      "lockfilePath": "crates/api/Cargo.lock",
      "dependencyCount": 1,
      "href": "/mona/octo-app/blob/main/crates%2Fapi%2FCargo.toml",
      "lockfileHref": "/mona/octo-app/blob/main/crates%2Fapi%2FCargo.lock"
    }
  ],
  "dependencies": [
    {
      "package": {
        "ecosystem": "cargo",
        "name": "sqlx",
        "href": "/packages/cargo/sqlx"
      },
      "version": "0.8",
      "relationship": "direct",
      "license": null,
      "manifestPath": "crates/api/Cargo.toml",
      "manifestHref": "/mona/octo-app/blob/main/crates%2Fapi%2FCargo.toml",
      "lockfilePath": "crates/api/Cargo.lock",
      "advisories": [],
      "detailsHref": "/packages/cargo/sqlx",
      "advisoryHref": null
    }
  ],
  "availability": {
    "enabled": true,
    "indexed": true,
    "supportedEcosystems": ["npm", "cargo", "pip"],
    "message": "Dependency graph is indexed from supported manifest and lock files.",
    "unavailableReason": null
  },
  "export": {
    "supported": true,
    "href": "/api/repos/mona/octo-app/network/dependencies/sbom",
    "latestStatus": null
  }
}`,
    notes: [
      "Supported ecosystems are npm, cargo, and pip. Unsupported manifests are ignored truthfully; malformed supported manifests produce no rows instead of leaking parser internals.",
      "Supported relationship filters are direct and transitive. q is bounded to 120 characters and matches package name, version, or manifest path.",
      "Extraction reads repository_files from the resolved default branch, handles duplicate package declarations deterministically, and never calls the upstream GitHub API.",
      "Private repository outsiders receive not_found; dependency_graph_unavailable states use structured 422 envelopes for disabled or unindexed graphs.",
      "Responses never include raw session rows, OAuth data, token hashes, environment secrets, storage keys, stack traces, private repository paths, or private consumer names.",
    ],
  },
  {
    id: "repo-dependency-graph-sbom-export",
    method: "POST",
    path: "/api/repos/{owner}/{repo}/network/dependencies/sbom",
    title: "Create repository Dependency graph SBOM export",
    description:
      "Creates a ready SPDX JSON SBOM artifact from the current indexed dependency graph, records repository audit metadata, and returns a signed download affordance for the browser Export SBOM flow.",
    auth: "Signed opengithub session cookie with repository read permission",
    response: `{
  "id": "export_01",
  "status": "ready",
  "format": "spdx-json",
  "artifactSha256": "b7f2...",
  "artifactByteSize": 4096,
  "downloadHref": "/api/repos/mona/octo-app/network/dependencies/sbom/export_01",
  "pollHref": "/api/repos/mona/octo-app/network/dependencies/sbom/export_01",
  "expiresAt": "2026-05-06T00:00:00Z",
  "createdAt": "2026-05-05T00:00:00Z",
  "completedAt": "2026-05-05T00:00:01Z"
}`,
    notes: [
      "Exports are generated from currently indexed rows, not mock data, and include SPDX-2.3 package and relationship sections.",
      "Empty or unindexed graphs return dependency_graph_unavailable with a 422 status so the browser can show a disabled export state.",
      "Successful exports write dependency_graph.sbom_export repository_settings_audit_events with package and manifest counts.",
      "The response includes artifact hashes and byte size, but never returns raw storage keys, session cookies, OAuth data, token hashes, environment secrets, stack traces, or private package rows.",
    ],
  },
  {
    id: "repo-dependency-graph-sbom-download",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/network/dependencies/sbom/{export_id}",
    title: "Download repository Dependency graph SBOM export",
    description:
      "Downloads the ready SPDX JSON artifact for an SBOM export that belongs to the requested repository.",
    auth: "Signed opengithub session cookie with repository read permission",
    response: `{
  "spdxVersion": "SPDX-2.3",
  "dataLicense": "CC0-1.0",
  "SPDXID": "SPDXRef-DOCUMENT",
  "name": "mona/octo-app dependency graph",
  "packages": [],
  "relationships": []
}`,
    notes: [
      "Ready downloads return application/json with an attachment Content-Disposition filename.",
      "Unknown export IDs return not_found within the repository scope and cannot be used to discover exports from another private repository.",
      "Expired or non-ready artifacts remain metadata-only until regenerated by POST /api/repos/{owner}/{repo}/network/dependencies/sbom.",
      "Downloaded artifacts are derived from package names, versions, licenses, and manifest paths only; they never include OAuth data, raw session rows, token hashes, storage keys, environment secrets, stack traces, or private dependent repository names.",
    ],
  },
  {
    id: "repo-dependency-graph-dependents",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/network/dependents?package=npm%3A%40namuh%2Fflow&owner=acme",
    title: "Repository Dependency graph dependents",
    description:
      "Returns the public Dependents tab contract for packages indexed from a public source repository, including package filter options, owner filtering, approximate counts, hidden private counts, and dependent repository rows.",
    auth: "Signed opengithub session cookie; source repository must be public and readable",
    response: `{
  "repository": {
    "ownerLogin": "mona",
    "name": "octo-app",
    "visibility": "public",
    "viewerPermission": "read",
    "href": "/mona/octo-app"
  },
  "filters": {
    "package": "npm:@namuh/flow",
    "owner": "acme"
  },
  "summary": {
    "repositoryCount": 1,
    "packageCount": 1,
    "hiddenPrivateCount": 2,
    "approximate": true
  },
  "packages": [
    {
      "package": {
        "ecosystem": "npm",
        "name": "@namuh/flow",
        "href": "/packages/npm/%40namuh%2Fflow"
      },
      "dependentCount": 1,
      "selected": true
    }
  ],
  "dependents": [
    {
      "ownerLogin": "acme",
      "name": "workflow-tools",
      "visibility": "public",
      "package": {
        "ecosystem": "npm",
        "name": "@namuh/flow"
      },
      "manifestPath": "package.json",
      "href": "/acme/workflow-tools",
      "ownerHref": "/acme",
      "packageHref": "/packages/npm/%40namuh%2Fflow"
    }
  ]
}`,
    notes: [
      "Dependents are shown only for public source repositories. Private source repositories return dependency_graph_unavailable with 422, even when the actor can read the source.",
      "The package filter accepts either a package name or ecosystem:name; owner is bounded to 80 URL-safe username characters.",
      "Dependent rows are limited to public repositories. Private consumers contribute only to hiddenPrivateCount and are never named in rows, links, empty states, or errors.",
      "Counts are approximate because they are derived from public indexed dependency graph rows and explicit dependent projection rows.",
      "Responses never include private repository names, private owner logins, raw session rows, OAuth data, token hashes, storage keys, environment secrets, stack traces, or private manifest contents.",
    ],
  },
  {
    id: "repo-dependabot-alerts",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/security/dependabot?state=open&package=npm%3A%40playwright%2Ftest&sort=most_important",
    title: "List repository Dependabot alerts",
    description:
      "Returns the screen-ready Dependabot alerts list derived from indexed dependency graph rows and security advisories, including filter metadata, open/closed counts, selectable alert rows, disabled states, and viewer write permissions.",
    auth: "Signed opengithub session cookie with repository read permission; private outsiders receive not_found",
    response: `{
  "availability": {
    "enabled": true,
    "indexed": true,
    "message": "Dependabot alerts are monitored.",
    "settingsHref": "/mona/octo-app/settings/security"
  },
  "filters": {
    "state": "open",
    "query": null,
    "package": "npm:@playwright/test",
    "ecosystem": null,
    "manifest": null,
    "scope": null,
    "severity": null,
    "sort": "most_important"
  },
  "counts": { "open": 2, "closed": 1, "total": 3, "visible": 1 },
  "alerts": [
    {
      "id": "alert_01",
      "number": 1,
      "state": "open",
      "scope": "production",
      "package": {
        "ecosystem": "npm",
        "name": "@playwright/test",
        "href": "/packages/npm/%40playwright%2Ftest"
      },
      "advisory": {
        "identifier": "GHSA-demo-0001",
        "severity": "high",
        "title": "Playwright test runner demo advisory",
        "href": "/advisories/GHSA-demo-0001"
      },
      "manifestPath": "package.json",
      "manifestHref": "/mona/octo-app/blob/main/package.json",
      "fixedVersion": "1.56.0",
      "href": "/mona/octo-app/security/dependabot/1"
    }
  ]
}`,
    notes: [
      "Alerts are materialized from repository_dependencies joined to dependency_advisories; the endpoint does not use fake data or call the upstream GitHub API.",
      "Supported filters are state, q, package, ecosystem, manifest, scope, severity, and sort. Invalid filter values return validation_failed with a 422 status.",
      "Disabled repository settings return an enabled=false availability payload with no rows and a concrete settingsHref for maintainers.",
      "Rows include concrete package, manifest, and alert detail destinations; private repository access failures are redacted as not_found.",
    ],
  },
  {
    id: "repo-dependabot-alert-detail",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/security/dependabot/{alert_id}",
    title: "Read repository Dependabot alert detail",
    description:
      "Returns one Dependabot alert detail view with vulnerable dependency metadata, advisory data, affected and fixed versions, assignee options, timeline rows, and security-update PR affordances.",
    auth: "Signed opengithub session cookie with repository read permission",
    response: `{
  "alert": {
    "id": "alert_01",
    "number": 1,
    "state": "open",
    "scope": "production",
    "vulnerableRequirements": "< 1.56.0",
    "currentVersion": "1.55.0",
    "fixedVersion": "1.56.0"
  },
  "advisory": {
    "identifier": "GHSA-demo-0001",
    "severity": "high",
    "title": "Playwright test runner demo advisory",
    "vulnerableRange": "< 1.56.0"
  },
  "dependency": {
    "package": { "ecosystem": "npm", "name": "@playwright/test" },
    "manifestPath": "package.json",
    "manifestHref": "/mona/octo-app/blob/main/package.json",
    "currentVersion": "1.55.0",
    "relationship": "direct"
  },
  "timeline": [
    { "eventType": "created", "message": "Dependabot opened this alert from the dependency graph." }
  ],
  "securityUpdate": {
    "supported": true,
    "status": "available",
    "href": "/api/repos/mona/octo-app/security/dependabot/1/security-update"
  }
}`,
    notes: [
      "Readers can view detail data but receive forbidden on mutation routes.",
      "Timeline rows come from security_alert_events and redacted audit metadata; no private payloads, session values, token hashes, or provider secrets are serialized.",
      "securityUpdate reports unsupported truthfully when the ecosystem or manifest cannot be edited deterministically.",
    ],
  },
  {
    id: "repo-dependabot-alert-update",
    method: "PATCH",
    path: "/api/repos/{owner}/{repo}/security/dependabot/{alert_id}",
    title: "Update repository Dependabot alert",
    description:
      "Lets maintainers dismiss, reopen, or assign one Dependabot alert while recording security alert timeline events, security audit events, and notification updates.",
    auth: "Signed opengithub session cookie with repository maintainer permission",
    request: `{
  "action": "dismiss",
  "dismissalReason": "not_used",
  "dismissalComment": "Only a browser smoke fixture uses this dependency."
}`,
    response: `{
  "alert": {
    "id": "alert_01",
    "number": 1,
    "state": "dismissed",
    "dismissalReason": "not_used"
  },
  "timeline": [
    { "eventType": "dismissed", "message": "Dismissed this alert as not_used." }
  ]
}`,
    notes: [
      "Supported actions are dismiss, reopen, and assign. Dismiss requires a bounded dismissalReason and optional bounded comment.",
      "Archived repositories, disabled Dependabot settings, invalid assignees, fixed-alert reopen attempts, and malformed state transitions return structured validation or conflict errors.",
      "Successful writes update dependabot_alerts, security_alert_events, security_audit_events, and assignee notification rows atomically.",
    ],
  },
  {
    id: "repo-dependabot-alerts-bulk",
    method: "POST",
    path: "/api/repos/{owner}/{repo}/security/dependabot/bulk",
    title: "Bulk update repository Dependabot alerts",
    description:
      "Dismisses or reopens selected Dependabot alerts from the list page and returns per-alert results for the browser bulk triage controls.",
    auth: "Signed opengithub session cookie with repository maintainer permission",
    request: `{
  "action": "dismiss",
  "alertIds": ["alert_01", "alert_02"],
  "dismissalReason": "fix_started",
  "dismissalComment": "Tracked in the security update queue."
}`,
    response: `{
  "requestedCount": 2,
  "updatedCount": 2,
  "results": [
    { "alertId": "alert_01", "state": "dismissed", "href": "/mona/octo-app/security/dependabot/1" }
  ],
  "message": "2 Dependabot alerts updated."
}`,
    notes: [
      "alertIds must be non-empty, deduplicated, and scoped to the requested repository.",
      "Mixed results keep failed rows addressable without revealing hidden private repositories or alerts.",
      "Each successful row writes timeline, audit, and notification updates with redacted metadata.",
    ],
  },
  {
    id: "repo-dependabot-security-update",
    method: "POST",
    path: "/api/repos/{owner}/{repo}/security/dependabot/{alert_id}/security-update",
    title: "Create Dependabot security update pull request",
    description:
      "Creates or reuses a deterministic security update branch and pull request for a supported open Dependabot alert using the repository snapshot and existing pull request contracts.",
    auth: "Signed opengithub session cookie with repository maintainer permission",
    response: `{
  "status": "created",
  "branch": "dependabot/npm/playwright-test-1",
  "commitOid": "abc123",
  "pullRequestHref": "/mona/octo-app/pull/12",
  "message": "Security update pull request created."
}`,
    notes: [
      "Supported npm manifest updates edit the default-branch snapshot, create a commit/ref, open a pull request, and link dependabot_alerts.security_update_pull_request_id.",
      "Repeated requests return the existing linked pull request instead of creating duplicates.",
      "Unsupported ecosystems, missing fixed versions, archived repositories, closed alerts, and disabled settings return truthful error or unsupported states.",
    ],
  },
  {
    id: "repo-code-scanning-alerts",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/security/code-scanning?state=open&tool=CodeQL&sort=most_important",
    title: "List repository Code scanning alerts",
    description:
      "Returns the screen-ready Code scanning alerts list from SARIF and Actions analysis, including filter metadata, tool status summaries, open/closed counts, selectable alert rows, disabled states, and viewer write permissions.",
    auth: "Signed opengithub session cookie with repository read permission; private outsiders receive not_found",
    response: `{
  "availability": {
    "enabled": true,
    "indexed": true,
    "message": "Code scanning alerts are normalized from SARIF analysis.",
    "settingsHref": "/mona/octo-app/settings/security"
  },
  "filters": {
    "state": "open",
    "query": null,
    "severity": null,
    "securitySeverity": null,
    "tool": "CodeQL",
    "branch": null,
    "ref": null,
    "tag": null,
    "applicationCode": "true",
    "sort": "most_important"
  },
  "counts": { "open": 2, "closed": 1, "fixed": 1, "total": 4, "visible": 2 },
  "tools": [
    {
      "name": "CodeQL",
      "version": "2.18.0",
      "latestStatus": "completed",
      "latestUploadAt": "2026-05-05T00:00:00Z",
      "checkRunHref": "/mona/octo-app/actions/runs/42"
    }
  ],
  "alerts": [
    {
      "id": "alert_01",
      "number": 1,
      "state": "open",
      "ruleName": "Unsanitized SQL query",
      "severity": "error",
      "securitySeverity": "critical",
      "toolName": "CodeQL",
      "path": "crates/api/src/routes/search.rs",
      "startLine": 42,
      "pathHref": "/mona/octo-app/blob/refs%2Fheads%2Fmain/crates/api/src/routes/search.rs#L42",
      "href": "/mona/octo-app/security/code-scanning/1"
    }
  ]
}`,
    notes: [
      "Supported filters are state, q, severity, security_severity, tool, branch, ref, tag, application_code, and sort. Invalid values return validation_failed with a 422 status.",
      "Disabled repository settings return enabled=false with zero counts and no historical rows, so disabled states do not leak past private alert volume.",
      "Tool summaries expose reader-safe SARIF upload and check-run destinations without raw storage keys, workflow secrets, environment variables, token hashes, OAuth data, or stack traces.",
      "List rows include concrete alert detail and file destinations; private repository access failures are redacted as not_found.",
    ],
  },
  {
    id: "repo-code-scanning-alert-detail",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/security/code-scanning/{alert_id}",
    title: "Read repository Code scanning alert detail",
    description:
      "Returns one Code scanning alert detail view with rule metadata, source location, sanitized snippet/help content, timeline rows, assignee options, linked issue state, and PR/check annotation destinations.",
    auth: "Signed opengithub session cookie with repository read permission",
    response: `{
  "alert": {
    "id": "alert_01",
    "number": 1,
    "state": "open",
    "ruleId": "rust/sql-injection",
    "ruleName": "Unsanitized SQL query",
    "severity": "error",
    "securitySeverity": "critical"
  },
  "location": {
    "path": "crates/api/src/routes/search.rs",
    "startLine": 42,
    "snippet": "sqlx::query(&format!(...))",
    "pathHref": "/mona/octo-app/blob/refs%2Fheads%2Fmain/crates/api/src/routes/search.rs#L42"
  },
  "rule": {
    "id": "rust/sql-injection",
    "name": "Unsanitized SQL query",
    "helpMarkdown": "Use parameterized queries."
  },
  "timeline": [
    { "eventType": "created", "message": "Code scanning opened this alert from analysis results." }
  ],
  "linkedIssue": {
    "canLink": true,
    "issue": null,
    "createHref": "/api/repos/mona/octo-app/security/code-scanning/1/issue"
  }
}`,
    notes: [
      "Readers can inspect permitted detail and PR annotations but receive forbidden on mutation routes.",
      "Rendered Markdown and snippets are sanitized; script tags, unsafe URLs, raw SARIF storage metadata, session values, token hashes, OAuth payloads, and provider secrets are never serialized.",
      "Missing source snippets and long paths remain screen-safe: the response keeps concrete file/action hrefs and bounded strings for mobile layouts.",
    ],
  },
  {
    id: "repo-code-scanning-alert-update",
    method: "PATCH",
    path: "/api/repos/{owner}/{repo}/security/code-scanning/{alert_id}",
    title: "Update repository Code scanning alert",
    description:
      "Lets maintainers dismiss, reopen, assign, or link an existing issue to one Code scanning alert while recording timeline events, security audit events, and notification updates.",
    auth: "Signed opengithub session cookie with repository maintainer permission",
    request: `{
  "action": "dismiss",
  "dismissalReason": "false_positive",
  "dismissalComment": "Confirmed by security review."
}`,
    response: `{
  "alert": {
    "id": "alert_01",
    "number": 1,
    "state": "dismissed",
    "dismissalReason": "false_positive"
  },
  "timeline": [
    { "eventType": "dismissed", "message": "Dismissed this alert as false_positive." }
  ]
}`,
    notes: [
      "Supported actions are dismiss, reopen, assign, and link_issue. Dismiss requires a bounded dismissalReason and optional bounded comment.",
      "Archived repositories, disabled Code scanning settings, invalid assignees, stale alert states, malformed issue links, and invalid transitions return structured validation or conflict errors.",
      "Successful writes update code_scanning_alerts, code_scanning_alert_events, security_audit_events, linked issue state, and assignee notification rows atomically.",
    ],
  },
  {
    id: "repo-code-scanning-alert-issue",
    method: "POST",
    path: "/api/repos/{owner}/{repo}/security/code-scanning/{alert_id}/issue",
    title: "Create Code scanning linked issue",
    description:
      "Creates or reuses a repository issue from Code scanning rule, location, and remediation data, then links it back to the alert detail view.",
    auth: "Signed opengithub session cookie with repository maintainer permission",
    response: `{
  "linkedIssue": {
    "issue": {
      "number": 12,
      "title": "Code scanning: Unsanitized SQL query",
      "href": "/mona/octo-app/issues/12"
    },
    "canLink": true
  },
  "timeline": [
    { "eventType": "issue_linked", "message": "Linked this alert to issue #12." }
  ]
}`,
    notes: [
      "Duplicate requests return the existing linked issue instead of creating a second issue.",
      "Issue bodies are generated from bounded alert fields and sanitized remediation text; raw SARIF blobs and private environment metadata are not copied.",
      "Issue creation records normal issue notifications plus a redacted Code scanning alert timeline event.",
    ],
  },
  {
    id: "repo-code-scanning-sarif-upload",
    method: "POST",
    path: "/api/repos/{owner}/{repo}/code-scanning/sarifs",
    title: "Upload repository Code scanning SARIF",
    description:
      "Accepts a bounded SARIF 2.1.0 payload from Actions or REST clients, stores redacted upload metadata, normalizes alerts by fingerprint/location/rule, updates fixed-alert state, and links reader-safe check-run annotations.",
    auth: "Signed opengithub session cookie or API token with repository write permission",
    request: `{
  "ref": "main",
  "commitSha": "abc123",
  "sarif": {
    "version": "2.1.0",
    "runs": [
      {
        "tool": { "driver": { "name": "CodeQL", "version": "2.18.0" } },
        "results": []
      }
    ]
  }
}`,
    response: `{
  "status": "processed",
  "processedAlerts": 1,
  "fixedAlerts": 1,
  "toolName": "CodeQL",
  "toolVersion": "2.18.0",
  "artifactStorageKey": "redacted://code-scanning/repository/upload.sarif"
}`,
    notes: [
      "Uploads larger than 2 MiB return 413; malformed JSON, missing runs, missing tool.driver.name, invalid locations, or unknown refs return standard validation_failed envelopes.",
      "Repeated uploads de-duplicate by stable fingerprint, path, line, rule, and ref; alerts absent from the latest analysis are marked fixed unless already dismissed.",
      "Responses expose only redacted storage identifiers and reader-safe PR/check annotations, never S3 object keys, raw SARIF blobs, secrets, token hashes, session cookies, OAuth payloads, environment variables, or stack traces.",
    ],
  },
  {
    id: "repo-secret-scanning-alerts",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/security/secret-scanning?state=open&provider=GitHub&sort=recently_detected",
    title: "List repository Secret scanning alerts",
    description:
      "Returns the screen-ready Secret scanning alerts list from committed blob indexing and push-protection outcomes, including provider/default and generic result tabs, filter metadata, open/resolved counts, disabled states, redacted alert rows, and viewer write permissions.",
    auth: "Signed opengithub session cookie with repository read permission; private outsiders receive not_found",
    response: `{
  "availability": {
    "enabled": true,
    "indexed": true,
    "message": "Secret scanning alerts are monitored.",
    "settingsHref": "/mona/octo-app/settings/security_analysis"
  },
  "filters": {
    "state": "open",
    "query": null,
    "provider": "GitHub",
    "secretType": null,
    "validity": null,
    "resolution": null,
    "bypassed": null,
    "resultKind": "provider",
    "sort": "recently_detected"
  },
  "counts": {
    "open": 2,
    "resolved": 1,
    "provider": 2,
    "generic": 1,
    "bypassed": 1,
    "visible": 1
  },
  "alerts": [
    {
      "id": "alert_01",
      "number": 1,
      "state": "open",
      "resultKind": "provider",
      "redactedSecret": "ghp_************",
      "pattern": {
        "provider": "GitHub",
        "secretType": "github_personal_access_token",
        "displayName": "GitHub personal access token",
        "pushProtectionEnabled": true
      },
      "validity": { "state": "active", "checkedAt": "2026-05-05T00:00:00Z" },
      "primaryLocation": {
        "path": ".env",
        "startLine": 12,
        "pathHref": "/mona/octo-app/blob/refs%2Fheads%2Fmain/.env#L12"
      },
      "bypass": {
        "status": "pending_review",
        "reason": "Needed for local example fixture."
      },
      "href": "/mona/octo-app/security/secret-scanning/1"
    }
  ]
}`,
    notes: [
      "Supported filters are state, q, provider, secret_type, validity, resolution, bypassed, result_kind, team, topic, and sort. Invalid values return validation_failed with a 422 status.",
      "Disabled repository settings return enabled=false with zero counts and no historical rows, so disabled states do not leak private alert volume.",
      "Rows include concrete alert detail, file, commit, and settings destinations while exposing only redacted snippets and keyed fingerprints.",
      "Responses never include plaintext secrets, token hashes, session cookies, OAuth payloads, storage keys, environment variables, raw Git object bytes, stack traces, or private repository metadata.",
    ],
  },
  {
    id: "repo-secret-scanning-alert-detail",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/security/secret-scanning/{alert_id}",
    title: "Read repository Secret scanning alert detail",
    description:
      "Returns one Secret scanning alert detail view with redacted evidence, file and commit location, provider pattern metadata, validity checks, push-protection bypass metadata, assignment options, and timeline rows.",
    auth: "Signed opengithub session cookie with repository read permission",
    response: `{
  "alert": {
    "id": "alert_01",
    "number": 1,
    "state": "open",
    "resolution": null,
    "redactedSecret": "ghp_************"
  },
  "pattern": {
    "provider": "GitHub",
    "secretType": "github_personal_access_token",
    "displayName": "GitHub personal access token",
    "resultKind": "provider"
  },
  "locations": [
    {
      "path": ".env",
      "startLine": 12,
      "redactedSnippet": "TOKEN=ghp_************",
      "commitHref": "/mona/octo-app/commit/abc123",
      "pathHref": "/mona/octo-app/blob/refs%2Fheads%2Fmain/.env#L12"
    }
  ],
  "validity": {
    "state": "active",
    "provider": "GitHub",
    "message": "Provider reported the credential is active."
  },
  "bypass": {
    "status": "pending_review",
    "reason": "Needed for local example fixture.",
    "redactedSnippet": "TOKEN=ghp_************"
  },
  "timeline": [
    { "eventType": "created", "message": "Secret scanning opened this alert from committed content." }
  ]
}`,
    notes: [
      "Readers can inspect permitted redacted detail data but receive forbidden on mutation routes.",
      "The endpoint serializes redacted evidence and fingerprints only; plaintext secret bytes are not stored, returned, logged, or copied into audit payloads.",
      "Long paths, missing validity data, and deleted source files remain screen-safe with bounded strings and concrete fallback actions.",
    ],
  },
  {
    id: "repo-secret-scanning-alert-update",
    method: "PATCH",
    path: "/api/repos/{owner}/{repo}/security/secret-scanning/{alert_id}",
    title: "Update repository Secret scanning alert",
    description:
      "Lets maintainers resolve, reopen, assign, or update permitted validity metadata for one Secret scanning alert while recording redacted timeline events, security audit events, and notification updates.",
    auth: "Signed opengithub session cookie with repository maintainer permission",
    request: `{
  "action": "resolve",
  "resolution": "false_positive",
  "comment": "The redacted fixture is not a real credential."
}`,
    response: `{
  "alert": {
    "id": "alert_01",
    "number": 1,
    "state": "resolved",
    "resolution": "false_positive"
  },
  "timeline": [
    { "eventType": "resolved", "message": "Resolved this alert as false_positive." }
  ]
}`,
    notes: [
      "Supported actions are resolve, reopen, assign, and validity. Resolve requires a reason of revoked, false_positive, used_in_tests, or wont_fix plus an optional bounded comment.",
      "Archived repositories, disabled Secret scanning settings, invalid assignees, stale states, malformed transitions, and unsupported validity updates return structured validation or conflict errors.",
      "Successful writes update secret_scanning_alerts, secret_scanning_alert_events, security_audit_events, and assignee notification rows atomically with redacted metadata only.",
    ],
  },
  {
    id: "repo-secret-scanning-push-protection",
    method: "POST",
    path: "/api/repos/{owner}/{repo}/git/receive-pack",
    title: "Secret scanning push protection",
    description:
      "Scans incoming Git smart-HTTP pushes before accepting refs when Secret scanning and push protection are enabled, creates redacted alerts for provider/default or generic matches, and records approved bypass outcomes.",
    auth: "Personal access token or signed session with repository write permission",
    request: `{
  "ref": "refs/heads/main",
  "commitOid": "abc123",
  "pushProtectionBypassReason": "Needed for local example fixture."
}`,
    response: `{
  "status": "accepted_with_bypass",
  "createdAlerts": 1,
  "blocked": false,
  "bypass": {
    "status": "pending_review",
    "reason": "Needed for local example fixture."
  }
}`,
    notes: [
      "Protected provider matches block or warn before ref updates unless the actor supplies a bounded bypass reason that policy allows.",
      "Existing blob backfills and new pushed commits de-duplicate alerts by provider pattern, blob/commit identity, path/line, and keyed secret hash.",
      "Push-protection responses, bypass rows, alert events, audit events, and notifications carry only redacted snippets and never echo plaintext secrets.",
      "Binary files, oversized blobs, archived repositories, disabled settings, malformed bypass reasons, and permission failures return truthful no-secret envelopes.",
    ],
  },
  {
    id: "repo-discussions-list",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/discussions?q=is%3Aopen&label=help-wanted&sort=latest&page=1&page_size=30",
    title: "List repository Discussions",
    description:
      "Returns the screen-ready repository Discussions list with URL-backed search filters, pinned cards, category rail data, labels, helpful contributors, community links, pagination, and viewer vote/create affordances.",
    auth: "Signed opengithub session cookie with repository read permission; private outsiders receive not_found",
    response: `{
  "repository": {
    "owner": "mona",
    "name": "octo-app",
    "visibility": "public",
    "discussionsHref": "/mona/octo-app/discussions"
  },
  "viewer": {
    "authenticated": true,
    "permission": "write",
    "canRead": true,
    "canVote": true,
    "canCreate": true
  },
  "enabled": true,
  "filters": {
    "query": "is:open",
    "label": "help-wanted",
    "state": "open",
    "answered": null,
    "locked": null,
    "pinned": null,
    "sort": "latest",
    "category": null,
    "page": 1,
    "pageSize": 30
  },
  "pinned": [
    {
      "position": 1,
      "discussion": {
        "number": 12,
        "title": "How should import previews handle large manifests?",
        "answered": true,
        "pinned": true,
        "votesCount": 14,
        "viewerVoted": false,
        "href": "/mona/octo-app/discussions/12"
      }
    }
  ],
  "categories": [
    {
      "slug": "general",
      "name": "General",
      "emoji": "💬",
      "count": 12,
      "openCount": 10,
      "href": "/mona/octo-app/discussions/categories/general",
      "active": false
    }
  ],
  "items": [
    {
      "number": 12,
      "title": "How should import previews handle large manifests?",
      "state": "open",
      "category": { "slug": "general", "name": "General" },
      "labels": [{ "name": "help-wanted", "color": "var(--accent)" }],
      "commentsCount": 8,
      "votesCount": 14,
      "viewerVoted": false,
      "href": "/mona/octo-app/discussions/12"
    }
  ],
  "openCount": 10,
  "closedCount": 2,
  "total": 12,
  "hasNextPage": false
}`,
    notes: [
      "Supported filters are q, label, state, answered, locked, pinned, sort, page, and page_size. Invalid booleans, states, sort keys, or pagination values return validation_failed envelopes.",
      "When organization policy disables Discussions, enabled=false returns with empty rows and a disabledReason so historical private discussion volume is not leaked.",
      "Pinned cards are capped and ordered by pin position; list rows include concrete detail, category, label, author, comment, and vote metadata for the Editorial repository workspace.",
      "Responses never include raw session rows, OAuth provider payloads, notification internals, environment variables, storage keys, private repository metadata for outsiders, or stack traces.",
    ],
  },
  {
    id: "repo-discussions-category-list",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/discussions/categories/{slug}?q=is%3Aopen&sort=top",
    title: "List repository Discussions by category",
    description:
      "Returns the same Discussions list contract scoped to one category slug, with active category rail state and category-specific empty-state data.",
    auth: "Signed opengithub session cookie with repository read permission; unknown private categories return not_found without repository metadata leaks",
    response: `{
  "filters": {
    "query": "is:open",
    "sort": "top",
    "category": "general",
    "page": 1,
    "pageSize": 30
  },
  "categories": [
    {
      "slug": "general",
      "name": "General",
      "emoji": "💬",
      "description": "General project conversation.",
      "active": true
    }
  ],
  "items": [],
  "total": 0,
  "hasNextPage": false
}`,
    notes: [
      "Category slugs are normalized through the same validation path as discussion filters. Unknown slugs return not_found rather than an empty global list.",
      "Search, label, state, answered, locked, pinned, sort, and pagination filters compose with category scope and preserve the active rail destination.",
      "Empty category responses provide enough metadata for a category-specific New discussion CTA without implementing creation in this feature.",
      "Poll category slugs and renamed categories with format=poll return poll row metadata such as categoryQualifier, pollSummary, resultsVisible, viewerCanVote, viewerVoteOptionIds, and pollUnavailableReasons.",
    ],
  },
  {
    id: "repo-discussion-vote-create",
    method: "PUT",
    path: "/api/repos/{owner}/{repo}/discussions/{discussion_number}/vote",
    title: "Upvote repository Discussion",
    description:
      "Creates the current viewer's upvote for a Discussion, reconciles the aggregate vote count, records a discussion activity event, and notifies the discussion author when appropriate.",
    auth: "Signed opengithub session cookie with repository read permission",
    response: `{
  "discussionId": "discussion_01",
  "discussionNumber": 12,
  "viewerVoted": true,
  "votesCount": 15
}`,
    notes: [
      "Repeated PUT requests are idempotent and return the current server vote count without creating duplicate vote rows.",
      "Anonymous callers receive 401. Archived repositories, disabled Discussions, missing discussions, and private access failures return stable no-secret error envelopes.",
      "Successful first votes write discussion_votes, discussion_activity_events, and notification rows with bounded metadata only.",
    ],
  },
  {
    id: "repo-discussion-vote-delete",
    method: "DELETE",
    path: "/api/repos/{owner}/{repo}/discussions/{discussion_number}/vote",
    title: "Remove repository Discussion upvote",
    description:
      "Removes the current viewer's upvote from a Discussion and returns the reconciled count for optimistic browser rollback or confirmation.",
    auth: "Signed opengithub session cookie with repository read permission",
    response: `{
  "discussionId": "discussion_01",
  "discussionNumber": 12,
  "viewerVoted": false,
  "votesCount": 14
}`,
    notes: [
      "DELETE is idempotent when the viewer has not voted; it still returns the authoritative count and viewerVoted=false.",
      "Vote removal records a discussion activity event but does not create duplicate author notification rows.",
      "The response never includes raw notification payloads, session cookies, OAuth profile data, token hashes, environment variables, or stack traces.",
    ],
  },
  {
    id: "repo-discussion-poll-vote",
    method: "PUT",
    path: "/api/repos/{owner}/{repo}/discussions/{discussion_number}/poll/vote",
    title: "Vote in repository Discussion poll",
    description:
      "Creates or updates the current viewer's poll vote, enforces single-choice or multiple-choice policy, reconciles result percentages, and returns the refreshed poll contract for the detail page.",
    auth: "Signed opengithub session cookie with repository read permission and poll voting enabled",
    request: `{
  "optionIds": ["option_01", "option_02"]
}`,
    response: `{
  "discussionId": "discussion_03",
  "discussionNumber": 44,
  "poll": {
    "id": "poll_01",
    "question": "Which branch policy should ship first?",
    "multipleChoice": true,
    "allowsVoteChanges": true,
    "viewerCanVote": true,
    "resultsVisible": true,
    "viewerVoteOptionIds": ["option_01", "option_02"],
    "totalVotes": 2,
    "options": [
      {
        "id": "option_01",
        "body": "Linear history",
        "votesCount": 1,
        "percentage": 50
      },
      {
        "id": "option_02",
        "body": "Required reviews",
        "votesCount": 1,
        "percentage": 50
      }
    ]
  }
}`,
    notes: [
      "Anonymous callers receive 401 and browser clients render a sign-in prompt instead of dead controls.",
      "Single-choice polls accept one option id. Multiple-choice polls accept the selected option set. Invalid option ids, cross-poll options, and empty selections return validation envelopes.",
      "Vote changes are accepted only when allowsVoteChanges is true; otherwise a second different selection returns a policy validation error while preserving the existing vote.",
      "Locked, archived, deleted, disabled, and private-inaccessible discussions reject votes without leaking result counts, session rows, OAuth payloads, environment variables, storage keys, or stack traces.",
      "Successful writes update discussion_poll_votes, discussion_activity_events, and audit rows with bounded metadata; comments, replies, reactions, subscriptions, and notification state remain intact.",
    ],
  },
  {
    id: "repo-discussion-create-metadata",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/discussions/new",
    title: "Read repository Discussion creation metadata",
    description:
      "Returns the chooser-ready category cards, selected-category form fallback, community links, similar-search requirement, and viewer create affordances for the Editorial new Discussion flow.",
    auth: "Signed opengithub session cookie with repository read permission; private outsiders receive not_found",
    response: `{
  "repository": {
    "owner": "mona",
    "name": "octo-app",
    "visibility": "public",
    "href": "/mona/octo-app",
    "discussionsHref": "/mona/octo-app/discussions"
  },
  "viewer": {
    "authenticated": true,
    "permission": "write",
    "canRead": true,
    "canVote": true,
    "canCreate": true
  },
  "enabled": true,
  "disabledReason": null,
  "categories": [
    {
      "slug": "q-a",
      "name": "Q&A",
      "emoji": "🙏",
      "acceptsAnswers": true,
      "isPoll": false,
      "count": 3,
      "openCount": 2,
      "formHref": "/mona/octo-app/discussions/new?category=q-a"
    }
  ],
  "selectedCategory": null,
  "similarSearch": {
    "required": true,
    "query": "",
    "href": "/mona/octo-app/discussions?q="
  }
}`,
    notes: [
      "Organization policy-disabled repositories return enabled=false with a disabledReason and no create-capable affordance instead of dead browser controls.",
      "Archived repositories and read-only viewers can still read public chooser metadata, but viewer.canCreate=false tells the browser to render non-submitting states.",
      "The response is bounded for browser cards and omits raw discussion template source, session rows, OAuth payloads, environment variables, storage keys, and stack traces.",
    ],
  },
  {
    id: "repo-discussion-create-category-metadata",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/discussions/new/categories/{slug}?q=import%20preview",
    title: "Read selected Discussion category form",
    description:
      "Returns one category's composer contract, including parsed DISCUSSION_TEMPLATE YAML fields when valid, generic fallback metadata when invalid, poll-category flags, and the title-derived similar-search URL.",
    auth: "Signed opengithub session cookie with repository read permission; private outsiders and unknown private categories receive not_found",
    response: `{
  "selectedCategory": {
    "slug": "q-a",
    "name": "Q&A",
    "emoji": "🙏",
    "acceptsAnswers": true,
    "isPoll": false
  },
  "form": {
    "categorySlug": "q-a",
    "templatePath": ".github/DISCUSSION_TEMPLATE/q-a.yml",
    "description": "Add enough context for a maintainer to answer.",
    "fallback": false,
    "valid": true,
    "fields": [
      {
        "id": "context",
        "fieldType": "textarea",
        "label": "Context",
        "required": true,
        "options": []
      },
      {
        "id": "area",
        "fieldType": "dropdown",
        "label": "Area",
        "required": false,
        "options": ["UI", "API"]
      }
    ]
  },
  "similarSearch": {
    "required": true,
    "query": "import preview",
    "href": "/mona/octo-app/discussions?q=import+preview"
  }
}`,
    notes: [
      "Supported YAML field types are input, textarea, dropdown, and checkboxes. Unsupported or oversized fields are ignored safely; invalid templates fall back to the generic title/body composer.",
      "Markdown descriptions are sanitized before they reach the browser. Raw template files, parse stack traces, and private Git blob metadata are never returned.",
      "Poll categories set selectedCategory.isPoll=true and the browser renders poll question/options controls instead of YAML form fields.",
    ],
  },
  {
    id: "repo-discussion-create",
    method: "POST",
    path: "/api/repos/{owner}/{repo}/discussions",
    title: "Create repository Discussion",
    description:
      "Creates a Discussion from the chooser/composer flow, persists the initial body comment, optional YAML form answers, optional poll options, bounded attachment metadata, viewer subscription, notifications, and repository activity events.",
    auth: "Signed opengithub session cookie with repository write permission",
    request: `{
  "categorySlug": "q-a",
  "title": "How should import previews handle large manifests?",
  "body": "The preview should stay responsive for large lockfiles.",
  "similarSearchAcknowledged": true,
  "formAnswers": [
    { "fieldId": "context", "value": "Users repeat search discussions." }
  ],
  "poll": null,
  "attachmentDrafts": [
    {
      "fileName": "sketch.txt",
      "contentType": "text/plain",
      "byteSize": 128,
      "storageKey": "discussion-drafts/user/sketch.txt"
    }
  ]
}`,
    response: `{
  "discussionId": "discussion_01",
  "discussionNumber": 42,
  "href": "/mona/octo-app/discussions/42",
  "title": "How should import previews handle large manifests?",
  "category": {
    "slug": "q-a",
    "name": "Q&A",
    "acceptsAnswers": true,
    "isPoll": false
  }
}`,
    notes: [
      "The similar-search acknowledgement is required before creation. Missing title, body, required YAML answers, invalid category slugs, archived repositories, disabled Discussions, and read-only viewers return stable validation or permission envelopes.",
      "Poll payloads require a question plus two to ten unique non-empty options, optional multiple-choice/change-vote policy, and cannot be combined with category form answers.",
      "Creation writes discussions, discussion_comments, discussion_form_answers, discussion_polls/options when present, discussion_attachments metadata, subscriptions, notifications, and repository_activity_events in one transaction.",
      "Markdown preview is browser-side and sanitized before display; previewing does not create discussion, comment, attachment, subscription, notification, or activity rows.",
      "Responses and errors never include raw session cookies, OAuth profile payloads, token hashes, environment variables, plaintext storage credentials, raw attachment object bytes, or stack traces.",
    ],
  },
  {
    id: "repo-discussion-detail",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/discussions/{discussion_number}?sort=oldest&page=1&page_size=30",
    title: "Read repository Discussion detail",
    description:
      "Returns the screen-ready Discussion detail contract with sanitized Markdown body, form and poll summaries, sorted timeline comments, nested replies, reactions, answer summary, subscription state, sidebar metadata, participants, events, and viewer moderation affordances.",
    auth: "Signed opengithub session cookie with repository read permission; private outsiders receive not_found",
    response: `{
  "discussion": {
    "number": 42,
    "title": "How should import previews handle large manifests?",
    "state": "open",
    "answered": true,
    "category": { "slug": "q-a", "name": "Q&A", "acceptsAnswers": true },
    "bodyHtml": "<p>The preview should stay responsive.</p>",
    "commentsCount": 8,
    "votesCount": 14,
    "href": "/mona/octo-app/discussions/42"
  },
  "answer": {
    "commentId": "comment_02",
    "markedBy": { "login": "maintainer" },
    "href": "/mona/octo-app/discussions/42#discussioncomment-comment_02"
  },
  "poll": {
    "id": "poll_01",
    "question": "Which branch policy should ship first?",
    "multipleChoice": false,
    "allowsVoteChanges": true,
    "viewerCanVote": true,
    "resultsVisible": true,
    "viewerVoteOptionIds": ["option_01"],
    "totalVotes": 12,
    "options": [
      { "id": "option_01", "body": "Linear history", "votesCount": 8, "percentage": 67 },
      { "id": "option_02", "body": "Required reviews", "votesCount": 4, "percentage": 33 }
    ]
  },
  "timeline": [
    {
      "kind": "comment",
      "comment": {
        "id": "comment_02",
        "bodyHtml": "<p>Use streamed parsing.</p>",
        "answer": true,
        "reactions": [{ "content": "thumbs_up", "count": 2 }],
        "replies": [{ "id": "reply_01", "bodyHtml": "<p>Confirmed.</p>" }]
      }
    }
  ],
  "viewer": {
    "canComment": true,
    "canModerate": true,
    "subscriptionState": "subscribed"
  }
}`,
    notes: [
      "Supported sort values are oldest, newest, and top; malformed discussion numbers, sort keys, or pagination values return validation_failed envelopes.",
      "Comment and reply Markdown is sanitized server-side. Unsafe HTML, raw attachment objects, session rows, OAuth data, environment variables, storage credentials, and stack traces are never serialized.",
      "Poll details include voting controls, result visibility, current viewer selections, result bars, and policy reasons while suppressing counts from unauthorized private viewers.",
      "The sidebar includes category, labels, participants, notification state, and bounded event history so the browser can render answer, close/reopen, category, and label controls without dead placeholders.",
      "Archived repositories, disabled Discussions, private repository outsiders, deleted comments, and moderated rows preserve privacy-safe unavailable or collapsed states.",
    ],
  },
  {
    id: "repo-discussion-comment-create",
    method: "POST",
    path: "/api/repos/{owner}/{repo}/discussions/{discussion_number}/comments",
    title: "Create repository Discussion comment",
    description:
      "Adds a top-level comment to a Discussion, sanitizes Markdown, validates bounded attachment metadata, subscribes the participant, records activity, notifies participants, and returns the refreshed detail contract.",
    auth: "Signed opengithub session cookie with repository read permission and unlocked discussion access",
    request: `{
  "body": "Use streamed parsing and summarize large lockfiles.",
  "attachmentDrafts": []
}`,
    response: `{
  "discussion": { "number": 42, "commentsCount": 9 },
  "createdAnchor": "#discussioncomment-comment_09",
  "timeline": []
}`,
    notes: [
      "Blank bodies, oversized bodies, invalid attachment draft metadata, locked discussions, archived repositories, disabled Discussions, and private access failures return stable no-secret envelopes.",
      "Successful writes update discussion_comments, comment counts, participant subscription state, discussion_activity_events, and notification rows in one transaction.",
      "Attachment object upload remains a separate storage workflow; this endpoint accepts only bounded draft metadata and never returns raw object bytes or storage credentials.",
    ],
  },
  {
    id: "repo-discussion-reply-create",
    method: "POST",
    path: "/api/repos/{owner}/{repo}/discussions/{discussion_number}/comments/{comment_id}/replies",
    title: "Create repository Discussion reply",
    description:
      "Adds a nested reply under a top-level Discussion comment with the same sanitization, attachment metadata validation, activity, subscription, notification, and refreshed-detail behavior as top-level comments.",
    auth: "Signed opengithub session cookie with repository read permission and unlocked discussion access",
    request: `{
  "body": "That shape works for the importer path.",
  "attachmentDrafts": []
}`,
    response: `{
  "createdAnchor": "#discussionreply-reply_01",
  "timeline": []
}`,
    notes: [
      "Replies can target only comments in the same Discussion; nested reply-to-reply ids, deleted parents, locked discussions, and malformed comment ids return validation or not_found envelopes.",
      "Successful replies update discussion_comment_replies, aggregate counts, discussion_activity_events, participant subscriptions, and bounded notification fanout.",
      "Responses never include raw notification payloads, private repository metadata for outsiders, session cookies, OAuth provider data, environment variables, storage keys, or stack traces.",
    ],
  },
  {
    id: "repo-discussion-reaction-toggle",
    method: "PUT",
    path: "/api/repos/{owner}/{repo}/discussions/{discussion_number}/comments/{comment_id}/reactions",
    title: "React to repository Discussion content",
    description:
      "Adds the current viewer's reaction to a Discussion, top-level comment, or nested reply target and returns the refreshed reaction aggregate for optimistic browser reconciliation.",
    auth: "Signed opengithub session cookie with repository read permission",
    request: `{
  "content": "thumbs_up",
  "replyId": null
}`,
    response: `{
  "targetType": "comment",
  "targetId": "comment_02",
  "reactions": [
    { "content": "thumbs_up", "count": 3, "viewerReacted": true }
  ]
}`,
    notes: [
      "PUT is idempotent for the same viewer/content/target tuple; DELETE on the same path removes the reaction and returns the authoritative aggregate.",
      "Supported targets are the discussion itself, top-level comments, and replies. Invalid reaction content, cross-discussion targets, deleted targets, or private access failures use stable envelopes.",
      "Reaction writes update discussion_reactions and discussion_activity_events without exposing raw actor session data, OAuth payloads, notification internals, or stack traces.",
    ],
  },
  {
    id: "repo-discussion-subscription",
    method: "PUT",
    path: "/api/repos/{owner}/{repo}/discussions/{discussion_number}/subscription",
    title: "Subscribe to repository Discussion",
    description:
      "Sets the current viewer's Discussion notification state to subscribed or unsubscribed and returns the refreshed sidebar notification state.",
    auth: "Signed opengithub session cookie with repository read permission",
    response: `{
  "discussionId": "discussion_01",
  "discussionNumber": 42,
  "subscriptionState": "subscribed",
  "reason": "manual"
}`,
    notes: [
      "PUT subscribes and DELETE unsubscribes idempotently; both preserve read permission and private repository privacy checks.",
      "Successful writes update discussion_subscriptions and record bounded discussion_activity_events metadata only.",
      "The response never includes notification delivery internals, session rows, OAuth data, token hashes, environment variables, storage credentials, or stack traces.",
    ],
  },
  {
    id: "repo-discussion-answer",
    method: "PUT",
    path: "/api/repos/{owner}/{repo}/discussions/{discussion_number}/answer",
    title: "Mark repository Discussion answer",
    description:
      "Marks or unmarks a top-level comment as the accepted answer for answer-enabled Discussion categories, updates the Discussion answered state, records events, notifies the author, and returns the refreshed detail contract.",
    auth: "Signed opengithub session cookie with triage, write, admin, or owner repository permission",
    request: `{
  "commentId": "comment_02"
}`,
    response: `{
  "discussion": { "number": 42, "answered": true },
  "answer": { "commentId": "comment_02", "href": "/mona/octo-app/discussions/42#discussioncomment-comment_02" }
}`,
    notes: [
      "DELETE on the same endpoint unmarks the answer and clears the accepted-answer pointer. Both operations reject normal categories that do not accept answers.",
      "Validation enforces triage-or-greater permission, same-discussion top-level comments, unlocked/unarchived repositories, enabled Discussions, and stable no-secret error envelopes.",
      "Successful writes update discussion_answers, discussions.answer_comment_id, discussion_activity_events, and notification rows with redacted metadata.",
    ],
  },
  {
    id: "repo-discussion-state-update",
    method: "PUT",
    path: "/api/repos/{owner}/{repo}/discussions/{discussion_number}/state",
    title: "Close or reopen repository Discussion",
    description:
      "Transitions a Discussion between open and closed states with a bounded reason, records timeline events, sends notification fanout, and returns the refreshed detail contract.",
    auth: "Signed opengithub session cookie with triage, write, admin, or owner repository permission",
    request: `{
  "state": "closed",
  "reason": "resolved"
}`,
    response: `{
  "discussion": { "number": 42, "state": "closed", "closedReason": "resolved" },
  "sidebar": { "events": [{ "eventType": "closed" }] }
}`,
    notes: [
      "Supported reasons are bounded server enums for resolved, duplicate, outdated, and not_planned close flows; reopening clears the close reason.",
      "Archived repositories, disabled Discussions, malformed state or reason values, and insufficient permissions return stable validation or permission envelopes.",
      "Close and reopen writes update discussions, discussion_activity_events, and notification rows without leaking session cookies, OAuth profiles, private repository metadata, or stack traces.",
    ],
  },
  {
    id: "repo-milestones-list",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/milestones?state=open&sort=updated-desc&page=1&pageSize=30",
    title: "List repository milestones",
    description:
      "Returns repository milestones with state tabs, full sort support, progress counts, due dates, issue-count links, and viewer edit capability for the Editorial Milestones page.",
    auth: "Anonymous for public repositories, signed opengithub session cookie for private repositories",
    response: `{
  "items": [
    {
      "id": "milestone_01",
      "title": "Launch readiness",
      "state": "open",
      "dueOn": "2026-05-20T00:00:00Z",
      "progress": {
        "openCount": 3,
        "closedCount": 2,
        "totalCount": 5,
        "percentComplete": 40
      },
      "openIssuesHref": "/mona/octo-app/issues?state=open&milestone=Launch%20readiness",
      "closedIssuesHref": "/mona/octo-app/issues?state=closed&milestone=Launch%20readiness",
      "href": "/mona/octo-app/milestones/milestone_01"
    }
  ],
  "openCount": 1,
  "closedCount": 0,
  "filters": { "state": "open", "sort": "updated-desc" },
  "viewer": { "permission": "write", "canEditMilestones": true }
}`,
    notes: [
      "state supports open, closed, and all; sort supports recently updated, due-date, completion, alphabetical, reverse alphabetical, most issues, and fewest issues modes.",
      "Private repositories return not_found outside the viewer boundary; public repositories remain readable without exposing raw permission rows or private item bodies.",
      "Progress counts include issues and pull requests through pull-request backing issues, so list rows, detail progress, and issue-count links share one Rust-owned source of truth.",
    ],
  },
  {
    id: "repo-milestones-create",
    method: "POST",
    path: "/api/repos/{owner}/{repo}/milestones",
    title: "Create repository milestone",
    description:
      "Creates a repository milestone with title, optional Markdown description, optional due date, rendered description cache, event evidence, and audit rows.",
    auth: "Signed opengithub session cookie with write, admin, or owner repository permission",
    request: `{
  "title": "Launch readiness",
  "description": "Track blockers before launch.",
  "dueOn": "2026-05-20T00:00:00Z"
}`,
    response: `{
  "id": "milestone_01",
  "title": "Launch readiness",
  "descriptionHtml": "<p>Track blockers before launch.</p>",
  "state": "open",
  "progress": { "openCount": 0, "closedCount": 0, "totalCount": 0, "percentComplete": 0 },
  "viewer": { "canEditMilestones": true }
}`,
    notes: [
      "Validation rejects blank titles, case-insensitive duplicate titles, oversized descriptions, invalid dates, archived repositories, and insufficient permissions with stable error envelopes.",
      "Descriptions are rendered through the Rust Markdown sanitizer and cached without returning parser stack traces, session rows, database URLs, or environment values.",
      "Create writes milestone_events and audit evidence before the browser refreshes the list or redirects to detail.",
    ],
  },
  {
    id: "repo-milestones-detail",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/milestones/{milestone_id}",
    title: "Read repository milestone detail",
    description:
      "Returns the planning view for one milestone, including rendered description, open and closed issue or pull-request rows, selected-item metadata, progress, and reorder capability.",
    auth: "Anonymous for public repositories, signed opengithub session cookie for private repositories",
    response: `{
  "id": "milestone_01",
  "title": "Launch readiness",
  "descriptionHtml": "<p>Track blockers before launch.</p>",
  "progress": { "openCount": 2, "closedCount": 1, "totalCount": 3, "percentComplete": 33 },
  "items": [
    {
      "id": "issue_42",
      "number": 42,
      "title": "Stabilize importer",
      "state": "open",
      "isPullRequest": false,
      "href": "/mona/octo-app/issues/42"
    }
  ],
  "order": { "canReorder": true, "reason": null, "version": "order-v1" }
}`,
    notes: [
      "Rows include labels, assignees, comment counts, pull-request identity, and concrete links without serializing private comments or raw join rows.",
      "order.canReorder is false for readers, archived repositories, and milestones with more than 500 open items; reason explains the disabled state for the browser.",
      "Missing or inaccessible milestones return repository-safe not_found envelopes so private repository names and milestone titles are not leaked.",
    ],
  },
  {
    id: "repo-milestones-update",
    method: "PATCH",
    path: "/api/repos/{owner}/{repo}/milestones/{milestone_id}",
    title: "Update repository milestone",
    description:
      "Updates milestone title, Markdown description, and due date while preserving issue and pull request assignments.",
    auth: "Signed opengithub session cookie with write, admin, or owner repository permission",
    request: `{
  "title": "Launch readiness v2",
  "description": "Updated launch blockers.",
  "dueOn": null
}`,
    response: `{
  "id": "milestone_01",
  "title": "Launch readiness v2",
  "dueOn": null,
  "descriptionHtml": "<p>Updated launch blockers.</p>"
}`,
    notes: [
      "Partial edits go through the same validation as create and reject cross-repository ids, duplicate titles, archived repositories, and reader sessions.",
      "Successful edits refresh rendered_markdown_cache, milestone_events, and audit_events while keeping issue and pull request milestone_id values intact.",
      "The response contains the refreshed milestone contract only, not raw audit metadata, session data, OAuth profile data, or storage internals.",
    ],
  },
  {
    id: "repo-milestones-close",
    method: "POST",
    path: "/api/repos/{owner}/{repo}/milestones/{milestone_id}/close",
    title: "Close repository milestone",
    description:
      "Transitions an open milestone to closed and returns the refreshed milestone detail contract for list and detail lifecycle controls.",
    auth: "Signed opengithub session cookie with write, admin, or owner repository permission",
    response: `{
  "id": "milestone_01",
  "state": "closed",
  "closedAt": "2026-05-20T00:00:00Z"
}`,
    notes: [
      "Closing is idempotent for already-closed milestones and does not close assigned issues or pull requests.",
      "Archived repositories, missing milestones, and insufficient permissions return structured errors without stack traces or private repository metadata.",
      "Successful transitions write milestone_events and audit rows so QA can trace state changes.",
    ],
  },
  {
    id: "repo-milestones-reopen",
    method: "POST",
    path: "/api/repos/{owner}/{repo}/milestones/{milestone_id}/reopen",
    title: "Reopen repository milestone",
    description:
      "Transitions a closed milestone back to open and clears its closed timestamp without changing assigned work items.",
    auth: "Signed opengithub session cookie with write, admin, or owner repository permission",
    response: `{
  "id": "milestone_01",
  "state": "open",
  "closedAt": null
}`,
    notes: [
      "Reopening preserves milestone_item_order, due date, description, and all issue or pull request assignments.",
      "State changes use the same repository write permission boundary as create, update, delete, and reorder.",
      "Responses never include session cookies, OAuth tokens, database URLs, raw audit payloads, or environment values.",
    ],
  },
  {
    id: "repo-milestones-delete",
    method: "DELETE",
    path: "/api/repos/{owner}/{repo}/milestones/{milestone_id}",
    title: "Delete repository milestone",
    description:
      "Deletes a milestone, clears associated issue and pull-request backing issue milestone assignments, records timeline evidence, and leaves conversations intact.",
    auth: "Signed opengithub session cookie with write, admin, or owner repository permission",
    response: `{
  "deleted": true,
  "clearedAssignments": { "issues": 2, "pullRequests": 1 }
}`,
    notes: [
      "Deletion clears issues.milestone_id for issues and pull-request backing issues while preserving issue numbers, pull request numbers, comments, labels, assignees, notifications, and subscriptions.",
      "Each cleared assignment writes metadata timeline evidence so issue and pull request sidebars can explain why the milestone chip disappeared.",
      "Missing, cross-repository, archived, or unauthorized deletes fail through stable no-secret envelopes.",
    ],
  },
  {
    id: "repo-milestones-order",
    method: "PATCH",
    path: "/api/repos/{owner}/{repo}/milestones/{milestone_id}/order",
    title: "Reorder milestone work items",
    description:
      "Persists writer-controlled priority order for a milestone's open issues and pull requests when the milestone has 500 or fewer open items.",
    auth: "Signed opengithub session cookie with write, admin, or owner repository permission",
    request: `{
  "orderedItemIds": ["issue_44", "issue_42"],
  "expectedVersion": "order-v1"
}`,
    response: `{
  "id": "milestone_01",
  "items": [
    { "id": "issue_44", "title": "Polish release notes" },
    { "id": "issue_42", "title": "Stabilize importer" }
  ],
  "order": { "canReorder": true, "reason": null, "version": "order-v2" }
}`,
    notes: [
      "orderedItemIds must exactly reference open issue rows in the same repository and milestone; cross-repository, duplicate, missing, closed, or stale item sets are rejected before persistence.",
      "expectedVersion provides stale-order conflict detection for concurrent edits; the browser keeps the user's attempted order visible while showing the structured conflict message.",
      "Milestones with more than 500 open items return an explicit disabled reorder reason and keep deterministic updated-time ordering instead of accepting a partial priority list.",
    ],
  },
  {
    id: "repo-labels-list",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/labels?q=bug&sort=name&direction=asc&page=1&pageSize=100",
    title: "List repository labels",
    description:
      "Returns repository labels with search, sort, pagination, open issue and pull request counts, discussion counts, and concrete filter hrefs for every collaboration surface.",
    auth: "Anonymous for public repositories, signed opengithub session cookie for private repositories",
    response: `{
  "items": [
    {
      "id": "label_bug",
      "name": "bug",
      "color": "b85c38",
      "description": "Something is not working",
      "counts": {
        "openIssues": 3,
        "openPullRequests": 1,
        "discussions": 2,
        "totalIssueCount": 4
      },
      "issuesHref": "/mona/octo-app/issues?q=is%3Aissue+state%3Aopen+label%3Abug&labels=bug",
      "pullRequestsHref": "/mona/octo-app/pulls?q=is%3Apr+state%3Aopen+label%3Abug&labels=bug",
      "discussionsHref": "/mona/octo-app/discussions?label=bug"
    }
  ],
  "total": 1,
  "page": 1,
  "pageSize": 100
}`,
    notes: [
      "Search matches label names and descriptions; sort supports name and total issue count with ascending or descending direction.",
      "Private repositories return not_found to unauthorized viewers, while public repository labels remain readable without a session.",
      "Counts are computed from issues, pull-request backing issues, and discussions without exposing private conversation bodies or raw join rows.",
    ],
  },
  {
    id: "repo-labels-create",
    method: "POST",
    path: "/api/repos/{owner}/{repo}/labels",
    title: "Create repository label",
    description:
      "Creates a repository-owned label, normalizes its color, records label event evidence, and returns the created label summary used by the Editorial Labels page.",
    auth: "Signed opengithub session cookie with write, admin, or owner repository permission",
    request: `{
  "name": "needs review",
  "description": "Needs maintainer attention",
  "color": "b85c38"
}`,
    response: `{
  "eventId": "event_01",
  "label": {
    "id": "label_needs_review",
    "name": "needs review",
    "color": "b85c38",
    "description": "Needs maintainer attention"
  }
}`,
    notes: [
      "Validation rejects blank names, duplicate names case-insensitively, malformed colors, oversized descriptions, archived repositories, and insufficient permissions.",
      "The API accepts colors with or without a leading hash but always returns six lowercase hexadecimal characters.",
      "Errors use the standard envelope and never serialize session cookies, OAuth profile data, stack traces, database URLs, or environment values.",
    ],
  },
  {
    id: "repo-labels-update",
    method: "PATCH",
    path: "/api/repos/{owner}/{repo}/labels/{label_id}",
    title: "Update repository label",
    description:
      "Renames or recolors an existing repository label and lets issue, pull request, and discussion chips reflect the updated label through their existing label joins.",
    auth: "Signed opengithub session cookie with write, admin, or owner repository permission",
    request: `{
  "name": "confirmed bug",
  "description": "Confirmed defect",
  "color": "7f6a42"
}`,
    response: `{
  "eventId": "event_02",
  "label": {
    "id": "label_bug",
    "name": "confirmed bug",
    "color": "7f6a42"
  }
}`,
    notes: [
      "Partial updates preserve omitted fields and reject duplicate names, invalid colors, cross-repository label ids, archived repositories, and readers.",
      "Successful updates write repository_label_events and audit evidence so QA can trace renamed labels without duplicating join rows.",
      "Responses return the refreshed label contract only, not raw audit payloads or permission internals.",
    ],
  },
  {
    id: "repo-labels-delete",
    method: "DELETE",
    path: "/api/repos/{owner}/{repo}/labels/{label_id}",
    title: "Delete repository label",
    description:
      "Deletes a repository label and removes its issue, pull request, and discussion assignments without deleting the linked conversations.",
    auth: "Signed opengithub session cookie with write, admin, or owner repository permission",
    response: `{
  "eventId": "event_03",
  "deleted": true,
  "removedAssignments": {
    "issues": 3,
    "pullRequests": 1,
    "discussions": 2
  }
}`,
    notes: [
      "Deletion cascades through issue_labels, pull-request issue labels, and discussion_labels while preserving issues, pull requests, discussion threads, comments, subscriptions, and notifications.",
      "Missing or cross-repository label ids return not_found; readers and archived repositories receive stable permission or validation envelopes.",
      "The Labels page, issue sidebar, pull request sidebar, and discussion sidebar all consume the same label source of truth after deletion.",
    ],
  },
  {
    id: "repo-issue-metadata-labels-update",
    method: "PATCH",
    path: "/api/repos/{owner}/{repo}/issues/{issue_number}/metadata",
    title: "Update issue labels",
    description:
      "Replaces issue sidebar label assignments with repository-owned labels, records metadata timeline evidence, and returns the refreshed issue detail contract.",
    auth: "Signed opengithub session cookie with triage, write, admin, or owner repository permission",
    request: `{
  "labelIds": ["label_bug", "label_docs"],
  "assigneeUserIds": [],
  "milestoneId": null
}`,
    response: `{
  "issue": { "number": 42 },
  "labels": [{ "id": "label_bug", "name": "bug" }],
  "timeline": [{ "eventType": "metadata_changed" }]
}`,
    notes: [
      "Label ids must belong to the same repository; cross-repository labels are rejected before any issue_labels rows are changed.",
      "The mutation computes added and removed labels transactionally and writes timeline/audit metadata with bounded label names and ids.",
      "The same endpoint keeps assignee and milestone payloads compatible with existing issue metadata controls.",
    ],
  },
  {
    id: "repo-pull-metadata-labels-update",
    method: "PATCH",
    path: "/api/repos/{owner}/{repo}/pulls/{pull_number}/metadata",
    title: "Update pull request labels",
    description:
      "Replaces pull request sidebar label assignments through the pull request's backing issue row and returns the refreshed pull request detail contract.",
    auth: "Signed opengithub session cookie with triage, write, admin, or owner repository permission",
    request: `{
  "labelIds": ["label_bug"],
  "assigneeUserIds": [],
  "milestoneId": null
}`,
    response: `{
  "pullRequest": { "number": 7 },
  "labels": [{ "id": "label_bug", "name": "bug" }],
  "timeline": [{ "eventType": "metadata_changed" }]
}`,
    notes: [
      "Pull request labels share the repository label table and issue_labels join semantics used by issues, so label edits and deletes propagate consistently.",
      "Cross-repository labels, archived repositories, missing pull requests, and insufficient permissions fail without exposing private repository metadata.",
      "Timeline, audit, and notification side effects use redacted old/new label metadata.",
    ],
  },
  {
    id: "repo-discussion-metadata-update",
    method: "PATCH",
    path: "/api/repos/{owner}/{repo}/discussions/{discussion_number}/metadata",
    title: "Update repository Discussion metadata",
    description:
      "Updates maintainer-controlled Discussion sidebar metadata such as category and labels, validates category compatibility, replaces label assignments, records events, and returns the refreshed detail contract.",
    auth: "Signed opengithub session cookie with triage, write, admin, or owner repository permission",
    request: `{
  "categorySlug": "ideas",
  "labelIds": ["label_help_wanted", "label_api"]
}`,
    response: `{
  "discussion": {
    "number": 42,
    "category": { "slug": "ideas", "name": "Ideas" }
  },
  "sidebar": {
    "labels": [{ "name": "help wanted" }],
    "events": [{ "eventType": "labels_changed" }]
  }
}`,
    notes: [
      "Category changes reject missing categories, poll/form incompatibilities, archived repositories, disabled Discussions, and insufficient permissions.",
      "Label updates validate repository-owned label ids and replace discussion_labels atomically so the sidebar and list row stay consistent.",
      "Successful metadata edits write discussion_activity_events and notification rows with bounded old/new metadata; responses never expose raw audit payloads, session rows, OAuth data, storage keys, environment variables, or stack traces.",
    ],
  },
  {
    id: "repo-discussion-pin",
    method: "PUT",
    path: "/api/repos/{owner}/{repo}/discussions/{discussion_number}/pin",
    title: "Pin repository Discussion",
    description:
      "Pins a Discussion globally or within its category, stores optional custom pinned copy, writes moderation events, and returns the refreshed detail contract.",
    auth: "Signed opengithub session cookie with triage, write, admin, or owner repository permission",
    request: `{
  "target": "category",
  "categorySlug": "general",
  "title": "Read before filing importer questions",
  "body": "Collect known workarounds here."
}`,
    response: `{
  "discussion": { "number": 42, "pinned": true },
  "moderation": {
    "globalPin": null,
    "categoryPin": {
      "target": "category",
      "categorySlug": "general",
      "customTitle": "Read before filing importer questions",
      "position": 2
    }
  }
}`,
    notes: [
      "Targets are global or category. Category pins require a valid non-poll category and reject incompatible Discussion/category formats.",
      "The server enforces at most four global pins and four pins per category, unique pin targets, private repository privacy, enabled Discussions, and archived repository guardrails.",
      "Successful pins write discussion_pins, discussion_activity_events, audit_events, and bounded notification rows without exposing raw session, OAuth, environment, storage, or audit payload data.",
    ],
  },
  {
    id: "repo-discussion-pin-update",
    method: "PATCH",
    path: "/api/repos/{owner}/{repo}/discussions/{discussion_number}/pin",
    title: "Update pinned Discussion copy",
    description:
      "Updates the custom title/body shown by pinned Discussion cards while preserving the existing pin target and position.",
    auth: "Signed opengithub session cookie with triage, write, admin, or owner repository permission",
    request: `{
  "title": "Importer troubleshooting index",
  "body": "Start with the supported manifest list."
}`,
    response: `{
  "discussion": { "number": 42, "pinned": true },
  "moderation": {
    "globalPin": {
      "target": "global",
      "customTitle": "Importer troubleshooting index",
      "customBody": "Start with the supported manifest list."
    }
  }
}`,
    notes: [
      "Blank custom fields clear the override and the list/detail pages fall back to the Discussion title/body excerpt.",
      "Updates reject missing pins, disabled Discussions, archived repositories, and insufficient permissions with stable no-secret envelopes.",
      "Successful updates write repository.discussion.pin_update activity/audit rows and return the same screen-ready detail shape as the pin endpoint.",
    ],
  },
  {
    id: "repo-discussion-unpin",
    method: "DELETE",
    path: "/api/repos/{owner}/{repo}/discussions/{discussion_number}/pin",
    title: "Unpin repository Discussion",
    description:
      "Removes all active pin records for a Discussion, reorders remaining pins, records moderation history, and returns the refreshed detail contract.",
    auth: "Signed opengithub session cookie with triage, write, admin, or owner repository permission",
    response: `{
  "discussion": { "number": 42, "pinned": false },
  "moderation": { "globalPin": null, "categoryPin": null }
}`,
    notes: [
      "DELETE is idempotent for an already-unpinned Discussion and still returns the detail payload needed by the sidebar.",
      "Unpin writes discussion_activity_events and audit_events with bounded metadata only; remaining pin positions are normalized for list rendering.",
      "Private access failures, archived repositories, disabled Discussions, and malformed discussion numbers never leak private repository or session internals.",
    ],
  },
  {
    id: "repo-discussion-lock",
    method: "PUT",
    path: "/api/repos/{owner}/{repo}/discussions/{discussion_number}/lock",
    title: "Lock repository Discussion",
    description:
      "Locks a Discussion conversation with an explicit reaction policy so comments/replies are blocked while optional reactions can remain available.",
    auth: "Signed opengithub session cookie with triage, write, admin, or owner repository permission",
    request: `{
  "allowReactions": false
}`,
    response: `{
  "discussion": {
    "number": 42,
    "locked": true,
    "lockAllowsReactions": false
  },
  "sidebar": { "events": [{ "eventType": "locked" }] }
}`,
    notes: [
      "Locked Discussions reject new comments, replies, answer changes, and metadata writes that require an unlocked conversation unless the specific moderator endpoint explicitly allows locked updates.",
      "The allowReactions policy is stored with the lock state so the browser can render reaction controls truthfully.",
      "Successful locks write discussion_activity_events, audit_events, and notification rows without serializing raw session rows, OAuth profiles, storage keys, environment variables, or stack traces.",
    ],
  },
  {
    id: "repo-discussion-unlock",
    method: "DELETE",
    path: "/api/repos/{owner}/{repo}/discussions/{discussion_number}/lock",
    title: "Unlock repository Discussion",
    description:
      "Clears a Discussion lock, restores normal conversation affordances, records an unlock event, and returns the refreshed detail contract.",
    auth: "Signed opengithub session cookie with triage, write, admin, or owner repository permission",
    response: `{
  "discussion": {
    "number": 42,
    "locked": false,
    "lockAllowsReactions": true
  },
  "sidebar": { "events": [{ "eventType": "unlocked" }] }
}`,
    notes: [
      "DELETE is safe for an already-unlocked Discussion and keeps the browser state synchronized through the returned detail contract.",
      "Archived repositories, disabled Discussions, missing Discussions, and insufficient permissions return standard no-secret error envelopes.",
      "Unlock history is retained in discussion_activity_events and audit_events; response bodies never expose raw audit payloads.",
    ],
  },
  {
    id: "repo-discussion-recategorize",
    method: "PATCH",
    path: "/api/repos/{owner}/{repo}/discussions/{discussion_number}/category",
    title: "Recategorize repository Discussion",
    description:
      "Moves a non-poll Discussion into another compatible category, updates sidebar/list metadata, records the category-change event, and returns the refreshed detail contract.",
    auth: "Signed opengithub session cookie with triage, write, admin, or owner repository permission",
    request: `{
  "categorySlug": "ideas"
}`,
    response: `{
  "discussion": {
    "number": 42,
    "category": { "slug": "ideas", "name": "Ideas" }
  },
  "sidebar": { "events": [{ "eventType": "category_changed" }] }
}`,
    notes: [
      "Poll Discussions cannot be moved to non-poll categories and normal Discussions cannot be moved into poll-only categories.",
      "Unknown category slugs, archived repositories, disabled Discussions, locked-incompatible operations, and private access failures return stable validation or permission envelopes.",
      "Successful recategorization writes discussion_activity_events, audit_events, and notification rows while preserving discussion numbers, comments, answers, labels, subscriptions, and reactions.",
    ],
  },
  {
    id: "repo-discussion-transfer-targets",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/discussions/{discussion_number}/transfer-targets",
    title: "List Discussion transfer targets",
    description:
      "Returns repositories under the allowed same-owner transfer constraints plus destination category choices for the transfer dialog.",
    auth: "Signed opengithub session cookie with write, admin, or owner repository permission",
    response: `{
  "currentRepository": { "owner": "mona", "name": "octo-app" },
  "discussionNumber": 42,
  "targets": [
    {
      "repositoryId": "repo_docs",
      "owner": "mona",
      "name": "docs",
      "visibility": "private",
      "href": "/mona/docs",
      "discussionsHref": "/mona/docs/discussions",
      "categoryOptions": [{ "slug": "general", "name": "General" }]
    }
  ]
}`,
    notes: [
      "The list excludes the current repository, archived repositories, repositories with Discussions disabled, and repositories the actor cannot write.",
      "Target categories are pre-filtered for compatibility so the browser never renders a dead transfer option.",
      "Private destination metadata is returned only when the signed-in actor has access; no unavailable repository names or counts are leaked.",
    ],
  },
  {
    id: "repo-discussion-transfer",
    method: "POST",
    path: "/api/repos/{owner}/{repo}/discussions/{discussion_number}/transfer",
    title: "Transfer repository Discussion",
    description:
      "Moves a Discussion to an allowed destination repository/category, assigns the destination Discussion number, links source and destination timeline events, and returns the navigation target.",
    auth: "Signed opengithub session cookie with write, admin, or owner repository permission",
    request: `{
  "repositoryId": "repo_docs",
  "categorySlug": "general"
}`,
    response: `{
  "discussionId": "discussion_01",
  "sourceHref": "/mona/octo-app/discussions/42",
  "destinationHref": "/mona/docs/discussions/17",
  "destinationOwner": "mona",
  "destinationRepo": "docs",
  "destinationNumber": 17
}`,
    notes: [
      "Transfers are constrained to allowed same-owner repositories and reject disabled Discussions, archived destinations, incompatible categories, and stale repository ids.",
      "The transaction preserves body, comments, answers, reactions, labels where compatible, subscriptions, notifications, and audit history while writing source and destination events.",
      "Responses include only the concrete destination href and never expose storage keys, session cookies, OAuth profiles, private repository internals, or raw audit payloads.",
    ],
  },
  {
    id: "repo-discussion-delete",
    method: "DELETE",
    path: "/api/repos/{owner}/{repo}/discussions/{discussion_number}/delete",
    title: "Delete repository Discussion",
    description:
      "Deletes a Discussion only after explicit confirmation, stores a tombstone/audit trail, redacts deleted content, and returns the Discussions list destination.",
    auth: "Signed opengithub session cookie with write, admin, or owner repository permission",
    request: `{
  "confirmation": "delete discussion 42",
  "reason": "Spam cleanup"
}`,
    response: `{
  "discussionId": "discussion_01",
  "deleted": true,
  "tombstoneId": "tombstone_01",
  "discussionsHref": "/mona/octo-app/discussions"
}`,
    notes: [
      "Confirmation text must match the server contract before destructive deletion is accepted.",
      "Deletion creates a tombstone and audit/event rows but response bodies and list/detail reads do not leak deleted body/comment contents.",
      "Repeated deletes, malformed ids, disabled Discussions, archived repositories, and private access failures return stable envelopes with no stack traces or secret values.",
    ],
  },
  {
    id: "repo-issue-discussion-conversion-read",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/issues/{issue_number}/convert-to-discussion",
    title: "Read issue-to-Discussion conversion options",
    description:
      "Returns maintainer conversion eligibility, available Discussion categories, comment-copy summary, and already-converted destination metadata for the issue sidebar dialog.",
    auth: "Signed opengithub session cookie with triage, write, admin, or owner repository permission",
    response: `{
  "issue": { "number": 42, "title": "Import fails for workspaces" },
  "eligible": true,
  "alreadyConverted": false,
  "categories": [{ "slug": "q-a", "name": "Q&A", "acceptsAnswers": true }],
  "commentCount": 3
}`,
    notes: [
      "The metadata read is permission-gated and hides conversion affordances from readers and signed-out viewers.",
      "Already-converted issues return the concrete destination Discussion href so duplicate submissions can navigate instead of creating another Discussion.",
      "Disabled Discussions, archived repositories, missing issues, and poll-only category constraints are represented as bounded unavailable reasons for the browser dialog.",
    ],
  },
  {
    id: "repo-issue-discussion-conversion-create",
    method: "POST",
    path: "/api/repos/{owner}/{repo}/issues/{issue_number}/convert-to-discussion",
    title: "Convert issue to Discussion",
    description:
      "Creates a Discussion from an issue title/body/comment metadata, links both timelines, marks the issue as converted, notifies subscribers, and returns the new Discussion href.",
    auth: "Signed opengithub session cookie with triage, write, admin, or owner repository permission",
    request: `{
  "categorySlug": "q-a"
}`,
    response: `{
  "discussionId": "discussion_02",
  "discussionNumber": 91,
  "discussionHref": "/mona/octo-app/discussions/91",
  "issueHref": "/mona/octo-app/issues/42"
}`,
    notes: [
      "Conversion is idempotent for already-converted issues and rejects poll categories, missing categories, disabled Discussions, archived repositories, and insufficient permissions.",
      "The transaction copies bounded issue title/body/comment metadata into Discussion rows, writes issue_timeline_events and discussion_activity_events, and records audit/notification fanout.",
      "Responses and events redact session rows, OAuth data, raw storage keys, private repository metadata unavailable to the actor, environment variables, and stack traces.",
    ],
  },
  {
    id: "repo-discussion-category-settings-read",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/settings/discussions/categories",
    title: "Read repository Discussion category settings",
    description:
      "Returns the maintainer settings contract for Discussion categories, sections, category formats, counts, template metadata, permissions, and admin limits.",
    auth: "Signed opengithub session cookie with repository admin or owner permission",
    response: `{
  "repository": {
    "owner": "mona",
    "name": "octo-app",
    "settingsHref": "/mona/octo-app/discussions/categories/edit"
  },
  "viewer": { "canManage": true },
  "limits": { "maxCategories": 25, "maxSections": 12 },
  "sections": [
    { "id": "section_01", "name": "Product", "position": 1, "categoryCount": 2 }
  ],
  "categories": [
    {
      "id": "cat_01",
      "slug": "q-a",
      "emoji": "🙏",
      "name": "Q&A",
      "description": "Ask and answer product questions.",
      "format": "question_answer",
      "sectionId": "section_01",
      "position": 1,
      "discussionCount": 8,
      "templatePath": ".github/DISCUSSION_TEMPLATE/q-a.yml"
    }
  ]
}`,
    notes: [
      "Readers, anonymous callers, and private repository outsiders cannot read admin-only settings metadata; private access failures use not_found where appropriate.",
      "Archived repositories and organization-policy-disabled Discussions return explicit disabled capabilities so the browser does not render dead mutation controls.",
      "The response contains parsed metadata and counts only. It never includes raw YAML source, session rows, OAuth payloads, token hashes, environment variables, storage keys, or stack traces.",
    ],
  },
  {
    id: "repo-discussion-category-create",
    method: "POST",
    path: "/api/repos/{owner}/{repo}/settings/discussions/categories",
    title: "Create repository Discussion category",
    description:
      "Creates a Discussion category with bounded emoji/name/description metadata, a supported category format, optional section placement, stable ordering, and audit/activity side effects.",
    auth: "Signed opengithub session cookie with repository admin or owner permission",
    request: `{
  "emoji": "💡",
  "name": "Ideas",
  "description": "Share product ideas.",
  "format": "open_ended",
  "sectionId": "section_01"
}`,
    response: `{
  "category": {
    "id": "cat_02",
    "slug": "ideas",
    "name": "Ideas",
    "format": "open_ended",
    "position": 2
  },
  "settings": { "categories": [], "sections": [] }
}`,
    notes: [
      "The server enforces the 25-category limit, unique normalized names/slugs, bounded emoji/name/description lengths, valid formats, and repository-owned section ids.",
      "Supported formats are announcement, open_ended, poll, and question_answer. Poll categories cannot be paired with YAML form templates.",
      "Successful writes record repository.discussion_category.create activity plus audit metadata without copying raw session, OAuth, environment, or storage secrets.",
    ],
  },
  {
    id: "repo-discussion-category-update-delete",
    method: "PATCH",
    path: "/api/repos/{owner}/{repo}/settings/discussions/categories/{category_id}",
    title: "Update or delete repository Discussion category",
    description:
      "Updates category metadata/format/section assignment, or deletes a category after moving existing Discussions to a chosen destination category.",
    auth: "Signed opengithub session cookie with repository admin or owner permission",
    request: `{
  "emoji": "✅",
  "name": "Answered questions",
  "description": "Resolved support conversations.",
  "format": "question_answer",
  "sectionId": null,
  "moveToCategoryId": "cat_02"
}`,
    response: `{
  "category": {
    "id": "cat_01",
    "slug": "answered-questions",
    "format": "question_answer",
    "sectionId": null
  },
  "settings": { "categories": [], "sections": [] }
}`,
    notes: [
      "PATCH updates metadata and category format; DELETE on the same path requires moveToCategoryId when the category owns Discussions, rejects self-destinations, and blocks deleting the final category.",
      "Delete-with-move updates affected discussion category ids atomically while preserving discussion numbers, timelines, answers, polls, form answers, labels, subscriptions, and notifications.",
      "Format changes preserve existing Discussions and validate poll/template compatibility instead of silently corrupting creation or detail behavior.",
    ],
  },
  {
    id: "repo-discussion-category-order",
    method: "PUT",
    path: "/api/repos/{owner}/{repo}/settings/discussions/categories/order",
    title: "Reorder repository Discussion categories",
    description:
      "Persists the maintainer-defined category order and section assignment order used by the settings page, chooser, and category rail.",
    auth: "Signed opengithub session cookie with repository admin or owner permission",
    request: `{
  "items": [
    { "categoryId": "cat_01", "sectionId": "section_01", "position": 1 },
    { "categoryId": "cat_02", "sectionId": null, "position": 2 }
  ]
}`,
    response: `{
  "settings": { "categories": [], "sections": [] }
}`,
    notes: [
      "Every category id must belong to the repository; section ids are optional but must be repository-owned when present.",
      "The server normalizes contiguous positions so repeated saves are idempotent and stable across list/create/detail responses.",
      "Ordering writes record bounded audit/activity metadata and never expose raw audit rows, session cookies, OAuth data, environment variables, or stack traces.",
    ],
  },
  {
    id: "repo-discussion-section-create",
    method: "POST",
    path: "/api/repos/{owner}/{repo}/settings/discussions/sections",
    title: "Create repository Discussion category section",
    description:
      "Creates a one-level Discussion category section for grouping category rows in the settings page, chooser, and category rail.",
    auth: "Signed opengithub session cookie with repository admin or owner permission",
    request: `{
  "name": "Product"
}`,
    response: `{
  "section": { "id": "section_01", "name": "Product", "position": 1 },
  "settings": { "categories": [], "sections": [] }
}`,
    notes: [
      "Section names are unique per repository after normalization and are bounded for browser rendering.",
      "Sections do not nest; the API rejects parent ids or cross-repository section references.",
      "Archived repositories, disabled Discussions, readers, and private outsiders receive stable permission or not_found envelopes without secret leakage.",
    ],
  },
  {
    id: "repo-discussion-section-update-delete",
    method: "PATCH",
    path: "/api/repos/{owner}/{repo}/settings/discussions/sections/{section_id}",
    title: "Update or delete repository Discussion category section",
    description:
      "Renames a Discussion category section or deletes it while moving its categories back to the unsectioned category group.",
    auth: "Signed opengithub session cookie with repository admin or owner permission",
    request: `{
  "name": "Support"
}`,
    response: `{
  "section": { "id": "section_01", "name": "Support", "position": 1 },
  "settings": { "categories": [], "sections": [] }
}`,
    notes: [
      "PATCH renames the section; DELETE on the same path removes only the grouping row and preserves all categories and Discussions.",
      "Deleting a section clears category section ids atomically and then re-normalizes category and section positions.",
      "Every write records repository.discussion_category_section.* activity plus audit metadata without exposing raw audit payloads or session internals.",
    ],
  },
  {
    id: "repo-discussion-section-order",
    method: "PUT",
    path: "/api/repos/{owner}/{repo}/settings/discussions/sections/order",
    title: "Reorder repository Discussion category sections",
    description:
      "Persists the maintainer-defined order for top-level Discussion category sections.",
    auth: "Signed opengithub session cookie with repository admin or owner permission",
    request: `{
  "sectionIds": ["section_01", "section_02"]
}`,
    response: `{
  "settings": { "categories": [], "sections": [] }
}`,
    notes: [
      "All ids must belong to the repository and each id may appear once. Missing or duplicate ids return validation_failed.",
      "Section ordering is independent from per-section category ordering; clients save category order through the category order endpoint.",
      "Responses never include raw session rows, OAuth provider payloads, token hashes, environment variables, storage credentials, or stack traces.",
    ],
  },
  {
    id: "repo-discussion-category-template-read",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/settings/discussions/categories/{category_id}/template",
    title: "Read repository Discussion category template",
    description:
      "Returns the YAML editor contract for a category's `.github/DISCUSSION_TEMPLATE/*.yml` file, parsed form preview, current content hash, and commit/propose affordances.",
    auth: "Signed opengithub session cookie with repository admin or owner permission",
    response: `{
  "category": { "id": "cat_01", "slug": "q-a", "format": "question_answer" },
  "template": {
    "path": ".github/DISCUSSION_TEMPLATE/q-a.yml",
    "content": "body:\\n  - type: textarea\\n    attributes:\\n      label: Context",
    "contentSha": "abc123",
    "parsed": { "valid": true, "fields": [] },
    "canCommit": true,
    "canProposeChange": true
  }
}`,
    notes: [
      "Poll categories return a validation envelope because poll creation and category forms are mutually exclusive.",
      "Malformed YAML remains maintainer-visible in this settings endpoint while reader creation flows keep using a safe generic fallback.",
      "The response is admin-only and never exposes private Git object storage keys, raw session rows, OAuth payloads, environment variables, or stack traces.",
    ],
  },
  {
    id: "repo-discussion-category-template-preview",
    method: "POST",
    path: "/api/repos/{owner}/{repo}/settings/discussions/categories/{category_id}/template/preview",
    title: "Preview repository Discussion category template",
    description:
      "Parses submitted YAML without committing it and returns the sanitized form schema or validation errors for the template editor preview pane.",
    auth: "Signed opengithub session cookie with repository admin or owner permission",
    request: `{
  "content": "body:\\n  - type: input\\n    id: context\\n    attributes:\\n      label: Context\\n    validations:\\n      required: true"
}`,
    response: `{
  "valid": true,
  "parseError": null,
  "fields": [
    { "id": "context", "fieldType": "input", "label": "Context", "required": true }
  ]
}`,
    notes: [
      "Preview uses the same bounded parser as Discussion creation: input, textarea, dropdown, and checkboxes are supported; oversized or unsupported fields are rejected or ignored safely.",
      "Preview does not write Git objects, commits, category rows, form caches, audit events, activity events, notifications, or storage objects.",
      "Markdown descriptions and labels are sanitized before returning to the browser, and raw parser stack traces are never serialized.",
    ],
  },
  {
    id: "repo-discussion-category-template-commit",
    method: "PUT",
    path: "/api/repos/{owner}/{repo}/settings/discussions/categories/{category_id}/template",
    title: "Commit repository Discussion category template",
    description:
      "Writes or updates a category form template file on the default branch or a proposed branch, validates expected content sha, refreshes the parsed form cache, and records audit metadata.",
    auth: "Signed opengithub session cookie with repository admin or owner permission",
    request: `{
  "content": "body:\\n  - type: textarea\\n    id: context\\n    attributes:\\n      label: Context",
  "expectedContentSha": "abc123",
  "commitMessage": "Update Q&A discussion template",
  "targetBranch": "main",
  "proposeChange": false
}`,
    response: `{
  "templatePath": ".github/DISCUSSION_TEMPLATE/q-a.yml",
  "contentSha": "def456",
  "commitOid": "1234abcd",
  "branchName": "main",
  "proposedChangeHref": null,
  "parsed": { "valid": true, "fields": [] }
}`,
    notes: [
      "expectedContentSha protects concurrent edits; conflicts return a stable validation envelope with no raw Git object internals.",
      "Successful commits update repository_files, commits, repository_git_refs or a proposed branch snapshot, category template_path, discussion_category_forms cache rows, repository_activity_events, and audit_events.",
      "YAML size, field count, option count, branch name, commit message, archived repository, poll category, and permission checks all run before any write is committed.",
    ],
  },
  {
    id: "repo-releases-list",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/releases?page=1&pageSize=30",
    title: "List repository releases",
    description:
      "Lists published releases newest first with release badges, author metadata, contributors, rendered notes excerpts, assets, reaction counts, archive links, and bounded pagination.",
    auth: "Public repositories are readable; private repositories require read permission; drafts require write/admin access",
    response: `{
  "items": [
    {
      "id": "release_01",
      "tagName": "v2.0.0",
      "title": "Stable Editorial release",
      "latest": true,
      "prerelease": false,
      "draft": false,
      "assets": [{ "name": "opengithub.tar.gz", "downloadCount": 42 }],
      "reactions": { "rocket": 3, "viewerReaction": null }
    }
  ],
  "total": 1,
  "page": 1,
  "pageSize": 30
}`,
    notes: [
      "Latest means the newest published non-prerelease release; prereleases and drafts never become latest.",
      "Release notes are rendered through the server sanitizer before bodyHtml/bodyExcerpt is returned.",
      "Private repository outsiders receive not_found without leaking release or tag counts.",
    ],
  },
  {
    id: "repo-releases-detail",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/releases/latest",
    title: "Read repository release detail",
    description:
      "Reads the latest release, a release by ID, or a release by tag with full sanitized notes, asset metadata, source archive links, signature state, reaction summary, and viewer permissions.",
    auth: "Public repositories are readable; private repositories require read permission; drafts require write/admin access",
    response: `{
  "id": "release_01",
  "tagName": "v2.0.0",
  "title": "Stable Editorial release",
  "bodyHtml": "<h2>Highlights</h2><p>Safe release notes.</p>",
  "immutable": false,
  "tagSignatureSummary": "Verified tag",
  "viewer": { "canManage": true }
}`,
    notes: [
      "GET /api/repos/{owner}/{repo}/releases/{release_id} reads by ID; GET /api/repos/{owner}/{repo}/releases/tag/{tag} reads tags that may contain slashes.",
      "Missing latest releases and unauthorized private releases return the standard error envelope without revealing private refs.",
    ],
  },
  {
    id: "repo-releases-tags",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/releases/tags?page=1&pageSize=30",
    title: "List repository release tags",
    description:
      "Lists repository tags with target commit metadata, expandable signature details, release notes linkage, source archive URLs, and compare links for the Tags tab.",
    auth: "Public repositories are readable; private repositories require read permission",
    response: `{
  "items": [
    {
      "name": "v2.0.0",
      "shortOid": "abc1234",
      "verified": true,
      "signatureSummary": "Verified tag signature from a trusted public key",
      "releaseId": "release_01",
      "releaseHref": "/mona/octo-app/releases/tag/v2.0.0"
    }
  ],
  "total": 1,
  "page": 1,
  "pageSize": 30
}`,
    notes: [
      "Tag rows reuse repository visibility checks and do not expose private repository refs to outsiders.",
      "zipballHref and tarballHref are authorization-checked API URLs, not raw storage keys.",
      "Requesting zipball or tarball metadata records a repository_archives cache row so worker-backed source archive generation can resume or reuse the same tag target.",
    ],
  },
  {
    id: "repo-releases-manage-context",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/releases/manage",
    title: "Load release management context",
    description:
      "Returns the write-gated context used by the dedicated new/edit release forms: available tags, refs, target defaults, previous-tag candidates, latest policy options, upload limits, and an optional release snapshot.",
    auth: "Signed opengithub session cookie with repository write, maintain, admin, or owner access",
    response: `{
  "canWrite": true,
  "defaultTarget": "main",
  "availableTags": [{ "name": "v2.0.0", "kind": "tag" }],
  "availableRefs": [{ "name": "main", "kind": "branch" }],
  "latestPolicyOptions": [{ "value": "automatic", "label": "Automatic" }],
  "uploadLimits": { "maxAssetBytes": 2147483648, "allowedStorageKinds": ["local", "s3"] }
}`,
    notes: [
      "GET /api/repos/{owner}/{repo}/releases/manage?releaseId={release_id} loads the same contract for edit pages.",
      "Non-writers receive permission envelopes; private repositories do not leak tags, refs, draft releases, or upload limits to outsiders.",
      "Archived repositories and immutable releases return context for display while mutation controls remain disabled.",
    ],
  },
  {
    id: "repo-releases-generated-notes",
    method: "POST",
    path: "/api/repos/{owner}/{repo}/releases/manage/generated-notes",
    title: "Generate release notes preview",
    description:
      "Generates bounded Markdown release notes from opengithub-owned commits and merged pull requests between the selected previous tag and target ref.",
    auth: "Signed opengithub session cookie with repository write, maintain, admin, or owner access",
    request: `{
  "tagName": "v2.0.1",
  "target": "main",
  "previousTagName": "v2.0.0"
}`,
    response: `{
  "title": "v2.0.1",
  "body": "## Changes\\n\\n- abc1234 Merge pull request #42",
  "commitCount": 8,
  "mergedPullRequestCount": 2
}`,
    notes: [
      "Generated notes never call GitHub APIs; all commit, pull request, and contributor data comes from the local repository database.",
      "Empty ranges return deterministic Markdown that users can edit before publishing.",
      "Invalid previous-tag or target-ref selections return validation_failed without modifying draft content.",
    ],
  },
  {
    id: "repo-releases-create",
    method: "POST",
    path: "/api/repos/{owner}/{repo}/releases",
    title: "Create repository release",
    description:
      "Creates a draft or published release for an existing or selected tag, validates write permission, persists sanitized notes, and writes release audit metadata.",
    auth: "Signed opengithub session cookie with repository write, maintain, admin, or owner access",
    request: `{
  "tagName": "v2.0.1",
  "target": "main",
  "title": "Managed release",
  "body": "Release notes",
  "draft": true,
  "prerelease": false
}`,
    response: `{
  "id": "release_02",
  "tagName": "v2.0.1",
  "draft": true,
  "viewer": { "canManage": true }
}`,
    notes: [
      "Duplicate active releases for the same tag return 409; soft-deleted releases do not block future tags.",
      "latestPolicy controls whether publication recalculates, preserves, or explicitly assigns the latest release marker.",
      "Archived repositories and immutable release policies reject write attempts before audit state changes.",
      "Every successful create writes a release_audit_events row with redacted before/after state.",
    ],
  },
  {
    id: "repo-releases-update",
    method: "PATCH",
    path: "/api/repos/{owner}/{repo}/releases/{release_id}",
    title: "Update or delete repository release",
    description:
      "Updates release title, notes, prerelease/draft metadata, or soft-deletes a release after permission, archived repository, immutable release, and tag conflict checks.",
    auth: "Signed opengithub session cookie with repository write, maintain, admin, or owner access",
    request: `{
  "title": "Managed release",
  "body": "Updated notes",
  "draft": false,
  "prerelease": false
}`,
    response: `{
  "id": "release_02",
  "title": "Managed release",
  "draft": false
}`,
    notes: [
      "DELETE /api/repos/{owner}/{repo}/releases/{release_id} soft-deletes the release and hides it from readers.",
      "Delete requests accept deleteTag=true only after explicit UI confirmation; the matching refs/tags/{tag} ref is preserved unless that flag is accepted.",
      "PATCH never accepts asset storage keys or untrusted rendered HTML from callers.",
      "Immutable releases and archived repositories return validation or conflict envelopes without partial updates.",
    ],
  },
  {
    id: "repo-releases-publish",
    method: "POST",
    path: "/api/repos/{owner}/{repo}/releases/{release_id}/publish",
    title: "Publish draft release",
    description:
      "Publishes a draft release, recalculates latest release state, preserves existing asset metadata, and records an audit event.",
    auth: "Signed opengithub session cookie with repository write, maintain, admin, or owner access",
    response: `{
  "id": "release_02",
  "draft": false,
  "publishedAt": "2026-05-03T00:00:00Z",
  "latest": true
}`,
    notes: [
      "Publishing a prerelease does not mark it latest.",
      "Release assets remain local/S3-pluggable metadata until the storage provider returns authorized download metadata.",
    ],
  },
  {
    id: "repo-releases-upload-intents",
    method: "POST",
    path: "/api/repos/{owner}/{repo}/releases/manage/upload-intents",
    title: "Create release asset upload intent",
    description:
      "Creates a bounded local or S3-compatible upload intent before an asset row exists, validating file metadata, checksum, repository mutability, and write permission.",
    auth: "Signed opengithub session cookie with repository write, maintain, admin, or owner access",
    request: `{
  "releaseId": "release_02",
  "assetName": "opengithub.tar.gz",
  "contentType": "application/gzip",
  "byteSize": 128,
  "checksumSha256": "abc123"
}`,
    response: `{
  "id": "intent_01",
  "assetName": "opengithub.tar.gz",
  "storageKind": "local",
  "uploadUrl": "/api/repos/mona/octo-app/releases/manage/upload-intents/intent_01/local-upload",
  "handoffToken": "local-upload-intent_01",
  "status": "pending"
}`,
    notes: [
      "Responses expose short-lived upload URLs or local handoff tokens, never raw S3 or local storage keys.",
      "Duplicate asset names, expired intents, oversized files, unsupported metadata, immutable releases, and archived repositories fail before asset rows are created.",
      "Production S3 signed PUT/GET behavior is provider-backed; local development uses compatible handoff metadata.",
    ],
  },
  {
    id: "repo-releases-upload-complete",
    method: "POST",
    path: "/api/repos/{owner}/{repo}/releases/manage/upload-intents/{intent_id}/complete",
    title: "Complete release asset upload intent",
    description:
      "Completes a pending upload intent, verifies the handoff token and checksum metadata, creates the release asset row, and returns the updated release.",
    auth: "Signed opengithub session cookie with repository write, maintain, admin, or owner access",
    request: `{
  "handoffToken": "local-upload-intent_01",
  "checksumSha256": "abc123"
}`,
    response: `{
  "id": "release_02",
  "assets": [{ "name": "opengithub.tar.gz", "downloadCount": 0 }]
}`,
    notes: [
      "POST /api/repos/{owner}/{repo}/releases/manage/upload-intents/{intent_id}/cancel marks pending intents cancelled without creating asset rows.",
      "Completion records audit and webhook/activity side effects with storage identifiers redacted.",
      "Asset deletion uses DELETE /api/repos/{owner}/{repo}/releases/{release_id}/assets/{asset_id} and preserves download audit history.",
    ],
  },
  {
    id: "repo-releases-assets",
    method: "POST",
    path: "/api/repos/{owner}/{repo}/releases/{release_id}/assets",
    title: "Create or delete release asset metadata",
    description:
      "Adds release asset metadata for the current storage contract and removes asset rows while keeping storage keys redacted from API responses.",
    auth: "Signed opengithub session cookie with repository write, maintain, admin, or owner access",
    request: `{
  "name": "opengithub.tar.gz",
  "label": "Source archive",
  "contentType": "application/gzip",
  "byteSize": 128,
  "checksum": "sha256:abc123"
}`,
    response: `{
  "assets": [
    {
      "id": "asset_01",
      "name": "opengithub.tar.gz",
      "downloadCount": 0,
      "downloadUrl": "/mona/octo-app/releases/assets/asset_01"
    }
  ]
}`,
    notes: [
      "DELETE /api/repos/{owner}/{repo}/releases/{release_id}/assets/{asset_id} removes an asset metadata row after the same permission checks.",
      "Responses never expose S3 or local storage keys; download URLs route back through the Rust API.",
      "Large-file limits, checksums, and real S3 signed upload/download behavior require provider QA.",
    ],
  },
  {
    id: "repo-releases-downloads",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/releases/assets/{asset_id}",
    title: "Authorize release asset download",
    description:
      "Authorizes an asset download, increments counters transactionally, records release_downloads, and returns bounded local/S3-pluggable download metadata.",
    auth: "Public repository assets are readable; private repository assets require read permission",
    response: `{
  "assetId": "asset_01",
  "releaseTagName": "v2.0.0",
  "fileName": "opengithub.tar.gz",
  "contentType": "application/gzip",
  "downloadUrl": "/api/repos/mona/octo-app/releases/assets/asset_01"
}`,
    notes: [
      "GET /api/repos/{owner}/{repo}/releases/zipball/{tag} and /tarball/{tag} authorize source archive downloads and record release_downloads rows.",
      "Download responses expose authorized URLs/metadata only and never raw object storage keys.",
    ],
  },
  {
    id: "repo-releases-reactions",
    method: "POST",
    path: "/api/repos/{owner}/{repo}/releases/{release_id}/reactions",
    title: "Toggle release reaction",
    description:
      "Toggles the signed-in viewer's release reaction and returns the updated reaction summary used by the release cards.",
    auth: "Signed opengithub session cookie with repository read access",
    request: `{
  "content": "rocket"
}`,
    response: `{
  "totalCount": 4,
  "rocket": 3,
  "heart": 1,
  "viewerReaction": "rocket"
}`,
    notes: [
      "Unsupported reaction names return validation_failed; anonymous viewers receive 401.",
      "Repeated toggles are idempotent per user/release/reaction and update the same summary contract.",
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
    id: "org-people-admin",
    method: "GET",
    path: "/api/orgs/{org}/people/admin?tab=members&q=member&page=1&pageSize=30",
    title: "Administer organization people",
    description:
      "Loads the owner/admin people administration state with tab counts, filtered member or invitation rows, row action capabilities, export links, and pagination.",
    auth: "Signed opengithub session cookie with organization owner or admin role",
    response: `{
  "organization": { "login": "namuh", "name": "Namuh Engineering" },
  "viewerState": { "role": "owner", "canAdmin": true },
  "tab": "members",
  "counts": {
    "members": 2,
    "outsideCollaborators": 0,
    "pendingCollaborators": 0,
    "invitations": 1,
    "failedInvitations": 1,
    "securityManagers": 0
  },
  "rows": {
    "items": [
      {
        "login": "mona",
        "role": "owner",
        "membershipVisibility": "public",
        "actionState": { "canChangeRole": false, "canRemove": false, "finalOwner": true }
      }
    ],
    "total": 1,
    "page": 1,
    "pageSize": 30
  }
}`,
    notes: [
      "Private organizations return not_found to outsiders; authenticated non-admin organization members receive 403.",
      "Invitation token hashes, raw session data, private emails outside authorized rows, stack traces, and provider secrets are never returned.",
      "Supported tabs are members, outside_collaborators, pending_collaborators, invitations, failed_invitations, and security_managers.",
    ],
  },
  {
    id: "org-people-invitations",
    method: "POST",
    path: "/api/orgs/{org}/people/invitations",
    title: "Invite organization member",
    description:
      "Creates a pending organization invitation for a username or verified email, stores only a hashed token, records the SES delivery handoff state, and returns fresh admin people state.",
    auth: "Signed opengithub session cookie with organization owner or admin role",
    request: `{
  "emailOrLogin": "member@example.com",
  "role": "member",
  "teamIds": []
}`,
    response: `{
  "tab": "members",
  "counts": { "invitations": 1, "failedInvitations": 1 },
  "invitations": {
    "items": [
      {
        "invitedEmail": "member@example.com",
        "role": "member",
        "emailDeliveryStatus": "degraded",
        "canRetry": true,
        "canCancel": true
      }
    ]
  }
}`,
    notes: [
      "Invitations expire after 7 days and duplicate pending invitations return a conflict envelope.",
      "Missing or local SES credentials persist emailDeliveryStatus=degraded or failed instead of faking successful delivery.",
      "Existing members, invalid roles, and invalid team choices use structured validation_failed or conflict envelopes.",
    ],
  },
  {
    id: "org-people-invitation-retry",
    method: "POST",
    path: "/api/orgs/{org}/people/invitations/{invitation_id}/retry",
    title: "Retry organization invitation delivery",
    description:
      "Retries a failed organization invitation delivery, keeps the same redacted invitation contract, and returns refreshed people administration state.",
    auth: "Signed opengithub session cookie with organization owner or admin role",
    response: `{
  "counts": { "invitations": 1, "failedInvitations": 0 },
  "invitations": {
    "items": [
      { "emailDeliveryStatus": "degraded", "canRetry": true, "canCancel": true }
    ]
  }
}`,
    notes: [
      "Retry writes an organization audit event and never exposes the invitation token hash.",
      "Canceled, accepted, expired, or unknown invitations return a standard not_found or validation envelope.",
    ],
  },
  {
    id: "org-people-invitation-cancel",
    method: "DELETE",
    path: "/api/orgs/{org}/people/invitations/{invitation_id}",
    title: "Cancel organization invitation",
    description:
      "Cancels a pending organization invitation, preserves audit history, and removes it from active invitation tabs.",
    auth: "Signed opengithub session cookie with organization owner or admin role",
    response: `{
  "counts": { "invitations": 0, "failedInvitations": 0 },
  "invitations": { "items": [], "total": 0, "page": 1, "pageSize": 30 }
}`,
    notes: [
      "Cancel is idempotent for active pending invitations only; accepted or expired invitations are not silently mutated.",
      "Audit metadata redacts target emails where the viewer is not authorized to see them.",
    ],
  },
  {
    id: "org-people-visibility",
    method: "PATCH",
    path: "/api/orgs/{org}/people/members/{user_id}/visibility",
    title: "Update organization membership visibility",
    description:
      "Toggles a member between public and private organization membership visibility and returns fresh people administration state.",
    auth: "Signed opengithub session cookie with organization owner or admin role",
    request: `{
  "visibility": "private"
}`,
    response: `{
  "rows": {
    "items": [
      { "login": "mona", "membershipVisibility": "private" }
    ]
  }
}`,
    notes: [
      "Public membership is reflected by the public people endpoint; private membership remains admin-visible only.",
      "Every successful write inserts an organization.people.visibility audit event.",
    ],
  },
  {
    id: "org-people-role",
    method: "PATCH",
    path: "/api/orgs/{org}/people/members/{user_id}/role",
    title: "Update organization member role",
    description:
      "Changes an organization member role after confirmation, enforces final-owner protections, and returns fresh admin people state.",
    auth: "Signed opengithub session cookie with organization owner role",
    request: `{
  "role": "admin"
}`,
    response: `{
  "rows": {
    "items": [
      { "login": "mona", "role": "admin" }
    ]
  }
}`,
    notes: [
      "Demoting the final owner is blocked with a conflict envelope and the actionState.finalOwner flag explains the disabled browser control.",
      "Role changes write organization.people.role audit events without leaking session payloads or stack traces.",
    ],
  },
  {
    id: "org-people-remove",
    method: "DELETE",
    path: "/api/orgs/{org}/people/members/{user_id}",
    title: "Remove organization member",
    description:
      "Removes an organization member, cleans up team memberships, enforces final-owner protections, and returns fresh admin people state.",
    auth: "Signed opengithub session cookie with organization owner role",
    response: `{
  "rows": { "items": [], "total": 0, "page": 1, "pageSize": 30 }
}`,
    notes: [
      "Removing the final owner is blocked with a conflict envelope.",
      "Successful removals write organization.people.remove audit events and do not delete the user account.",
    ],
  },
  {
    id: "org-people-export",
    method: "GET",
    path: "/api/orgs/{org}/people/export?format=csv&tab=members&q=member",
    title: "Export filtered organization people",
    description:
      "Downloads the current owner/admin people filter as JSON or CSV with the same tab, search, and authorization rules used by the admin table.",
    auth: "Signed opengithub session cookie with organization owner or admin role",
    response: `login,display_name,role,membership_visibility,membership_source,team_count,roles_count,two_factor_enabled,has_active_session
mona,Mona Lisa,owner,public,organization,0,1,true,true`,
    notes: [
      "format=json returns an application/json array; format=csv returns text/csv with content-disposition attachment headers.",
      "CSV output is escaped for commas, quotes, and newlines, and never includes invitation tokens, raw session rows, provider secrets, or stack traces.",
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
    id: "issues-global-list",
    method: "GET",
    path: "/api/issues?scope=assigned&state=open&page=1&pageSize=30",
    title: "List global issues",
    description:
      "Lists issues that involve the signed-in user across every readable repository.",
    auth: "Signed opengithub session cookie",
    response: `{
  "items": [
    {
      "repositoryOwner": "mona",
      "repositoryName": "octo-app",
      "number": 18,
      "title": "Polish issue queue",
      "state": "open",
      "href": "/mona/octo-app/issues/18"
    }
  ],
  "total": 1,
  "counts": {
    "created": 4,
    "assigned": 2,
    "mentioned": 1
  }
}`,
    notes: [
      "Supported scopes are created, assigned, and mentioned; mentioned uses notification evidence for issue mentions.",
      "Pull-request backing issues are excluded so the global issue dashboard does not duplicate the global pull request queue.",
      "Repository visibility is enforced before rows, repository filter options, labels, milestones, or project filters are returned.",
    ],
  },
  {
    id: "pulls-global-list",
    method: "GET",
    path: "/api/pulls?scope=review_requests&state=open&page=1&pageSize=30",
    title: "List global pull requests",
    description:
      "Lists pull requests that involve the signed-in user across every readable repository.",
    auth: "Signed opengithub session cookie",
    response: `{
  "items": [
    {
      "repositoryOwner": "mona",
      "repositoryName": "octo-app",
      "number": 42,
      "title": "Improve review queue",
      "state": "open",
      "href": "/mona/octo-app/pull/42"
    }
  ],
  "total": 1,
  "counts": {
    "created": 4,
    "assigned": 2,
    "mentioned": 1,
    "reviewRequests": 3
  }
}`,
    notes: [
      "Supported scopes are created, assigned, mentioned, and review_requests; mentioned uses notification evidence for pull-request mentions.",
      "Repository visibility is enforced before rows, repository filter options, labels, or milestones are returned.",
    ],
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
    id: "pulls-checks",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/pulls/{number}/checks",
    title: "Read pull request check runs",
    description:
      "Returns the PR Checks tab contract by syncing Actions jobs on the head SHA into durable check_runs rows, aggregating required-check state, and exposing per-step annotations.",
    auth: "Public repositories are readable; private repositories require read permission",
    response: `{
  "summary": { "status": "completed", "conclusion": "failure", "totalCount": 2 },
  "requiredStatusChecks": ["ci/test"],
  "checkRuns": [
    {
      "id": "check_01",
      "name": "ci/test",
      "status": "completed",
      "conclusion": "failure",
      "required": true,
      "annotations": [{ "path": "src/app.ts", "startLine": 12, "level": "failure" }]
    }
  ]
}`,
    notes: [
      "Check summaries also feed pull request mergeability and commit status badges.",
      "Workflow command annotations from job logs are stored as check annotations without leaking runner tokens or secrets.",
    ],
  },
  {
    id: "pulls-checks-rerun",
    method: "POST",
    path: "/api/repos/{owner}/{repo}/pulls/{number}/checks/{check_run_id}/rerun",
    title: "Re-run pull request check job",
    description:
      "Queues a single-job workflow re-run for a check run backed by an Actions job and returns the refreshed Actions run detail.",
    auth: "Signed opengithub session cookie with write access",
    response: `{
  "run": { "status": "queued" },
  "jobs": [{ "name": "ci/test", "status": "queued" }]
}`,
    notes: [
      "The check run must belong to the addressed repository and pull request head SHA.",
      "Reruns reuse the existing Actions attempt limit and audit event behavior.",
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
    id: "actions-job-log-stream",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/actions/jobs/{job_id}/logs/stream?after=120",
    title: "Stream workflow job log lines",
    description:
      "Returns a text/event-stream replay window for lines appended after the supplied cursor, plus a cursor event for near-real-time browser refresh.",
    auth: "Optional signed opengithub session cookie; private repositories require read access",
    response: `event: line
id: 121
data: {"lineNumber":121,"content":"cargo test passed","anchor":"L121"}

event: cursor
data: {"nextCursor":121,"finalizedAt":null}`,
    notes: [
      "Line payloads are redacted with the same Actions secret masking used by detail and download endpoints.",
      "The Next.js job log page opens this SSE endpoint for live runs and falls back to timed refresh when EventSource is unavailable.",
    ],
  },
  {
    id: "actions-job-log-chunks",
    method: "POST",
    path: "/api/internal/actions/jobs/{job_id}/logs/chunks",
    title: "Append runner job log chunk",
    description:
      "Accepts stdout or stderr chunks from the assigned runner, appends normalized lines to the searchable log table, updates job_logs S3 metadata, and optionally finalizes the log.",
    auth: "Assigned runner registration token via runnerToken or x-opengithub-runner-token",
    request: `{
  "runnerToken": "ogr_example",
  "stepId": "step_01",
  "content": "cargo test\\nfinished",
  "finalize": true
}`,
    response: `{
  "jobId": "job_01",
  "runId": "run_01",
  "s3Key": "actions/run_01/unit-web/log.txt",
  "bytesWritten": 19,
  "appendedLines": 2,
  "nextCursor": 122,
  "finalizedAt": "2026-05-07T00:00:00Z"
}`,
    notes: [
      "Only the runner assigned to the job can append chunks; bad tokens return 403 without writing lines.",
      "The S3 key follows actions/{run_id}/{job}/log.txt and is never exposed by viewer-facing detail responses.",
    ],
  },
  {
    id: "actions-artifacts-caches",
    method: "POST",
    path: "/_apis/pipelines/workflows/{run_id}/artifacts",
    title: "Upload workflow artifact",
    description:
      "Registers a workflow artifact uploaded by a runner, records retention and storage metadata, and makes it available in the run detail artifact panel.",
    auth: "Signed opengithub session cookie with write access to the run repository",
    request: `{
  "name": "playwright-report",
  "sizeBytes": 2048,
  "digest": "sha256:abc123",
  "retentionDays": 14
}`,
    response: `{
  "id": "artifact_01",
  "name": "playwright-report",
  "sizeBytes": 2048,
  "retentionDays": 14,
  "downloadAvailable": true
}`,
    notes: [
      "Artifact bytes live behind the configured S3-compatible storage key; viewer APIs expose download metadata, not raw storage credentials.",
      "DELETE /api/repos/{owner}/{repo}/actions/artifacts/{artifact_id} marks an artifact deleted for writers.",
    ],
  },
  {
    id: "actions-cache-api",
    method: "GET",
    path: "/_apis/artifactcache/cache/{owner}/{repo}",
    title: "Manage Actions dependency caches",
    description:
      "Lists, reserves, refreshes, and deletes dependency cache metadata for actions/cache-compatible workflow jobs and the repository cache table.",
    auth: "Optional signed session for listing public caches; write access for reserve/delete",
    request: `{
  "key": "node-linux-lock",
  "version": "v1-main",
  "sizeBytes": 2097152,
  "scope": "refs/heads/main"
}`,
    response: `{
  "caches": [{ "key": "node-linux-lock", "version": "v1-main", "sizeBytes": 2097152 }],
  "totalSizeBytes": 2097152,
  "limitBytes": 10737418240
}`,
    notes: [
      "The server applies repository-local LRU eviction after the active cache set exceeds 10 GB.",
      "The Editorial cache page uses /api/repos/{owner}/{repo}/actions/caches and proxies deletes through the same Rust API.",
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
    id: "actions-runners-settings",
    method: "GET",
    path: "/api/repos/{owner}/{repo}/settings/actions/runners",
    title: "Read Actions runner pool",
    description:
      "Returns the admin-only repository runner pool contract: runner health, labels, queue depth, concurrency policy, and a bounded setup command for registering self-hosted runners.",
    auth: "Signed opengithub session cookie with admin access",
    response: `{
  "runners": [
    {
      "id": "runner_01",
      "name": "linux-build-1",
      "labels": ["self-hosted", "ubuntu-latest"],
      "status": "busy",
      "lastHeartbeat": "2026-05-07T00:03:00Z",
      "currentJob": { "jobName": "build", "runNumber": 42 }
    }
  ],
  "queue": {
    "queuedJobs": 3,
    "onlineRunners": 2,
    "busyRunners": 1,
    "offlineRunners": 1,
    "concurrencyLimit": 4,
    "cancelInProgress": false
  },
  "workflowPermissions": {
    "githubTokenPermission": "read",
    "allowPullRequestApproval": false,
    "githubTokenScopes": ["contents:read", "metadata:read", "packages:read"]
  }
}`,
    notes: [
      "Runners with stale heartbeats are marked offline and stale in-progress assignments are timed out before the response is returned.",
      "Only repository admins receive registration tokens and scheduling controls.",
      "Workflow permission policy narrows the run-scoped GITHUB_TOKEN minted for each job; environment secrets remain blocked until protection rules approve release.",
    ],
  },
  {
    id: "actions-runners-create",
    method: "POST",
    path: "/api/repos/{owner}/{repo}/settings/actions/runners",
    title: "Register Actions runner",
    description:
      "Registers a repository-scoped self-hosted runner with normalized labels and returns fresh runner settings.",
    auth: "Signed opengithub session cookie with admin access",
    request: `{
  "name": "linux-build-1",
  "labels": ["self-hosted", "ubuntu-latest"]
}`,
    response: `{
  "runners": [{ "name": "linux-build-1", "status": "offline" }],
  "setup": { "registrationToken": "ogr_...", "expiresInMinutes": 60 }
}`,
    notes: [
      "Duplicate runner names return a conflict envelope.",
      "Blank names or empty label sets return validation_failed without leaking stack traces.",
    ],
  },
  {
    id: "actions-runners-schedule",
    method: "POST",
    path: "/api/repos/{owner}/{repo}/settings/actions/runners/schedule",
    title: "Assign queued Actions jobs",
    description:
      "Schedules queued workflow jobs onto online runners whose labels satisfy each job's runs-on label while respecting repository concurrency limits.",
    auth: "Signed opengithub session cookie with admin access",
    response: `{
  "assigned": [
    { "jobName": "build", "workflowName": "CI", "runNumber": 42 }
  ],
  "queuedJobs": 0
}`,
    notes: [
      "Assigned jobs are moved to in_progress, workflow runs are started, runner status changes to busy, and workflow_job_assignments stores durable audit data.",
      "POST /settings/actions/runners/heartbeat lets a runner report online, busy, or offline health without exposing internal session state.",
      "PATCH /settings/actions/runners updates concurrencyLimit, cancelInProgress, githubTokenPermission, and allowPullRequestApproval policy.",
    ],
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
    id: "package-user-detail",
    method: "GET",
    path: "/api/users/{username}/packages/{package_type}/{package_name}?version=sha256:{digest}",
    title: "Read user package detail",
    description:
      "Reads an owner-scoped package detail page contract with selected version, install commands, digest/platform metadata, blobs, linked repository, README/about content, and viewer admin state.",
    auth: "Anonymous for public packages; signed opengithub session cookie for private/internal package or linked repository access",
    response: `{
  "owner": { "kind": "user", "login": "mona", "href": "/mona" },
  "name": "octo-image",
  "packageType": "container",
  "visibility": "public",
  "selectedVersion": {
    "version": "2.0.0",
    "digest": "sha256:bbbb...",
    "platformOs": "linux",
    "platformArch": "arm64",
    "installCommands": [
      { "label": "Docker", "command": "docker pull ghcr.io/mona/octo-image:2.0.0@sha256:bbbb..." }
    ]
  },
  "versions": [],
  "blobs": [],
  "about": { "html": "<h1>README</h1>", "empty": false },
  "admin": { "canAdmin": true, "settingsHref": "/mona/container/octo-image/settings" }
}`,
    notes: [
      "version accepts a tag or immutable digest and returns 422 for malformed selections.",
      "Storage keys for package blobs are never serialized in this detail contract.",
      "Rendering the detail page does not create package_downloads rows or increment counters.",
    ],
  },
  {
    id: "package-org-detail",
    method: "GET",
    path: "/api/orgs/{org}/packages/{package_type}/{package_name}?version=1.0.0",
    title: "Read organization package detail",
    description:
      "Reads the organization-scoped package detail contract with the same selected-version and README/about shape as user packages while enforcing organization membership, package grants, and linked repository permissions.",
    auth: "Anonymous for public packages; signed opengithub session cookie for private/internal package, organization member, package grant, or linked repository access",
    response: `{
  "owner": { "kind": "organization", "login": "namuh", "href": "/orgs/namuh" },
  "name": "opengithub-web",
  "packageType": "npm",
  "visibility": "internal",
  "selectedVersion": { "version": "1.0.0", "digest": "sha256:aaaa..." },
  "admin": { "canAdmin": false, "settingsHref": null }
}`,
    notes: [
      "Unreadable private/internal organization packages return 404-style redaction without package metadata.",
      "Organization owners receive admin settings hrefs; members with read access do not.",
    ],
  },
  {
    id: "package-download-metadata",
    method: "GET",
    path: "/api/users/{username}/packages/{package_type}/{package_name}/download?version=1.0.0",
    title: "Record package download metadata",
    description:
      "Records a bounded package_downloads increment and returns redacted download metadata for an explicit registry/source download handoff.",
    auth: "Same read rules as package detail",
    response: `{
  "packageId": "package_01",
  "versionId": "version_01",
  "version": "1.0.0",
  "digest": "sha256:aaaa...",
  "downloadCount": 42
}`,
    notes: [
      "Organization packages use /api/orgs/{org}/packages/{package_type}/{package_name}/download with the same query shape.",
      "This metadata endpoint is the only packages-002 read path that increments package_downloads; page rendering and version selection do not.",
      "The response deliberately omits package blob storage keys and signed object URLs; byte streaming belongs to packages-003.",
    ],
  },
  {
    id: "package-settings-read",
    method: "GET",
    path: "/api/users/{username}/packages/{package_type}/{package_name}/settings",
    title: "Read package settings",
    description:
      "Reads the admin-only package settings contract for visibility, explicit package grants, linked repositories, inherited repository access, recent activity, and registry lifecycle capabilities.",
    auth: "Signed opengithub session cookie with package admin, owner, organization owner, or linked repository admin access",
    response: `{
  "package": {
    "name": "octo-image",
    "visibility": "public",
    "latestVersion": "2.0.0",
    "latestDigest": "sha256:bbbb..."
  },
  "explicitPermissions": [],
  "linkedRepositories": [],
  "inheritedRepositoryAccess": [],
  "registryWriteCapabilities": [
    { "key": "visibility", "enabled": true, "reason": "Admins can change package visibility." },
    { "key": "version_lifecycle", "enabled": true, "reason": "Admins can delete or restore package versions." }
  ],
  "recentActivity": []
}`,
    notes: [
      "Organization package settings use /api/orgs/{org}/packages/{package_type}/{package_name}/settings with the same redaction rules.",
      "Readable non-admin viewers receive 403 without package settings data; unreadable packages receive 404 redaction.",
      "The settings mutation endpoint supports visibility updates, direct package grants, repository link changes, package delete/restore, and version delete/restore while retaining audit history.",
      "Settings responses never expose blob storage keys, workflow token hashes, or registry upload storage paths.",
    ],
  },
  {
    id: "package-settings-update",
    method: "PATCH",
    path: "/api/users/{username}/packages/{package_type}/{package_name}/settings",
    title: "Mutate package settings",
    description:
      "Applies admin-gated package lifecycle changes used by the package settings page and container registry retention controls.",
    auth: "Signed opengithub session cookie with package admin, owner, organization owner, or linked repository admin access",
    request: `{
  "action": "updateVisibility",
  "visibility": "private"
}

{
  "action": "grantAccess",
  "login": "teammate",
  "permission": "write"
}

{
  "action": "deleteVersion",
  "versionId": "version_01"
}`,
    response: `{
  "settings": {
    "package": { "name": "octo-image", "visibility": "private" },
    "explicitPermissions": [{ "login": "teammate", "permission": "write" }],
    "linkedRepositories": [],
    "recentActivity": []
  }
}`,
    notes: [
      "Organization packages use /api/orgs/{org}/packages/{package_type}/{package_name}/settings with the same action envelope.",
      "Supported actions are updateVisibility, grantAccess, revokeAccess, linkRepository, unlinkRepository, deletePackage, restorePackage, deleteVersion, and restoreVersion.",
      "Delete actions are soft deletes; pull/tag reads hide deleted versions while blob bytes and audit rows are retained.",
      "Validation, conflict, and forbidden errors use the standard JSON error envelope without echoing sensitive registry metadata.",
    ],
  },
  {
    id: "oci-registry-v2",
    method: "GET",
    path: "/v2/",
    title: "Container registry challenge",
    description:
      "Exposes the Docker Registry HTTP API v2 challenge surface for opengithub container packages.",
    auth: "PAT or workflow package token; anonymous requests receive a Bearer challenge",
    response: `{}`,
    notes: [
      "Use docker login opengithub.namuh.co with a PAT that has packages:read or packages:write.",
      "Workflow jobs may use a short-lived opengithub workflow package token scoped to their repository instead of a long-lived PAT.",
      "The challenge realm is /v2/token and uses service=opengithub-registry.",
      "Local smoke tests can target http://localhost:3016 after seeding a PAT and setting OPENGITHUB_PACKAGE_REGISTRY_STORAGE_DIR.",
    ],
  },
  {
    id: "oci-registry-manifest",
    method: "GET",
    path: "/v2/{namespace}/{image}/manifests/{reference}",
    title: "Read or publish OCI manifests",
    description:
      "Reads manifests by tag or digest and publishes tag-targeted OCI/Docker manifests after referenced blobs have been uploaded.",
    auth: "Anonymous for public pulls; packages:read for private pulls; packages:write PAT or workflow token for pushes",
    request: `{
  "schemaVersion": 2,
  "mediaType": "application/vnd.oci.image.manifest.v1+json",
  "config": { "digest": "sha256:...", "size": 128 },
  "layers": [{ "digest": "sha256:...", "size": 2048 }]
}`,
    response: `{
  "headers": {
    "docker-content-digest": "sha256:manifest..."
  }
}`,
    notes: [
      "GET and HEAD read manifests; PUT publishes a tag-targeted manifest. Pushing by digest is rejected; clients publish to a tag and then pull by the returned digest.",
      "The config blob is inspected for org.opencontainers.image.source, description, licenses, revision, and URL labels.",
      "When a workflow token publishes, the package inherits the workflow repository link, version rows store workflow run/job IDs, and package webhooks are queued.",
      "Storage keys are never serialized in manifest responses or audit payloads.",
      "Digest pulls use the docker-content-digest header returned by PUT or tag reads.",
    ],
  },
  {
    id: "oci-registry-blobs",
    method: "POST",
    path: "/v2/{namespace}/{image}/blobs/uploads/ and /v2/{namespace}/{image}/blobs/{digest}",
    title: "Upload and pull OCI blobs",
    description:
      "Handles resumable blob upload sessions, SHA-256 completion validation, blob pulls, and download accounting.",
    auth: "packages:write for uploads; same read rules as manifests for pulls",
    request: `# publish from CI
echo "$OPENGITHUB_TOKEN" | docker login opengithub.namuh.co -u "$OPENGITHUB_ACTOR" --password-stdin
docker build -t opengithub.namuh.co/mona/octo-image:latest .
docker push opengithub.namuh.co/mona/octo-image:latest
docker pull opengithub.namuh.co/mona/octo-image:latest
docker pull opengithub.namuh.co/mona/octo-image@sha256:manifest...`,
    response: `{
  "digest": "sha256:layer...",
  "range": "0-2047"
}`,
    notes: [
      "POST starts an upload, PATCH appends bytes, PUT completes by digest, DELETE cancels, and GET/HEAD read uploaded blobs.",
      "Only body transfers increment package_downloads; HEAD checks do not count as downloads.",
      "Local development stores bytes under OPENGITHUB_PACKAGE_REGISTRY_STORAGE_DIR; production should back the same storage_key contract with S3.",
      "Upload cancel/expiry preserves audit history without exposing storage paths.",
    ],
  },
  {
    id: "oci-registry-manifest-delete",
    method: "DELETE",
    path: "/v2/{namespace}/{image}/manifests/{reference}",
    title: "Delete a container tag or manifest",
    description:
      "Soft-deletes a tag or digest reference for package admins and packages:write credentials while preserving blobs, downloads, provenance, and audit rows.",
    auth: "packages:write PAT or workflow token with package write/admin permission",
    request: `DELETE /v2/mona/octo-image/manifests/latest

DELETE /v2/mona/octo-image/manifests/sha256:manifest...`,
    response: `202 Accepted
docker-content-digest: sha256:manifest...`,
    notes: [
      "After deletion, tag lists and manifest reads hide the deleted version until an admin restores it from package settings.",
      "Blob storage is retained for audit and retention policy; physical garbage collection is a separate provider-backed maintenance job.",
      "Delete audit rows record actor kind and reference without local or S3 storage paths.",
    ],
  },
  {
    id: "oci-registry-tags",
    method: "GET",
    path: "/v2/{namespace}/{image}/tags/list",
    title: "List container tags",
    description:
      "Returns Docker-compatible tag lists for visible container packages.",
    auth: "Anonymous for public packages; packages:read PAT or workflow token for private/internal packages",
    response: `{
  "name": "mona/octo-image",
  "tags": ["latest", "sha-abc123"]
}`,
    notes: [
      "Private package tag lists return 401 for anonymous clients and redacted 404-style failures for unauthorized tokens.",
      "Workflow tokens can list tags only when their repository is linked to the package.",
      "Deleted package versions are omitted from tag lists until restored by an admin.",
    ],
  },
  {
    id: "repository-watch-settings",
    method: "PATCH",
    path: "/api/repos/{owner}/{repo}/watch",
    title: "Repository watch settings",
    description:
      "Reads or updates a signed-in reader's repository-level notification watch state, including custom event filters and ignore behavior.",
    auth: "Signed opengithub session cookie with repository read access",
    request: `PATCH /api/repos/mona/octo-app/watch
{
  "level": "custom",
  "customEvents": ["issues", "pull_requests", "actions"]
}`,
    response: `{
  "watching": true,
  "level": "custom",
  "label": "Custom",
  "customEvents": ["issues", "pull_requests", "actions"],
  "availableEvents": [
    "issues",
    "pull_requests",
    "releases",
    "discussions",
    "actions",
    "security_alerts",
    "repository_invitations"
  ],
  "watchersCount": 12,
  "ignoreWarning": "Ignoring this repository suppresses repository watch notifications until you choose another watch level."
}`,
    notes: [
      "Supported levels are participating, all, ignore, and custom.",
      "Custom requires at least one selected event; duplicate customEvents are normalized server-side.",
      "PUT and DELETE /api/repos/{owner}/{repo}/watch remain compatibility aliases for participating and unwatch.",
      "watchersCount excludes ignore rows and private repository reads never leak to unauthorized users.",
    ],
  },
  {
    id: "issue-thread-subscription-settings",
    method: "PATCH",
    path: "/api/repos/{owner}/{repo}/issues/{number}/subscription",
    title: "Issue thread notification settings",
    description:
      "Subscribes, unsubscribes, or customizes state-change events for one issue thread without changing repository-wide watch settings.",
    auth: "Signed opengithub session cookie with repository read access",
    request: `{
  "subscribed": true,
  "customEvents": ["closed", "reopened"]
}`,
    response: `{
  "subscribed": true,
  "reason": "subscribed",
  "customEvents": ["closed", "reopened"],
  "canCustomize": true
}`,
    notes: [
      "Issue thread settings override repository watch preferences for the same thread.",
      "customEvents supports closed and reopened for issue state changes.",
      "Participation, mentions, and other direct reactivation reasons can resubscribe later notifications.",
      "Unauthorized private issue reads and writes return redacted not_found-style errors.",
    ],
  },
  {
    id: "pull-request-thread-subscription-settings",
    method: "PATCH",
    path: "/api/repos/{owner}/{repo}/pulls/{number}/subscription",
    title: "Pull request thread notification settings",
    description:
      "Subscribes, unsubscribes, or customizes state-change events for one pull request thread without changing repository-wide watch settings.",
    auth: "Signed opengithub session cookie with repository read access",
    request: `{
  "subscribed": true,
  "customEvents": ["merged", "closed"]
}`,
    response: `{
  "subscribed": true,
  "reason": "subscribed",
  "customEvents": ["merged", "closed"],
  "canCustomize": true
}`,
    notes: [
      "Pull request thread settings override repository watch preferences for the same thread.",
      "customEvents supports merged, closed, and reopened for pull request state changes.",
      "Review requests and direct mentions reactivate delivery after a manual unsubscribe.",
      "Fanout de-dupes recipients after repository watch state, thread overrides, permissions, and actor exclusion are evaluated.",
    ],
  },
  {
    id: "notifications-inbox",
    method: "GET",
    path: "/api/notifications?folder=inbox&tab=unread&group=repository&page=1&pageSize=30",
    title: "Notification inbox",
    description:
      "Returns the signed-in user's notification inbox with folder facets, read-state filters, repository buckets, grouping, and bounded pagination.",
    auth: "Signed opengithub session cookie",
    response: `{
  "query": { "folder": "inbox", "tab": "unread", "group": "repository" },
  "folders": [
    { "id": "inbox", "label": "Inbox", "count": 4, "active": true },
    { "id": "saved", "label": "Saved", "count": 1, "active": false },
    { "id": "done", "label": "Done", "count": 2, "active": false }
  ],
  "groups": [
    {
      "id": "mona/octo-app",
      "label": "mona/octo-app",
      "rows": [
        {
          "id": "notif_01",
          "title": "Triage dashboard setup workflow",
          "unread": true,
          "saved": false,
          "done": false,
          "subscribed": true,
          "openHref": "/notifications/notif_01/open?next=/mona/octo-app/issues/42"
        }
      ]
    }
  ],
  "unreadCount": 4
}`,
    notes: [
      "folder=inbox excludes done notifications and manually unsubscribed threads.",
      "folder=saved retains saved notifications even after they are moved to Done.",
      "folder=done returns completed notifications that can be moved back to Inbox.",
      "Repository and subject links are permission-aware and do not reveal private repository metadata to unauthorized users.",
    ],
  },
  {
    id: "notifications-read-state",
    method: "PATCH",
    path: "/api/notifications/{notification_id}/read and /api/notifications/{notification_id}/unread",
    title: "Update notification read state",
    description:
      "Marks one notification read or unread and returns fresh row state plus folder and unread counts for optimistic UI reconciliation.",
    auth: "Signed opengithub session cookie for the notification owner",
    response: `{
  "id": "notif_01",
  "unread": false,
  "saved": false,
  "done": false,
  "subscribed": true,
  "lastReadAt": "2026-05-04T00:00:00Z",
  "unreadCount": 3,
  "folderCounts": { "inbox": 4, "saved": 1, "done": 2 }
}`,
    notes: [
      "Unknown or cross-user notification IDs return notification_not_found.",
      "Read/unread mutations preserve saved, done, and subscription state.",
    ],
  },
  {
    id: "notifications-retention-state",
    method: "PATCH",
    path: "/api/notifications/{notification_id}/save, /unsave, /done, and /inbox",
    title: "Update notification retention state",
    description:
      "Saves, unsaves, completes, or restores one notification while returning server-confirmed row state and counts.",
    auth: "Signed opengithub session cookie for the notification owner",
    response: `{
  "id": "notif_01",
  "unread": true,
  "saved": true,
  "done": true,
  "subscribed": true,
  "savedAt": "2026-05-04T00:00:00Z",
  "folderCounts": { "inbox": 3, "saved": 2, "done": 3 }
}`,
    notes: [
      "Done removes rows from Inbox but does not clear unread or saved state.",
      "Move to inbox clears done_at and makes a subscribed thread visible in Inbox again.",
      "Saved notifications remain addressable from Saved until explicitly unsaved.",
    ],
  },
  {
    id: "notifications-subscription-state",
    method: "PATCH",
    path: "/api/notifications/{notification_id}/subscribe and /api/notifications/{notification_id}/unsubscribe",
    title: "Update notification subscription state",
    description:
      "Subscribes or unsubscribes the notification's thread and returns row/count state for the current user.",
    auth: "Signed opengithub session cookie for the notification owner",
    response: `{
  "id": "notif_01",
  "subscribed": false,
  "unreadCount": 3,
  "folderCounts": { "inbox": 3, "saved": 1, "done": 2 }
}`,
    notes: [
      "Unsubscribe hides the thread from Inbox but leaves Saved and Done retention queryable.",
      "Participation, direct mentions, team mentions, and review requests reactivate a thread subscription on later notification creation.",
      "Repository watch state is a fallback only; direct thread unsubscribe wins until a reactivation reason occurs.",
    ],
  },
  {
    id: "notifications-bulk-triage",
    method: "POST",
    path: "/api/notifications/bulk",
    title: "Bulk notification triage",
    description:
      "Applies one triage action to up to 100 notification IDs and returns per-row success or failure details for partial rollback.",
    auth: "Signed opengithub session cookie for the notification owner",
    request: `{
  "notificationIds": ["notif_01", "notif_02"],
  "action": "done"
}`,
    response: `{
  "action": "done",
  "updated": [
    {
      "id": "notif_01",
      "unread": true,
      "saved": false,
      "done": true,
      "subscribed": true
    }
  ],
  "failed": [
    {
      "id": "notif_missing",
      "code": "notification_not_found",
      "message": "Notification was not found."
    }
  ],
  "unreadCount": 3,
  "folderCounts": { "inbox": 3, "saved": 1, "done": 3 }
}`,
    notes: [
      "Supported actions are read, unread, save, unsave, done, inbox, subscribe, and unsubscribe.",
      "Empty, duplicate, or more than 100 notificationIds return validation_failed.",
      "Failed rows stay selected in the browser so the client can retry or inspect them.",
    ],
  },
  {
    id: "notifications-custom-filters",
    method: "POST",
    path: "/api/notifications/custom-filters",
    title: "Create notification custom filter",
    description:
      "Creates one signed-in user's saved notification inbox filter and returns the full default/custom filter settings payload.",
    auth: "Signed opengithub session cookie",
    request: `{
  "name": "My review queue",
  "queryString": "repo:mona/octo-app reason:review_requested is:unread"
}`,
    response: `{
  "limit": 15,
  "remaining": 14,
  "allowedQualifiers": ["repo", "org", "author", "is", "reason"],
  "defaultFilters": [
    { "id": "assigned", "name": "Assigned", "queryString": "reason:assigned" }
  ],
  "customFilters": [
    {
      "id": "filter_01",
      "name": "My review queue",
      "queryString": "repo:mona/octo-app reason:review_requested is:unread",
      "position": 1,
      "href": "/notifications?q=repo%3Amona%2Focto-app%20reason%3Areview_requested%20is%3Aunread"
    }
  ]
}`,
    notes: [
      "GET /api/notifications/custom-filters returns the same settings payload without creating a filter.",
      "PATCH or DELETE /api/notifications/custom-filters/{filter_id} updates or removes one owned custom filter.",
      "Each user can store at most 15 custom filters; create returns validation_failed after the limit.",
      "Validation accepts repo:, org:, author:, is:, and reason: only; NOT, exclusion, unsupported qualifiers, and full-text tokens are rejected.",
      "repo: and org: qualifiers validate visibility or membership without revealing inaccessible private names.",
    ],
  },
  {
    id: "notifications-delivery-preferences",
    method: "PATCH",
    path: "/api/notifications/delivery-preferences",
    title: "Update notification delivery preferences",
    description:
      "Reads or saves personal notification delivery channels for web, email, and CLI delivery across subscription and system event categories.",
    auth: "Signed opengithub session cookie",
    request: `{
  "defaultEmailId": "email_01",
  "preferences": [
    { "key": "watching", "channels": ["web", "email"] },
    { "key": "actions", "channels": ["web", "cli"] }
  ]
}`,
    response: `{
  "defaultEmailId": "email_01",
  "emailChannelAvailable": true,
  "sesSenderReady": true,
  "emails": [
    {
      "id": "email_01",
      "email": "mona@example.com",
      "verified": true,
      "isPrimary": true
    }
  ],
  "preferences": [
    {
      "key": "watching",
      "label": "Watching",
      "section": "subscriptions",
      "channels": ["web", "email"],
      "supportedChannels": ["web", "email", "cli"],
      "disabled": false
    },
    {
      "key": "dependabot",
      "label": "Dependabot",
      "section": "system",
      "channels": ["web"],
      "supportedChannels": ["web", "email", "cli"],
      "disabled": false
    }
  ]
}`,
    notes: [
      "GET /api/notifications/delivery-preferences returns the same settings payload without changing preferences.",
      "Email channels require a verified user_email_addresses row selected as defaultEmailId.",
      "Successful writes insert notifications.delivery_preferences.update security audit events.",
      "Dependabot alert triage writes notification rows for assignees and repository watchers when alert state or assignments change.",
      "notification_email_deliveries stores future SES delivery attempts without exposing provider secrets to the browser.",
    ],
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
    id: "admin-search-index",
    method: "GET",
    path: "/api/admin/search",
    title: "Search index pipeline status",
    description:
      "Returns the admin observability contract for write-time search indexing events, document counts, and repositories needing attention.",
    auth: "Signed opengithub session cookie",
    response: `{
  "documents": [
    { "kind": "code", "total": 42, "latestIndexedAt": "2026-05-07T00:00:00Z" }
  ],
  "events": { "queued": 0, "running": 0, "completed": 12, "failed": 0 },
  "recentEvents": [
    { "eventType": "repo.push.code.reindex", "resourceKind": "code", "status": "completed" }
  ],
  "staleRepositories": []
}`,
    notes: [
      "Issue, pull request, code, and commit index writers append search_index_events rows.",
      "The admin page at /admin/search renders this response with Editorial status chips and no write-side JavaScript auth.",
      "Failures use the standard code/message error envelope and do not expose database or session secrets.",
    ],
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
  {
    id: "api-rate-limit-read",
    method: "GET",
    path: "/rate_limit",
    title: "Read REST API rate limits",
    description:
      "Returns the current core and search buckets for the signed session, bearer token, or anonymous IP address.",
    auth: "Optional signed opengithub session cookie or Bearer personal access token",
    response: `{
  "resources": {
    "core": { "limit": 5000, "remaining": 4999, "reset": 1778155200, "used": 1, "resource": "core" },
    "search": { "limit": 30, "remaining": 30, "reset": 1778151660, "used": 0, "resource": "search" }
  },
  "rate": { "limit": 5000, "remaining": 4999, "reset": 1778155200, "used": 1, "resource": "core" }
}`,
    notes: [
      "Every REST response includes X-RateLimit-Limit, X-RateLimit-Remaining, X-RateLimit-Reset, X-RateLimit-Used, X-RateLimit-Resource, and X-GitHub-Api-Version-Selected.",
      "Authenticated requests receive 5000 core requests per hour; anonymous IPs receive 60 core requests per hour; search receives 30 requests per minute.",
      "Requests over quota return 403 with code rate_limited and no stack traces or token material.",
      "X-GitHub-Api-Version pins the selected REST API version; missing headers default to 2022-11-28.",
    ],
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
