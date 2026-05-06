import { apiBaseUrl } from "@/lib/api";

type RouteContext = {
  params: Promise<{ owner: string; repo: string; cacheId: string }>;
};

export async function DELETE(request: Request, { params }: RouteContext) {
  const { owner, repo, cacheId } = await params;
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(
      decodeURIComponent(owner),
    )}/${encodeURIComponent(
      decodeURIComponent(repo),
    )}/actions/caches/${encodeURIComponent(decodeURIComponent(cacheId))}`,
    {
      method: "DELETE",
      headers: request.headers.get("cookie")
        ? { cookie: request.headers.get("cookie") as string }
        : undefined,
      cache: "no-store",
    },
  );
  const text = await response.text();
  return new Response(text, {
    status: response.status,
    headers: {
      "content-type":
        response.headers.get("content-type") ?? "application/json",
    },
  });
}
