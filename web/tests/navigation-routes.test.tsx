import { existsSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import {
  activeOrganizationTab,
  activeProfileTab,
  activeRepositoryTab,
  activeSearchType,
  activeSettingsSection,
  CREATE_NAV_ITEMS,
  createJumpSuggestions,
  GLOBAL_NAV_ITEMS,
  isActivePath,
  navigationHrefs,
  ORGANIZATION_TABS,
  organizationHref,
  organizationProjectHref,
  organizationRepositoryListHref,
  organizationSettingsHref,
  organizationTabHref,
  organizationTeamHref,
  PROFILE_TABS,
  profileRepositoryTabHref,
  profileTabHref,
  queryJumpSuggestions,
  REPOSITORY_TABS,
  repositoryJumpHref,
  repositoryTabHref,
  SEARCH_TABS,
  SETTINGS_NAV_ITEMS,
  searchQueryHref,
  searchTypeHref,
} from "@/lib/navigation";
import { isProtectedPath } from "@/lib/protected-routes";

function routeFileForHref(href: string) {
  const segments = href.split("/").filter(Boolean);
  return join(process.cwd(), "src", "app", ...segments, "page.tsx");
}

function repositoryRouteFileForHref(href: string) {
  const [, , , ...segments] = href.split("/");
  return join(
    process.cwd(),
    "src",
    "app",
    "[owner]",
    "[repo]",
    ...segments,
    "page.tsx",
  );
}

function hasRouteFile(pathSegments: string[]) {
  return existsSync(join(process.cwd(), "src", "app", ...pathSegments));
}

describe("navigation route registry", () => {
  it("points every static signed-in destination at a real route file", () => {
    const missingRoutes = navigationHrefs().filter(
      (href) => !existsSync(routeFileForHref(href)),
    );

    expect(missingRoutes).toEqual([]);
  });

  it("has no inert targets and keeps signed-in destinations protected", () => {
    const items = [
      ...GLOBAL_NAV_ITEMS,
      ...CREATE_NAV_ITEMS,
      ...SETTINGS_NAV_ITEMS,
    ];

    expect(items.map((item) => item.href)).not.toContain("#");
    for (const item of items) {
      expect(item.href).toMatch(/^\//);
      expect(isProtectedPath(item.href)).toBe(item.protected);
    }
  });

  it("matches active global and settings paths deterministically", () => {
    expect(
      isActivePath("/settings/security/sessions", "/settings/security"),
    ).toBe(true);
    expect(isActivePath("/settings/security", "/settings/security")).toBe(true);
    expect(isActivePath("/settings/security", "/settings/profile")).toBe(false);
    expect(activeSettingsSection("/settings/keys")).toBe("keys");
    expect(activeSettingsSection("/settings/notifications/email")).toBe(
      "notifications",
    );
  });

  it("builds repository tab hrefs and active states without losing owner context", () => {
    const tabs = REPOSITORY_TABS.map((tab) => ({
      href: repositoryTabHref("namuh", "opengithub", tab),
      label: tab.label,
    }));

    expect(tabs).toContainEqual({
      href: "/namuh/opengithub",
      label: "Code",
    });
    expect(tabs).toContainEqual({
      href: "/namuh/opengithub/pulls",
      label: "Pull requests",
    });
    expect(tabs).toContainEqual({
      href: "/namuh/opengithub/settings",
      label: "Settings",
    });
    expect(activeRepositoryTab("/namuh/opengithub/pull/42/files")).toBe(
      "pulls",
    );
    expect(activeRepositoryTab("/namuh/opengithub/graphs/contributors")).toBe(
      "pulse",
    );
    expect(activeRepositoryTab("/namuh/opengithub/actions/runs/123")).toBe(
      "actions",
    );
    expect(activeRepositoryTab("/namuh/opengithub/actions/caches")).toBe(
      "actions",
    );
    expect(activeRepositoryTab("/namuh/opengithub/issues/42")).toBe("issues");
  });

  it("points every repository tab at a concrete workspace route", () => {
    const missingRoutes = REPOSITORY_TABS.filter((tab) => tab.segment).filter(
      (tab) =>
        !existsSync(
          repositoryRouteFileForHref(
            repositoryTabHref("namuh", "opengithub", tab),
          ),
        ),
    );

    expect(missingRoutes.map((tab) => tab.label)).toEqual([]);
  });

  it("builds profile, organization, team, and search tabs with preserved context", () => {
    expect(activeProfileTab(undefined)).toBe("overview");
    expect(activeProfileTab("stars")).toBe("stars");
    expect(activeProfileTab("unknown")).toBe("overview");
    expect(profileTabHref("mona lisa", "repositories")).toBe(
      "/mona%20lisa?tab=repositories",
    );
    expect(
      profileRepositoryTabHref(
        "mona lisa",
        {
          query: "api server",
          repositoryType: "forks",
          language: "TypeScript",
          sort: "stars-desc",
        },
        { type: "all" },
      ),
    ).toBe(
      "/mona%20lisa?tab=repositories&q=api+server&language=TypeScript&sort=stars-desc",
    );
    expect(
      profileRepositoryTabHref(
        "mona lisa",
        {
          query: "api server",
          repositoryType: "forks",
          language: "TypeScript",
          sort: "stars-desc",
        },
        { q: null, language: null, sort: "updated-desc" },
      ),
    ).toBe("/mona%20lisa?tab=repositories&type=forks");

    expect(activeOrganizationTab("people")).toBe("people");
    expect(organizationHref("namuh labs")).toBe("/orgs/namuh%20labs");
    expect(organizationTabHref("namuh", "teams")).toBe("/orgs/namuh?tab=teams");
    expect(
      organizationRepositoryListHref(
        "namuh labs",
        {
          query: "api server",
          repositoryType: "forks",
          language: "Rust",
          sort: "stars-desc",
          density: "compact",
          page: 3,
        },
        { type: "all", page: "1" },
      ),
    ).toBe(
      "/orgs/namuh%20labs/repositories?q=api+server&language=Rust&sort=stars-desc&density=compact",
    );
    expect(organizationProjectHref("namuh")).toBe("/orgs/namuh/projects");
    expect(organizationSettingsHref("namuh")).toBe("/orgs/namuh/settings");
    expect(organizationTeamHref("namuh", "platform team")).toBe(
      "/orgs/namuh/teams/platform%20team",
    );

    expect(activeSearchType(undefined)).toBe("repositories");
    expect(activeSearchType("pull_requests")).toBe("pull_requests");
    expect(searchTypeHref("code", "router guards")).toBe(
      "/search?q=router+guards&type=code",
    );
    expect(searchQueryHref("router guards")).toBe(
      "/search?q=router+guards&type=repositories",
    );
    expect(repositoryJumpHref("mona lisa", "editorial app")).toBe(
      "/mona%20lisa/editorial%20app",
    );
  });

  it("builds typed header jump suggestions with concrete destinations", () => {
    expect(createJumpSuggestions()).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          href: "/new",
          kind: "create",
          section: "Create",
        }),
      ]),
    );
    expect(queryJumpSuggestions("router guards")).toEqual([
      expect.objectContaining({
        href: "/search?q=router+guards&type=repositories",
        kind: "search",
        section: "Search",
      }),
    ]);
    expect(queryJumpSuggestions("   ")).toEqual([]);
  });

  it("keeps phase 4 skeleton routes concrete without colliding with repository pages", () => {
    expect(hasRouteFile(["[owner]", "page.tsx"])).toBe(true);
    expect(hasRouteFile(["[owner]", "[repo]", "page.tsx"])).toBe(true);
    expect(
      hasRouteFile(["[owner]", "[repo]", "actions", "caches", "page.tsx"]),
    ).toBe(true);
    expect(
      hasRouteFile(["[owner]", "[repo]", "actions", "deployments", "page.tsx"]),
    ).toBe(true);
    expect(
      hasRouteFile([
        "[owner]",
        "[repo]",
        "actions",
        "attestations",
        "page.tsx",
      ]),
    ).toBe(true);
    expect(
      hasRouteFile(["[owner]", "[repo]", "actions", "usage", "page.tsx"]),
    ).toBe(true);
    expect(
      hasRouteFile(["[owner]", "[repo]", "actions", "performance", "page.tsx"]),
    ).toBe(true);
    expect(hasRouteFile(["orgs", "[org]", "page.tsx"])).toBe(true);
    expect(hasRouteFile(["orgs", "[org]", "repositories", "page.tsx"])).toBe(
      true,
    );
    expect(hasRouteFile(["orgs", "[org]", "projects", "page.tsx"])).toBe(true);
    expect(hasRouteFile(["orgs", "[org]", "settings", "page.tsx"])).toBe(true);
    expect(
      hasRouteFile(["orgs", "[org]", "teams", "[teamSlug]", "page.tsx"]),
    ).toBe(true);
    expect(hasRouteFile(["organizations", "new", "page.tsx"])).toBe(true);
    expect(hasRouteFile(["[org]", "[teamSlug]", "page.tsx"])).toBe(false);

    expect(isProtectedPath("/organizations/new")).toBe(true);
    expect(isProtectedPath("/orgs/namuh/settings")).toBe(true);
    expect(isProtectedPath("/orgs/namuh/teams/platform")).toBe(false);
  });

  it("does not define inert profile, organization, or search tab targets", () => {
    const hrefs = [
      ...PROFILE_TABS.map((tab) => profileTabHref("mona", tab.value)),
      ...ORGANIZATION_TABS.map((tab) =>
        organizationTabHref("namuh", tab.value),
      ),
      ...SEARCH_TABS.map((tab) => searchTypeHref(tab.value, "query")),
      organizationProjectHref("namuh"),
      organizationSettingsHref("namuh"),
      organizationTeamHref("namuh", "platform"),
    ];

    expect(hrefs).not.toContain("#");
    for (const href of hrefs) {
      expect(href).toMatch(/^\//);
    }
  });
});
