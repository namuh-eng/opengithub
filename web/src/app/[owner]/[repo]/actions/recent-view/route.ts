import { apiBaseUrl } from "@/lib/api";

type RouteContext = {
  params: Promise<{ owner: string; repo: string }>;
};

export async function POST(request: Request, { params }: RouteContext) {
  const { owner, repo } = await params;
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(
      decodeURIComponent(owner),
    )}/${encodeURIComponent(decodeURIComponent(repo))}/actions/recent-view`,
    {
      body: await request.text(),
      headers: {
        "content-type":
          request.headers.get("content-type") ?? "application/json",
        ...(request.headers.get("cookie")
          ? { cookie: request.headers.get("cookie") as string }
          : {}),
      },
      method: "POST",
      cache: "no-store",
    },
  );

  const body = await response.text();
  return new Response(body, {
    status: response.status,
    headers: {
      "content-type":
        response.headers.get("content-type") ?? "application/json",
    },
  });
}
