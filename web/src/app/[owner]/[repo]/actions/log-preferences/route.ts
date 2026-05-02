import { apiBaseUrl } from "@/lib/api";

type RouteContext = {
  params: Promise<{ owner: string; repo: string }>;
};

export async function PATCH(request: Request, { params }: RouteContext) {
  const { owner, repo } = await params;
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(
      decodeURIComponent(owner),
    )}/${encodeURIComponent(decodeURIComponent(repo))}/actions/log-preferences`,
    {
      method: "PATCH",
      headers: {
        "content-type":
          request.headers.get("content-type") ?? "application/json",
        ...(request.headers.get("cookie")
          ? { cookie: request.headers.get("cookie") as string }
          : {}),
      },
      body: await request.text(),
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
