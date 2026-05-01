import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  type MergeMethod,
  mergeRepositoryPullRequestFromCookie,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ owner: string; repo: string; number: string }>;
};

function requestedMethod(value: unknown): MergeMethod {
  return value === "merge_commit" || value === "rebase" || value === "squash"
    ? value
    : "squash";
}

export async function POST(request: NextRequest, context: RouteContext) {
  const { owner, repo, number } = await context.params;
  const body = await request.json().catch(() => null);
  const method =
    typeof body === "object" && body !== null
      ? requestedMethod(body.method)
      : "squash";
  const commitTitle =
    typeof body === "object" &&
    body !== null &&
    typeof body.commitTitle === "string"
      ? body.commitTitle
      : null;
  const commitBody =
    typeof body === "object" &&
    body !== null &&
    typeof body.commitBody === "string"
      ? body.commitBody
      : null;
  const deleteBranch =
    typeof body === "object" && body !== null && body.deleteBranch === true;

  try {
    const pullRequest = await mergeRepositoryPullRequestFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(owner),
      decodeURIComponent(repo),
      decodeURIComponent(number),
      { method, commitTitle, commitBody, deleteBranch },
    );
    return NextResponse.json(pullRequest);
  } catch (error) {
    const envelope = (
      error instanceof Error ? error.cause : null
    ) as ApiErrorEnvelope | null;
    return NextResponse.json(
      envelope ?? {
        error: {
          code: "pull_merge_failed",
          message: "Pull request could not be merged.",
        },
        status: 500,
      },
      { status: envelope?.status ?? 500 },
    );
  }
}
