import { apiBaseUrl } from "@/lib/api";

type RouteContext = {
  params: Promise<{ owner: string; repo: string; number: string }>;
};

function rawFormat(value: string) {
  if (value.endsWith(".diff")) {
    return { format: "diff", number: value.slice(0, -".diff".length) };
  }
  if (value.endsWith(".patch")) {
    return { format: "patch", number: value.slice(0, -".patch".length) };
  }
  return null;
}

export async function GET(request: Request, { params }: RouteContext) {
  const { owner, repo, number } = await params;
  const raw = rawFormat(decodeURIComponent(number));
  if (!raw) {
    return Response.json(
      {
        error: {
          code: "not_found",
          message: "Use .diff or .patch for raw pull request text.",
        },
        status: 404,
      },
      { status: 404 },
    );
  }

  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(
      decodeURIComponent(owner),
    )}/${encodeURIComponent(decodeURIComponent(repo))}/pulls/${encodeURIComponent(
      raw.number,
    )}.${raw.format}`,
    {
      headers: request.headers.get("cookie")
        ? { cookie: request.headers.get("cookie") as string }
        : undefined,
      cache: "no-store",
    },
  );

  return new Response(response.body, {
    status: response.status,
    headers: {
      "content-type":
        response.headers.get("content-type") ?? "text/plain; charset=utf-8",
    },
  });
}
