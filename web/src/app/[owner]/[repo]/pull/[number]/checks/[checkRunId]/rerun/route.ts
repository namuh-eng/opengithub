import { type NextRequest, NextResponse } from "next/server";
import { apiBaseUrl } from "@/lib/api";

type RerunRouteParams = {
  params: Promise<{
    owner: string;
    repo: string;
    number: string;
    checkRunId: string;
  }>;
};

export async function POST(request: NextRequest, { params }: RerunRouteParams) {
  const { owner, repo, number, checkRunId } = await params;
  const response = await fetch(
    `${apiBaseUrl()}/api/repos/${encodeURIComponent(decodeURIComponent(owner))}/${encodeURIComponent(
      decodeURIComponent(repo),
    )}/pulls/${encodeURIComponent(decodeURIComponent(number))}/checks/${encodeURIComponent(
      decodeURIComponent(checkRunId),
    )}/rerun`,
    {
      method: "POST",
      headers: {
        cookie: request.headers.get("cookie") ?? "",
      },
      cache: "no-store",
    },
  );
  const body = await response.json().catch(() => null);
  return NextResponse.json(body, { status: response.status });
}
