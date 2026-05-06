import { fireEvent, render, screen, within } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { RepositoryFileFinderPage } from "@/components/RepositoryFileFinderPage";
import type { RepositoryFileFinderResult, RepositoryOverview } from "@/lib/api";

function repositoryOverview(): RepositoryOverview {
  return {
    id: "repo-1",
    owner_user_id: "user-1",
    owner_organization_id: null,
    owner_login: "mona",
    name: "octo-app",
    description: "A repository for testing the finder",
    visibility: "public",
    default_branch: "main",
    is_archived: false,
    created_by_user_id: "user-1",
    created_at: "2026-04-30T00:00:00Z",
    updated_at: "2026-04-30T00:00:00Z",
    viewerPermission: "owner",
    branchCount: 2,
    tagCount: 1,
    defaultBranchRef: null,
    latestCommit: null,
    rootEntries: [],
    files: [],
    readme: null,
    sidebar: {
      about: null,
      websiteUrl: null,
      topics: [],
      starsCount: 0,
      watchersCount: 0,
      forksCount: 0,
      releasesCount: 0,
      deploymentsCount: 0,
      contributorsCount: 0,
      languages: [],
    },
    viewerState: {
      starred: false,
      watching: false,
      forkedRepositoryHref: null,
    },
    cloneUrls: {
      https: "https://opengithub.namuh.co/mona/octo-app.git",
      git: "git@opengithub.namuh.co:mona/octo-app.git",
      zip: "/mona/octo-app/archive/refs/heads/main.zip",
    },
  };
}

function finderResult(): RepositoryFileFinderResult {
  return {
    items: [
      {
        path: "README.md",
        name: "README.md",
        kind: "file",
        href: "/mona/octo-app/blob/main/README.md",
        byteSize: 42,
        language: "Markdown",
      },
      {
        path: "src/app/page.tsx",
        name: "page.tsx",
        kind: "file",
        href: "/mona/octo-app/blob/main/src/app/page.tsx",
        byteSize: 2048,
        language: "TypeScript",
      },
      {
        path: "crates/api/src/routes/repositories.rs",
        name: "repositories.rs",
        kind: "file",
        href: "/mona/octo-app/blob/main/crates/api/src/routes/repositories.rs",
        byteSize: 8192,
        language: "Rust",
      },
    ],
    total: 3,
    page: 1,
    pageSize: 100,
    resolvedRef: {
      kind: "branch",
      shortName: "main",
      qualifiedName: "refs/heads/main",
      targetOid: "abcdef1234567890",
      recoveryHref: "/mona/octo-app/tree/main",
    },
    defaultBranchHref: "/mona/octo-app/tree/main",
    recoveryHref: "/mona/octo-app/tree/main",
  };
}

describe("RepositoryFileFinderPage", () => {
  it("renders the cached path list and filters with client-side fuzzy scoring", () => {
    render(
      <RepositoryFileFinderPage
        finder={finderResult()}
        repository={repositoryOverview()}
      />,
    );

    expect(
      screen.getByRole("combobox", { name: "Fuzzy-find a file path" }),
    ).toBeVisible();
    expect(screen.getByText("3 cached paths")).toBeVisible();
    expect(screen.getByRole("option", { name: /README.md/ })).toBeVisible();

    fireEvent.change(
      screen.getByRole("combobox", { name: "Fuzzy-find a file path" }),
      { target: { value: "sap" } },
    );

    const listbox = screen.getByRole("listbox");
    expect(
      within(listbox).getByRole("option", { name: /src\/app\/page.tsx/ }),
    ).toBeVisible();
    expect(
      within(listbox).queryByRole("option", { name: /README.md/ }),
    ).not.toBeInTheDocument();
    expect(screen.getByText("2 matching paths")).toBeVisible();
  });

  it("supports arrow navigation, enter-to-open, escape clearing, and concrete links", () => {
    const assign = vi.fn();
    Object.defineProperty(window, "location", {
      configurable: true,
      value: { assign },
    });
    render(
      <RepositoryFileFinderPage
        finder={finderResult()}
        repository={repositoryOverview()}
      />,
    );
    const input = screen.getByRole("combobox", {
      name: "Fuzzy-find a file path",
    });

    fireEvent.keyDown(input, { key: "ArrowDown" });
    fireEvent.keyDown(input, { key: "Enter" });
    expect(assign).toHaveBeenCalledWith("/mona/octo-app/blob/main/README.md");

    const links = screen.getAllByRole("option");
    for (const link of links) {
      expect(link).toHaveAttribute("href");
      expect(link).not.toHaveAttribute("href", "#");
    }

    fireEvent.change(input, { target: { value: "zzz" } });
    expect(screen.getByRole("status")).toHaveTextContent("No matching files");
    fireEvent.keyDown(input, { key: "Escape" });
    expect(input).toHaveValue("");
  });
});
