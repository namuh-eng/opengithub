import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  type PullRequestState,
  updateRepositoryPullRequestStateFromCookie,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ owner: string; repo: string; number: string }>;
};

function requestedState(value: unknown): PullRequestState | null {
  return value === "open" || value === "closed" ? value : null;
}

export async function PATCH(request: NextRequest, context: RouteContext) {
  const { owner, repo, number } = await context.params;
  const body = await request.json().catch(() => null);
  const state =
    typeof body === "object" && body !== null
      ? requestedState(body.state)
      : null;
  if (!state) {
    return NextResponse.json(
      {
        error: {
          code: "validation_failed",
          message: "state must be open or closed.",
        },
        status: 422,
      },
      { status: 422 },
    );
  }

  try {
    const pullRequest = await updateRepositoryPullRequestStateFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(owner),
      decodeURIComponent(repo),
      decodeURIComponent(number),
      state,
    );
    return NextResponse.json(pullRequest);
  } catch (error) {
    const envelope = (
      error instanceof Error ? error.cause : null
    ) as ApiErrorEnvelope | null;
    return NextResponse.json(
      envelope ?? {
        error: {
          code: "pull_state_failed",
          message: "Pull request state could not be updated.",
        },
        status: 500,
      },
      { status: envelope?.status ?? 500 },
    );
  }
}
