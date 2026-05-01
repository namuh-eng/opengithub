import { apiBaseUrl } from "@/lib/api";

type RouteContext = {
  params: Promise<{ owner: string; repo: string; runId: string }>;
};

export async function POST(request: Request, { params }: RouteContext) {
  const { owner, repo, runId } = await params;
  const body = await request.text();
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(
      decodeURIComponent(owner),
    )}/${encodeURIComponent(
      decodeURIComponent(repo),
    )}/actions/runs/${encodeURIComponent(decodeURIComponent(runId))}/rerun`,
    {
      method: "POST",
      headers: {
        ...(request.headers.get("cookie")
          ? { cookie: request.headers.get("cookie") as string }
          : {}),
        "content-type": "application/json",
      },
      body: body || "{}",
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
