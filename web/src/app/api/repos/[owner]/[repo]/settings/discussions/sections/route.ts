import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  type CreateDiscussionCategorySectionRequest,
  createRepositoryDiscussionCategorySectionFromCookie,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ owner: string; repo: string }>;
};

function parseSection(
  input: unknown,
): CreateDiscussionCategorySectionRequest | null {
  if (!input || typeof input !== "object" || Array.isArray(input)) return null;
  const name = (input as Record<string, unknown>).name;
  if (typeof name !== "string" || !name.trim()) return null;
  return { name: name.trim() };
}

export async function POST(request: NextRequest, context: RouteContext) {
  const { owner, repo } = await context.params;
  const mutation = parseSection(await request.json().catch(() => null));
  if (!mutation) {
    return NextResponse.json(
      {
        error: {
          code: "validation_failed",
          message: "Discussion category section name is required.",
        },
        status: 422,
      },
      { status: 422 },
    );
  }

  try {
    const settings = await createRepositoryDiscussionCategorySectionFromCookie(
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
          code: "repository_discussion_category_section_failed",
          message: "Discussion category section could not be created.",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}
