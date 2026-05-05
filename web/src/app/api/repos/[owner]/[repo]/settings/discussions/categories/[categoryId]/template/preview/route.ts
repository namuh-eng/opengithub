import { type NextRequest, NextResponse } from "next/server";
import { previewRepositoryDiscussionCategoryTemplateFromCookie } from "@/lib/api";

type RouteContext = {
  params: Promise<{ categoryId: string; owner: string; repo: string }>;
};

export async function POST(request: NextRequest, context: RouteContext) {
  const { categoryId, owner, repo } = await context.params;
  const body = (await request.json().catch(() => null)) as {
    content?: unknown;
  } | null;
  if (typeof body?.content !== "string") {
    return NextResponse.json(
      {
        error: {
          code: "validation_failed",
          message: "Template content is required.",
        },
        status: 422,
      },
      { status: 422 },
    );
  }
  try {
    const response =
      await previewRepositoryDiscussionCategoryTemplateFromCookie(
        request.headers.get("cookie"),
        decodeURIComponent(owner),
        decodeURIComponent(repo),
        decodeURIComponent(categoryId),
        { content: body.content },
      );
    return NextResponse.json(response);
  } catch (error) {
    return NextResponse.json(
      {
        error: {
          code: "repository_discussion_category_template_preview_failed",
          message:
            error instanceof Error
              ? error.message
              : "Discussion category template preview could not be generated.",
        },
        status: 422,
      },
      { status: 422 },
    );
  }
}
