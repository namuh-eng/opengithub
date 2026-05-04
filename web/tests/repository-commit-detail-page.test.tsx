import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { RepositoryCommitDetailPage } from "@/components/RepositoryCommitDetailPage";
import type { RepositoryCommitDetailView } from "@/lib/api";

function commitDetail(
  overrides: Partial<RepositoryCommitDetailView> = {},
): RepositoryCommitDetailView {
  const base: RepositoryCommitDetailView = {
    repository: {
      ownerLogin: "mona",
      name: "octo-app",
      defaultBranch: "main",
      visibility: "public",
      href: "/mona/octo-app",
      commitHistoryHref: "/mona/octo-app/commits/main",
    },
    commit: {
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
      committerLogin: "mona",
      committerAvatarUrl: null,
    },
    parents: [
      {
        oid: "1234567890abcdef",
        shortOid: "1234567",
        href: "/mona/octo-app/commit/1234567890abcdef",
      },
    ],
    branches: [
      {
        name: "main",
        qualifiedName: "refs/heads/main",
        kind: "branch",
        href: "/mona/octo-app/commits/main",
      },
    ],
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
    diffPlaceholder: {
      state: "pending_phase",
      message: "Diff rendering arrives in the next commit-detail slice.",
      nextPhase: "Phase 2: Diff File Tree and Unified Diff Rendering",
    },
  };
  return { ...base, ...overrides };
}

describe("RepositoryCommitDetailPage", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("renders commit summary metadata with concrete destinations", () => {
    const { container } = render(
      <RepositoryCommitDetailPage detail={commitDetail()} />,
    );

    expect(
      screen.getByRole("heading", {
        name: "Refactor router into per-resource modules",
      }),
    ).toBeVisible();
    expect(screen.getByRole("link", { name: "octo-app" })).toHaveAttribute(
      "href",
      "/mona/octo-app",
    );
    expect(screen.getByRole("link", { name: "Browse files" })).toHaveAttribute(
      "href",
      "/mona/octo-app/tree/abcdef1234567890",
    );
    expect(
      screen.getByRole("link", { name: "Commit history" }),
    ).toHaveAttribute("href", "/mona/octo-app/commits/main");
    expect(screen.getByRole("link", { name: "3 checks passed" })).toHaveClass(
      "chip",
      "ok",
    );
    expect(screen.getByText("Verified")).toHaveClass("chip", "ok");
    expect(screen.getByRole("link", { name: "1234567" })).toHaveAttribute(
      "href",
      "/mona/octo-app/commit/1234567890abcdef",
    );
    expect(screen.getByRole("link", { name: "#12" })).toHaveAttribute(
      "href",
      "/mona/octo-app/pull/12",
    );
    expect(screen.getByText(/Diff rendering arrives/)).toBeVisible();
    expect(
      container.querySelectorAll('a[href="#"], a:not([href])'),
    ).toHaveLength(0);
  });

  it("copies the full SHA with visible feedback", async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    Object.defineProperty(navigator, "clipboard", {
      configurable: true,
      value: { writeText },
    });
    render(<RepositoryCommitDetailPage detail={commitDetail()} />);

    fireEvent.click(screen.getByRole("button", { name: "Copy full SHA" }));

    await waitFor(() =>
      expect(writeText).toHaveBeenCalledWith("abcdef1234567890"),
    );
    expect(screen.getByRole("status")).toHaveTextContent("Full SHA copied");
  });

  it("keeps Editorial guardrails and root-commit fallback", () => {
    const { container } = render(
      <RepositoryCommitDetailPage
        detail={commitDetail({
          parents: [],
          branches: [],
          pullRequests: [],
          verification: {
            verified: false,
            signatureState: "unverified",
            signatureSummary: null,
          },
          status: {
            status: "pending",
            conclusion: null,
            totalCount: 0,
            completedCount: 0,
            failedCount: 0,
            href: "/mona/octo-app/actions?commit=abcdef1234567890",
          },
        })}
      />,
    );

    expect(screen.getByText("Root commit")).toHaveClass("chip", "soft");
    expect(screen.getByText("No checks")).toHaveClass("chip", "soft");
    expect(screen.getByText("Unverified")).toHaveClass("chip", "soft");
    expect(screen.getByText("No linked pull request.")).toBeVisible();
    expect(container.querySelector(".card")).not.toBeNull();
    expect(
      container.querySelectorAll('a[href="#"], a:not([href])'),
    ).toHaveLength(0);

    const source = readFileSync(
      resolve(process.cwd(), "src/components/RepositoryCommitDetailPage.tsx"),
      "utf8",
    );
    expect(source).not.toMatch(
      /#(0969da|1f883d|1a7f37|cf222e|82071e|f6f8fa|1f2328|d0d7de|59636e|f1aeb5|fff1f3)\b|@primer\/|Octicon/,
    );
    expect(source).not.toContain('href="#"');
    expect(source).not.toContain("onClick={() => {}}");
  });
});
