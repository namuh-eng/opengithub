import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  voteRepositoryDiscussionPollFromCookie,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{
    owner: string;
    repo: string;
    number: string;
  }>;
};

function pollVoteErrorResponse(error: unknown) {
  const envelope = (
    error instanceof Error ? error.cause : null
  ) as ApiErrorEnvelope | null;
  return NextResponse.json(
    envelope ?? {
      error: {
        code: "discussion_poll_vote_failed",
        message: "Discussion poll vote could not be updated.",
      },
      status: 502,
    },
    { status: envelope?.status ?? 502 },
  );
}

export async function PUT(request: NextRequest, context: RouteContext) {
  const { owner, repo, number } = await context.params;
  const body = await request.json().catch(() => null);
  const optionIds =
    typeof body === "object" && body !== null && Array.isArray(body.optionIds)
      ? body.optionIds.map((optionId: unknown) => String(optionId))
      : [];

  try {
    const response = await voteRepositoryDiscussionPollFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(owner),
      decodeURIComponent(repo),
      decodeURIComponent(number),
      { optionIds },
    );
    return NextResponse.json(response);
  } catch (error) {
    return pollVoteErrorResponse(error);
  }
}
