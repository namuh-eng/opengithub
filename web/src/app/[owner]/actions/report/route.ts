import { type NextRequest, NextResponse } from "next/server";
import { type ApiErrorEnvelope, reportUserFromCookie } from "@/lib/api";

type RouteContext = {
  params: Promise<{ owner: string }>;
};

export async function POST(request: NextRequest, context: RouteContext) {
  const { owner } = await context.params;
  const body = await request.json().catch(() => ({}));
  try {
    const report = await reportUserFromCookie(
      request.headers.get("cookie"),
      owner,
      {
        details: typeof body.details === "string" ? body.details : undefined,
        reason: typeof body.reason === "string" ? body.reason : "",
      },
    );
    return NextResponse.json(report, { status: 201 });
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
