import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  publishRepositorySecurityAdvisoryFromCookie,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ owner: string; repo: string; ghsaId: string }>;
};

export async function POST(request: NextRequest, context: RouteContext) {
  const { owner, repo, ghsaId } = await context.params;
  try {
    const advisory = await publishRepositorySecurityAdvisoryFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(owner),
      decodeURIComponent(repo),
      decodeURIComponent(ghsaId),
    );
    return NextResponse.json(advisory);
  } catch (error) {
    const envelope = (
      error instanceof Error ? error.cause : null
    ) as ApiErrorEnvelope | null;
    return NextResponse.json(
      envelope ?? {
        error: {
          code: "security_advisory_publish_failed",
          message: "Repository security advisory publish failed.",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}
