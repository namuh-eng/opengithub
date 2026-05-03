import { headers } from "next/headers";
import { NextResponse } from "next/server";
import { updateNotificationDeliverySettingsFromCookie } from "@/lib/api";

export async function PATCH(request: Request) {
  const requestHeaders = await headers();
  const cookie = requestHeaders.get("cookie");
  const input = await request.json().catch(() => null);
  if (!input || typeof input !== "object") {
    return NextResponse.json(
      {
        error: {
          code: "invalid_json",
          message: "Request body must be valid JSON.",
        },
        status: 400,
      },
      { status: 400 },
    );
  }

  try {
    const settings = await updateNotificationDeliverySettingsFromCookie(
      cookie,
      input as {
        defaultEmailId?: string | null;
        preferences?: { key: string; channels: string[] }[];
      },
    );
    return NextResponse.json(settings);
  } catch (error) {
    const cause = error instanceof Error ? error.cause : null;
    const envelope =
      cause && typeof cause === "object" && "error" in cause
        ? (cause as {
            error: { code: string; message: string };
            status?: number;
          })
        : null;
    return NextResponse.json(
      envelope ?? {
        error: {
          code: "notification_delivery_failed",
          message: "Notification delivery settings could not be saved.",
        },
        status: 422,
      },
      { status: envelope?.status ?? 422 },
    );
  }
}
