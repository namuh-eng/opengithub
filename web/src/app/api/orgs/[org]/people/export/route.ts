import { type NextRequest, NextResponse } from "next/server";
import { apiBaseUrl } from "@/lib/api";

type RouteContext = {
  params: Promise<{ org: string }>;
};

export async function GET(request: NextRequest, context: RouteContext) {
  const { org } = await context.params;
  const upstream = new URL(
    `${apiBaseUrl()}/api/orgs/${encodeURIComponent(decodeURIComponent(org))}/people/export`,
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
  const headers = new Headers();
  const contentType = response.headers.get("content-type");
  const disposition = response.headers.get("content-disposition");
  if (contentType) {
    headers.set("content-type", contentType);
  }
  if (disposition) {
    headers.set("content-disposition", disposition);
  }

  return new NextResponse(body, {
    status: response.status,
    headers,
  });
}
