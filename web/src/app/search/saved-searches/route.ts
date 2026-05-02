import { headers } from "next/headers";
import { type NextRequest, NextResponse } from "next/server";
import { createSavedSearchFromCookie } from "@/lib/api";

export async function POST(request: NextRequest) {
  const requestHeaders = await headers();
  const body = await request.json().catch(() => ({}));
  const result = await createSavedSearchFromCookie(
    requestHeaders.get("cookie"),
    {
      name: typeof body.name === "string" ? body.name : "",
      query: typeof body.query === "string" ? body.query : "",
      scope: typeof body.scope === "string" ? body.scope : undefined,
    },
  );

  return NextResponse.json(result, {
    status: "error" in result ? result.status : 201,
  });
}
