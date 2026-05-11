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
      state: "ready",
      message: "Diff file tree and unified rows are available.",
      nextPhase: "Phase 3: Diff Filter, In-Page Search, and Focus Behavior",
    },
    diffSummary: {
      totalFiles: 2,
      additions: 2,
      deletions: 1,
    },
    fileTree: [
      {
        path: "crates/api/src/routes/repositories.rs",
        name: "repositories.rs",
        depth: 3,
        status: "modified",
        additions: 1,
        deletions: 1,
        href: "#diff-crates-api-src-routes-repositories-rs",
      },
      {
        path: "web/src/components/Commit.tsx",
        name: "Commit.tsx",
        depth: 3,
        status: "added",
        additions: 1,
        deletions: 0,
        href: "#diff-web-src-components-Commit-tsx",
      },
    ],
    files: [
      {
        path: "crates/api/src/routes/repositories.rs",
        previousPath: null,
        status: "modified",
        additions: 1,
        deletions: 1,
        byteSize: 120,
        blobOid: "blob-1",
        language: "Rust",
        anchor: "diff-crates-api-src-routes-repositories-rs",
        href: "/mona/octo-app/commit/abcdef1234567890#diff-crates-api-src-routes-repositories-rs",
        rawHref:
          "/mona/octo-app/raw/abcdef1234567890/crates/api/src/routes/repositories.rs",
        viewHref:
          "/mona/octo-app/blob/abcdef1234567890/crates/api/src/routes/repositories.rs",
        isBinary: false,
        isLarge: false,
        hunks: [
          {
            id: "diff-crates-api-src-routes-repositories-rs-hunk-1",
            header: "@@ -1,2 +1,2 @@ crates/api/src/routes/repositories.rs",
            oldStart: 1,
            oldLines: 2,
            newStart: 1,
            newLines: 2,
            lines: [
              {
                kind: "context",
                oldLine: 1,
                newLine: 1,
                content: "pub fn routes() {",
                position: 1,
              },
              {
                kind: "removed",
                oldLine: 2,
                newLine: null,
                content: "  todo!()",
                position: 2,
              },
              {
                kind: "added",
                oldLine: null,
                newLine: 2,
                content: "  commit_detail()",
                position: 3,
              },
            ],
          },
        ],
      },
      {
        path: "web/src/components/Commit.tsx",
        previousPath: null,
        status: "added",
        additions: 1,
        deletions: 0,
        byteSize: 60,
        blobOid: "blob-2",
        language: "TypeScript",
        anchor: "diff-web-src-components-Commit-tsx",
        href: "/mona/octo-app/commit/abcdef1234567890#diff-web-src-components-Commit-tsx",
        rawHref:
          "/mona/octo-app/raw/abcdef1234567890/web/src/components/Commit.tsx",
        viewHref:
          "/mona/octo-app/blob/abcdef1234567890/web/src/components/Commit.tsx",
        isBinary: false,
        isLarge: false,
        hunks: [],
      },
    ],
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
    expect(screen.getByText(/files changed with/)).toBeVisible();
    expect(
      screen.getByRole("navigation", { name: "Changed file tree" }),
    ).toBeVisible();
    expect(
      screen.getByRole("button", { name: /repositories.rs/ }),
    ).toBeVisible();
    expect(
      screen.getByText("@@ -1,2 +1,2 @@ crates/api/src/routes/repositories.rs"),
    ).toBeVisible();
    expect(screen.getByText("commit_detail()")).toBeVisible();
    expect(screen.getAllByRole("link", { name: "Raw" })[0]).toHaveAttribute(
      "href",
      "/mona/octo-app/raw/abcdef1234567890/crates/api/src/routes/repositories.rs",
    );
    expect(
      screen.getAllByRole("link", { name: "View file" })[0],
    ).toHaveAttribute(
      "href",
      "/mona/octo-app/blob/abcdef1234567890/crates/api/src/routes/repositories.rs",
    );
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
    expect(screen.getByText("Full SHA copied")).toHaveAttribute(
      "role",
      "status",
    );
  });

  it("filters changed files, clears filters, and reports empty states", () => {
    render(<RepositoryCommitDetailPage detail={commitDetail()} />);

    fireEvent.change(screen.getByRole("textbox", { name: "Filter files" }), {
      target: { value: "Commit.tsx" },
    });

    expect(screen.getByRole("button", { name: /Commit.tsx/ })).toBeVisible();
    expect(
      screen.queryByRole("button", { name: /repositories.rs/ }),
    ).not.toBeInTheDocument();
    expect(
      screen.getByRole("article", {
        name: /Diff for web\/src\/components\/Commit.tsx/,
      }),
    ).toBeVisible();
    expect(
      screen.queryByRole("article", {
        name: /Diff for crates\/api\/src\/routes\/repositories.rs/,
      }),
    ).not.toBeInTheDocument();
    expect(screen.getByRole("status")).toHaveTextContent("1 visible file");

    fireEvent.change(screen.getByRole("textbox", { name: "Filter files" }), {
      target: { value: "missing.rs" },
    });
    expect(
      screen.getAllByText("No changed files match this filter."),
    ).toHaveLength(2);

    fireEvent.click(
      screen.getAllByRole("button", { name: "Clear filters" })[0],
    );
    expect(
      screen.getByRole("button", { name: /repositories.rs/ }),
    ).toBeVisible();
    expect(screen.getByRole("button", { name: /Commit.tsx/ })).toBeVisible();
  });

  it("highlights code search safely and reports visible match counts", () => {
    render(<RepositoryCommitDetailPage detail={commitDetail()} />);

    fireEvent.change(
      screen.getByRole("textbox", { name: "Search within code" }),
      {
        target: { value: "commit_detail" },
      },
    );

    expect(screen.getByRole("status")).toHaveTextContent("1 match");
    const highlight = screen.getByText("commit_detail");
    expect(highlight.tagName).toBe("MARK");

    fireEvent.change(
      screen.getByRole("textbox", { name: "Search within code" }),
      {
        target: { value: "<script>" },
      },
    );
    expect(screen.getByRole("status")).toHaveTextContent("0 matches");
    expect(
      screen.getByText("No visible diff lines match this search."),
    ).toBeVisible();
  });

  it("expands hunk context through the same-origin API with feedback", async () => {
    const expandedLine = {
      kind: "context",
      oldLine: 4,
      newLine: 4,
      content: "expanded context from API",
      position: 44,
    };
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: () =>
        Promise.resolve({
          path: "crates/api/src/routes/repositories.rs",
          hunkId: "diff-crates-api-src-routes-repositories-rs-hunk-1",
          lines: [expandedLine],
          expanded: true,
          message: "Expanded context lines loaded.",
        }),
    });
    vi.stubGlobal("fetch", fetchMock);

    render(<RepositoryCommitDetailPage detail={commitDetail()} />);
    fireEvent.click(
      screen.getAllByRole("button", { name: "Expand all lines" })[0],
    );

    await waitFor(() =>
      expect(fetchMock).toHaveBeenCalledWith(
        expect.stringContaining(
          "/api/repos/mona/octo-app/commits/abcdef1234567890/context?",
        ),
        { cache: "no-store" },
      ),
    );
    expect(await screen.findByText("expanded context from API")).toBeVisible();
    expect(screen.getByRole("button", { name: "Expanded" })).toBeDisabled();
  });

  it("shows bounded binary, large, renamed, and removed diff states", () => {
    const detail = commitDetail();
    detail.files = [
      {
        ...detail.files[0],
        path: "src/routes/new.rs",
        previousPath: "src/routes/old.rs",
        status: "renamed",
        anchor: "diff-src-routes-new-rs",
        isBinary: false,
        isLarge: false,
      },
      {
        ...detail.files[0],
        path: "assets/app.bin",
        previousPath: null,
        status: "modified",
        anchor: "diff-assets-app-bin",
        byteSize: 4096,
        isBinary: true,
        isLarge: false,
        hunks: [],
      },
      {
        ...detail.files[0],
        path: "logs/large.txt",
        previousPath: null,
        status: "modified",
        anchor: "diff-logs-large-txt",
        byteSize: 320_000,
        isBinary: false,
        isLarge: true,
        hunks: [],
      },
      {
        ...detail.files[0],
        path: "docs/removed.md",
        previousPath: null,
        status: "removed",
        anchor: "diff-docs-removed-md",
      },
    ];
    detail.fileTree = detail.files.map((file) => ({
      path: file.path,
      name: file.path.split("/").pop() ?? file.path,
      depth: file.path.split("/").length - 1,
      status: file.status,
      additions: file.additions,
      deletions: file.deletions,
      href: `#${file.anchor}`,
    }));
    detail.diffSummary = {
      totalFiles: detail.files.length,
      additions: 4,
      deletions: 3,
    };

    render(<RepositoryCommitDetailPage detail={detail} />);

    expect(screen.getByText("Renamed")).toHaveClass("chip", "soft");
    expect(screen.getByText(/Renamed from/)).toBeVisible();
    expect(screen.getByText("Removed")).toHaveClass("chip", "soft");
    expect(
      screen.getByText("Binary file diff is not rendered inline."),
    ).toBeVisible();
    expect(screen.getByText(/Large file diff is bounded inline/)).toBeVisible();
  });

  it("focuses a diff file from the file tree without placeholder handlers", async () => {
    const scrollIntoView = vi.fn();
    const focus = vi.fn();
    Object.defineProperty(HTMLElement.prototype, "scrollIntoView", {
      configurable: true,
      value: scrollIntoView,
    });
    const animationFrame = vi
      .spyOn(window, "requestAnimationFrame")
      .mockImplementation((callback) => {
        callback(0);
        return 1;
      });
    vi.spyOn(HTMLElement.prototype, "focus").mockImplementation(focus);

    render(<RepositoryCommitDetailPage detail={commitDetail()} />);

    const fileButton = screen.getByRole("button", { name: /Commit.tsx/ });
    fireEvent.click(fileButton);

    await waitFor(() => expect(scrollIntoView).toHaveBeenCalled());
    expect(focus).toHaveBeenCalled();
    expect(fileButton).toHaveAttribute("aria-pressed", "true");
    expect(
      screen.getByRole("article", {
        name: /Diff for web\/src\/components\/Commit.tsx selected/,
      }),
    ).toBeVisible();
    animationFrame.mockRestore();
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
    expect(source).not.toContain("dangerouslySetInnerHTML");
    expect(source).toContain("var(--accent-soft)");
    expect(source).toContain("var(--ok)");
    expect(source).toContain("var(--err)");
  });

  it("keeps semantic chips and accessible diff controls for final QA", () => {
    render(<RepositoryCommitDetailPage detail={commitDetail()} />);

    expect(screen.getByText("Verified")).toHaveClass("chip", "ok");
    expect(screen.getByRole("link", { name: "3 checks passed" })).toHaveClass(
      "chip",
      "ok",
    );
    expect(screen.getByText("Modified")).toHaveClass("chip", "soft");
    expect(screen.getByText("Added")).toHaveClass("chip", "soft");
    expect(
      screen.getByRole("textbox", { name: "Filter files" }),
    ).toHaveAttribute("placeholder", "Path or filename");
    expect(
      screen.getByRole("textbox", { name: "Search within code" }),
    ).toHaveAttribute("placeholder", "Function, selector, or text");
    expect(
      screen.getByRole("button", { name: "Expand all lines" }),
    ).toBeEnabled();
    expect(
      screen.getAllByRole("button", { name: "More actions" })[0],
    ).toBeVisible();
    expect(screen.getByLabelText("Resize diff panes")).toHaveAttribute(
      "type",
      "range",
    );
    fireEvent.change(screen.getByLabelText("Resize diff panes"), {
      target: { value: "340" },
    });
    expect(screen.getByLabelText("Resize diff panes")).toHaveValue("340");
    expect(
      screen.getByRole("article", {
        name: /Diff for crates\/api\/src\/routes\/repositories.rs/,
      }),
    ).toHaveAttribute("tabindex", "-1");
  });
});
