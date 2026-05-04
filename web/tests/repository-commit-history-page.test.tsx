import { render, screen, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { RepositoryCommitHistoryPage } from "@/components/RepositoryCommitHistoryPage";
import type { RepositoryCommitHistoryView } from "@/lib/api";

function commitHistory(
  overrides: Partial<RepositoryCommitHistoryView> = {},
): RepositoryCommitHistoryView {
  const base: RepositoryCommitHistoryView = {
    repository: {
      ownerLogin: "mona",
      name: "octo-app",
      defaultBranch: "main",
      visibility: "public",
    },
    resolvedRef: {
      shortName: "main",
      qualifiedName: "refs/heads/main",
      kind: "branch",
      targetOid: "abcdef1234567890",
      href: "/mona/octo-app/tree/main",
    },
    filters: {
      path: null,
      author: null,
      until: null,
    },
    groups: [
      {
        date: "2026-04-30",
        commits: [
          {
            oid: "abcdef1234567890",
            shortOid: "abcdef1",
            message:
              "Refactor router into per-resource modules\n\nMove repository routes behind typed handlers.",
            subject: "Refactor router into per-resource modules",
            body: "Move repository routes behind typed handlers.",
            href: "/mona/octo-app/commit/abcdef1234567890",
            browseHref: "/mona/octo-app/tree/abcdef1234567890",
            committedAt: "2026-04-30T00:00:00Z",
            authorLogin: "mona",
            authorAvatarUrl: null,
            pullRequests: [
              {
                number: 12,
                title: "Router cleanup",
                href: "/mona/octo-app/pull/12",
                state: "merged",
              },
            ],
            status: {
              status: "completed",
              conclusion: "success",
              totalCount: 3,
              completedCount: 3,
              failedCount: 0,
              href: "/mona/octo-app/actions?commit=abcdef1234567890",
            },
            verification: {
              verified: true,
              signatureState: "verified",
              signatureSummary: "Verified signature from an active GPG key.",
            },
          },
        ],
      },
    ],
    authorOptions: [
      {
        login: "mona",
        avatarUrl: null,
        count: 1,
        active: false,
      },
    ],
    total: 1,
    page: 1,
    pageSize: 30,
    hasNextPage: false,
    hasPreviousPage: false,
  };
  return { ...base, ...overrides };
}

describe("RepositoryCommitHistoryPage", () => {
  it("renders grouped commit history with concrete destinations", () => {
    const { container } = render(
      <RepositoryCommitHistoryPage history={commitHistory()} />,
    );

    expect(
      screen.getByRole("heading", { name: "Commit history" }),
    ).toBeVisible();
    expect(screen.getByText("April 30, 2026")).toBeVisible();
    expect(
      screen.getByRole("link", {
        name: "Refactor router into per-resource modules",
      }),
    ).toHaveAttribute("href", "/mona/octo-app/commit/abcdef1234567890");
    expect(screen.getByRole("link", { name: "#12" })).toHaveAttribute(
      "href",
      "/mona/octo-app/pull/12",
    );
    expect(
      screen.getByRole("link", { name: "3 checks passed" }),
    ).toHaveAttribute("href", "/mona/octo-app/actions?commit=abcdef1234567890");
    expect(
      screen.getByRole("link", { name: "Browse repository at abcdef1" }),
    ).toHaveAttribute("href", "/mona/octo-app/tree/abcdef1234567890");
    expect(screen.getByRole("link", { name: "abcdef1" })).toHaveAttribute(
      "href",
      "/mona/octo-app/commit/abcdef1234567890",
    );
    expect(screen.getByText("Verified")).toBeVisible();
    expect(
      container.querySelectorAll('a[href="#"], a:not([href])'),
    ).toHaveLength(0);
  });

  it("renders active filters, pagination, and empty-state recovery links", () => {
    render(
      <RepositoryCommitHistoryPage
        history={commitHistory({
          filters: {
            path: "src/main.rs",
            author: "mona",
            until: "2026-04-30T00:00:00Z",
          },
          groups: [],
          total: 0,
          page: 2,
          hasPreviousPage: true,
          hasNextPage: true,
          authorOptions: [
            {
              login: "mona",
              avatarUrl: null,
              count: 4,
              active: true,
            },
          ],
        })}
      />,
    );

    expect(screen.getByText(/Path/)).toBeVisible();
    expect(screen.getByText("src/main.rs")).toBeVisible();
    expect(screen.getByText("Until 2026-04-30")).toBeVisible();
    expect(screen.getByRole("link", { name: "Clear filters" })).toHaveAttribute(
      "href",
      "/mona/octo-app/commits/main/src/main.rs",
    );
    expect(
      screen.getByRole("heading", { name: "No commits found" }),
    ).toBeVisible();
    const pagination = screen.getByRole("navigation", {
      name: "Commit pagination",
    });
    expect(
      within(pagination).getByRole("link", { name: "Previous" }),
    ).toHaveAttribute(
      "href",
      "/mona/octo-app/commits/main/src/main.rs?author=mona&until=2026-04-30T00%3A00%3A00Z",
    );
    expect(
      within(pagination).getByRole("link", { name: "Next" }),
    ).toHaveAttribute(
      "href",
      "/mona/octo-app/commits/main/src/main.rs?author=mona&until=2026-04-30T00%3A00%3A00Z&page=3",
    );
  });
});
