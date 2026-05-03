import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  type ReleaseReactionContent,
  toggleRepositoryReleaseReactionFromCookie,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{
    owner: string;
    repo: string;
  }>;
};

const reactions = new Set<ReleaseReactionContent>([
  "thumbs_up",
  "thumbs_down",
  "laugh",
  "hooray",
  "confused",
  "heart",
  "rocket",
  "eyes",
]);

export async function POST(request: NextRequest, context: RouteContext) {
  const { owner, repo } = await context.params;
  const body = await request.json().catch(() => null);
  const releaseId =
    typeof body === "object" && body !== null && "releaseId" in body
      ? String(body.releaseId)
      : "";
  const content =
    typeof body === "object" && body !== null && "content" in body
      ? String(body.content)
      : "";

  if (!releaseId.trim()) {
    return NextResponse.json(
      {
        error: {
          code: "validation_failed",
          message: "release id is required",
        },
        status: 422,
      },
      { status: 422 },
    );
  }

  if (!reactions.has(content as ReleaseReactionContent)) {
    return NextResponse.json(
      {
        error: {
          code: "validation_failed",
          message: "reaction is not supported",
        },
        status: 422,
      },
      { status: 422 },
    );
  }

  try {
    const summary = await toggleRepositoryReleaseReactionFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(owner),
      decodeURIComponent(repo),
      releaseId,
      content as ReleaseReactionContent,
    );
    return NextResponse.json(summary, { status: 201 });
  } catch (error) {
    const envelope = (
      error instanceof Error ? error.cause : null
    ) as ApiErrorEnvelope | null;
    return NextResponse.json(
      envelope ?? {
        error: {
          code: "reaction_failed",
          message: "Release reaction could not be updated.",
        },
        status: 500,
      },
      { status: envelope?.status ?? 500 },
    );
  }
}
