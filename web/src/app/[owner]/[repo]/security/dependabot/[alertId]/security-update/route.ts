import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  createRepositoryDependabotSecurityUpdateFromCookie,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ owner: string; repo: string; alertId: string }>;
};

export async function POST(request: NextRequest, context: RouteContext) {
  const { owner, repo, alertId } = await context.params;

  try {
    const result = await createRepositoryDependabotSecurityUpdateFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(owner),
      decodeURIComponent(repo),
      decodeURIComponent(alertId),
    );
    return NextResponse.json(result, { status: 201 });
  } catch (error) {
    const envelope = (
      error instanceof Error ? error.cause : null
    ) as ApiErrorEnvelope | null;
    return NextResponse.json(
      envelope ?? {
        error: {
          code: "dependabot_security_update_failed",
          message: "Dependabot security update failed.",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}
