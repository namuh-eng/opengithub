import { afterEach, describe, expect, it, vi } from "vitest";
import { getRepositoryIssuesFromCookie } from "@/lib/api";

describe("API client issue list handling", () => {
  afterEach(() => {
    vi.restoreAllMocks();
    vi.unstubAllEnvs();
  });

  it("returns an API error instead of an unsafe issue list when the API shape is stale", async () => {
    vi.stubEnv("API_URL", "http://api.test");
    const fetchMock = vi.fn(async () => {
      return new Response(
        JSON.stringify({
          items: [],
          total: 0,
          page: 1,
          pageSize: 30,
        }),
        { status: 200 },
      );
    });
    vi.stubGlobal("fetch", fetchMock);

    const result = await getRepositoryIssuesFromCookie(
      "og_session=test",
      "mona",
      "octo-app",
      { state: "open" },
    );

    expect(fetchMock).toHaveBeenCalledWith(
      "http://api.test/api/repos/mona/octo-app/issues?state=open",
      {
        headers: { cookie: "og_session=test" },
        cache: "no-store",
      },
    );
    expect(result).toEqual({
      error: {
        code: "invalid_issues_response",
        message:
          "Issues are temporarily unavailable because the API returned an outdated response shape.",
      },
      status: 502,
      details: {
        reason:
          "Restart the API server so the frontend receives issue filters and metadata.",
      },
    });
  });
});
