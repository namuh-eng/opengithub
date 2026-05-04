import { headers } from "next/headers";
import { NextResponse } from "next/server";
import { updateOrganizationProfileSettingsFromCookie } from "@/lib/api";

type OrganizationProfileActionsRouteProps = {
  params: Promise<{ org: string }>;
};

export async function PATCH(
  request: Request,
  { params }: OrganizationProfileActionsRouteProps,
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
    const settings = await updateOrganizationProfileSettingsFromCookie(
      cookie,
      decodeURIComponent(org),
      input,
    );
    return NextResponse.json(settings);
  } catch (error) {
    const message =
      error instanceof Error
        ? error.message
        : "Organization profile settings update failed";
    return NextResponse.json(
      { error: { code: "validation_failed", message }, status: 422 },
      { status: 422 },
    );
  }
}
