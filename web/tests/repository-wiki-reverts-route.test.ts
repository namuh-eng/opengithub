import type { NextRequest } from "next/server";
import { describe, expect, it, vi } from "vitest";
import { POST } from "@/app/api/repos/[owner]/[repo]/wiki/reverts/route";
import { revertRepositoryWikiPageFromCookie } from "@/lib/api";

vi.mock("@/lib/api", async (importOriginal) => {
  const actual = await importOriginal<typeof import("@/lib/api")>();
  return {
    ...actual,
    revertRepositoryWikiPageFromCookie: vi.fn(),
  };
});

const mockedRevert = vi.mocked(revertRepositoryWikiPageFromCookie);

function revertRequest(cookie?: string) {
  return new Request(
    "http://localhost/api/repos/namuh-eng/opengithub/wiki/reverts",
    {
      method: "POST",
      headers: {
        "content-type": "application/json",
        ...(cookie ? { cookie } : {}),
      },
      body: JSON.stringify({
        pageSlug: "Home",
        baseRevisionId: "rev-base",
        expectedHeadRevisionId: "rev-head",
      }),
    },
  ) as NextRequest;
}

describe("repository wiki reverts route", () => {
  it("forwards the request with the session cookie", async () => {
    mockedRevert.mockResolvedValue({
      page: {
        id: "page-1",
        title: "Home",
        slug: "Home",
        href: "/namuh-eng/opengithub/wiki",
        html: "<h1>Home</h1>",
        markdown: "# Home",
        path: "Home.md",
        contentSha: "sha-content",
        outline: [],
        editHref: "/namuh-eng/opengithub/wiki/Home/_edit",
        historyHref: "/namuh-eng/opengithub/wiki/Home/_history",
        revision: {
          id: "rev-restored",
          author: null,
          commitOid: "abc1234567890",
          shortOid: "abc1234",
          message: "Revert wiki page to def5678",
          createdAt: "2026-05-13T00:00:00Z",
          href: "/namuh-eng/opengithub/wiki/Home/_history/abc1234567890",
        },
      },
      gitCommit: {
        id: "commit-1",
        oid: "abc1234567890",
        shortOid: "abc1234",
        branch: "main",
        message: "Revert wiki page to def5678",
        storagePath: "wiki.git",
        createdAt: "2026-05-13T00:00:00Z",
      },
      revertEventId: "event-1",
      restoredRevisionId: "rev-restored",
      redirectHref: "/namuh-eng/opengithub/wiki/Home/_history",
    });

    const response = await POST(revertRequest("og_session=test"), {
      params: Promise.resolve({ owner: "namuh-eng", repo: "opengithub" }),
    });
    const body = await response.json();

    expect(response.status).toBe(200);
    expect(body.redirectHref).toBe("/namuh-eng/opengithub/wiki/Home/_history");
    expect(mockedRevert).toHaveBeenCalledWith(
      "og_session=test",
      "namuh-eng",
      "opengithub",
      {
        pageSlug: "Home",
        baseRevisionId: "rev-base",
        expectedHeadRevisionId: "rev-head",
      },
    );
  });

  it("preserves upstream wiki revert error status codes", async () => {
    mockedRevert.mockRejectedValue(
      new Error("Wiki head revision changed.", {
        cause: {
          status: 409,
          error: {
            code: "wiki_revision_conflict",
            message: "Wiki head revision changed.",
          },
        },
      }),
    );

    const response = await POST(revertRequest(), {
      params: Promise.resolve({ owner: "namuh-eng", repo: "opengithub" }),
    });
    const body = await response.json();

    expect(response.status).toBe(409);
    expect(body.error.code).toBe("wiki_revision_conflict");
    expect(body.error.message).toBe("Wiki head revision changed.");
  });
});
