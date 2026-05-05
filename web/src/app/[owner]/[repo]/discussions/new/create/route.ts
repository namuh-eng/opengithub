import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  type CreateDiscussionRequest,
  createRepositoryDiscussionFromCookie,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ owner: string; repo: string }>;
};

function validationError(message: string, field: string) {
  return NextResponse.json(
    {
      error: {
        code: "validation_failed",
        message,
      },
      status: 422,
      details: {
        field,
        reason: message,
      },
    },
    { status: 422 },
  );
}

export async function POST(request: NextRequest, context: RouteContext) {
  const { owner, repo } = await context.params;
  let body: CreateDiscussionRequest;

  try {
    body = (await request.json()) as CreateDiscussionRequest;
  } catch {
    return validationError(
      "Discussion creation payload must be valid JSON.",
      "payload",
    );
  }

  if (!body.categorySlug?.trim()) {
    return validationError("Discussion category is required.", "categorySlug");
  }
  if (!body.title?.trim()) {
    return validationError("Discussion title is required.", "title");
  }
  if (!body.similarSearchAcknowledged) {
    return validationError(
      "Confirm that you searched for similar discussions before starting a discussion.",
      "similarSearchAcknowledged",
    );
  }

  try {
    const discussion = await createRepositoryDiscussionFromCookie(
      request.headers.get("cookie"),
      owner,
      repo,
      {
        ...body,
        categorySlug: body.categorySlug.trim(),
        title: body.title.trim(),
        body: body.body?.trim() ? body.body : null,
      },
    );
    return NextResponse.json(discussion, { status: 201 });
  } catch (error) {
    const envelope = (
      error instanceof Error ? error.cause : null
    ) as ApiErrorEnvelope | null;
    return NextResponse.json(
      envelope ?? {
        error: {
          code: "discussion_create_failed",
          message: "Discussion could not be created.",
        },
        status: 500,
      },
      { status: envelope?.status ?? 500 },
    );
  }
}
