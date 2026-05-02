import { type NextRequest, NextResponse } from "next/server";
import { type ApiErrorEnvelope, reportUserFromCookie } from "@/lib/api";

type RouteContext = { params: Promise<{ owner: string }> };

export async function POST(request: NextRequest, context: RouteContext) {
  const { owner } = await context.params;
  const input = (await request.json().catch(() => ({}))) as {
    reason?: string;
    details?: string;
  };
  try {
    const receipt = await reportUserFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(owner),
      { reason: input.reason ?? "other", details: input.details },
    );
    return NextResponse.json(receipt, { status: 201 });
  } catch (error) {
    const envelope = (
      error instanceof Error ? error.cause : null
    ) as ApiErrorEnvelope | null;
    return NextResponse.json(
      envelope ?? {
        error: {
          code: "profile_report_failed",
          message: "Profile report failed",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}
