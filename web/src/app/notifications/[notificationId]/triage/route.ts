import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  type NotificationTriageAction,
  updateNotificationTriageFromCookie,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ notificationId: string }>;
};

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

export async function PATCH(request: NextRequest, context: RouteContext) {
  const { notificationId } = await context.params;
  const body = await request.json().catch(() => null);
  const action =
    typeof body === "object" && body !== null && "action" in body
      ? String(body.action)
      : "";

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
    const response = await updateNotificationTriageFromCookie(
      request.headers.get("cookie"),
      notificationId,
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
          code: "notification_triage_failed",
          message: "Notification could not be updated.",
        },
        status: 500,
      },
      { status: envelope?.status ?? 500 },
    );
  }
}
