import { type NextRequest, NextResponse } from "next/server";
import { type ApiErrorEnvelope, setUserBlockFromCookie } from "@/lib/api";

type RouteContext = { params: Promise<{ owner: string }> };

async function updateBlock(
  request: NextRequest,
  context: RouteContext,
  blocked: boolean,
) {
  const { owner } = await context.params;
  try {
    const state = await setUserBlockFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(owner),
      blocked,
    );
    return NextResponse.json(state);
  } catch (error) {
    const envelope = (
      error instanceof Error ? error.cause : null
    ) as ApiErrorEnvelope | null;
    return NextResponse.json(
      envelope ?? {
        error: {
          code: "profile_block_failed",
          message: "Profile block update failed",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}

export async function PUT(request: NextRequest, context: RouteContext) {
  return updateBlock(request, context, true);
}

export async function DELETE(request: NextRequest, context: RouteContext) {
  return updateBlock(request, context, false);
}
