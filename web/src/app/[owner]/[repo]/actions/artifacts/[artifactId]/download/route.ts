import { apiBaseUrl } from "@/lib/api";

type RouteContext = {
  params: Promise<{ owner: string; repo: string; artifactId: string }>;
};

export async function GET(request: Request, { params }: RouteContext) {
  const { owner, repo, artifactId } = await params;
  const source = new URL(request.url);
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(
      decodeURIComponent(owner),
    )}/${encodeURIComponent(
      decodeURIComponent(repo),
    )}/actions/artifacts/${encodeURIComponent(
      decodeURIComponent(artifactId),
    )}/download`,
    {
      headers: request.headers.get("cookie")
        ? { cookie: request.headers.get("cookie") as string }
        : undefined,
      cache: "no-store",
    },
  );
  const text = await response.text();
  if (!response.ok || source.searchParams.get("metadata") === "1") {
    return new Response(text, {
      status: response.status,
      headers: {
        "content-type":
          response.headers.get("content-type") ?? "application/json",
      },
    });
  }

  const body = JSON.parse(text) as {
    filename?: string;
    name?: string;
    storageKey?: string;
  };
  const filename = body.filename ?? `${body.name ?? "artifact"}.zip`;
  return new Response(
    `opengithub artifact: ${body.name ?? filename}\nstorage key: ${
      body.storageKey ?? "local"
    }\n`,
    {
      status: 200,
      headers: {
        "content-disposition": `attachment; filename="${filename}"`,
        "content-type": "application/octet-stream",
      },
    },
  );
}

export async function DELETE(request: Request, { params }: RouteContext) {
  const { owner, repo, artifactId } = await params;
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(
      decodeURIComponent(owner),
    )}/${encodeURIComponent(
      decodeURIComponent(repo),
    )}/actions/artifacts/${encodeURIComponent(decodeURIComponent(artifactId))}`,
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
