import { type NextRequest, NextResponse } from "next/server";
import { type ApiErrorEnvelope, blockUserFromCookie } from "@/lib/api";

type RouteContext = {
  params: Promise<{ owner: string }>;
};

export async function PUT(request: NextRequest, context: RouteContext) {
  const { owner } = await context.params;
  const body = await request.json().catch(() => ({}));
  try {
    const state = await blockUserFromCookie(
      request.headers.get("cookie"),
      owner,
      typeof body.reason === "string" ? body.reason : undefined,
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
          message: "Profile block failed",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}
