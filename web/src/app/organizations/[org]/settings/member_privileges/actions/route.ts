import { headers } from "next/headers";
import { NextResponse } from "next/server";
import { updateOrganizationMemberPrivilegesFromCookie } from "@/lib/api";

type OrganizationMemberPrivilegesActionsRouteProps = {
  params: Promise<{ org: string }>;
};

export async function PATCH(
  request: Request,
  { params }: OrganizationMemberPrivilegesActionsRouteProps,
) {
  const [{ org }, requestHeaders] = await Promise.all([params, headers()]);
  const cookie = requestHeaders.get("cookie");
  const input = await request.json().catch(() => null);
  if (!input || typeof input !== "object") {
    return NextResponse.json(
      {
        error: {
          code: "invalid_json",
          message: "Request body must be valid JSON",
        },
        status: 400,
      },
      { status: 400 },
    );
  }

  try {
    const settings = await updateOrganizationMemberPrivilegesFromCookie(
      cookie,
      decodeURIComponent(org),
      input,
    );
    return NextResponse.json(settings);
  } catch (error) {
    const cause =
      error instanceof Error && error.cause && typeof error.cause === "object"
        ? (error.cause as {
            details?: Record<string, unknown> | null;
            error?: { code?: string };
            status?: number;
          })
        : null;
    const status = cause?.status ?? 422;
    const code = cause?.error?.code ?? "validation_failed";
    const message =
      error instanceof Error
        ? error.message
        : "Organization member privileges update failed";
    return NextResponse.json(
      { details: cause?.details ?? null, error: { code, message }, status },
      { status },
    );
  }
}
