import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  updateRepositoryPullRequestSubscriptionFromCookie,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ owner: string; repo: string; number: string }>;
};

export async function PATCH(request: NextRequest, context: RouteContext) {
  const { owner, repo, number } = await context.params;
  const body = await request.json().catch(() => null);
  const subscribed =
    typeof body === "object" && body !== null && "subscribed" in body
      ? Boolean(body.subscribed)
      : false;
  const customEvents =
    typeof body === "object" &&
    body !== null &&
    Array.isArray(body.customEvents)
      ? body.customEvents.filter(
          (event: unknown): event is string => typeof event === "string",
        )
      : [];

  try {
    const subscription =
      await updateRepositoryPullRequestSubscriptionFromCookie(
        request.headers.get("cookie"),
        decodeURIComponent(owner),
        decodeURIComponent(repo),
        decodeURIComponent(number),
        subscribed,
        customEvents,
      );
    return NextResponse.json(subscription);
  } catch (error) {
    const envelope = (
      error instanceof Error ? error.cause : null
    ) as ApiErrorEnvelope | null;
    return NextResponse.json(
      envelope ?? {
        error: {
          code: "subscription_failed",
          message: "Notification subscription could not be updated.",
        },
        status: 500,
      },
      { status: envelope?.status ?? 500 },
    );
  }
}
