import { type NextRequest, NextResponse } from "next/server";
import { apiBaseUrl, organizationSlugAvailabilityPath } from "@/lib/api";

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

  let response: Response;
  try {
    response = await fetch(
      `${apiBaseUrl()}${organizationSlugAvailabilityPath(name)}`,
      {
        headers: request.headers.get("cookie")
          ? { cookie: request.headers.get("cookie") as string }
          : undefined,
        cache: "no-store",
      },
    );
  } catch {
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

  const body = await response.json().catch(() => null);

  if (!response.ok) {
    return NextResponse.json(
      body ?? {
        error: {
          code: "availability_unavailable",
          message: "Organization slug availability could not be checked",
        },
        status: response.status,
      },
      { status: response.status },
    );
  }

  return NextResponse.json(body);
}
