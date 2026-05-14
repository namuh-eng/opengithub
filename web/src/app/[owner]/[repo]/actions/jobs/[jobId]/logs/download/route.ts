import { apiBaseUrl } from "@/lib/api";

type RouteContext = {
  params: Promise<{ owner: string; repo: string; jobId: string }>;
};

export async function GET(request: Request, { params }: RouteContext) {
  const { owner, repo, jobId } = await params;
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(
      decodeURIComponent(owner),
    )}/${encodeURIComponent(
      decodeURIComponent(repo),
    )}/actions/jobs/${encodeURIComponent(
      decodeURIComponent(jobId),
    )}/logs/download`,
    {
      headers: request.headers.get("cookie")
        ? { cookie: request.headers.get("cookie") as string }
        : undefined,
      cache: "no-store",
    },
  );

  const headers = new Headers();
  headers.set(
    "content-type",
    response.headers.get("content-type") ?? "text/plain; charset=utf-8",
  );
  const disposition = response.headers.get("content-disposition");
  if (disposition) {
    headers.set("content-disposition", disposition);
  }

  return new Response(await response.arrayBuffer(), {
    status: response.status,
    headers,
  });
}
