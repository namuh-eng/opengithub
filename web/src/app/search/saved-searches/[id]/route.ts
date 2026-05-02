import { headers } from "next/headers";
import { type NextRequest, NextResponse } from "next/server";
import { deleteSavedSearchFromCookie } from "@/lib/api";

type RouteContext = {
  params: Promise<{ id: string }>;
};

export async function DELETE(_request: NextRequest, context: RouteContext) {
  const requestHeaders = await headers();
  const { id } = await context.params;
  const result = await deleteSavedSearchFromCookie(
    requestHeaders.get("cookie"),
    id,
  );

  if (!("error" in result)) {
    return new NextResponse(null, { status: 204 });
  }

  return NextResponse.json(result, { status: result.status });
}
