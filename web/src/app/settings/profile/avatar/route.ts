import { headers } from "next/headers";
import { NextResponse } from "next/server";
import { updatePersonalAvatarFromCookie } from "@/lib/api";

export async function PATCH(request: Request) {
  const requestHeaders = await headers();
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
    const settings = await updatePersonalAvatarFromCookie(cookie, input);
    return NextResponse.json(settings);
  } catch (error) {
    const message =
      error instanceof Error ? error.message : "Avatar update failed";
    return NextResponse.json(
      { error: { code: "validation_failed", message }, status: 422 },
      { status: 422 },
    );
  }
}
