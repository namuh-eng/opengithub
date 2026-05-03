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
