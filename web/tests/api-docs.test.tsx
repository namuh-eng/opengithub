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
    expect(apiEndpointDocs.length).toBeGreaterThanOrEqual(49);

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
      screen.getByText(/default organization_policy_settings/),
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
