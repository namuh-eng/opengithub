import { apiBaseUrl } from "@/lib/api";

export async function GET(request: Request) {
  const source = new URL(request.url);
  const upstream = new URL(`${apiBaseUrl()}/api/settings/security-log/export`);
  const action = source.searchParams.get("action");
  const format = source.searchParams.get("format") ?? "csv";
  if (action) upstream.searchParams.set("action", action);
  upstream.searchParams.set("format", format);

  const response = await fetch(upstream, {
    headers: request.headers.get("cookie")
      ? { cookie: request.headers.get("cookie") as string }
      : undefined,
    cache: "no-store",
  });
  const body = await response.arrayBuffer();
  return new Response(body, {
    status: response.status,
    headers: {
      "content-disposition":
        response.headers.get("content-disposition") ??
        `attachment; filename="opengithub-security-log.${format === "json" ? "json" : "csv"}"`,
      "content-type":
        response.headers.get("content-type") ??
        (format === "json"
          ? "application/json; charset=utf-8"
          : "text/csv; charset=utf-8"),
    },
  });
}
