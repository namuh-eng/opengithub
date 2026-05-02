import { type NextRequest, NextResponse } from "next/server";
import { type ApiErrorEnvelope, setUserFollowFromCookie } from "@/lib/api";

type RouteContext = {
  params: Promise<{ owner: string }>;
};

async function updateFollow(
  request: NextRequest,
  context: RouteContext,
  following: boolean,
) {
  const { owner } = await context.params;
  try {
    const state = await setUserFollowFromCookie(
      request.headers.get("cookie"),
      owner,
      following,
    );
    return NextResponse.json(state);
  } catch (error) {
    const envelope = (
      error instanceof Error ? error.cause : null
    ) as ApiErrorEnvelope | null;
    return NextResponse.json(
      envelope ?? {
        error: {
          code: "profile_follow_failed",
          message: "Profile follow update failed",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}

export async function PUT(request: NextRequest, context: RouteContext) {
  return updateFollow(request, context, true);
}

export async function DELETE(request: NextRequest, context: RouteContext) {
  return updateFollow(request, context, false);
}
