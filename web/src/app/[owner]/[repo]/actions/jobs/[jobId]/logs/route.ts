import { apiBaseUrl } from "@/lib/api";

type RouteContext = {
  params: Promise<{ owner: string; repo: string; jobId: string }>;
};

export async function GET(request: Request, { params }: RouteContext) {
  const { owner, repo, jobId } = await params;
  const source = new URL(request.url);
  const query = source.searchParams.toString();
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(
      decodeURIComponent(owner),
    )}/${encodeURIComponent(
      decodeURIComponent(repo),
    )}/actions/jobs/${encodeURIComponent(decodeURIComponent(jobId))}/logs${
      query ? `?${query}` : ""
    }`,
    {
      headers: request.headers.get("cookie")
        ? { cookie: request.headers.get("cookie") as string }
        : undefined,
      cache: "no-store",
    },
  );

  return new Response(await response.text(), {
    status: response.status,
    headers: {
      "content-type":
        response.headers.get("content-type") ?? "application/json",
    },
  });
}
