import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  createRepositoryCodeScanningIssueFromCookie,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ owner: string; repo: string; alertId: string }>;
};

export async function POST(request: NextRequest, context: RouteContext) {
  const { owner, repo, alertId } = await context.params;

  try {
    const detail = await createRepositoryCodeScanningIssueFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(owner),
      decodeURIComponent(repo),
      decodeURIComponent(alertId),
    );
    return NextResponse.json(detail, { status: 201 });
  } catch (error) {
    const envelope = (
      error instanceof Error ? error.cause : null
    ) as ApiErrorEnvelope | null;
    return NextResponse.json(
      envelope ?? {
        error: {
          code: "code_scanning_issue_link_failed",
          message: "Code scanning issue link failed.",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}
