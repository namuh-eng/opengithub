import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { AdminSearchIndexPage } from "@/components/AdminSearchIndexPage";
import type { SearchIndexStatus } from "@/lib/api";

const status: SearchIndexStatus = {
  documents: [
    {
      kind: "code",
      total: 4,
      latestIndexedAt: "2026-05-07T00:00:00Z",
    },
    {
      kind: "issue",
      total: 2,
      latestIndexedAt: "2026-05-07T00:01:00Z",
    },
  ],
  events: {
    queued: 1,
    running: 1,
    completed: 8,
    failed: 1,
  },
  recentEvents: [
    {
      id: "event-1",
      eventType: "repo.push.code.reindex",
      repositoryId: "repo-1",
      resourceKind: "code",
      resourceId: "repo-1:main",
      status: "completed",
      attempts: 1,
      lastError: null,
      metadata: {},
      completedAt: "2026-05-07T00:02:00Z",
      createdAt: "2026-05-07T00:02:00Z",
      updatedAt: "2026-05-07T00:02:00Z",
    },
  ],
  staleRepositories: [
    {
      repositoryId: "repo-1",
      ownerLogin: "namuh-eng",
      name: "opengithub",
      visibility: "private",
      defaultBranch: "main",
      latestDocumentIndexedAt: null,
      latestEventAt: "2026-05-07T00:03:00Z",
      pendingEvents: 1,
      failedEvents: 1,
    },
  ],
};

describe("AdminSearchIndexPage", () => {
  it("renders indexing pipeline health without dead links", () => {
    render(<AdminSearchIndexPage status={status} />);

    expect(
      screen.getByRole("heading", { name: "Indexing pipeline" }),
    ).toBeVisible();
    expect(screen.getByText("repo.push.code.reindex")).toBeVisible();
    expect(screen.getByText("repo-1:main")).toBeVisible();
    expect(screen.getAllByText("code").length).toBeGreaterThan(0);
    expect(
      screen.getByRole("link", { name: "namuh-eng/opengithub" }),
    ).toHaveAttribute("href", "/namuh-eng/opengithub");
    expect(document.body.innerHTML).not.toContain('href="#"');
    expect(document.body.innerHTML).not.toMatch(/#0969da|Octicon|@primer\//);
  });

  it("renders the unavailable state from the API envelope", () => {
    render(
      <AdminSearchIndexPage
        status={{
          error: {
            code: "not_authenticated",
            message: "Authentication required.",
          },
          status: 401,
        }}
      />,
    );

    expect(
      screen.getByRole("heading", { name: "Index status unavailable" }),
    ).toBeVisible();
    expect(
      screen.getByRole("link", { name: "Return to dashboard" }),
    ).toHaveAttribute("href", "/dashboard");
  });
});
