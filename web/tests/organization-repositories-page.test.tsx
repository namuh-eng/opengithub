import { render, screen, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { OrganizationRepositoriesPage } from "@/components/OrganizationRepositoriesPage";
import type {
  OrganizationRepositoryList,
  OrganizationRepositoryListItem,
} from "@/lib/api";

function repository(
  overrides: Partial<OrganizationRepositoryListItem> = {},
): OrganizationRepositoryListItem {
  return {
    id: "repo-1",
    owner: "namuh",
    name: "opengithub",
    fullName: "namuh/opengithub",
    description: "A rust-first collaboration platform.",
    visibility: "public",
    href: "/namuh/opengithub",
    defaultBranch: "main",
    primaryLanguage: {
      language: "Rust",
      color: "#b7410e",
      byteCount: 9000,
    },
    languages: [],
    topics: ["developer-tools", "forge"],
    starsCount: 142,
    forksCount: 18,
    openIssuesCount: 5,
    openPullRequestsCount: 2,
    license: { slug: "mit", name: "MIT" },
    isArchived: false,
    isFork: false,
    isTemplate: true,
    isMirror: false,
    canAdmin: false,
    contributedByViewer: false,
    forkSource: null,
    createdAt: "2026-04-01T00:00:00Z",
    updatedAt: "2026-05-01T00:00:00Z",
    ...overrides,
  };
}

function repositoryList(
  overrides: Partial<OrganizationRepositoryList> = {},
): OrganizationRepositoryList {
  const items = overrides.items ?? [repository()];
  return {
    items,
    total: overrides.total ?? items.length,
    page: overrides.page ?? 1,
    pageSize: overrides.pageSize ?? 30,
    mode: "repositories",
    filters: {
      query: null,
      repositoryType: "all",
      language: null,
      sort: "updated-desc",
      density: "comfortable",
      page: 1,
      pageSize: 30,
      ...overrides.filters,
    },
    availableLanguages: [
      { value: "Rust", label: "Rust", count: 1 },
      { value: "TypeScript", label: "TypeScript", count: 1 },
    ],
    availableTypes: [
      { value: "all", label: "All", count: 1 },
      { value: "contributed", label: "Contributed by me", count: 0 },
      { value: "admin", label: "Admin access", count: 0 },
      { value: "public", label: "Public", count: 1 },
      { value: "sources", label: "Sources", count: 1 },
      { value: "forks", label: "Forks", count: 0 },
      { value: "archived", label: "Archived", count: 0 },
      { value: "templates", label: "Templates", count: 1 },
    ],
    tabCounts: {
      repositories: 1,
      projects: 0,
      packages: 0,
      people: 1,
      sponsoring: 0,
    },
    viewerState: {
      authenticated: false,
      isMember: false,
      role: null,
      canViewInternal: false,
      canAdmin: false,
      isFollowing: false,
    },
    ...overrides,
  };
}

describe("OrganizationRepositoriesPage", () => {
  it("renders dense Editorial organization repository rows with concrete links", () => {
    const { container } = render(
      <OrganizationRepositoriesPage
        list={repositoryList({
          items: [
            repository({
              canAdmin: true,
              contributedByViewer: true,
              forkSource: {
                owner: "upstream",
                name: "forge",
                href: "/upstream/forge",
              },
              isFork: true,
              visibility: "internal",
            }),
          ],
        })}
        org="namuh"
      />,
    );

    expect(screen.getByRole("heading", { name: "Repositories" })).toBeVisible();
    expect(screen.getByText("1-1 of 1")).toBeVisible();
    expect(screen.getByRole("link", { name: "opengithub" })).toHaveAttribute(
      "href",
      "/namuh/opengithub",
    );
    expect(screen.getByText("namuh/opengithub · main")).toBeVisible();
    expect(
      screen.getByText("A rust-first collaboration platform."),
    ).toBeVisible();
    expect(screen.getByText("developer-tools")).toBeVisible();
    expect(screen.getByText("Rust")).toBeVisible();
    expect(screen.getByText("142 stars")).toBeVisible();
    expect(screen.getByText("18 forks")).toBeVisible();
    expect(screen.getByText("MIT")).toBeVisible();
    expect(screen.getByText("5 issues")).toBeVisible();
    expect(screen.getByText("2 PRs")).toBeVisible();
    expect(screen.getByText("internal")).toBeVisible();
    expect(screen.getByText("fork")).toBeVisible();
    expect(screen.getByText("admin")).toBeVisible();
    expect(screen.getByText("contributed")).toBeVisible();
    expect(
      screen.getByRole("link", { name: "upstream/forge" }),
    ).toHaveAttribute("href", "/upstream/forge");
    expect(screen.getByText("template")).toBeVisible();
    expect(container.querySelector('a[href="#"], a:not([href])')).toBeNull();
  });

  it("renders URL-backed filters, density controls, and empty-state recovery", () => {
    render(
      <OrganizationRepositoriesPage
        list={repositoryList({
          items: [],
          total: 0,
          filters: {
            query: "api server",
            repositoryType: "forks",
            language: "Rust",
            sort: "stars-desc",
            density: "compact",
            page: 1,
            pageSize: 30,
          },
        })}
        org="namuh"
      />,
    );

    expect(
      screen.getByLabelText("Search organization repositories"),
    ).toHaveValue("api server");
    expect(screen.getByLabelText("Repository type")).toHaveValue("forks");
    expect(screen.getByLabelText("Language")).toHaveValue("Rust");
    expect(screen.getByLabelText("Sort")).toHaveValue("stars-desc");
    expect(screen.getByRole("button", { name: "Filter" })).toHaveAttribute(
      "type",
      "submit",
    );

    const density = screen.getByRole("group", { name: "Display density" });
    expect(
      within(density).getByRole("link", { name: "Comfortable density" }),
    ).toHaveAttribute(
      "href",
      "/orgs/namuh/repositories?q=api+server&type=forks&language=Rust&sort=stars-desc",
    );
    expect(
      within(density).getByRole("link", { name: "Compact density" }),
    ).toHaveAttribute(
      "href",
      "/orgs/namuh/repositories?q=api+server&type=forks&language=Rust&sort=stars-desc&density=compact",
    );
    const typeFilters = screen.getByRole("navigation", {
      name: "Repository type filters",
    });
    expect(
      within(typeFilters).getByRole("link", { name: "All 1" }),
    ).toHaveAttribute(
      "href",
      "/orgs/namuh/repositories?q=api+server&language=Rust&sort=stars-desc&density=compact",
    );
    expect(
      within(typeFilters).getByRole("link", { name: "Sources 1" }),
    ).toHaveAttribute(
      "href",
      "/orgs/namuh/repositories?q=api+server&type=sources&language=Rust&sort=stars-desc&density=compact",
    );
    expect(
      screen.getByRole("link", { name: "Search: api server x" }),
    ).toHaveAttribute(
      "href",
      "/orgs/namuh/repositories?type=forks&language=Rust&sort=stars-desc&density=compact",
    );
    expect(screen.getByRole("link", { name: "Forks x" })).toHaveAttribute(
      "href",
      "/orgs/namuh/repositories?q=api+server&language=Rust&sort=stars-desc&density=compact",
    );
    expect(screen.getByRole("link", { name: "Rust x" })).toHaveAttribute(
      "href",
      "/orgs/namuh/repositories?q=api+server&type=forks&sort=stars-desc&density=compact",
    );
    expect(screen.getByRole("link", { name: "Sort: Stars x" })).toHaveAttribute(
      "href",
      "/orgs/namuh/repositories?q=api+server&type=forks&language=Rust&density=compact",
    );
    expect(
      screen.getByRole("link", { name: "Compact density x" }),
    ).toHaveAttribute(
      "href",
      "/orgs/namuh/repositories?q=api+server&type=forks&language=Rust&sort=stars-desc",
    );
    expect(
      screen.getAllByRole("link", { name: "Clear filters" })[0],
    ).toHaveAttribute("href", "/orgs/namuh/repositories");
    expect(
      screen.getByText("No repositories matched these filters."),
    ).toBeVisible();
  });

  it("renders pagination links that preserve current filters", () => {
    render(
      <OrganizationRepositoriesPage
        list={repositoryList({
          items: [repository()],
          total: 45,
          page: 2,
          pageSize: 10,
          filters: {
            query: "forge",
            repositoryType: "public",
            language: "Rust",
            sort: "stars-desc",
            density: "compact",
            page: 2,
            pageSize: 10,
          },
        })}
        org="namuh"
      />,
    );

    const pagination = screen.getByRole("navigation", {
      name: "Repository pagination",
    });
    expect(
      within(pagination).getByRole("link", { name: "Previous" }),
    ).toHaveAttribute(
      "href",
      "/orgs/namuh/repositories?q=forge&type=public&language=Rust&sort=stars-desc&density=compact&pageSize=10",
    );
    expect(
      within(pagination).getByRole("link", { name: "Next" }),
    ).toHaveAttribute(
      "href",
      "/orgs/namuh/repositories?q=forge&type=public&language=Rust&sort=stars-desc&density=compact&page=3&pageSize=10",
    );
  });

  it("uses disabled real buttons at pagination boundaries", () => {
    render(
      <OrganizationRepositoriesPage
        list={repositoryList({ items: [repository()], total: 1 })}
        org="namuh"
      />,
    );

    const pagination = screen.getByRole("navigation", {
      name: "Repository pagination",
    });
    expect(
      within(pagination).getByRole("button", { name: "Previous" }),
    ).toBeDisabled();
    expect(
      within(pagination).getByRole("button", { name: "Next" }),
    ).toBeDisabled();
  });
});
