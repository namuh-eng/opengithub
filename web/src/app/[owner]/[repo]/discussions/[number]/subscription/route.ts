import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  setRepositoryDiscussionSubscriptionFromCookie,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ owner: string; repo: string; number: string }>;
};

async function updateSubscription(
  request: NextRequest,
  context: RouteContext,
  subscribed: boolean,
) {
  const { owner, repo, number } = await context.params;
  try {
    const subscription = await setRepositoryDiscussionSubscriptionFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(owner),
      decodeURIComponent(repo),
      decodeURIComponent(number),
      subscribed,
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

export async function PUT(request: NextRequest, context: RouteContext) {
  return updateSubscription(request, context, true);
}

export async function DELETE(request: NextRequest, context: RouteContext) {
  return updateSubscription(request, context, false);
}
