import { fireEvent, render, screen, within } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { ApiDocsPage } from "@/components/ApiDocsPage";
import { apiEndpointDocs } from "@/lib/api-docs";

describe("ApiDocsPage", () => {
  it("documents every implemented api-001 resource family", {
    timeout: 60_000,
  }, () => {
    render(<ApiDocsPage />);

    expect(
      screen.getByRole("heading", {
        name: "Build against implemented opengithub APIs",
      }),
    ).toBeVisible();
    expect(apiEndpointDocs.length).toBeGreaterThanOrEqual(62);

    for (const endpoint of apiEndpointDocs) {
      const card = screen
        .getByRole("heading", { name: endpoint.title })
        .closest("section");
      expect(card).not.toBeNull();
      expect(
        within(card as HTMLElement).getByText(endpoint.method),
      ).toBeVisible();
      expect(
        within(card as HTMLElement).getByText(endpoint.path),
      ).toBeVisible();
      expect(
        within(card as HTMLElement).getByText(endpoint.auth),
      ).toBeVisible();
    }

    expect(screen.getByText("/api/user")).toBeVisible();
    expect(
      screen.getByText(
        "/api/orgs/{org}/repositories?q=router&type=public&language=Rust&page=1&pageSize=30",
      ),
    ).toBeVisible();
    expect(
      screen.getByText("/api/orgs/{org}/people?q=member&page=1&pageSize=30"),
    ).toBeVisible();
    expect(
      screen.getAllByText(/Private organizations return not_found/)[0],
    ).toBeVisible();
    expect(
      screen.getByText(
        /Signed-out and outside viewers see public members only/,
      ),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/orgs/{org}/people/admin?tab=members&q=member&page=1&pageSize=30",
      ),
    ).toBeVisible();
    expect(
      screen.getByText("/api/orgs/{org}/people/invitations"),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/orgs/{org}/people/invitations/{invitation_id}/retry",
      ),
    ).toBeVisible();
    expect(
      screen.getByText("/api/orgs/{org}/people/invitations/{invitation_id}"),
    ).toBeVisible();
    expect(
      screen.getByText("/api/orgs/{org}/people/members/{user_id}/visibility"),
    ).toBeVisible();
    expect(
      screen.getByText("/api/orgs/{org}/people/members/{user_id}/role"),
    ).toBeVisible();
    expect(
      screen.getByText("/api/orgs/{org}/people/members/{user_id}"),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/orgs/{org}/people/export?format=csv&tab=members&q=member",
      ),
    ).toBeVisible();
    expect(
      screen.getByText(/Supported tabs are members, outside_collaborators/),
    ).toBeVisible();
    expect(
      screen.getByText(/emailDeliveryStatus=degraded or failed/),
    ).toBeVisible();
    expect(
      screen.getByText(/Demoting the final owner is blocked/),
    ).toBeVisible();
    expect(
      screen.getByText(/Removing the final owner is blocked/),
    ).toBeVisible();
    expect(screen.getByText(/format=csv returns text\/csv/)).toBeVisible();
    expect(
      screen.getByText(/never includes invitation tokens, raw session rows/),
    ).toBeVisible();
    expect(
      screen.getByText("/api/organizations/slug-availability?name=Acme%20Labs"),
    ).toBeVisible();
    expect(screen.getByText("/api/organizations")).toBeVisible();
    expect(
      screen.getByText(
        /Reserved slugs and existing user or organization logins return available=false/,
      ),
    ).toBeVisible();
    expect(
      screen.getByText(
        /The creator receives the owner role in organization_memberships/,
      ),
    ).toBeVisible();
    expect(
      screen.getAllByText(/default organization_policy_settings/)[0],
    ).toBeVisible();
    expect(
      screen.getByText(
        /Duplicate, reserved, invalid email, missing terms, and rate-limit failures/,
      ),
    ).toBeVisible();
    expect(
      screen.getByText(/contact email and company fields are not written/),
    ).toBeVisible();
    expect(
      screen.getAllByText("/api/orgs/{org}/settings/profile")[0],
    ).toBeVisible();
    expect(
      screen.getByText("/api/orgs/{org}/settings/profile/rename"),
    ).toBeVisible();
    expect(
      screen.getByText(/organization members without owner role receive 403/),
    ).toBeVisible();
    expect(
      screen.getByText(/four bounded social account providers/),
    ).toBeVisible();
    expect(screen.getByText(/Partial patches preserve fields/)).toBeVisible();
    expect(
      screen.getByText(/organization.profile_settings.update audit event/),
    ).toBeVisible();
    expect(
      screen.getByText(/Reserved, duplicate user, and duplicate organization/),
    ).toBeVisible();
    expect(
      screen.getByText(/archive and delete execution remain unsupported/),
    ).toBeVisible();
    expect(
      screen.getAllByText("/api/orgs/{org}/settings/member-privileges")[0],
    ).toBeVisible();
    expect(
      screen.getByRole("heading", {
        name: "Read organization member privileges",
      }),
    ).toBeVisible();
    expect(
      screen.getByRole("heading", {
        name: "Update organization member privileges",
      }),
    ).toBeVisible();
    expect(
      screen.getByText(
        /Base repository permission is inherited by organization members/,
      ),
    ).toBeVisible();
    expect(
      screen.getByText(
        /Repository creation, team creation, Pages publishing, discussions, forking/,
      ),
    ).toBeVisible();
    expect(
      screen.getByText(
        /Base repository permission and Projects base permission changes return confirmation_required/,
      ),
    ).toBeVisible();
    expect(
      screen.getByText(/organization.policy.update audit events/),
    ).toBeVisible();
    expect(
      screen.getByText(
        /Policy-denied repository creation, Pages source updates, team creation/,
      ),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/users/{username}/projects?q=is%3Aopen&state=open&tab=projects&sort=recently_updated&page=1&pageSize=30",
      ),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/orgs/{org}/projects?q=roadmap&state=open&tab=projects&sort=name_asc&page=1&pageSize=30",
      ),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/projects?q=release&state=open&tab=projects&sort=created_desc&page=1&pageSize=30",
      ),
    ).toBeVisible();
    expect(screen.getByText("/api/projects/{project_id}/copies")).toBeVisible();
    expect(
      screen.getByText(
        "/api/projects/{project_id}/workspace?view=1&q=is%3Aopen&sort=manual&group=Status&slice=Priority&page=1&pageSize=50",
      ),
    ).toBeVisible();
    expect(
      screen.getByText("/api/projects/{project_id}/views/{view_id}/state"),
    ).toBeVisible();
    expect(
      screen.getByText("/api/projects/{project_id}/views/{view_id}/layout"),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/projects/{project_id}/items/{item_id}/fields/{field_id}",
      ),
    ).toBeVisible();
    expect(screen.getByText("/api/projects/{project_id}/items")).toBeVisible();
    expect(
      screen.getByText("/api/projects/{project_id}/items/bulk"),
    ).toBeVisible();
    expect(
      screen.getByText("/api/projects/{project_id}/items/{item_id}/position"),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/projects/{project_id}/views/{view_id}/roadmap-settings",
      ),
    ).toBeVisible();
    expect(
      screen.getAllByText("/api/projects/{project_id}/items/{item_id}")[0],
    ).toBeVisible();
    expect(
      screen.getAllByText("/api/projects/{project_id}/settings/fields")[0],
    ).toBeVisible();
    expect(
      screen.getByText("/api/projects/{project_id}/fields/{field_id}"),
    ).toBeVisible();
    expect(
      screen.getAllByText(
        "/api/projects/{project_id}/fields/{field_id}/options",
      )[0],
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/projects/{project_id}/fields/{field_id}/options/reorder",
      ),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/projects/{project_id}/fields/{field_id}/iterations/settings",
      ),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/projects/{project_id}/fields/{field_id}/iteration-breaks",
      ),
    ).toBeVisible();
    expect(
      screen.getByText(/Supported sort values are recently_updated/),
    ).toBeVisible();
    expect(
      screen.getByText(/Organization policy-disabled Projects return/),
    ).toBeVisible();
    expect(screen.getAllByText(/project_repositories/)[0]).toBeVisible();
    expect(
      screen.getByText(/Successful writes append audit_events/),
    ).toBeVisible();
    expect(
      screen.getByText(/Linked issue and pull request rows are omitted/),
    ).toBeVisible();
    expect(
      screen.getByText(/layoutChoices exposes Table, Board, and Roadmap/),
    ).toBeVisible();
    expect(
      screen.getByText(/boardConfig computes eligible column and swimlane/),
    ).toBeVisible();
    expect(
      screen.getByText(/roadmapConfig returns compatible start, target/),
    ).toBeVisible();
    expect(
      screen.getByText(/layout must be table, board, or roadmap/),
    ).toBeVisible();
    expect(
      screen.getByText(/Board moves can include groupFieldId and groupValue/),
    ).toBeVisible();
    expect(screen.getByText(/Only roadmap views can be updated/)).toBeVisible();
    expect(
      screen.getByText(/zoom must be month, quarter, or year/),
    ).toBeVisible();
    expect(screen.getAllByText(/expectedUpdatedAt protects/)[0]).toBeVisible();
    expect(
      screen.getByText(
        /Custom project fields update project_item_field_values/,
      ),
    ).toBeVisible();
    expect(
      screen.getByText(/Duplicate linked issues or pull requests are rejected/),
    ).toBeVisible();
    expect(
      screen.getByText(/Bulk requests validate every requested item/),
    ).toBeVisible();
    expect(
      screen.getByText(/Board moves can include groupFieldId/),
    ).toBeVisible();
    expect(
      screen.getByText(/Removal archives the project item relationship/),
    ).toBeVisible();
    expect(
      screen.getByRole("heading", { name: "Read Project item detail" }),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/projects/{project_id}/items/archived?itemType=draft_issue&page=1&pageSize=30",
      ),
    ).toBeVisible();
    expect(
      screen.getByText("/api/projects/{project_id}/items/{item_id}/draft"),
    ).toBeVisible();
    expect(
      screen.getByText("/api/projects/{project_id}/items/{item_id}/comments"),
    ).toBeVisible();
    expect(
      screen.getByText("/api/projects/{project_id}/conversion-targets"),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/projects/{project_id}/items/{item_id}/convert-to-issue",
      ),
    ).toBeVisible();
    expect(
      screen.getByText("/api/projects/{project_id}/items/{item_id}/archive"),
    ).toBeVisible();
    expect(
      screen.getByText("/api/projects/{project_id}/items/{item_id}/restore"),
    ).toBeVisible();
    expect(
      screen.getByText(/Draft comments and activity are project-only/),
    ).toBeVisible();
    expect(screen.getByText(/Duplicate submits are idempotent/)).toBeVisible();
    expect(
      screen.getByText(/active workspace read no longer returns the item/),
    ).toBeVisible();
    expect(
      screen.getByText(/Built-in fields are returned with editable=false/),
    ).toBeVisible();
    expect(
      screen.getByText(
        /Creating an iteration field seeds three default cycles/,
      ),
    ).toBeVisible();
    expect(
      screen.getAllByText(/DELETE \/api\/projects\/\{project_id\}\/fields/)[0],
    ).toBeVisible();
    expect(
      screen.getByText(
        /PATCH \/api\/projects\/\{project_id\}\/fields\/\{field_id\}\/options\/reorder/,
      ),
    ).toBeVisible();
    expect(
      screen.getByText(/Workspace filters understand iteration values/),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/orgs/{org}/teams?q=platform&visibility=all&page=1&pageSize=30",
      ),
    ).toBeVisible();
    expect(screen.getByText("/api/orgs/{org}/teams")).toBeVisible();
    expect(screen.getByText("/api/orgs/{org}/teams/{team_slug}")).toBeVisible();
    expect(
      screen.getByText(
        /Supported visibility filters are all, visible, and secret/,
      ),
    ).toBeVisible();
    expect(
      screen.getByText(
        /Visible teams are discoverable and @mentionable by organization members/,
      ),
    ).toBeVisible();
    expect(
      screen.getByText(/Secret teams cannot be nested under any parent/),
    ).toBeVisible();
    expect(
      screen.getByText(/parent cycles are rejected with validation_failed/),
    ).toBeVisible();
    expect(
      screen.getByText(
        /notificationsEnabled flag controls team-mention fanout/,
      ),
    ).toBeVisible();
    expect(
      screen.getByText(
        /Parent team repository permissions cascade to children/,
      ),
    ).toBeVisible();
    expect(
      screen.getByText(/direct versus inherited team grants/),
    ).toBeVisible();
    expect(
      screen.getByText(/never invitation tokens or private member records/),
    ).toBeVisible();
    expect(screen.getByText("/api/repos/{owner}/{repo}/issues")).toBeVisible();
    expect(
      screen.getAllByText("/api/repos/{owner}/{repo}/settings")[0],
    ).toBeVisible();
    expect(
      screen.getByText(
        /Every successful write inserts a repository.settings.update audit event/,
      ),
    ).toBeVisible();
    expect(
      screen.getByText(
        /Archived repositories reject every settings mutation except unarchive/,
      ),
    ).toBeVisible();
    expect(
      screen.getAllByText("/api/repos/{owner}/{repo}/settings/access")[0],
    ).toBeVisible();
    expect(
      screen.getByText("/api/repos/{owner}/{repo}/settings/access/teams"),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/settings/access/collaborators/{user_id}",
      ),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/settings/access/teams/{team_id}",
      ),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/settings/access/invitations/{invitation_id}",
      ),
    ).toBeVisible();
    expect(
      screen.getByText(/Demoting the final owner\/admin access path/),
    ).toBeVisible();
    expect(
      screen.getAllByText(/emailDeliveryStatus=degraded/)[0],
    ).toBeVisible();
    expect(
      screen.getAllByText("/api/repos/{owner}/{repo}/settings/branches")[0],
    ).toBeVisible();
    expect(
      screen.getByText("/api/repos/{owner}/{repo}/settings/branches/rules"),
    ).toBeVisible();
    expect(
      screen.getByText("/api/repos/{owner}/{repo}/settings/branches/rulesets"),
    ).toBeVisible();
    expect(
      screen.getByText(
        /Non-admin readers can see active and evaluate policy explanations/,
      ),
    ).toBeVisible();
    expect(
      screen.getByText(/branch_policy_blocked for locked branches/),
    ).toBeVisible();
    expect(
      screen.getByText(
        /Matching previews use the same bounded fnmatch-style pattern matcher/,
      ),
    ).toBeVisible();
    expect(
      screen.getAllByText("/api/repos/{owner}/{repo}/settings/hooks")[0],
    ).toBeVisible();
    expect(
      screen.getAllByText(
        "/api/repos/{owner}/{repo}/settings/hooks/{hook_id}",
      )[0],
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/settings/hooks/{hook_id}/ping",
      ),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/settings/hooks/{hook_id}/deliveries/{delivery_id}",
      ),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/settings/hooks/{hook_id}/deliveries/{delivery_id}/redeliver",
      ),
    ).toBeVisible();
    expect(
      screen.getByText(/Plaintext webhook secrets are never returned/),
    ).toBeVisible();
    expect(screen.getByText(/x-hub-signature-256/)).toBeVisible();
    expect(
      screen.getByText(/Oversized request and response bodies/),
    ).toBeVisible();
    expect(
      screen.getAllByText("/api/repos/{owner}/{repo}/settings/secrets")[0],
    ).toBeVisible();
    expect(
      screen.getByText("/api/repos/{owner}/{repo}/settings/secrets/secrets"),
    ).toBeVisible();
    expect(
      screen.getByText("/api/repos/{owner}/{repo}/settings/secrets/variables"),
    ).toBeVisible();
    expect(
      screen.getByText(/Secret responses expose only metadata/),
    ).toBeVisible();
    expect(
      screen.getByText(/blocks secrets for untrusted fork pull_request events/),
    ).toBeVisible();
    expect(
      screen.getByText(
        /Validation and conflict responses never echo submitted secret values/,
      ),
    ).toBeVisible();
    expect(
      screen.getByText(/secret values cannot and are masked from job logs/),
    ).toBeVisible();
    expect(
      screen.getAllByText("/api/repos/{owner}/{repo}/settings/pages")[0],
    ).toBeVisible();
    expect(
      screen.getByText("/api/repos/{owner}/{repo}/settings/pages/source"),
    ).toBeVisible();
    expect(
      screen.getByText("/api/repos/{owner}/{repo}/settings/pages/domain"),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/settings/pages/domain/recheck",
      ),
    ).toBeVisible();
    expect(
      screen.getByText("/api/repos/{owner}/{repo}/settings/pages/https"),
    ).toBeVisible();
    expect(
      screen.getByText("/api/repos/{owner}/{repo}/settings/pages/deployments"),
    ).toBeVisible();
    expect(
      screen.getByText("/api/repos/{owner}/{repo}/settings/pages/unpublish"),
    ).toBeVisible();
    expect(
      screen.getByText(/Non-admin readers can inspect public live status/),
    ).toBeVisible();
    expect(
      screen.getByText(/CloudFront alias activation remains gated/),
    ).toBeVisible();
    expect(
      screen.getByText(/Unpublish never deletes repository Git objects/),
    ).toBeVisible();
    expect(
      screen.getByText("/api/repos/{owner}/{repo}/pulse?period=1w"),
    ).toBeVisible();
    expect(
      screen.getByRole("heading", { name: "Repository Pulse insights" }),
    ).toBeVisible();
    expect(
      screen.getAllByText(/Supported period values are 24h, 3d, 1w, and 1m/)[0],
    ).toBeVisible();
    expect(
      screen.getByText(/metric hrefs so browser cards navigate/),
    ).toBeVisible();
    expect(
      screen.getAllByText(/authorStatus and isBot metadata/)[0],
    ).toBeVisible();
    expect(
      screen.getAllByText(/repository_insight_snapshots stores/)[0],
    ).toBeVisible();
    expect(
      screen.getAllByText(/Private repository outsiders receive not_found/)[0],
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/graphs/contributors?period=1w&start=2026-05-01T00:00:00Z&end=2026-05-07T00:00:00Z",
      ),
    ).toBeVisible();
    expect(
      screen.getByRole("heading", {
        name: "Repository Contributors insights",
      }),
    ).toBeVisible();
    expect(
      screen.getByText(
        /repository-wide weekly commit buckets, top contributor rows/,
      ),
    ).toBeVisible();
    expect(
      screen.getByText(/default branch through repository_git_refs/),
    ).toBeVisible();
    expect(
      screen.getByText(
        /Optional start and end range bounds are parsed as RFC3339/,
      ),
    ).toBeVisible();
    expect(
      screen.getByText(/Merge commits and empty commits are excluded/),
    ).toBeVisible();
    expect(
      screen.getByText(/authorStatus and isBot metadata for active, bot/),
    ).toBeVisible();
    expect(
      screen.getByText(/repository_contributors_weekly stores bounded rollups/),
    ).toBeVisible();
    expect(screen.getAllByText(/private commit OIDs/).length).toBeGreaterThan(
      2,
    );
    expect(
      screen.getByText("/api/repos/{owner}/{repo}/graphs/traffic"),
    ).toBeVisible();
    expect(
      screen.getByRole("heading", {
        name: "Repository Traffic insights",
      }),
    ).toBeVisible();
    expect(
      screen.getByText(/14-day UTC clone and visitor series/),
    ).toBeVisible();
    expect(
      screen.getAllByText(/repository write, admin, or owner access/)[0],
    ).toBeVisible();
    expect(
      screen.getByText(
        /Clone and visitor series update hourly; referrers and popular content update daily/,
      ),
    ).toBeVisible();
    expect(
      screen.getByText(/traffic_access_required with countsVisible=false/),
    ).toBeVisible();
    expect(screen.getByText(/zero-filled sparse days/)).toBeVisible();
    expect(screen.getByText(/noopener noreferrer/)).toBeVisible();
    expect(
      screen.getByText(
        /repository_traffic_daily, repository_referrers_daily, and repository_popular_content_daily/,
      ),
    ).toBeVisible();
    expect(screen.getAllByText(/environment secrets/)[0]).toBeVisible();
    expect(
      screen.getByText("/api/repos/{owner}/{repo}/security"),
    ).toBeVisible();
    expect(
      screen.getByRole("heading", {
        name: "Repository Security overview",
      }),
    ).toBeVisible();
    expect(
      screen.getByText(
        /screen-ready Security and quality overview, including sanitized SECURITY.md/,
      ),
    ).toBeVisible();
    expect(
      screen.getByText(/Draft advisories remain hidden from overview readers/),
    ).toBeVisible();
    expect(
      screen.getByText(/Maintainers receive concrete private counts/),
    ).toBeVisible();
    expect(
      screen.getAllByText(/Script tags, unsafe URLs, raw session rows/)[0],
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/security/advisories?state=published&severity=high&page=1&page_size=10",
      ),
    ).toBeVisible();
    expect(
      screen.getByRole("heading", {
        name: "List repository security advisories",
      }),
    ).toBeVisible();
    expect(
      screen.getByText(
        /draft rows require maintainer or advisory collaborator/,
      ),
    ).toBeVisible();
    expect(
      screen.getAllByText(
        "/api/repos/{owner}/{repo}/security/advisories/{ghsa_id}",
      )[0],
    ).toBeVisible();
    expect(
      screen.getByRole("heading", {
        name: "Get repository security advisory detail",
      }),
    ).toBeVisible();
    expect(
      screen.getByText(/CVSS score and base metrics, CVE\/CWE disclosures/),
    ).toBeVisible();
    expect(
      screen.getAllByText("/api/repos/{owner}/{repo}/security/advisories")[0],
    ).toBeVisible();
    expect(
      screen.getByRole("heading", {
        name: "Create draft repository security advisory",
      }),
    ).toBeVisible();
    expect(screen.getByText(/generated GHSA-local identifier/)).toBeVisible();
    expect(
      screen.getByRole("heading", {
        name: "Update repository security advisory metadata",
      }),
    ).toBeVisible();
    expect(
      screen.getByText(/Credits and collaborators are replaced/),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/security/advisories/{ghsa_id}/publish",
      ),
    ).toBeVisible();
    expect(
      screen.getByRole("heading", {
        name: "Publish repository security advisory",
      }),
    ).toBeVisible();
    expect(
      screen.getByText(/dependency advisory feed rows when package metadata/),
    ).toBeVisible();
    expect(
      screen.getAllByText("/api/repos/{owner}/{repo}/security/policy")[0],
    ).toBeVisible();
    expect(
      screen.getByRole("heading", {
        name: "Repository Security policy",
      }),
    ).toBeVisible();
    expect(
      screen.getByText(/heading outline anchors, source\/raw\/history\/edit/),
    ).toBeVisible();
    expect(
      screen.getByText(
        /SECURITY.md, .github\/SECURITY.md, then docs\/SECURITY.md/,
      ),
    ).toBeVisible();
    expect(
      screen.getByText(/Relative Markdown links are rewritten/),
    ).toBeVisible();
    expect(
      screen.getByRole("heading", {
        name: "Create repository Security policy",
      }),
    ).toBeVisible();
    expect(
      screen.getByText(/repository file materialization path/),
    ).toBeVisible();
    expect(
      screen.getByText(/MVP does not create propose-change branches/),
    ).toBeVisible();
    expect(
      screen.getByRole("heading", {
        name: "Update repository Security policy",
      }),
    ).toBeVisible();
    expect(
      screen.getAllByText(/expectedContentSha protects concurrent edits/)[0],
    ).toBeVisible();
    expect(
      screen.getByText(/repository.security_policy.upsert audit events/),
    ).toBeVisible();
    expect(screen.getByText("/api/repos/{owner}/{repo}/network")).toBeVisible();
    expect(
      screen.getByRole("heading", {
        name: "Repository Network insights",
      }),
    ).toBeVisible();
    expect(
      screen.getAllByText(/50 most recently pushed readable forks/)[0],
    ).toBeVisible();
    expect(
      screen.getByText(
        /repository_network_forks stores bounded daily projection rows/,
      ),
    ).toBeVisible();
    expect(
      screen.getAllByText(/Branch names with slashes are encoded/)[0],
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/forks?period=1m&type=starred&sort=most_starred",
      ),
    ).toBeVisible();
    expect(
      screen.getByRole("heading", {
        name: "Repository Forks list",
      }),
    ).toBeVisible();
    expect(
      screen.getByText(/Supported period values are 24h, 3d, 1w, 1m, and all/),
    ).toBeVisible();
    expect(
      screen.getByText(/Supported sort values are most_starred/),
    ).toBeVisible();
    expect(
      screen.getByText(/hiddenPrivateForks reports omitted forks/),
    ).toBeVisible();
    expect(
      screen.getByText("/api/repos/{owner}/{repo}/forks/defaults"),
    ).toBeVisible();
    expect(
      screen.getByRole("heading", {
        name: "Save repository Forks defaults",
      }),
    ).toBeVisible();
    expect(
      screen.getByText(/actor-scoped in saved_fork_filter_defaults/),
    ).toBeVisible();
    expect(
      screen.getByText(/same period, repository type, and sort enums/),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/network/dependencies?q=sqlx&ecosystem=cargo&relationship=direct",
      ),
    ).toBeVisible();
    expect(
      screen.getByRole("heading", {
        name: "Repository Dependency graph dependencies",
      }),
    ).toBeVisible();
    expect(screen.getByText(/Supported ecosystems are npm/)).toBeVisible();
    expect(
      screen.getByText(/malformed supported manifests produce no rows/),
    ).toBeVisible();
    expect(
      screen.getByText(
        /dependency_graph_unavailable states use structured 422/,
      ),
    ).toBeVisible();
    expect(
      screen.getByText("/api/repos/{owner}/{repo}/network/dependencies/sbom"),
    ).toBeVisible();
    expect(
      screen.getByRole("heading", {
        name: "Create repository Dependency graph SBOM export",
      }),
    ).toBeVisible();
    expect(screen.getByText(/SPDX-2.3 package/)).toBeVisible();
    expect(screen.getByText(/dependency_graph.sbom_export/)).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/network/dependencies/sbom/{export_id}",
      ),
    ).toBeVisible();
    expect(
      screen.getByRole("heading", {
        name: "Download repository Dependency graph SBOM export",
      }),
    ).toBeVisible();
    expect(screen.getByText(/attachment Content-Disposition/)).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/network/dependents?package=npm%3A%40namuh%2Fflow&owner=acme",
      ),
    ).toBeVisible();
    expect(
      screen.getByRole("heading", {
        name: "Repository Dependency graph dependents",
      }),
    ).toBeVisible();
    expect(
      screen.getByText(
        /Dependents are shown only for public source repositories/,
      ),
    ).toBeVisible();
    expect(
      screen.getByText(
        /Private consumers contribute only to hiddenPrivateCount/,
      ),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/security/secret-scanning?state=open&provider=GitHub&sort=recently_detected",
      ),
    ).toBeVisible();
    expect(
      screen.getByRole("heading", {
        name: "List repository Secret scanning alerts",
      }),
    ).toBeVisible();
    expect(
      screen.getByText(/provider\/default and generic result tabs/),
    ).toBeVisible();
    expect(
      screen.getAllByText(
        "/api/repos/{owner}/{repo}/security/secret-scanning/{alert_id}",
      )[0],
    ).toBeVisible();
    expect(
      screen.getByRole("heading", {
        name: "Read repository Secret scanning alert detail",
      }),
    ).toBeVisible();
    expect(
      screen.getByRole("heading", {
        name: "Update repository Secret scanning alert",
      }),
    ).toBeVisible();
    expect(
      screen.getByText(/plaintext secret bytes are not stored/),
    ).toBeVisible();
    expect(
      screen.getAllByText("/api/repos/{owner}/{repo}/git/receive-pack")[0],
    ).toBeVisible();
    expect(
      screen.getByRole("heading", {
        name: "Secret scanning push protection",
      }),
    ).toBeVisible();
    expect(screen.getByText(/Protected provider matches/)).toBeVisible();
    expect(
      screen.getByText("/api/repos/{owner}/{repo}/releases?page=1&pageSize=30"),
    ).toBeVisible();
    expect(
      screen.getAllByText("/api/repos/{owner}/{repo}/releases/latest")[0],
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/releases/tags?page=1&pageSize=30",
      ),
    ).toBeVisible();
    expect(
      screen.getByText("/api/repos/{owner}/{repo}/releases/manage"),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/releases/manage/generated-notes",
      ),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/releases/manage/upload-intents",
      ),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/releases/manage/upload-intents/{intent_id}/complete",
      ),
    ).toBeVisible();
    expect(
      screen.getByText("/api/repos/{owner}/{repo}/releases/{release_id}"),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/releases/{release_id}/publish",
      ),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/releases/{release_id}/assets",
      ),
    ).toBeVisible();
    expect(
      screen.getByText("/api/repos/{owner}/{repo}/releases/assets/{asset_id}"),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/releases/{release_id}/reactions",
      ),
    ).toBeVisible();
    expect(
      screen.getByText(/Responses never expose S3 or local storage keys/),
    ).toBeVisible();
    expect(
      screen.getByText(/Delete requests accept deleteTag=true/),
    ).toBeVisible();
    expect(
      screen.getByText(/Generated notes never call GitHub APIs/),
    ).toBeVisible();
    expect(
      screen.getByText(/Completion records audit and webhook\/activity/),
    ).toBeVisible();
    expect(
      screen.getByText(/Publishing a prerelease does not mark it latest/),
    ).toBeVisible();
    expect(screen.getByText(/Repeated toggles are idempotent/)).toBeVisible();
    expect(screen.getByText("/api/repos/{owner}/{repo}/pulls")).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/commits?ref=main&path=src&author=mona&until=2026-04-30T23:59:59Z&page=1&pageSize=30",
      ),
    ).toBeVisible();
    expect(
      screen.getByRole("heading", { name: "Repository commit history" }),
    ).toBeVisible();
    expect(
      screen.getByText(/ref resolves against repository_git_refs/),
    ).toBeVisible();
    expect(screen.getByText(/missing refs return ref_not_found/)).toBeVisible();
    expect(
      screen.getByText(/path scopes history to commits touching/),
    ).toBeVisible();
    expect(
      screen.getByText(/author, until, before, page, and pageSize/),
    ).toBeVisible();
    expect(
      screen.getAllByText(/Anonymous callers receive 401/).length,
    ).toBeGreaterThan(0);
    expect(
      screen.getByText(/raw check logs, signing keys, and secret material/),
    ).toBeVisible();
    expect(
      screen.getByText("/api/repos/{owner}/{repo}/commits/{sha}"),
    ).toBeVisible();
    expect(
      screen.getByRole("heading", { name: "Repository commit detail" }),
    ).toBeVisible();
    expect(
      screen.getByText(
        /file tree, bounded unified diffs, Raw\/View file actions/,
      ),
    ).toBeVisible();
    expect(
      screen.getByText(
        /sha accepts an exact OID or an unambiguous abbreviation/,
      ),
    ).toBeVisible();
    expect(
      screen.getByText(/Root commits return an empty parents array/),
    ).toBeVisible();
    expect(
      screen.getByText(
        /Binary and large files keep concrete Raw\/View file actions/,
      ),
    ).toBeVisible();
    expect(screen.getByText(/repository_commit_recent_visits/)).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/commits/{sha}/context?path=src/router.rs&hunkId=diff-src-router-rs-hunk-1&contextLines=80",
      ),
    ).toBeVisible();
    expect(
      screen.getByRole("heading", {
        name: "Repository commit diff context",
      }),
    ).toBeVisible();
    expect(
      screen.getByText(/Expands one commit-detail diff hunk/),
    ).toBeVisible();
    expect(
      screen.getByText(/contextLines is clamped server-side/),
    ).toBeVisible();
    expect(
      screen.getByText(
        /same-origin proxy forwards the current Rust session cookie/,
      ),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/branches?tab=stale&q=release&page=1&pageSize=30",
      ),
    ).toBeVisible();
    expect(
      screen.getByRole("heading", {
        name: "Repository branches directory",
      }),
    ).toBeVisible();
    expect(
      screen.getByText(/Overview, Active, Stale, and All tabs/),
    ).toBeVisible();
    expect(
      screen.getByText(/tree, commits, activity, and rules destinations/),
    ).toBeVisible();
    expect(
      screen.getByText(/branch directory recent-visit telemetry/),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/branches/activity?branch=release%2Fold-tree",
      ),
    ).toBeVisible();
    expect(
      screen.getByRole("heading", { name: "Repository branch activity" }),
    ).toBeVisible();
    expect(
      screen.getByText(/recent commits, recent pull requests/),
    ).toBeVisible();
    expect(
      screen.getByText(/Missing branches return a non-leaky recovery payload/),
    ).toBeVisible();
    expect(
      screen.getByText(/raw rule bypass actors, check logs, tokens/),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/pulls/{number}/files?view=unified&whitespace=show",
      ),
    ).toBeVisible();
    expect(
      screen.getByText("/api/repos/{owner}/{repo}/pulls/{number}/reviews"),
    ).toBeVisible();
    expect(
      screen.getByText("/api/repos/{owner}/{repo}/pulls/{number}/merge"),
    ).toBeVisible();
    expect(
      screen.getByText("/api/repos/{owner}/{repo}/pulls/{number}.diff"),
    ).toBeVisible();
    expect(
      screen.getByText("/api/repos/{owner}/{repo}/pulls/{number}.patch"),
    ).toBeVisible();
    expect(
      screen.getAllByText(
        "/api/repos/{owner}/{repo}/settings/discussions/categories",
      )[0],
    ).toBeVisible();
    expect(
      screen.getByRole("heading", {
        name: "Read repository Discussion category settings",
      }),
    ).toBeVisible();
    expect(
      screen.getByRole("heading", {
        name: "Create repository Discussion category",
      }),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/settings/discussions/categories/{category_id}",
      ),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/settings/discussions/categories/order",
      ),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/settings/discussions/sections",
      ),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/settings/discussions/sections/{section_id}",
      ),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/settings/discussions/sections/order",
      ),
    ).toBeVisible();
    expect(
      screen.getAllByText(
        "/api/repos/{owner}/{repo}/settings/discussions/categories/{category_id}/template",
      )[0],
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/settings/discussions/categories/{category_id}/template/preview",
      ),
    ).toBeVisible();
    expect(
      screen.getByText(
        /Delete-with-move updates affected discussion category ids atomically/,
      ),
    ).toBeVisible();
    expect(
      screen.getByText(/Poll categories return a validation envelope/),
    ).toBeVisible();
    expect(
      screen.getAllByText(/expectedContentSha protects concurrent edits/)[0],
    ).toBeVisible();
    expect(
      screen.getByText(/discussion_category_forms cache rows/),
    ).toBeVisible();
    expect(
      screen.getByText(/Preview does not write Git objects/),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/actions/dashboard?q=ci&status=success&page=1&pageSize=30",
      ),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/actions/workflows/{workflow_file}/dashboard?status=success",
      ),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/actions/workflows/{workflow_file}/dispatches",
      ),
    ).toBeVisible();
    expect(
      screen.getByText(/Invalid YAML keeps the workflow visible/),
    ).toBeVisible();
    expect(
      screen.getAllByText(
        "/api/repos/{owner}/{repo}/actions/workflows?page=1&pageSize=30",
      )[0],
    ).toBeVisible();
    expect(
      screen.getAllByText("/api/repos/{owner}/{repo}/actions/workflows")[0],
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/actions/workflows/{workflow_id}",
      ),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/actions/runs?page=1&pageSize=30",
      ),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/actions/workflows/{workflow_id}/runs",
      ),
    ).toBeVisible();
    expect(
      screen.getAllByText("/api/repos/{owner}/{repo}/actions/runs/{run_id}")[0],
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/actions/runs/{run_id}/detail",
      ),
    ).toBeVisible();
    expect(
      screen.getByText("/api/repos/{owner}/{repo}/actions/runs/{run_id}/rerun"),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/actions/runs/{run_id}/cancel",
      ),
    ).toBeVisible();
    expect(
      screen.getByText("/api/repos/{owner}/{repo}/actions/runs/{run_id}/logs"),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/actions/runs/{run_id}/jobs/{job_id}/detail?q=error&match=1&timestamps=true&raw=false",
      ),
    ).toBeVisible();
    expect(
      screen.getByText("/api/repos/{owner}/{repo}/actions/log-preferences"),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/actions/jobs/{job_id}/logs/download",
      ),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/actions/runs/{run_id}/logs/archive",
      ),
    ).toBeVisible();
    expect(
      screen.getByText("/api/repos/{owner}/{repo}/actions/recent-view"),
    ).toBeVisible();
    expect(
      screen.getByText("/api/repos/{owner}/{repo}/packages"),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/users/{username}/packages/{package_type}/{package_name}?version=sha256:{digest}",
      ),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/orgs/{org}/packages/{package_type}/{package_name}?version=1.0.0",
      ),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/users/{username}/packages/{package_type}/{package_name}/download?version=1.0.0",
      ),
    ).toBeVisible();
    expect(
      screen.getAllByText(
        "/api/users/{username}/packages/{package_type}/{package_name}/settings",
      )[0],
    ).toBeVisible();
    expect(
      screen.getByText(/Supported actions are updateVisibility/),
    ).toBeVisible();
    expect(
      screen.getByText(/Rendering the detail page does not create/),
    ).toBeVisible();
    expect(screen.queryByText(/Reserved for packages-003/)).toBeNull();
    expect(
      screen.queryByText(/packages-002 exposes read-only settings state/),
    ).toBeNull();
    expect(screen.getByText(/Delete actions are soft deletes/)).toBeVisible();
    expect(screen.getByText("/v2/")).toBeVisible();
    expect(
      screen.getAllByText("/v2/{namespace}/{image}/manifests/{reference}")[0],
    ).toBeVisible();
    expect(
      screen.getByText(
        "/v2/{namespace}/{image}/blobs/uploads/ and /v2/{namespace}/{image}/blobs/{digest}",
      ),
    ).toBeVisible();
    expect(
      screen.getAllByText("/v2/{namespace}/{image}/manifests/{reference}")[1],
    ).toBeVisible();
    expect(screen.getByText("/v2/{namespace}/{image}/tags/list")).toBeVisible();
    expect(
      screen.getByText(
        /Workflow jobs may use a short-lived opengithub workflow package token/,
      ),
    ).toBeVisible();
    expect(
      screen.getByText(/package inherits the workflow repository link/),
    ).toBeVisible();
    expect(
      apiEndpointDocs.some((endpoint) =>
        endpoint.request?.includes(
          "docker pull opengithub.namuh.co/mona/octo-image@sha256:manifest",
        ),
      ),
    ).toBe(true);
    expect(
      screen.getByText(
        /Blob storage is retained for audit and retention policy/,
      ),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/notifications?folder=inbox&tab=unread&group=repository&page=1&pageSize=30",
      ),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/notifications/{notification_id}/read and /api/notifications/{notification_id}/unread",
      ),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/notifications/{notification_id}/save, /unsave, /done, and /inbox",
      ),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/notifications/{notification_id}/subscribe and /api/notifications/{notification_id}/unsubscribe",
      ),
    ).toBeVisible();
    expect(screen.getByText("/api/notifications/bulk")).toBeVisible();
    expect(screen.getByText("/api/notifications/custom-filters")).toBeVisible();
    expect(
      screen.getByText("/api/notifications/delivery-preferences"),
    ).toBeVisible();
    expect(
      screen.getByText(/folder=inbox excludes done notifications/),
    ).toBeVisible();
    expect(
      screen.getByText(
        /Done removes rows from Inbox but does not clear unread/,
      ),
    ).toBeVisible();
    expect(
      screen.getByText(/Participation, direct mentions, team mentions/),
    ).toBeVisible();
    expect(
      screen.getByText(/Failed rows stay selected in the browser/),
    ).toBeVisible();
    expect(
      screen.getByText(
        /Email channels require a verified user_email_addresses row/,
      ),
    ).toBeVisible();
    expect(
      screen.getByText(
        /notifications.delivery_preferences.update security audit events/,
      ),
    ).toBeVisible();
    expect(
      screen.getByText(/Each user can store at most 15 custom filters/),
    ).toBeVisible();
    expect(
      screen.getByText(
        /Validation accepts repo:, org:, author:, is:, and reason:/,
      ),
    ).toBeVisible();
    expect(screen.getByText("/api/repos/{owner}/{repo}/watch")).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/issues/{number}/subscription",
      ),
    ).toBeVisible();
    expect(
      screen.getByText("/api/repos/{owner}/{repo}/pulls/{number}/subscription"),
    ).toBeVisible();
    expect(
      screen.getByText(/Supported levels are participating, all, ignore/),
    ).toBeVisible();
    expect(
      screen.getByText(/Issue thread settings override repository watch/),
    ).toBeVisible();
    expect(
      screen.getByText(/Review requests and direct mentions reactivate/),
    ).toBeVisible();
    expect(
      screen.getByText(/Fanout de-dupes recipients after repository watch/),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/security/dependabot?state=open&package=npm%3A%40playwright%2Ftest&sort=most_important",
      ),
    ).toBeVisible();
    expect(
      screen.getAllByText(
        "/api/repos/{owner}/{repo}/security/dependabot/{alert_id}",
      )[0],
    ).toBeVisible();
    expect(
      screen.getByText("/api/repos/{owner}/{repo}/security/dependabot/bulk"),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/security/dependabot/{alert_id}/security-update",
      ),
    ).toBeVisible();
    expect(
      screen.getByText(
        /Alerts are materialized from repository_dependencies joined to dependency_advisories/,
      ),
    ).toBeVisible();
    expect(
      screen.getByText(
        /Successful writes update dependabot_alerts, security_alert_events/,
      ),
    ).toBeVisible();
    expect(
      screen.getByText(
        /Repeated requests return the existing linked pull request/,
      ),
    ).toBeVisible();
    expect(
      screen.getByText(/Dependabot alert triage writes notification rows/),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/security/code-scanning?state=open&tool=CodeQL&sort=most_important",
      ),
    ).toBeVisible();
    expect(
      screen.getAllByText(
        "/api/repos/{owner}/{repo}/security/code-scanning/{alert_id}",
      )[0],
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/security/code-scanning/{alert_id}/issue",
      ),
    ).toBeVisible();
    expect(
      screen.getByText("/api/repos/{owner}/{repo}/code-scanning/sarifs"),
    ).toBeVisible();
    expect(
      screen.getByText(
        /Supported filters are state, q, severity, security_severity, tool, branch, ref, tag, application_code, and sort/,
      ),
    ).toBeVisible();
    expect(
      screen.getByText(
        /Successful writes update code_scanning_alerts, code_scanning_alert_events/,
      ),
    ).toBeVisible();
    expect(
      screen.getByText(/Uploads larger than 2 MiB return 413; malformed JSON/),
    ).toBeVisible();
    expect(
      screen.getByText(/Repeated uploads de-duplicate by stable fingerprint/),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/discussions?q=is%3Aopen&label=help-wanted&sort=latest&page=1&page_size=30",
      ),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/discussions/categories/{slug}?q=is%3Aopen&sort=top",
      ),
    ).toBeVisible();
    expect(
      screen.getAllByText(
        "/api/repos/{owner}/{repo}/discussions/{discussion_number}/vote",
      )[0],
    ).toBeVisible();
    expect(
      screen.getByText(
        /Supported filters are q, label, state, answered, locked, pinned, sort, page, and page_size/,
      ),
    ).toBeVisible();
    expect(
      screen.getByText(
        /Successful first votes write discussion_votes, discussion_activity_events/,
      ),
    ).toBeVisible();
    expect(
      screen.getByText(
        /Vote removal records a discussion activity event but does not create duplicate author notification rows/,
      ),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/discussions/{discussion_number}/poll/vote",
      ),
    ).toBeVisible();
    expect(
      screen.getByText(/Creates or updates the current viewer's poll vote/),
    ).toBeVisible();
    expect(
      screen.getByText(
        /Vote changes are accepted only when allowsVoteChanges is true/,
      ),
    ).toBeVisible();
    expect(
      screen.getByText(/discussion_poll_votes, discussion_activity_events/),
    ).toBeVisible();
    expect(
      screen.getByText("/api/repos/{owner}/{repo}/discussions/new"),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/discussions/new/categories/{slug}?q=import%20preview",
      ),
    ).toBeVisible();
    expect(
      screen.getByText("/api/repos/{owner}/{repo}/discussions"),
    ).toBeVisible();
    expect(screen.getByText(/chooser-ready category cards/)).toBeVisible();
    expect(
      screen.getByText(/Supported YAML field types are input, textarea/),
    ).toBeVisible();
    expect(
      screen.getByText(/similar-search acknowledgement is required/),
    ).toBeVisible();
    expect(
      screen.getByText(/Poll payloads require a question plus two to ten/),
    ).toBeVisible();
    expect(
      screen.getByText(/optional multiple-choice\/change-vote policy/),
    ).toBeVisible();
    expect(
      screen.getByText(/previewing does not create discussion/),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/discussions/{discussion_number}?sort=oldest&page=1&page_size=30",
      ),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/discussions/{discussion_number}/comments",
      ),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/discussions/{discussion_number}/comments/{comment_id}/replies",
      ),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/discussions/{discussion_number}/comments/{comment_id}/reactions",
      ),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/discussions/{discussion_number}/subscription",
      ),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/discussions/{discussion_number}/answer",
      ),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/discussions/{discussion_number}/state",
      ),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/discussions/{discussion_number}/metadata",
      ),
    ).toBeVisible();
    expect(
      screen.getAllByText(
        "/api/repos/{owner}/{repo}/discussions/{discussion_number}/pin",
      ).length,
    ).toBeGreaterThanOrEqual(3);
    expect(
      screen.getAllByText(
        "/api/repos/{owner}/{repo}/discussions/{discussion_number}/lock",
      ).length,
    ).toBeGreaterThanOrEqual(2);
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/discussions/{discussion_number}/category",
      ),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/discussions/{discussion_number}/transfer-targets",
      ),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/discussions/{discussion_number}/transfer",
      ),
    ).toBeVisible();
    expect(
      screen.getByText(
        "/api/repos/{owner}/{repo}/discussions/{discussion_number}/delete",
      ),
    ).toBeVisible();
    expect(
      screen.getAllByText(
        "/api/repos/{owner}/{repo}/issues/{issue_number}/convert-to-discussion",
      ).length,
    ).toBeGreaterThanOrEqual(2);
    expect(
      screen.getByText(/Supported sort values are oldest, newest, and top/),
    ).toBeVisible();
    expect(
      screen.getByText(
        /Poll details include voting controls, result visibility/,
      ),
    ).toBeVisible();
    expect(
      screen.getByText(/Successful writes update discussion_comments/),
    ).toBeVisible();
    expect(
      screen.getByText(/PUT is idempotent for the same viewer/),
    ).toBeVisible();
    expect(
      screen.getByText(/DELETE on the same endpoint unmarks the answer/),
    ).toBeVisible();
    expect(
      screen.getByText(
        /Successful metadata edits write discussion_activity_events/,
      ),
    ).toBeVisible();
    expect(
      screen.getByText(/at most four global pins and four pins per category/),
    ).toBeVisible();
    expect(
      screen.getByText(/allowReactions policy is stored with the lock state/),
    ).toBeVisible();
    expect(
      screen.getByText(/Transfers are constrained to allowed same-owner/),
    ).toBeVisible();
    expect(
      screen.getByText(/Deletion creates a tombstone and audit\/event rows/),
    ).toBeVisible();
    expect(
      screen.getByText(/Conversion is idempotent for already-converted issues/),
    ).toBeVisible();
    expect(
      screen.getByText("/api/search?q=router&type=code&page=1&pageSize=30"),
    ).toBeVisible();
    expect(
      screen.getByText("/api/search/suggestions?q=router&scope=all&limit=8"),
    ).toBeVisible();
    expect(screen.getByText("/api/search/saved-searches")).toBeVisible();
    expect(screen.getByText("/api/search/saved-searches/{id}")).toBeVisible();
    expect(
      screen.getByText(/Duplicate names for the same viewer/),
    ).toBeVisible();
    expect(screen.getAllByText("/api/settings/tokens").length).toBeGreaterThan(
      1,
    );
    expect(screen.getByText("/api/settings/tokens/new")).toBeVisible();
    expect(screen.getByText("/api/settings/sudo")).toBeVisible();
    expect(screen.getByText("/api/settings/tokens/{token_id}")).toBeVisible();
    expect(
      screen.getByText(/returns the plaintext secret exactly once/i),
    ).toBeVisible();
    expect(
      screen.getByText(/The response never includes token_hash/),
    ).toBeVisible();
    expect(
      screen.getByText(/REST, Git, and package-registry token use updates/),
    ).toBeVisible();
    expect(
      screen.getByText(/fine-grained tokens can be limited to selected/),
    ).toBeVisible();
    expect(screen.getByText("/api/settings/keys")).toBeVisible();
    expect(screen.getByText("/api/settings/keys/ssh")).toBeVisible();
    expect(screen.getByText("/api/settings/keys/ssh/{key_id}")).toBeVisible();
    expect(screen.getByText("/api/settings/keys/gpg")).toBeVisible();
    expect(screen.getByText("/api/settings/keys/gpg/{key_id}")).toBeVisible();
    expect(screen.getByText("/api/settings/keys/vigilant-mode")).toBeVisible();
    expect(
      screen.getByText(/raw SSH public keys and armored GPG blocks/),
    ).toBeVisible();
    expect(
      screen.getByText(
        /Duplicate active fingerprints return validation_failed/,
      ),
    ).toBeVisible();
    expect(
      screen.getByText(
        /Active GPG fingerprints drive commit and tag signature/,
      ),
    ).toBeVisible();
    expect(
      screen.getByText(/users.vigilant_mode and writes a vigilant_mode.update/),
    ).toBeVisible();
  });

  it("opens examples without placeholder links or inert controls", () => {
    const { container } = render(<ApiDocsPage />);

    expect(
      container.querySelectorAll('a[href="#"], a:not([href])'),
    ).toHaveLength(0);
    const linkHrefs = screen
      .getAllByRole("link")
      .map((link) => link.getAttribute("href"));
    expect(linkHrefs).toEqual(
      expect.arrayContaining([
        "/docs/git",
        "/docs/get-started",
        "/settings/tokens",
      ]),
    );
    for (const href of linkHrefs) {
      expect(href).toMatch(/^\/(?:docs|settings)\//);
    }

    const firstSummary = screen.getAllByText(
      "Request and response examples",
    )[0];
    const details = firstSummary.closest("details");
    expect(details).not.toBeNull();
    expect(details).not.toHaveAttribute("open");

    fireEvent.click(firstSummary);

    expect(details).toHaveAttribute("open");
    expect(
      screen.getAllByText((content) => content.includes('"login": "mona"'))[0],
    ).toBeVisible();
    expect(
      screen.getByText((content) =>
        content.includes('"code": "validation_failed"'),
      ),
    ).toBeVisible();
    expect(
      screen.getByText((content) =>
        content.includes("GET /api/repos?page=2&pageSize=10"),
      ),
    ).toBeVisible();
  });

  it("copies request examples from docs snippets", {
    timeout: 20_000,
  }, async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    Object.defineProperty(navigator, "clipboard", {
      configurable: true,
      value: { writeText },
    });

    render(<ApiDocsPage />);

    fireEvent.click(screen.getAllByText("Request and response examples")[0]);
    fireEvent.click(screen.getAllByRole("button", { name: "Copy request" })[0]);

    expect(writeText).toHaveBeenCalledWith("GET /api/user");
    expect(await screen.findByRole("status")).toHaveTextContent("Copied");
  });
});
