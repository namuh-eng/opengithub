import { type NextRequest, NextResponse } from "next/server";
import { apiBaseUrl } from "@/lib/api";

type RouteContext = {
  params: Promise<{ owner: string; repo: string; sha: string }>;
};

export async function GET(request: NextRequest, context: RouteContext) {
  const { owner, repo, sha } = await context.params;
  const upstream = new URL(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(
      decodeURIComponent(owner),
    )}/${encodeURIComponent(decodeURIComponent(repo))}/commits/${encodeURIComponent(
      decodeURIComponent(sha),
    )}/context`,
  );
  for (const [key, value] of request.nextUrl.searchParams.entries()) {
    upstream.searchParams.append(key, value);
  }

  const response = await fetch(upstream, {
    headers: request.headers.get("cookie")
      ? { cookie: request.headers.get("cookie") ?? "" }
      : undefined,
    cache: "no-store",
  });
  const body = await response.text();
  return new NextResponse(body, {
    status: response.status,
    headers: {
      "content-type":
        response.headers.get("content-type") ?? "application/json",
    },
  });
}
