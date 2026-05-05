import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  type UpdateDiscussionCategoryRequest,
  updateRepositoryDiscussionCategoryFromCookie,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ categoryId: string; owner: string; repo: string }>;
};

function optionalString(value: unknown) {
  return typeof value === "string" && value.trim() ? value.trim() : null;
}

function parseUpdate(input: unknown): UpdateDiscussionCategoryRequest | null {
  if (!input || typeof input !== "object" || Array.isArray(input)) return null;
  const body = input as Record<string, unknown>;
  return {
    description: optionalString(body.description),
    emoji: optionalString(body.emoji),
    format:
      typeof body.format === "string"
        ? (body.format as UpdateDiscussionCategoryRequest["format"])
        : undefined,
    name: optionalString(body.name) ?? undefined,
    sectionId: optionalString(body.sectionId),
  };
}

export async function PATCH(request: NextRequest, context: RouteContext) {
  const { categoryId, owner, repo } = await context.params;
  const mutation = parseUpdate(await request.json().catch(() => null));
  if (!mutation) {
    return NextResponse.json(
      {
        error: {
          code: "validation_failed",
          message: "Discussion category update is invalid.",
        },
        status: 422,
      },
      { status: 422 },
    );
  }

  try {
    const settings = await updateRepositoryDiscussionCategoryFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(owner),
      decodeURIComponent(repo),
      decodeURIComponent(categoryId),
      mutation,
    );
    return NextResponse.json(settings);
  } catch (error) {
    const envelope = (
      error instanceof Error ? error.cause : null
    ) as ApiErrorEnvelope | null;
    return NextResponse.json(
      envelope ?? {
        error: {
          code: "repository_discussion_category_failed",
          message: "Discussion category could not be updated.",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}
