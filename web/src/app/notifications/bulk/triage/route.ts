import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  bulkUpdateNotificationTriageFromCookie,
  type NotificationTriageAction,
} from "@/lib/api";

const ACTIONS = new Set<NotificationTriageAction>([
  "read",
  "unread",
  "save",
  "unsave",
  "done",
  "inbox",
  "subscribe",
  "unsubscribe",
]);

export async function POST(request: NextRequest) {
  const body = await request.json().catch(() => null);
  const payload =
    typeof body === "object" && body !== null
      ? (body as { action?: unknown; notificationIds?: unknown })
      : null;
  const action = payload && "action" in payload ? String(payload.action) : "";
  const notificationIds =
    payload && Array.isArray(payload.notificationIds)
      ? payload.notificationIds.filter(
          (id): id is string => typeof id === "string",
        )
      : [];

  if (!ACTIONS.has(action as NotificationTriageAction)) {
    return NextResponse.json(
      {
        error: {
          code: "validation_failed",
          message:
            "action must be read, unread, save, unsave, done, inbox, subscribe, or unsubscribe",
        },
        status: 422,
      },
      { status: 422 },
    );
  }

  try {
    const response = await bulkUpdateNotificationTriageFromCookie(
      request.headers.get("cookie"),
      notificationIds,
      action as NotificationTriageAction,
    );
    return NextResponse.json(response);
  } catch (error) {
    const envelope = (
      error instanceof Error ? error.cause : null
    ) as ApiErrorEnvelope | null;
    return NextResponse.json(
      envelope ?? {
        error: {
          code: "notification_bulk_triage_failed",
          message: "Notifications could not be updated.",
        },
        status: 500,
      },
      { status: envelope?.status ?? 500 },
    );
  }
}
