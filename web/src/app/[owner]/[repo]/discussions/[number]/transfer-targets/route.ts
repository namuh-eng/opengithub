import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  getRepositoryDiscussionTransferTargetsFromCookie,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ owner: string; repo: string; number: string }>;
};

export async function GET(request: NextRequest, context: RouteContext) {
  const { owner, repo, number } = await context.params;
  try {
    const targets = await getRepositoryDiscussionTransferTargetsFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(owner),
      decodeURIComponent(repo),
      decodeURIComponent(number),
    );
    return NextResponse.json(targets);
  } catch (error) {
    const envelope = (
      error instanceof Error ? error.cause : null
    ) as ApiErrorEnvelope | null;
    return NextResponse.json(
      envelope ?? {
        error: {
          code: "discussion_transfer_targets_failed",
          message: "Discussion transfer targets could not be loaded.",
        },
        status: 500,
      },
      { status: envelope?.status ?? 500 },
    );
  }
}
