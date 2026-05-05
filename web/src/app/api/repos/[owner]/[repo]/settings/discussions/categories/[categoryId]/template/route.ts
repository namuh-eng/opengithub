import { type NextRequest, NextResponse } from "next/server";
import {
  commitRepositoryDiscussionCategoryTemplateFromCookie,
  type DiscussionCategoryTemplateCommitRequest,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ categoryId: string; owner: string; repo: string }>;
};

function parseCommit(
  input: unknown,
): DiscussionCategoryTemplateCommitRequest | null {
  if (!input || typeof input !== "object") return null;
  const body = input as Record<string, unknown>;
  if (
    typeof body.content !== "string" ||
    typeof body.commitMessage !== "string"
  ) {
    return null;
  }
  return {
    branch: typeof body.branch === "string" ? body.branch : null,
    commitMessage: body.commitMessage,
    content: body.content,
    expectedContentSha:
      typeof body.expectedContentSha === "string"
        ? body.expectedContentSha
        : null,
    proposeChange:
      typeof body.proposeChange === "boolean" ? body.proposeChange : null,
  };
}

export async function PUT(request: NextRequest, context: RouteContext) {
  const { categoryId, owner, repo } = await context.params;
  const mutation = parseCommit(await request.json().catch(() => null));
  if (!mutation) {
    return NextResponse.json(
      {
        error: {
          code: "validation_failed",
          message: "Template content and commit message are required.",
        },
        status: 422,
      },
      { status: 422 },
    );
  }
  try {
    const response = await commitRepositoryDiscussionCategoryTemplateFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(owner),
      decodeURIComponent(repo),
      decodeURIComponent(categoryId),
      mutation,
    );
    return NextResponse.json(response);
  } catch (error) {
    return NextResponse.json(
      {
        error: {
          code: "repository_discussion_category_template_failed",
          message:
            error instanceof Error
              ? error.message
              : "Discussion category template could not be committed.",
        },
        status: 422,
      },
      { status: 422 },
    );
  }
}
