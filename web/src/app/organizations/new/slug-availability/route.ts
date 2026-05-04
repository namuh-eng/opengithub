import { type NextRequest, NextResponse } from "next/server";
import { getOrganizationSlugAvailabilityFromCookie } from "@/lib/api";

export async function GET(request: NextRequest) {
  const name = request.nextUrl.searchParams.get("name");

  if (!name) {
    return NextResponse.json(
      {
        error: {
          code: "validation_failed",
          message: "name is required",
        },
        status: 422,
      },
      { status: 422 },
    );
  }

  const availability = await getOrganizationSlugAvailabilityFromCookie(
    request.headers.get("cookie"),
    name,
  );

  if (!availability) {
    return NextResponse.json(
      {
        error: {
          code: "availability_unavailable",
          message: "Organization slug availability could not be checked",
        },
        status: 502,
      },
      { status: 502 },
    );
  }

  return NextResponse.json(availability);
}
