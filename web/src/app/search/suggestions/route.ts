import { headers } from "next/headers";
import { type NextRequest, NextResponse } from "next/server";
import { getSearchSuggestionsFromCookie } from "@/lib/api";

export async function GET(request: NextRequest) {
  const requestHeaders = await headers();
  const result = await getSearchSuggestionsFromCookie(
    requestHeaders.get("cookie"),
    {
      limit: numberParam(request.nextUrl.searchParams.get("limit")),
      query: request.nextUrl.searchParams.get("q") ?? undefined,
      scope: request.nextUrl.searchParams.get("scope") ?? undefined,
    },
  );

  return NextResponse.json(result, {
    status: "error" in result ? result.status : 200,
  });
}

function numberParam(value: string | null): number | undefined {
  if (!value) {
    return undefined;
  }
  const parsed = Number.parseInt(value, 10);
  return Number.isFinite(parsed) ? parsed : undefined;
}
