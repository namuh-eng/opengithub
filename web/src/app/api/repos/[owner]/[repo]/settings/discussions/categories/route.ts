import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  type CreateDiscussionCategoryRequest,
  createRepositoryDiscussionCategoryFromCookie,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ owner: string; repo: string }>;
};

function optionalString(value: unknown) {
  return typeof value === "string" && value.trim() ? value.trim() : null;
}

function parseCreate(input: unknown): CreateDiscussionCategoryRequest | null {
  if (!input || typeof input !== "object" || Array.isArray(input)) return null;
  const body = input as Record<string, unknown>;
  const name = optionalString(body.name);
  if (!name) return null;
  return {
    description: optionalString(body.description),
    emoji: optionalString(body.emoji),
    format:
      typeof body.format === "string"
        ? (body.format as CreateDiscussionCategoryRequest["format"])
        : undefined,
    name,
    sectionId: optionalString(body.sectionId),
  };
}

export async function POST(request: NextRequest, context: RouteContext) {
  const { owner, repo } = await context.params;
  const mutation = parseCreate(await request.json().catch(() => null));
  if (!mutation) {
    return NextResponse.json(
      {
        error: {
          code: "validation_failed",
          message: "Discussion category name is required.",
        },
        status: 422,
      },
      { status: 422 },
    );
  }

  try {
    const settings = await createRepositoryDiscussionCategoryFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(owner),
      decodeURIComponent(repo),
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
          message: "Discussion category could not be created.",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}
