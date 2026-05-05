import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  setRepositoryDiscussionVoteFromCookie,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{
    owner: string;
    repo: string;
    number: string;
  }>;
};

function voteErrorResponse(error: unknown) {
  const envelope = (
    error instanceof Error ? error.cause : null
  ) as ApiErrorEnvelope | null;
  return NextResponse.json(
    envelope ?? {
      error: {
        code: "discussion_vote_failed",
        message: "Discussion vote could not be updated.",
      },
      status: 502,
    },
    { status: envelope?.status ?? 502 },
  );
}

async function updateVote(
  request: NextRequest,
  context: RouteContext,
  voted: boolean,
) {
  const { owner, repo, number } = await context.params;
  try {
    const response = await setRepositoryDiscussionVoteFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(owner),
      decodeURIComponent(repo),
      decodeURIComponent(number),
      voted,
    );
    return NextResponse.json(response);
  } catch (error) {
    return voteErrorResponse(error);
  }
}

export async function PUT(request: NextRequest, context: RouteContext) {
  return updateVote(request, context, true);
}

export async function DELETE(request: NextRequest, context: RouteContext) {
  return updateVote(request, context, false);
}
